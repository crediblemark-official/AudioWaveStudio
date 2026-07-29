import { SongMetadata } from '../types/visualizer';
import { rustBridge } from './rustBridge';

export class AudioEngine {
  private audioCtx: AudioContext | null = null;
  private analyser: AnalyserNode | null = null;
  private gainNode: GainNode | null = null;
  private mediaDestination: MediaStreamAudioDestinationNode | null = null;
  
  private audioBuffer: AudioBuffer | null = null;
  private sourceNode: AudioBufferSourceNode | null = null;
  
  private isPlaying: boolean = false;
  private startTime: number = 0;
  private pausedAt: number = 0;
  private volume: number = 0.8;
  
  private onTimeUpdateCb: ((time: number) => void) | null = null;
  private onEndedCb: (() => void) | null = null;
  private animationFrameId: number | null = null;

  private songMeta: SongMetadata | null = null;
  private rustDecoded: boolean = false;
  private pendingFileBytes: ArrayBuffer | null = null;
  private pendingFileExt: string = '';

  constructor() {
    // AudioContext will be initialized on user interaction or load
  }

  private async initContext() {
    if (!this.audioCtx) {
      const AudioCtxClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
      this.audioCtx = new AudioCtxClass();
      this.analyser = this.audioCtx.createAnalyser();
      this.gainNode = this.audioCtx.createGain();
      
      this.analyser.fftSize = 1024;
      this.analyser.smoothingTimeConstant = 0.8;
      this.gainNode.gain.value = this.volume;

      // Create stream destination for MediaRecorder MP4 Export
      this.mediaDestination = this.audioCtx.createMediaStreamDestination();

      // Connect nodes
      this.gainNode.connect(this.audioCtx.destination);
      this.gainNode.connect(this.mediaDestination);
    }

    if (this.audioCtx.state === 'suspended') {
      await this.audioCtx.resume();
    }
  }

  private createAudioBufferFromSamples(samples: number[], sampleRate: number, _channels: number): AudioBuffer | null {
    if (!this.audioCtx || samples.length === 0) return null;

    // Rust decoder always downmixes to mono (one sample per frame).
    // Always create a mono AudioBuffer using all samples directly.
    const numFrames = samples.length;
    const buffer = this.audioCtx.createBuffer(1, numFrames, sampleRate);

    const channelData = buffer.getChannelData(0);
    for (let i = 0; i < numFrames; i++) {
      channelData[i] = samples[i];
    }

    return buffer;
  }

  public async loadAudioPath(filePath: string): Promise<SongMetadata> {
    console.log('[AudioEngine] loadAudioPath called:', filePath);
    await this.initContext();
    this.stop();
    this.rustDecoded = false;
    this.pendingFileBytes = null;
    this.pendingFileExt = '';

    // Rust decode → PCM samples → create AudioBuffer (no browser GStreamer dependency)
    try {
      console.log('[AudioEngine] Calling Rust decode...');
      const result = await rustBridge.decodeAudioPlayback(filePath);
      console.log('[AudioEngine] Rust decode result:', {
        sampleRate: result.sample_rate,
        channels: result.channels,
        duration: result.duration,
        samplesLength: result.samples.length,
      });
      this.audioBuffer = this.createAudioBufferFromSamples(result.samples, result.sample_rate, result.channels);
      console.log('[AudioEngine] AudioBuffer created:', this.audioBuffer ? {
        duration: this.audioBuffer.duration,
        length: this.audioBuffer.length,
        numberOfChannels: this.audioBuffer.numberOfChannels,
        sampleRate: this.audioBuffer.sampleRate,
      } : 'NULL');
      // Update songMeta duration from actual decode
      if (this.songMeta) {
        this.songMeta.duration = result.duration;
      }
    } catch (e) {
      console.error('[AudioEngine] Rust audio decode FAILED:', e);
    }

    // Extract filename and title
    const fileName = filePath.split(/[/\\]/).pop() || 'Track';
    const fileNameWithoutExt = fileName.replace(/\.[^/.]+$/, '');
    const parts = fileNameWithoutExt.split(' - ');
    const title = parts.length > 1 ? parts.slice(1).join(' - ') : fileNameWithoutExt;
    const artist = parts.length > 1 ? parts[0] : 'Unknown Artist';

    this.songMeta = {
      fileName,
      title,
      artist,
      duration: this.audioBuffer ? this.audioBuffer.duration : 0,
      audioUrl: filePath,
    };

    this.pausedAt = 0;

    return this.songMeta;
  }

