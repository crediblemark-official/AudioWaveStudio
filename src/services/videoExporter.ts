import { VisualizerConfig } from '../types/visualizer';
import { audioEngine } from './audioEngine';
import { rustBridge } from './rustBridge';
import { CanvasRenderer } from './canvasRenderer';
import { save } from '@tauri-apps/plugin-dialog';
import { tempDir } from '@tauri-apps/api/path';

export type ExportMethod = 'screen_recording' | 'frame_by_frame';

export interface ExportProgress {
  status: 'preparing' | 'recording' | 'muxing' | 'rendering' | 'completed' | 'cancelled' | 'error';
  progress: number;
  currentFrame: number;
  totalFrames: number;
  elapsedTime: number;
  errorMessage?: string;
  outputPath?: string;
}

export function getExportDimensions(config: VisualizerConfig): { width: number; height: number } {
  let width = 1920;
  let height = 1080;
  if (config.export.resolution === '720p') { width = 1280; height = 720; }
  else if (config.export.resolution === '4K') { width = 3840; height = 2160; }
  if (config.export.aspectRatio === '9:16') { const t = width; width = height; height = t; }
  else if (config.export.aspectRatio === '1:1') { height = width; }
  return { width, height };
}

export class VideoExporter {
  private isExporting: boolean = false;
  private animFrameId: number | null = null;

  public getIsExporting(): boolean {
    return this.isExporting;
  }

  public cancelExport() {
    this.isExporting = false;
    if (this.animFrameId !== null) {
      clearInterval(this.animFrameId);
      this.animFrameId = null;
    }
  }

  public async exportToVideo(
    sourceCanvas: HTMLCanvasElement,
    config: VisualizerConfig,
    includeAudio: boolean,
    method: ExportMethod,
    onProgress: (progress: ExportProgress) => void
  ): Promise<Blob> {
    if (method === 'screen_recording') {
      return this.exportViaScreenRecording(sourceCanvas, config, includeAudio, onProgress);
    } else {
      return this.exportViaOffscreenCanvas(sourceCanvas, config, includeAudio, onProgress);
    }
  }

