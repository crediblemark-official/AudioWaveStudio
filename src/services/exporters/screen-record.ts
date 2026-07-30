import { VisualizerConfig } from '../../types/visualizer';
import { audioEngine } from '../audioEngine';
import { rustBridge } from '../rustBridge';
import { tempDir } from '@tauri-apps/api/path';
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
  const tmpCanvas = document.createElement('canvas');
  tmpCanvas.width = width;
  tmpCanvas.height = height;
  const tmpCtx = tmpCanvas.getContext('2d');
  if (!tmpCtx) throw new Error('Cannot get temporary canvas 2D context');

  const captureCtx = sourceCanvas.getContext('2d');
  if (!captureCtx) throw new Error('Cannot get canvas 2D context');

  const frameInterval = 1000 / fps;
  let frameCount = 0;
  let sessionStarted = false;

  try {
    onProgress({ status: 'recording', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

    audioEngine.setVolume(0);
    audioEngine.stop();
    await audioEngine.play();

    sourceCanvas.width = width;
    sourceCanvas.height = height;

    await rustBridge.startExportSession(fps, width, height, outputPath, audioFilePath, includeAudio);
    sessionStarted = true;

    while (!isCancelled() && frameCount < totalFrames) {
      const wallElapsed = (Date.now() - startTime) / 1000;
      if (wallElapsed > duration + 2.0) break;

      const frameStart = Date.now();

      const imageData = captureCtx.getImageData(0, 0, width, height);
      tmpCtx.putImageData(imageData, 0, 0);
      const jpegBlob = await new Promise<Blob>((resolve, reject) => {
        tmpCanvas.toBlob(
          (blob) => { if (blob) resolve(blob); else reject(new Error('JPEG encode failed')); },
          'image/jpeg', 0.95,
        );
      });
      const bytes = new Uint8Array(await jpegBlob.arrayBuffer());

      if (isCancelled()) break;
      await rustBridge.writeFrame(bytes);
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
    throw err;
  } finally {
    sourceCanvas.width = origWidth;
    sourceCanvas.height = origHeight;
    audioEngine.stop();
    audioEngine.setVolume(prevVolume);
  }
}