  public async loadAudioFile(file: File): Promise<SongMetadata> {
    await this.initContext();
    this.stop();
    this.rustDecoded = false;

    const filePath = (file as unknown as { path?: string }).path;
    const ext = file.name.split('.').pop() || 'mp3';

    let audioFilePath = filePath || '';

    // If no native path, save bytes to temp file so Rust can decode it
    if (!audioFilePath) {
      const arrayBuffer = await file.arrayBuffer();
      this.pendingFileBytes = arrayBuffer;
      this.pendingFileExt = ext;
      try {
        audioFilePath = await rustBridge.saveUploadToTemp(new Uint8Array(arrayBuffer), ext);
        console.log('[AudioEngine] Saved to temp:', audioFilePath);
      } catch (e) {
        console.warn('[AudioEngine] Failed to save to temp:', e);
      }
    } else {
      this.pendingFileBytes = null;
      this.pendingFileExt = '';
    }

    // Rust decode → PCM samples → create AudioBuffer
    if (audioFilePath) {
      try {
        console.log('[AudioEngine] Rust decode from path:', audioFilePath);
        const result = await rustBridge.decodeAudioPlayback(audioFilePath);
        console.log('[AudioEngine] Rust decode result:', {
          sampleRate: result.sample_rate,
          channels: result.channels,
          duration: result.duration,
          samplesLength: result.samples.length,
        });
        this.audioBuffer = this.createAudioBufferFromSamples(result.samples, result.sample_rate, result.channels);
      } catch (e) {
        console.error('[AudioEngine] Rust audio decode FAILED:', e);
      }
    }

    // Extract metadata
    const fileNameWithoutExt = file.name.replace(/\.[^/.]+$/, '');
    const parts = fileNameWithoutExt.split(' - ');
    const title = parts.length > 1 ? parts.slice(1).join(' - ') : fileNameWithoutExt;
    const artist = parts.length > 1 ? parts[0] : 'Unknown Artist';

    this.songMeta = {
      fileName: file.name,
      title: title || 'Untitled Track',
      artist: artist || 'Unknown Artist',
      duration: this.audioBuffer ? this.audioBuffer.duration : 0,
      audioUrl: audioFilePath,
    };

    this.pausedAt = 0;

    return this.songMeta;
  }

  /**
   * Lazy Rust decode — only called when export needs the full sample data.
   */
  public async ensureRustDecode(): Promise<void> {
    if (this.rustDecoded) return;
    const filePath = this.songMeta?.audioUrl;
    if (!filePath && !this.pendingFileBytes) return;

    let rustPath = filePath || '';

    // If no native path, save uploaded bytes to temp first
    if (!rustPath && this.pendingFileBytes) {
      rustPath = await rustBridge.saveUploadToTemp(
        new Uint8Array(this.pendingFileBytes),
        this.pendingFileExt,
      );
      if (this.songMeta) {
        this.songMeta.audioUrl = rustPath;
      }
      this.pendingFileBytes = null;
      this.pendingFileExt = '';
    }

    await rustBridge.decodeAudio(rustPath);
    this.rustDecoded = true;
  }

  public setFftSize(fftSize: number) {
    if (this.analyser) {
      this.analyser.fftSize = fftSize;
    }
  }

  public setSmoothing(smoothing: number) {
    if (this.analyser) {
      this.analyser.smoothingTimeConstant = Math.max(0, Math.min(0.99, smoothing));
    }
  }

