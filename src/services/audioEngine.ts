import { SongMetadata } from '../types/visualizer';
import { rustBridge } from './rustBridge';
import { listen } from '@tauri-apps/api/event';
import { resetVisualizerState } from './renderers/resetState';

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

  private async loadAudioBufferFromFile(filePath: string): Promise<AudioBuffer | null> {
    if (!this.audioCtx) return null;
    try {
      const b64 = await rustBridge.readFileB64(filePath);
      const binaryStr = atob(b64);
      const bytes = new Uint8Array(binaryStr.length);
      for (let i = 0; i < binaryStr.length; i++) {
        bytes[i] = binaryStr.charCodeAt(i);
      }
      const arrayBuffer = bytes.buffer.slice(0, bytes.byteLength) as ArrayBuffer;
      return await this.audioCtx.decodeAudioData(arrayBuffer);
    } catch (e) {
      console.warn('[AudioEngine] Failed to load audio from file, falling back to chunk loading:', e);
      return null;
    }
  }

  private async createAudioBufferFromChunks(sampleRate: number, totalFrames: number): Promise<AudioBuffer | null> {
    if (!this.audioCtx || totalFrames === 0) return null;

    try {
      return await this.createAudioBufferFromPcmFile(sampleRate, totalFrames);
    } catch (e) {
      console.warn('[AudioEngine] PCM file approach failed, falling back to legacy chunk loading:', e);
    }

    let buffer: AudioBuffer;
    try {
      buffer = this.audioCtx.createBuffer(1, totalFrames, sampleRate);
    } catch (e) {
      console.error(`[AudioEngine] Failed to create buffer (${totalFrames} frames, ${sampleRate}Hz):`, e);
      return null;
    }
    const channelData = buffer.getChannelData(0);

    const CHUNK_SEC = 0.5;
    let offset = 0;
    let failedChunks = 0;
    const durationSec = totalFrames / sampleRate;
    for (let t = 0; offset < totalFrames; t += CHUNK_SEC) {
      let b64: string;
      try {
        b64 = await rustBridge.getAudioChunkB64(t, CHUNK_SEC);
      } catch (e) {
        console.warn(`[AudioEngine] Chunk at ${t.toFixed(1)}s failed:`, e);
        failedChunks++;
        if (failedChunks >= 3) break;
        continue;
      }
      if (!b64) {
        console.warn(`[AudioEngine] Empty chunk at ${t.toFixed(1)}s — end of audio (loaded ${(offset / sampleRate).toFixed(1)}s / ${durationSec.toFixed(1)}s)`);
        break;
      }

      try {
        const binaryStr = atob(b64);
        const bytes = new Uint8Array(binaryStr.length);
        for (let i = 0; i < binaryStr.length; i++) {
          bytes[i] = binaryStr.charCodeAt(i);
        }
        const float32 = new Float32Array(bytes.buffer, 0, bytes.byteLength >> 2);
        channelData.set(float32, offset);
        offset += float32.length;
      } catch (e) {
        console.warn(`[AudioEngine] Failed to decode chunk at ${t.toFixed(1)}s:`, e);
        failedChunks++;
        if (failedChunks >= 3) break;
        continue;
      }
    }

    if (offset < totalFrames) {
      console.warn(`[AudioEngine] Loaded ${offset}/${totalFrames} samples (${(offset / sampleRate).toFixed(1)}s / ${durationSec.toFixed(1)}s) — audio truncated`);
      try {
        const trimmed = this.audioCtx.createBuffer(1, offset, sampleRate);
        trimmed.getChannelData(0).set(channelData.subarray(0, offset));
        return trimmed;
      } catch (e) {
        console.error('[AudioEngine] Failed to create trimmed buffer:', e);
        return null;
      }
    }

    return buffer;
  }

  private async createAudioBufferFromPcmFile(sampleRate: number, totalFrames: number): Promise<AudioBuffer | null> {
    if (!this.audioCtx || totalFrames === 0) return null;

    const pcmPath = await rustBridge.getPcmFilePath();
    let cleanupPath = pcmPath;

    try {
      const totalBytes = totalFrames * 4;
      const CHUNK_BYTES = 4 * 1024 * 1024;
      const allBytes = new Uint8Array(totalBytes);
      let offset = 0;

      while (offset < totalBytes) {
        const length = Math.min(CHUNK_BYTES, totalBytes - offset);
        const b64 = await rustBridge.readFileRangeB64(pcmPath, offset, length);
        if (!b64) break;
        const binaryStr = atob(b64);
        for (let i = 0; i < binaryStr.length; i++) {
          allBytes[offset + i] = binaryStr.charCodeAt(i);
        }
        offset += binaryStr.length;
      }

      if (offset === 0) return null;

      const actualFrames = offset >> 2;
      const float32 = new Float32Array(allBytes.buffer, 0, actualFrames);
      const buffer = this.audioCtx.createBuffer(1, actualFrames, sampleRate);
      buffer.getChannelData(0).set(float32);
      return buffer;
    } finally {
      rustBridge.deleteFile(cleanupPath).catch(() => {});
    }
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
      this.rustDecoded = true;
      this.audioBuffer = await this.loadAudioBufferFromFile(filePath);
      if (!this.audioBuffer) {
        this.audioBuffer = await this.createAudioBufferFromChunks(result.sample_rate, result.samples_count);
      }
      if (this.audioBuffer) {
        const loadedSec = this.audioBuffer.duration;
        console.log('[AudioEngine] AudioBuffer created:', {
          duration: loadedSec,
          length: this.audioBuffer.length,
          fullDuration: fullDuration,
        });
        if (fullDuration > 0 && loadedSec < fullDuration - 0.5) {
          console.warn(
            `[AudioEngine] AudioBuffer truncated: ${loadedSec.toFixed(1)}s loaded vs ${fullDuration.toFixed(1)}s full. ` +
            `Chunk loading failed after ~${loadedSec.toFixed(1)}s. Export methods needing JS AudioBuffer will be limited.`
          );
        }
      } else {
        console.log('[AudioEngine] AudioBuffer creation FAILED');
      }
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
        this.rustDecoded = true;
        this.audioBuffer = await this.loadAudioBufferFromFile(audioFilePath);
        if (!this.audioBuffer) {
          this.audioBuffer = await this.createAudioBufferFromChunks(result.sample_rate, result.samples_count);
        }
        if (this.audioBuffer) {
          const loadedSec = this.audioBuffer.duration;
          console.log('[AudioEngine] AudioBuffer created:', {
            duration: loadedSec,
            length: this.audioBuffer.length,
            fullDuration: fullDuration,
          });
          if (fullDuration > 0 && loadedSec < fullDuration - 0.5) {
            console.warn(
              `[AudioEngine] AudioBuffer truncated: ${loadedSec.toFixed(1)}s loaded vs ${fullDuration.toFixed(1)}s full. ` +
              `Chunk loading failed after ~${loadedSec.toFixed(1)}s.`
            );
          }
        } else {
          console.log('[AudioEngine] AudioBuffer creation FAILED');
        }
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

    if (rustPath) {
      try {
        await rustBridge.decodeAudio(rustPath);
        this.rustDecoded = true;
      } catch (e) {
        console.warn('[AudioEngine] ensureRustDecode warning:', e);
      }
    }
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
    if (startOffset === 0) {
      resetVisualizerState();
    }
    
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

  public getFullDuration(): number {
    return this.songMeta?.duration || this.getDuration();
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
