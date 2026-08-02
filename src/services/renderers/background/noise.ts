import { RenderContext } from '../types';

let noiseCanvas: HTMLCanvasElement | null = null;
let noiseCtx: CanvasRenderingContext2D | null = null;
let lastNoiseSeed = -1;

export function resetNoiseState() {
  lastNoiseSeed = -1;
}

function mulberry32(seed: number) {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6D2B79F5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function getNoisePattern(c: CanvasRenderingContext2D, frameTime: number): CanvasPattern | null {
  if (typeof document === 'undefined') return null;
  const seed = Math.floor(frameTime * 60);
  if (!noiseCanvas) {
    noiseCanvas = document.createElement('canvas');
    noiseCanvas.width = 128;
    noiseCanvas.height = 128;
    noiseCtx = noiseCanvas.getContext('2d');
  }
  if (noiseCtx && seed !== lastNoiseSeed) {
    lastNoiseSeed = seed;
    const rand = mulberry32(seed);
    const imgData = noiseCtx.createImageData(128, 128);
    const data = imgData.data;
    for (let i = 0; i < data.length; i += 4) {
      const val = Math.floor(rand() * 255);
      data[i] = val;
      data[i + 1] = val;
      data[i + 2] = val;
      data[i + 3] = 255;
    }
    noiseCtx.putImageData(imgData, 0, 0);
  }
  return noiseCanvas ? c.createPattern(noiseCanvas, 'repeat') : null;
}

export function renderNoise(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy, beatStrength, frameTime } = ctx;
  const bg = config.background;
  const baseGrainOpacity = bg.grainOpacity ?? 0.08;
  const alpha = Math.min(1.0, baseGrainOpacity + bassEnergy * 0.08 + beatStrength * 0.06);
  const pattern = getNoisePattern(c, frameTime);
  if (pattern) {
    c.save();
    c.fillStyle = pattern;
    c.globalAlpha = alpha;
    c.fillRect(0, 0, width, height);
    c.restore();
  }
}