  // ─── METHOD 1: Screen Recording (real-time capture from live canvas) ──
  // Streams frames directly to FFmpeg without buffering in memory
  private async exportViaScreenRecording(
    sourceCanvas: HTMLCanvasElement,
    config: VisualizerConfig,
    includeAudio: boolean,
    onProgress: (progress: ExportProgress) => void
  ): Promise<Blob> {
    if (this.isExporting) throw new Error('An export is already in progress');

    const duration = audioEngine.getDuration();
    if (duration <= 0) throw new Error('No audio track loaded or duration is 0');

    const audioFilePath = audioEngine.getSongFilePath();
    if (!audioFilePath) throw new Error('No audio file path available');

    await audioEngine.ensureRustDecode();
    this.isExporting = true;

    const { width, height } = getExportDimensions(config);
    const fps = config.export.fps || 60;
    const totalFrames = Math.ceil(duration * fps);
    const outputFileName = `${(config.text.songTitle || 'visualizer').replace(/[^a-zA-Z0-9]/g, '_')}_wave.mp4`;
    const tmpDir = await tempDir();
    const separator = tmpDir.endsWith('/') || tmpDir.endsWith('\\') ? '' : '/';
    const outputPath = `${tmpDir}${separator}${outputFileName}`;
    const startTime = Date.now();

    try {
      onProgress({ status: 'recording', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

      const prevVolume = audioEngine.getVolume();
      audioEngine.setVolume(0);
      await audioEngine.play();

      const origWidth = sourceCanvas.width;
      const origHeight = sourceCanvas.height;
      sourceCanvas.width = width;
      sourceCanvas.height = height;

      const captureCtx = sourceCanvas.getContext('2d');
      if (!captureCtx) throw new Error('Cannot get canvas 2D context');

      const frameInterval = 1000 / fps;
      let frameCount = 0;

      // Start FFmpeg session FIRST, then stream frames as they're captured
      await rustBridge.startExportSession(fps, width, height, outputPath, audioFilePath, includeAudio);

      while (this.isExporting && frameCount < totalFrames) {
        const frameStart = Date.now();

        const imageData = captureCtx.getImageData(0, 0, width, height);
        const tmpCanvas = document.createElement('canvas');
        tmpCanvas.width = width;
        tmpCanvas.height = height;
        const tmpCtx = tmpCanvas.getContext('2d');
        if (!tmpCtx) throw new Error('Cannot get temporary canvas context');
        tmpCtx.putImageData(imageData, 0, 0);
        const jpegBlob = await new Promise<Blob>((resolve, reject) => {
          tmpCanvas.toBlob(
            (blob) => { if (blob) resolve(blob); else reject(new Error('JPEG encode failed')); },
            'image/jpeg', 0.95
          );
        });
        const bytes = new Uint8Array(await jpegBlob.arrayBuffer());

        // Stream to FFmpeg immediately
        await rustBridge.writeFrame(bytes);
        frameCount++;

        const pct = Math.min(100, ((frameCount) * 100) / totalFrames);
        if (frameCount % 3 === 0 || frameCount >= totalFrames) {
          onProgress({
            status: 'recording',
            progress: pct,
            currentFrame: frameCount,
            totalFrames,
            elapsedTime: (Date.now() - startTime) / 1000,
          });
        }

        // Frame limiter: wait to match target FPS
        const frameDuration = Date.now() - frameStart;
        const waitTime = Math.max(0, frameInterval - frameDuration);
        if (waitTime > 0) await new Promise((r) => setTimeout(r, waitTime));
      }

      sourceCanvas.width = origWidth;
      sourceCanvas.height = origHeight;

      audioEngine.stop();
      audioEngine.setVolume(prevVolume);
      if (!this.isExporting) throw new Error('Export cancelled');

      await rustBridge.finishExportSession();

      this.isExporting = false;
      onProgress({ status: 'completed', progress: 100, currentFrame: frameCount, totalFrames, elapsedTime: (Date.now() - startTime) / 1000, outputPath });
      return new Blob([]);
    } catch (err: unknown) {
      this.isExporting = false;
      audioEngine.stop();
      const msg = err instanceof Error ? err.message : String(err);
      onProgress({ status: 'error', progress: 0, currentFrame: 0, totalFrames: 0, elapsedTime: (Date.now() - startTime) / 1000, errorMessage: `Export Error: ${msg}` });
      throw err;
    }
  }

  // ─── METHOD 2: Frame by Frame (precise, computes spectrum per frame) ──
  private async exportViaOffscreenCanvas(
    _sourceCanvas: HTMLCanvasElement,
    config: VisualizerConfig,
    includeAudio: boolean,
    onProgress: (progress: ExportProgress) => void
  ): Promise<Blob> {
    if (this.isExporting) throw new Error('An export is already in progress');

    const duration = audioEngine.getDuration();
    if (duration <= 0) throw new Error('No audio track loaded or duration is 0');

    const audioFilePath = audioEngine.getSongFilePath();
    if (!audioFilePath) throw new Error('No audio file path available');

    await audioEngine.ensureRustDecode();
    this.isExporting = true;

    const { width, height } = getExportDimensions(config);

    const outputFileName = `${(config.text.songTitle || 'visualizer').replace(/[^a-zA-Z0-9]/g, '_')}_wave.mp4`;
    const tmpDir = await tempDir();
    const separator = tmpDir.endsWith('/') || tmpDir.endsWith('\\') ? '' : '/';
    const outputPath = `${tmpDir}${separator}${outputFileName}`;
    const fps = config.export.fps || 60;
    const totalFrames = Math.ceil(duration * fps);
    const startTime = Date.now();

    const offscreen = document.createElement('canvas');
    offscreen.width = width;
    offscreen.height = height;
    const offCtx = offscreen.getContext('2d', { alpha: false });
    if (!offCtx) throw new Error('Failed to get offscreen canvas 2D context');

    const exportRenderer = new CanvasRenderer();
    exportRenderer.init(offscreen);

    if (config.background.customImageUri) exportRenderer.setCustomBackgroundImage(config.background.customImageUri);
    await exportRenderer.preloadImages();

    try {
      onProgress({ status: 'rendering', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

      await rustBridge.startExportSession(fps, width, height, outputPath, audioFilePath, includeAudio);

      const fftSize = config.reactivity.fftSize;
      const barCount = Math.min(fftSize / 2, 128);
      const bassMultiplier = config.reactivity.bassMultiplier;
      const smoothing = config.reactivity.smoothing;
      let exportBassEnergy = 0;

      let rotationAngle = 0;
      for (let frame = 0; frame < totalFrames; frame++) {
        if (!this.isExporting) throw new Error('Export cancelled');

        const timeSec = frame / fps;
        const rustResult = await rustBridge.computeSpectrum(timeSec, barCount, fftSize, smoothing, bassMultiplier);

        // Rust now returns dB-scaled + smoothed data directly
        const freqData = new Uint8Array(rustResult.freq_data);
        const timeData = new Uint8Array(rustResult.time_data);

        // Bass energy smoothing (kept in JS — single float, cheap)
        exportBassEnergy += (rustResult.bass_energy - exportBassEnergy) * 0.2;

        rotationAngle += 0.003;
        exportRenderer.setExportData(freqData, timeData, exportBassEnergy);
        exportRenderer.setRotationAngle(rotationAngle);
        exportRenderer.drawFrame(config);

        // Get raw pixels and send to Rust for JPEG encoding + pipe to FFmpeg
        const imageData = offCtx.getImageData(0, 0, width, height);
        await rustBridge.writeFrameRgba(width, height, new Uint8Array(imageData.data.buffer));

        if (frame % 10 === 0) {
          await new Promise((resolve) => setTimeout(resolve, 0));
        }

        const pct = Math.min(100, ((frame + 1) * 100) / totalFrames);
        if (frame % 3 === 0 || frame === totalFrames - 1) {
          onProgress({ status: 'rendering', progress: pct, currentFrame: frame + 1, totalFrames, elapsedTime: (Date.now() - startTime) / 1000 });
        }
      }

      await rustBridge.finishExportSession();
      exportRenderer.clearExportData();
      this.isExporting = false;
      onProgress({ status: 'completed', progress: 100, currentFrame: totalFrames, totalFrames, elapsedTime: (Date.now() - startTime) / 1000, outputPath });
      return new Blob([]);
    } catch (err: unknown) {
      this.isExporting = false;
      exportRenderer.clearExportData();
      const msg = err instanceof Error ? err.message : String(err);
      onProgress({ status: 'error', progress: 0, currentFrame: 0, totalFrames: 0, elapsedTime: (Date.now() - startTime) / 1000, errorMessage: `Export Error: ${msg}` });
      throw err;
    }
  }

  public async saveToFile(sourcePath: string, defaultFilename: string): Promise<boolean> {
    const destPath = await save({
      defaultPath: defaultFilename,
      filters: [{ name: 'MP4 Video', extensions: ['mp4'] }],
    });
    if (!destPath) return false;
    await rustBridge.copyFileToPath(sourcePath, destPath);
    try {
      await rustBridge.deleteFile(sourcePath);
    } catch {
      // Temp file cleanup is best-effort
    }
    return true;
  }
}

export const videoExporter = new VideoExporter();
