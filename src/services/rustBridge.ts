import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { VisualizerConfig } from '../types/visualizer';

export interface AudioMetadataRust {
  duration: number;
  sample_rate: number;
  channels: number;
}

export interface AudioDecodeResult {
  sample_rate: number;
  channels: number;
  duration: number;
  full_duration: number;
  samples_count: number;
}

export interface PrecomputedSpectra {
  freq_data_all: number[];
  time_data_all: number[];
  bass_energies: number[];
  bar_count: number;
}

export interface SpectrumResultRust {
  freq_data: number[];
  time_data: number[];
  bass_energy: number;
}

export interface RenderConfigRust {
  style: string;
  width: number;
  height: number;
  primary_color: [number, number, number, number];
  secondary_color: [number, number, number, number];
  accent_color: [number, number, number, number];
  bg_color: [number, number, number, number];
  bar_count: number;
  sensitivity: number;
  bass_multiplier: number;
  show_particles: boolean;
  title_text?: string;
  artist_text?: string;
  position_x?: number;
  position_y?: number;
}

export function hexToRgba(hex: string): [number, number, number, number] {
  let c = hex.replace('#', '');
  if (c.length === 3) {
    c = c.split('').map((x) => x + x).join('');
  }
  const num = parseInt(c, 16);
  if (isNaN(num)) return [0, 240, 255, 255];
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255, 255];
}

export function convertToRustConfig(config: VisualizerConfig, width = 1280, height = 720): RenderConfigRust {
  return {
    style: config.style,
    width,
    height,
    primary_color: hexToRgba(config.theme.primaryColor),
    secondary_color: hexToRgba(config.theme.secondaryColor),
    accent_color: hexToRgba(config.theme.accentColor),
    bg_color: hexToRgba(config.background.solidColor),
    bar_count: config.reactivity.barCount,
    sensitivity: config.reactivity.sensitivity,
    bass_multiplier: config.reactivity.bassMultiplier,
    show_particles: config.background.showParticles,
    title_text: config.text.showTitle ? config.text.songTitle : undefined,
    artist_text: config.text.showArtist ? config.text.artistName : undefined,
    position_x: config.positionX || 0,
    position_y: config.positionY || 0,
  };
}

export class RustBridge {
  public async decodeAudio(filePath: string): Promise<AudioMetadataRust> {
    return await invoke<AudioMetadataRust>('decode_audio', { filePath });
  }

  public async decodeAudioPlayback(filePath: string): Promise<AudioDecodeResult> {
    return await invoke<AudioDecodeResult>('decode_audio_playback', { filePath });
  }

  public async getAudioChunkB64(startSec: number, durationSec: number): Promise<string> {
    return await invoke<string>('get_audio_chunk_b64', { startSec, durationSec });
  }

  public async readFileBytes(path: string): Promise<Uint8Array> {
    const bytes = await invoke<number[]>('read_file_bytes', { path });
    return new Uint8Array(bytes);
  }

  public async readFileB64(path: string): Promise<string> {
    return await invoke<string>('read_file_b64', { path });
  }

  public async getPcmFilePath(): Promise<string> {
    return await invoke<string>('save_pcm_to_file');
  }

  public async readFileRangeB64(path: string, offset: number, length: number): Promise<string> {
    return await invoke<string>('read_file_range_b64', { path, offset, length });
  }

  public async writeTextFile(path: string, content: string): Promise<void> {
    await invoke('write_text_file', { path, content });
  }

  public async copyFileToPath(source: string, destination: string): Promise<void> {
    await invoke('copy_file_to_path', { source, destination });
  }

  public async deleteFile(path: string): Promise<void> {
    await invoke('delete_file', { path });
  }

  public async precomputeSpectra(
    fps: number,
    startFrame: number,
    numFrames: number,
    barCount: number,
    fftSize: number,
    smoothing: number,
    bassMultiplier: number,
  ): Promise<PrecomputedSpectra> {
    return await invoke<PrecomputedSpectra>('precompute_spectra', {
      fps,
      startFrame,
      numFrames,
      barCount,
      fftSize,
      smoothing,
      bassMultiplier,
    });
  }

  public async computeSpectrum(
    timeSec: number,
    barCount: number,
    fftSize: number,
    smoothing: number,
    bassMultiplier: number,
  ): Promise<SpectrumResultRust> {
    return await invoke<SpectrumResultRust>('compute_spectrum_rust', {
      timeSec,
      barCount,
      fftSize,
      smoothing,
      bassMultiplier,
    });
  }

  public async checkFfmpeg(): Promise<boolean> {
    return await invoke<boolean>('check_ffmpeg');
  }

  public async ffmpegAutoInstallSupported(): Promise<boolean> {
    return await invoke<boolean>('ffmpeg_auto_install_supported');
  }

  public async installFfmpeg(onProgress?: (phase: string) => void): Promise<string> {
    let unlisten: (() => void) | null = null;
    if (onProgress) {
      unlisten = await listen<string>('ffmpeg-install-progress', (event) => {
        onProgress(event.payload);
      });
    }
    try {
      return await invoke<string>('install_ffmpeg');
    } finally {
      if (unlisten) unlisten();
    }
  }

  public async ffmpegDownloadUrl(): Promise<string> {
    return await invoke<string>('ffmpeg_download_url');
  }

  public async saveUploadToTemp(bytes: Uint8Array, ext: string): Promise<string> {
    return await invoke<string>('save_upload_to_temp', { bytes, ext });
  }

  public async startExportSession(
    fps: number,
    width: number,
    height: number,
    outputMp4Path: string,
    audioFilePath: string,
    includeAudio: boolean,
    encoder: string,
  ): Promise<void> {
    await invoke('start_export_session', {
      fps,
      width,
      height,
      outputMp4Path,
      audioFilePath,
      includeAudio,
      encoderPreference: encoder,
    });
  }

  public async writeFrameRgba(width: number, height: number, rgbaData: Uint8Array): Promise<void> {
    await invoke('write_frame_rgba', { width, height, rgbaData });
  }

  public async finishExportSession(): Promise<string> {
    return await invoke<string>('finish_export_session');
  }

  public async exportGpu(
    config: VisualizerConfig,
    audioFilePath: string,
    outputPath: string,
    includeAudio: boolean,
  ): Promise<string> {
    return await invoke<string>('export_gpu', {
      config,
      audioFilePath,
      outputPath,
      includeAudio,
    });
  }

  public async cancelGpuExport(): Promise<void> {
    await invoke('cancel_gpu_export');
  }

  public async renderRustPreviewFrame(
    config: VisualizerConfig,
    freqData: Uint8Array,
    timeData: Uint8Array,
    frameTime: number,
    width: number,
    height: number,
    isPlaying: boolean,
  ): Promise<Uint8Array> {
    const raw = await invoke<number[]>('render_rust_preview_frame', {
      config,
      freqData: Array.from(freqData),
      timeData: Array.from(timeData),
      frameTime,
      width,
      height,
      isPlaying,
    });
    return new Uint8Array(raw);
  }

  public async convertWebmToMp4(
    webmPath: string,
    audioPath: string,
    outputMp4Path: string,
    includeAudio: boolean,
  ): Promise<string> {
    return await invoke<string>('convert_webm_to_mp4', {
      webmPath,
      audioPath,
      outputMp4Path,
      includeAudio,
    });
  }

  public async startSystemListen(): Promise<string> {
    return await invoke<string>('start_system_listen');
  }

  public async stopSystemListen(): Promise<void> {
    await invoke('stop_system_listen');
  }
}

export const rustBridge = new RustBridge();