  public async play(offsetSeconds?: number) {
    console.log('[AudioEngine] play() called', {
      hasAudioBuffer: !!this.audioBuffer,
      hasAudioCtx: !!this.audioCtx,
      hasAnalyser: !!this.analyser,
      hasGainNode: !!this.gainNode,
      audioCtxState: this.audioCtx?.state,
    });
    if (!this.audioBuffer || !this.audioCtx || !this.analyser || !this.gainNode) {
      console.warn('[AudioEngine] play() aborted - missing dependency');
      return;
    }

    await this.initContext();

    if (this.isPlaying) {
      this.stopSource();
    }

    const startOffset = offsetSeconds !== undefined ? offsetSeconds : this.pausedAt;
    
    // Create new AudioBufferSourceNode
    this.sourceNode = this.audioCtx.createBufferSource();
    this.sourceNode.buffer = this.audioBuffer;

    this.sourceNode.connect(this.analyser);
    this.analyser.connect(this.gainNode);

    this.startTime = this.audioCtx.currentTime - startOffset;
    this.sourceNode.start(0, startOffset);
    this.isPlaying = true;

    this.sourceNode.onended = () => {
      if (this.getCurrentTime() >= this.getDuration() - 0.1) {
        this.isPlaying = false;
        this.pausedAt = 0;
        if (this.onEndedCb) this.onEndedCb();
      }
    };

    this.startProgressTracking();
  }

  public pause() {
    if (!this.isPlaying) return;
    this.pausedAt = this.getCurrentTime();
    this.stopSource();
    this.isPlaying = false;
    this.stopProgressTracking();
  }

  public stop() {
    this.stopSource();
    this.pausedAt = 0;
    this.isPlaying = false;
    this.stopProgressTracking();
    if (this.onTimeUpdateCb) this.onTimeUpdateCb(0);
  }

  public seek(seconds: number) {
    const duration = this.getDuration();
    const target = Math.max(0, Math.min(duration, seconds));
    const wasPlaying = this.isPlaying;

    if (wasPlaying) {
      this.stopSource();
    }

    this.pausedAt = target;

    if (wasPlaying) {
      this.play(target);
    } else {
      if (this.onTimeUpdateCb) this.onTimeUpdateCb(target);
    }
  }

  private stopSource() {
    if (this.sourceNode) {
      try {
        this.sourceNode.stop();
        this.sourceNode.disconnect();
      } catch {
        // Source already stopped
      }
      this.sourceNode = null;
    }
  }

  public getVolume(): number {
    return this.volume;
  }

  public setVolume(val: number) {
    this.volume = Math.max(0, Math.min(1, val));
    if (this.gainNode) {
      this.gainNode.gain.setValueAtTime(this.volume, this.audioCtx?.currentTime || 0);
    }
  }

  public getCurrentTime(): number {
    if (!this.audioBuffer) return 0;
    if (this.isPlaying && this.audioCtx) {
      const elapsed = this.audioCtx.currentTime - this.startTime;
      return Math.min(elapsed, this.audioBuffer.duration);
    }
    return this.pausedAt;
  }

  public getDuration(): number {
    return this.audioBuffer ? this.audioBuffer.duration : 0;
  }

  public getIsPlaying(): boolean {
    return this.isPlaying;
  }

  public getFrequencyData(uint8Array: Uint8Array): Uint8Array {
    if (this.analyser) {
      this.analyser.getByteFrequencyData(uint8Array as unknown as Uint8Array<ArrayBuffer>);
    } else {
      uint8Array.fill(0);
    }
    return uint8Array;
  }

  public getTimeDomainData(uint8Array: Uint8Array): Uint8Array {
    if (this.analyser) {
      this.analyser.getByteTimeDomainData(uint8Array as unknown as Uint8Array<ArrayBuffer>);
    } else {
      uint8Array.fill(128);
    }
    return uint8Array;
  }

  public getAudioStreamDestination(): MediaStreamAudioDestinationNode | null {
    return this.mediaDestination;
  }

  public getAudioBuffer(): AudioBuffer | null {
    return this.audioBuffer;
  }

  public getSongFilePath(): string | null {
    return this.songMeta?.audioUrl || null;
  }

  public setTimeUpdateCallback(cb: (time: number) => void) {
    this.onTimeUpdateCb = cb;
  }

  public setEndedCallback(cb: () => void) {
    this.onEndedCb = cb;
  }

  private startProgressTracking() {
    this.stopProgressTracking();
    const update = () => {
      if (this.isPlaying && this.onTimeUpdateCb) {
        this.onTimeUpdateCb(this.getCurrentTime());
        this.animationFrameId = requestAnimationFrame(update);
      }
    };
    this.animationFrameId = requestAnimationFrame(update);
  }

  private stopProgressTracking() {
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
  }
}

export const audioEngine = new AudioEngine();
