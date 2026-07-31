import { VisualizerConfig } from '../../types/visualizer';
import { audioEngine } from '../audioEngine';
import { rustBridge } from '../rustBridge';
import { CanvasRenderer } from '../canvasRenderer';
import { tempDir } from '@tauri-apps/api/path';
import { ExportProgress, getExportDimensions } from './types';

export async function exportHybrid(
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

  const offscreen = document.createElement('canvas');
  offscreen.width = width;
  offscreen.height = height;
  const offCtx = offscreen.getContext('2d', { alpha: false });
  if (!offCtx) throw new Error('Failed to get offscreen canvas 2D context');

  const renderer = new CanvasRenderer();
  renderer.init(offscreen);

  if (config.background.customImageUri) renderer.setCustomBackgroundImage(config.background.customImageUri);
  if (config.background.radialCenterImageUri) renderer.setRadialCenterImage(config.background.radialCenterImageUri);
  
  if (isCancelled()) throw new Error('Export cancelled');
  await renderer.preloadImages();

  const barCount = config.reactivity.barCount;
  const fftSize = config.reactivity.fftSize;
  const bassMultiplier = config.reactivity.bassMultiplier;
  const smoothing = config.reactivity.smoothing;

  onProgress({ status: 'preparing', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

  if (isCancelled()) throw new Error('Export cancelled');

  const estimatedPayload = totalFrames * (barCount * 7 + 7);
  const useBatchSpectra = estimatedPayload < 7_000_000;

  let freqAll: Uint8Array | null = null;
  let timeAll: Uint8Array | null = null;
  let bassEnergies: number[] = [];

  if (useBatchSpectra) {
    const spectra = await rustBridge.precomputeSpectra(fps, totalFrames, barCount, fftSize, smoothing, bassMultiplier);
    freqAll = new Uint8Array(spectra.freq_data_all);
    timeAll = new Uint8Array(spectra.time_data_all);
    bassEnergies = spectra.bass_energies;
  }

  let rotationAngle = 0;
  let sessionStarted = false;

  try {
    if (isCancelled()) throw new Error('Export cancelled');
    onProgress({ status: 'recording', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

    await rustBridge.startExportSession(fps, width, height, outputPath, audioFilePath, includeAudio);
    sessionStarted = true;

    for (let frame = 0; frame < totalFrames; frame++) {
      if (isCancelled()) throw new Error('Export cancelled');

      let freqData: Uint8Array;
      let timeData: Uint8Array;
      let bassEnergy: number;

      if (freqAll) {
        const offset = frame * barCount;
        freqData = freqAll.subarray(offset, offset + barCount);
        timeData = timeAll!.subarray(offset, offset + barCount);
        bassEnergy = bassEnergies[frame] ?? 0;
      } else {
        const rustResult = await rustBridge.computeSpectrum(frame / fps, barCount, fftSize, smoothing, bassMultiplier);
        freqData = new Uint8Array(rustResult.freq_data);
        timeData = new Uint8Array(rustResult.time_data);
        bassEnergy = rustResult.bass_energy;
      }

      renderer.setExportData(freqData, timeData, bassEnergy);
      renderer.setFrameTime(frame / fps);
      rotationAngle += 0.003;
      renderer.setRotationAngle(rotationAngle);
      renderer.drawFrame(config);

      const jpegBlob = await new Promise<Blob>((resolve, reject) => {
        offscreen.toBlob(
          (blob) => { if (blob) resolve(blob); else reject(new Error('JPEG encode failed')); },
          'image/jpeg', 0.95,
        );
      });
      const bytes = new Uint8Array(await jpegBlob.arrayBuffer());
      await rustBridge.writeFrame(bytes);

      if (frame % 10 === 0) {
        await new Promise((resolve) => setTimeout(resolve, 0));
      }

      const pct = Math.min(100, ((frame + 1) * 100) / totalFrames);
      if (frame % 3 === 0 || frame === totalFrames - 1) {
        onProgress({ status: 'recording', progress: pct, currentFrame: frame + 1, totalFrames, elapsedTime: (Date.now() - startTime) / 1000 });
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
    throw err;
  }
}
