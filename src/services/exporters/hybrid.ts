import { VisualizerConfig } from '../../types/visualizer';
import { audioEngine } from '../audioEngine';
import { rustBridge } from '../rustBridge';
import { CanvasRenderer } from '../canvasRenderer';
import { resetVisualizerState } from '../renderers/resetState';
import { tempDir } from '@tauri-apps/api/path';
import { listen } from '@tauri-apps/api/event';
import { ExportProgress, getExportDimensions } from './types';

export function canUseGpuExport(config: VisualizerConfig): boolean {
  return config.export?.renderEngine === 'gpu';
}

interface GpuExportProgressPayload {
  percent: number;
  current_frame: number;
  total_frames: number;
  elapsed_time: number;
}

export async function exportHybrid(
  config: VisualizerConfig,
  includeAudio: boolean,
  onProgress: (progress: ExportProgress) => void,
  isCancelled: () => boolean,
  outputFileName: string = 'visualizer.mp4'
): Promise<string> {
  const tmpDir = await tempDir();
  const separator = tmpDir.includes('\\') ? '\\' : '/';
  const outputPath = `${tmpDir}${separator}${outputFileName}`;
  const startTime = Date.now();

  const duration = audioEngine.getDuration();
  const fps = config.export?.fps || 60;
  const totalFrames = Math.ceil(duration * fps);

  if (canUseGpuExport(config)) {
    try {
      return await exportGpuPath(config, includeAudio, onProgress, isCancelled, outputPath, totalFrames);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      if (!msg.includes('GPU unavailable')) {
        try {
          await rustBridge.deleteFile(outputPath);
        } catch {
          /* Best-effort */
        }
        throw err;
      }
      // Fallback to Canvas path if GPU fails
      onProgress({ status: 'preparing', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });
    }
  }

  return await exportCanvasPath(config, includeAudio, onProgress, isCancelled, outputPath, totalFrames, startTime);
}

async function exportGpuPath(
  config: VisualizerConfig,
  includeAudio: boolean,
  onProgress: (progress: ExportProgress) => void,
  isCancelled: () => boolean,
  outputPath: string,
  totalFrames: number
): Promise<string> {
  onProgress({ status: 'recording', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

  let unlisten: (() => void) | null = null;
  try {
    unlisten = await listen<GpuExportProgressPayload>('gpu-export-progress', event => {
      const p = event.payload;
      onProgress({
        status: 'recording',
        progress: p.percent,
        currentFrame: p.current_frame,
        totalFrames: p.total_frames || totalFrames,
        elapsedTime: p.elapsed_time,
      });
    });
  } catch {
    /* Progress events best-effort */
  }

  try {
    const audioFilePath = audioEngine.getSongFilePath();
    if (!audioFilePath) throw new Error('No audio file path available');

    const exportPromise = rustBridge.exportGpu(config, audioFilePath, outputPath, includeAudio);

    const cancelTimer = window.setInterval(() => {
      if (isCancelled()) {
        void rustBridge.cancelGpuExport();
      }
    }, 200);

    try {
      return await exportPromise;
    } finally {
      window.clearInterval(cancelTimer);
    }
  } finally {
    if (unlisten) unlisten();
  }
}

async function exportCanvasPath(
  config: VisualizerConfig,
  includeAudio: boolean,
  onProgress: (progress: ExportProgress) => void,
  isCancelled: () => boolean,
  outputPath: string,
  totalFrames: number,
  startTime: number
): Promise<string> {
  onProgress({ status: 'preparing', progress: 0, currentFrame: 0, totalFrames: 0, elapsedTime: 0 });

  const audioFilePath = audioEngine.getSongFilePath();
  if (!audioFilePath) {
    throw new Error('No audio file path available');
  }

  const fps = config.export?.fps || 60;
  const { width, height } = getExportDimensions(config);

  const exportCanvas = document.createElement('canvas');
  exportCanvas.width = width;
  exportCanvas.height = height;

  const exportCtx = exportCanvas.getContext('2d', { alpha: false, desynchronized: true });
  if (!exportCtx) {
    throw new Error('Failed to create 2D rendering context');
  }

  const renderer = new CanvasRenderer();
  renderer.init(exportCanvas);
  resetVisualizerState();
  await renderer.preloadImages();

  const barCount = config.reactivity.barCount;
  const fftSize = config.reactivity.fftSize;
  const bassMultiplier = config.reactivity.bassMultiplier;
  const smoothing = config.reactivity.smoothing;
  let sessionStarted = false;

  try {
    if (isCancelled()) throw new Error('Export cancelled');
    onProgress({ status: 'rendering', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

    await rustBridge.startExportSession(fps, width, height, outputPath, audioFilePath, includeAudio, config.export?.encoder || 'auto');
    sessionStarted = true;

    for (let frame = 0; frame < totalFrames; frame++) {
      if (isCancelled()) throw new Error('Export cancelled');

      const timeSec = frame / fps;
      const rustResult = await rustBridge.computeSpectrum(timeSec, barCount, fftSize, smoothing, bassMultiplier);

      const freqData = new Uint8Array(rustResult.freq_data);
      const timeData = new Uint8Array(rustResult.time_data);

      renderer.setExportData(freqData, timeData, rustResult.bass_energy);
      renderer.setFrameTime(timeSec);
      renderer.drawFrame(config);

      const imageData = exportCtx.getImageData(0, 0, width, height);
      await rustBridge.writeFrameRgba(width, height, new Uint8Array(imageData.data.buffer));

      if (frame % 10 === 0) {
        await new Promise(resolve => setTimeout(resolve, 0));
      }

      const pct = Math.min(100, ((frame + 1) * 100) / totalFrames);
      if (frame % 3 === 0 || frame === totalFrames - 1) {
        onProgress({
          status: 'rendering',
          progress: pct,
          currentFrame: frame + 1,
          totalFrames,
          elapsedTime: (Date.now() - startTime) / 1000,
        });
      }
    }

    if (sessionStarted) {
      onProgress({ status: 'muxing', progress: 99, currentFrame: totalFrames, totalFrames, elapsedTime: (Date.now() - startTime) / 1000 });
      await rustBridge.finishExportSession();
      sessionStarted = false;
    }
  } catch (err) {
    if (sessionStarted) {
      try {
        await rustBridge.finishExportSession();
      } catch {
        /* best effort */
      }
    }
    try {
      await rustBridge.deleteFile(outputPath);
    } catch {
      /* best effort */
    }
    throw err;
  }

  const totalElapsed = (Date.now() - startTime) / 1000;
  onProgress({ status: 'completed', progress: 100, currentFrame: totalFrames, totalFrames, elapsedTime: totalElapsed });

  return outputPath;
}
