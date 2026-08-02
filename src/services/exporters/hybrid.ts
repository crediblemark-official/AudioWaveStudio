import { VisualizerConfig } from '../../types/visualizer';
import { audioEngine } from '../audioEngine';
import { rustBridge } from '../rustBridge';
import { CanvasRenderer } from '../canvasRenderer';
import { resetVisualizerState } from '../renderers/resetState';
import { tempDir } from '@tauri-apps/api/path';
import { listen } from '@tauri-apps/api/event';
import { ExportProgress, getExportDimensions } from './types';

// Styles fully ported to the Rust wgpu renderer (Phase 4/5). Everything else
// (background images/effects, particles/notes, screen effects) falls back to
// the canvas exporter.
const GPU_STYLES = new Set([
  'spectrum',
  'radial',
  'oscilloscope',
  'equalizer',
  'minimal',
  'waveformFill',
  'circularBars',
  'smoothSpectrum',
  'pulseRings',
  'vuMeter',
  'auroraWave',
  'flameFire',
  'spiralGalaxy',
  'threeD',
  'api3D',
  'neonCity3D',
  'speaker3D',
  'speakerTrio',
  'speakerSplatter',
]);

// Screen effects reproducible in the GPU export path. The single-pass mesh
// pipeline covers shake/vignette/pulse/spotlight/strobe/scanline/hueShift;
// the rest run through the Rust post-processing pass in gpu_export.rs.
const GPU_SCREEN_EFFECTS = new Set([
  'shake',
  'vignette',
  'pulse',
  'spotlight',
  'strobe',
  'scanline',
  'hueShift',
  'glitch',
  'chromatic',
  'zoom',
  'invert',
  'bars',
  'shockwave',
  'pixelate',
  'tilt',
  'heatHaze',
]);

export function canUseGpuExport(config: VisualizerConfig): boolean {
  if (!GPU_STYLES.has(config.style)) return false;
  if (config.screenEffects.enabled && !GPU_SCREEN_EFFECTS.has(config.screenEffects.mainEffect)) return false;
  return true;
}

interface GpuExportProgressPayload {
  percent: number;
  current_frame: number;
  total_frames: number;
  elapsed_time: number;
}

async function exportGpuPath(
  config: VisualizerConfig,
  includeAudio: boolean,
  onProgress: (progress: ExportProgress) => void,
  isCancelled: () => boolean,
  outputPath: string,
  totalFrames: number,
): Promise<string> {
  onProgress({ status: 'recording', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

  let unlisten: (() => void) | null = null;
  try {
    unlisten = await listen<GpuExportProgressPayload>('gpu-export-progress', (event) => {
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
    // Progress events are best-effort; the invoke result still resolves.
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

  if (canUseGpuExport(config)) {
    try {
      return await exportGpuPath(config, includeAudio, onProgress, isCancelled, outputPath, totalFrames);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      if (!msg.includes('GPU unavailable')) {
        try { await rustBridge.deleteFile(outputPath); } catch { /* Best-effort */ }
        throw err;
      }
      // GPU unavailable -> fall through to the canvas exporter.
      onProgress({ status: 'preparing', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });
    }
  }

  const offscreen = document.createElement('canvas');
  offscreen.width = width;
  offscreen.height = height;
  const offCtx = offscreen.getContext('2d', { alpha: false });
  if (!offCtx) throw new Error('Failed to get offscreen canvas 2D context');

  resetVisualizerState();

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
  const binsPerFrame = fftSize / 2;

  onProgress({ status: 'preparing', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

  if (isCancelled()) throw new Error('Export cancelled');

  // Precompute all spectra up front (a handful of IPC calls instead of one per
  // frame). Chunks keep each call's payload at the old safe threshold, and the
  // Rust side carries the smoothing state across chunks.
  const perFramePayload = binsPerFrame * 7 + 7;
  const chunkFrames = Math.max(1, Math.floor(7_000_000 / perFramePayload));
  const freqChunks: Uint8Array[] = [];
  const timeChunks: Uint8Array[] = [];
  const bassEnergies: number[] = [];
  for (let start = 0; start < totalFrames; start += chunkFrames) {
    const count = Math.min(chunkFrames, totalFrames - start);
    const chunk = await rustBridge.precomputeSpectra(fps, start, count, barCount, fftSize, smoothing, bassMultiplier);
    freqChunks.push(new Uint8Array(chunk.freq_data_all));
    timeChunks.push(new Uint8Array(chunk.time_data_all));
    bassEnergies.push(...chunk.bass_energies);
  }
  const freqAll = concatUint8Arrays(freqChunks);
  const timeAll = concatUint8Arrays(timeChunks);

  let sessionStarted = false;

  try {
    if (isCancelled()) throw new Error('Export cancelled');
    onProgress({ status: 'recording', progress: 0, currentFrame: 0, totalFrames, elapsedTime: 0 });

    await rustBridge.startExportSession(fps, width, height, outputPath, audioFilePath, includeAudio);
    sessionStarted = true;

    for (let frame = 0; frame < totalFrames; frame++) {
      if (isCancelled()) throw new Error('Export cancelled');

      const offset = frame * binsPerFrame;
      const freqData = freqAll.subarray(offset, offset + binsPerFrame);
      const timeData = timeAll.subarray(offset, offset + binsPerFrame);
      const bassEnergy = bassEnergies[frame] ?? 0;

      renderer.setExportData(freqData, timeData, bassEnergy);
      renderer.setFrameTime(frame / fps);
      renderer.drawFrame(config);

      const imageData = offCtx.getImageData(0, 0, width, height);
      await rustBridge.writeFrameRgba(width, height, new Uint8Array(imageData.data.buffer));

      if (frame % 30 === 0) {
        await new Promise((resolve) => requestAnimationFrame(resolve));
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
    try { await rustBridge.deleteFile(outputPath); } catch { /* Temp cleanup is best-effort */ }
    throw err;
  }
}

function concatUint8Arrays(chunks: Uint8Array[]): Uint8Array {
  const total = chunks.reduce((n, c) => n + c.length, 0);
  const out = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}
