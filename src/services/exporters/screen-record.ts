import { VisualizerConfig } from '../../types/visualizer';
import { audioEngine } from '../audioEngine';
import { rustBridge } from '../rustBridge';
import { tempDir } from '@tauri-apps/api/path';
import { resetVisualizerState } from '../renderers/resetState';
import { ExportProgress, getExportDimensions } from './types';

export async function exportScreenRecord(
  sourceCanvas: HTMLCanvasElement,
  config: VisualizerConfig,
  includeAudio: boolean,
  onProgress: (progress: ExportProgress) => void,
  isCancelled: () => boolean,
): Promise<string> {
  const duration = audioEngine.getFullDuration();
  if (duration <= 0) throw new Error('No audio track loaded or duration is 0');

  const audioFilePath = audioEngine.getSongFilePath();
  if (!audioFilePath) throw new Error('No audio file path available');

  onProgress({ status: 'preparing', progress: 0, currentFrame: 0, totalFrames: 0, elapsedTime: 0 });

  if (isCancelled()) throw new Error('Export cancelled');
  await audioEngine.ensureRustDecode();

  const { width, height } = getExportDimensions(config);
  const fps = config.export.fps || 60;
  const totalFrames = Math.ceil(duration * fps);
  const outputFileName = `${(config.text.songTitle || 'visualizer').replace(/[^a-zA-Z0-9]/g, '_')}_wave.mp4`;
  const tmpDir = await tempDir();
  const separator = tmpDir.endsWith('/') || tmpDir.endsWith('\\') ? '' : '/';
  const outputPath = `${tmpDir}${separator}${outputFileName}`;
  const startTime = Date.now();

  const audioBufferDuration = audioEngine.getDuration();
  if (audioBufferDuration < duration - 0.5) {
    console.warn(
      `[Export] AudioBuffer truncated (${audioBufferDuration.toFixed(1)}s vs full ${duration.toFixed(1)}s). ` +
      `Visualizer may freeze after audio ends. Use "hybrid" or "offscreen" method for correct full-length output.`,
    );
  }

  const prevVolume = audioEngine.getVolume();
  const origWidth = sourceCanvas.width;
  const origHeight = sourceCanvas.height;

  const captureCtx = sourceCanvas.getContext('2d');
  if (!captureCtx) throw new Error('Cannot get canvas 2D context');

  const frameInterval = 1000 / fps;
  let frameCount = 0;
  let sessionStarted = false;

  try {
    if (isCancelled()) throw new Error('Export cancelled');
    onProgress({ status: 'recording', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

    audioEngine.setVolume(0);
    audioEngine.stop();
    resetVisualizerState();
    await audioEngine.play();

    sourceCanvas.width = width;
    sourceCanvas.height = height;

    await rustBridge.startExportSession(fps, width, height, outputPath, audioFilePath, includeAudio, config.export.encoder || 'auto');
    sessionStarted = true;

    while (!isCancelled() && frameCount < totalFrames) {
      const frameStart = Date.now();

      const imageData = captureCtx.getImageData(0, 0, width, height);

      if (isCancelled()) break;
      await rustBridge.writeFrameRgba(width, height, new Uint8Array(imageData.data.buffer));
      frameCount++;

      const pct = Math.min(100, (frameCount * 100) / totalFrames);
      if (frameCount % 3 === 0 || frameCount >= totalFrames) {
        onProgress({
          status: 'recording',
          progress: pct,
          currentFrame: frameCount,
          totalFrames,
          elapsedTime: (Date.now() - startTime) / 1000,
        });
      }

      const frameDuration = Date.now() - frameStart;
      const waitTime = Math.max(0, frameInterval - frameDuration);
      if (waitTime > 0) await new Promise((r) => setTimeout(r, waitTime));
    }

    if (sessionStarted) {
      await rustBridge.finishExportSession();
      sessionStarted = false;
    }

    if (isCancelled()) throw new Error('Export cancelled');

    return outputPath;
  } catch (err: unknown) {
    if (sessionStarted) {
      try { await rustBridge.finishExportSession(); } catch { /* FFmpeg already gone */ }
      sessionStarted = false;
    }
    try { await rustBridge.deleteFile(outputPath); } catch { /* Temp cleanup is best-effort */ }
    throw err;
  } finally {
    sourceCanvas.width = origWidth;
    sourceCanvas.height = origHeight;
    audioEngine.stop();
    audioEngine.setVolume(prevVolume);
  }
}
