import { VisualizerConfig } from '../../types/visualizer';
import { audioEngine } from '../audioEngine';
import { rustBridge } from '../rustBridge';
import { CanvasRenderer } from '../canvasRenderer';
import { resetVisualizerState } from '../renderers/resetState';
import { tempDir } from '@tauri-apps/api/path';
import { ExportProgress, getExportDimensions } from './types';

export async function exportOffscreen(
  config: VisualizerConfig,
  includeAudio: boolean,
  onProgress: (progress: ExportProgress) => void,
  isCancelled: () => boolean,
): Promise<string> {
  const duration = audioEngine.getFullDuration();
  if (duration <= 0) throw new Error('No audio track loaded or duration is 0');

  const audioFilePath = audioEngine.getSongFilePath();
  if (!audioFilePath) throw new Error('No audio file path available');

  const fps = config.export.fps || 60;
  const totalFrames = Math.ceil(duration * fps);

  onProgress({ status: 'preparing', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

  if (isCancelled()) throw new Error('Export cancelled');
  await audioEngine.ensureRustDecode();

  const { width, height } = getExportDimensions(config);
  const outputFileName = `${(config.text.songTitle || 'visualizer').replace(/[^a-zA-Z0-9]/g, '_')}_wave.mp4`;
  const tmpDir = await tempDir();
  const separator = tmpDir.endsWith('/') || tmpDir.endsWith('\\') ? '' : '/';
  const outputPath = `${tmpDir}${separator}${outputFileName}`;
  const startTime = Date.now();

  const offscreen = document.createElement('canvas');
  offscreen.width = width;
  offscreen.height = height;
  const offCtx = offscreen.getContext('2d', { alpha: false });
  if (!offCtx) throw new Error('Failed to get offscreen canvas 2D context');

  const renderer = new CanvasRenderer();
  renderer.init(offscreen);
  resetVisualizerState();

  if (config.background.customImageUri) renderer.setCustomBackgroundImage(config.background.customImageUri);
  if (config.background.radialCenterImageUri) renderer.setRadialCenterImage(config.background.radialCenterImageUri);
  
  if (isCancelled()) throw new Error('Export cancelled');
  await renderer.preloadImages();

  const barCount = config.reactivity.barCount;
  const fftSize = config.reactivity.fftSize;
  const bassMultiplier = config.reactivity.bassMultiplier;
  const smoothing = config.reactivity.smoothing;
  let rotationAngle = 0;
  let sessionStarted = false;

  try {
    if (isCancelled()) throw new Error('Export cancelled');
    onProgress({ status: 'rendering', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

    await rustBridge.startExportSession(fps, width, height, outputPath, audioFilePath, includeAudio);
    sessionStarted = true;

    for (let frame = 0; frame < totalFrames; frame++) {
      if (isCancelled()) throw new Error('Export cancelled');

      const timeSec = frame / fps;
      const rustResult = await rustBridge.computeSpectrum(timeSec, barCount, fftSize, smoothing, bassMultiplier);

      const freqData = new Uint8Array(rustResult.freq_data);
      const timeData = new Uint8Array(rustResult.time_data);

      rotationAngle += 0.003;
      renderer.setExportData(freqData, timeData, rustResult.bass_energy);
      renderer.setFrameTime(timeSec);
      renderer.setRotationAngle(rotationAngle);
      renderer.drawFrame(config);

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

    if (sessionStarted) {
      await rustBridge.finishExportSession();
      sessionStarted = false;
    }

    renderer.clearExportData();
    return outputPath;
  } catch (err: unknown) {
    if (sessionStarted) {
      try { await rustBridge.finishExportSession(); } catch { /* FFmpeg already gone */ }
      sessionStarted = false;
    }
    renderer.clearExportData();
    try { await rustBridge.deleteFile(outputPath); } catch { /* Temp cleanup is best-effort */ }
    throw err;
  }
}
