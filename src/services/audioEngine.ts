import { SongMetadata } from '../types/visualizer';
import { rustBridge } from './rustBridge';
import { listen } from '@tauri-apps/api/event';

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

  private listenFreqData: Uint8Array | null = null;
  private listenTimeData: Uint8Array | null = null;
  private listenUnlisten: (() => void) | null = null;
  private isListening: boolean = false;

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

      // Connect nodes: sourceNode -> analyser -> gainNode -> destination
      this.analyser.connect(this.gainNode);
      this.gainNode.connect(this.audioCtx.destination);
      this.gainNode.connect(this.mediaDestination);
    }

    if (this.audioCtx.state === 'suspended') {
      await this.audioCtx.resume();
    }
  }

  private async createAudioBufferFromChunks(sampleRate: number, totalFrames: number): Promise<AudioBuffer | null> {
    if (!this.audioCtx || totalFrames === 0) return null;

    const buffer = this.audioCtx.createBuffer(1, totalFrames, sampleRate);
    const channelData = buffer.getChannelData(0);

    // Fetch samples in 2-second chunks to avoid IPC size limits
    const CHUNK_SEC = 2.0;
    let offset = 0;
    let failedChunks = 0;
    for (let t = 0; offset < totalFrames; t += CHUNK_SEC) {
      let b64: string;
      try {
        b64 = await rustBridge.getAudioChunkB64(t, CHUNK_SEC);
      } catch (e) {
        console.warn(`[AudioEngine] Chunk at ${t}s failed:`, e);
        failedChunks++;
        if (failedChunks >= 3) break;
        continue;
      }
      if (!b64) {
        console.warn(`[AudioEngine] Empty chunk at ${t}s — end of audio`);
        break;
      }

      const binaryStr = atob(b64);
      const bytes = new Uint8Array(binaryStr.length);
      for (let i = 0; i < binaryStr.length; i++) {
        bytes[i] = binaryStr.charCodeAt(i);
      }
      const float32 = new Float32Array(bytes.buffer, 0, bytes.byteLength >> 2);
      channelData.set(float32, offset);
      offset += float32.length;
    }

    // Trim buffer if fewer frames were loaded
    if (offset < totalFrames) {
      console.warn(`[AudioEngine] Loaded ${offset}/${totalFrames} samples — audio truncated`);
      const trimmed = this.audioCtx.createBuffer(1, offset, sampleRate);
      trimmed.getChannelData(0).set(channelData.subarray(0, offset));
      return trimmed;
    }

    return buffer;
  }

  public async startListening(_deviceId?: string): Promise<string> {
    if (this.isListening) throw new Error('Already listening');
    this.stop();
    this.listenFreqData = null;
    this.listenTimeData = null;

    const src = await rustBridge.startSystemListen();

    this.listenUnlisten = await listen<{ freq_data: number[]; time_data: number[] }>(
      'listen-freq-data',
      (event) => {
        this.listenFreqData = new Uint8Array(event.payload.freq_data);
        if (event.payload.time_data) {
          this.listenTimeData = new Uint8Array(event.payload.time_data);
        }
      },
    );

    this.isListening = true;
    return src;
  }

  public async stopListening(): Promise<void> {
    if (this.listenUnlisten) {
      this.listenUnlisten();
      this.listenUnlisten = null;
    }
    await rustBridge.stopSystemListen();
    this.isListening = false;
    this.listenFreqData = null;
    this.listenTimeData = null;
  }

  public getIsListening(): boolean {
    return this.isListening;
  }

  public async loadAudioPath(filePath: string): Promise<SongMetadata> {
    console.log('[AudioEngine] loadAudioPath called:', filePath);
    await this.initContext();
    this.stop();
    this.rustDecoded = false;
    this.pendingFileBytes = null;
    this.pendingFileExt = '';

    let fullDuration = 0;

    try {
      console.log('[AudioEngine] Calling Rust decode...');
      const result = await rustBridge.decodeAudioPlayback(filePath);
      console.log('[AudioEngine] Rust decode result:', {
        sampleRate: result.sample_rate,
        channels: result.channels,
        duration: result.duration,
        samplesCount: result.samples_count,
        fullDuration: result.full_duration,
      });
      fullDuration = result.full_duration;
      this.audioBuffer = await this.createAudioBufferFromChunks(result.sample_rate, result.samples_count);
      console.log('[AudioEngine] AudioBuffer created:', this.audioBuffer ? {
        duration: this.audioBuffer.duration,
        length: this.audioBuffer.length,
      } : 'NULL');
    } catch (e) {
      console.error('[AudioEngine] Rust audio decode FAILED:', e);
    }

    const fileName = filePath.split(/[/\\]/).pop() || 'Track';
    const fileNameWithoutExt = fileName.replace(/\.[^/.]+$/, '');
    const parts = fileNameWithoutExt.split(' - ');
    const title = parts.length > 1 ? parts.slice(1).join(' - ') : fileNameWithoutExt;
    const artist = parts.length > 1 ? parts[0] : 'Unknown Artist';

    this.songMeta = {
      fileName,
      title,
      artist,
      duration: fullDuration || (this.audioBuffer ? this.audioBuffer.duration : 0),
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
    let fullDuration = 0;
    if (audioFilePath) {
      try {
        console.log('[AudioEngine] Rust decode from path:', audioFilePath);
        const result = await rustBridge.decodeAudioPlayback(audioFilePath);
        console.log('[AudioEngine] Rust decode result:', {
          sampleRate: result.sample_rate,
          channels: result.channels,
          duration: result.duration,
          samplesCount: result.samples_count,
          fullDuration: result.full_duration,
        });
        fullDuration = result.full_duration;
        this.audioBuffer = await this.createAudioBufferFromChunks(result.sample_rate, result.samples_count);
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
      duration: fullDuration || (this.audioBuffer ? this.audioBuffer.duration : 0),
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
    if (this.isListening && this.listenFreqData) {
      const len = Math.min(uint8Array.length, this.listenFreqData.length);
      uint8Array.set(this.listenFreqData.subarray(0, len));
      if (len < uint8Array.length) {
        uint8Array.fill(0, len);
      }
      return uint8Array;
    }
    if (this.audioBuffer && this.analyser) {
      this.analyser.getByteFrequencyData(uint8Array as unknown as Uint8Array<ArrayBuffer>);
    } else {
      uint8Array.fill(0);
    }
    return uint8Array;
  }

  public getTimeDomainData(uint8Array: Uint8Array): Uint8Array {
    if (this.isListening && this.listenTimeData) {
      const len = Math.min(uint8Array.length, this.listenTimeData.length);
      uint8Array.set(this.listenTimeData.subarray(0, len));
      if (len < uint8Array.length) {
        uint8Array.fill(128, len);
      }
      return uint8Array;
    }
    if (this.audioBuffer && this.analyser) {
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
