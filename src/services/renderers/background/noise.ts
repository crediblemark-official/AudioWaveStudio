import { RenderContext } from '../types';

let noiseCanvas: HTMLCanvasElement | null = null;
let noiseCtx: CanvasRenderingContext2D | null = null;
let lastNoiseUpdate = 0;

function getNoisePattern(c: CanvasRenderingContext2D): CanvasPattern | null {
  if (typeof document === 'undefined') return null;
  const now = Date.now();
  if (!noiseCanvas) {
    noiseCanvas = document.createElement('canvas');
    noiseCanvas.width = 128;
    noiseCanvas.height = 128;
    noiseCtx = noiseCanvas.getContext('2d');
  }
  if (noiseCtx && now - lastNoiseUpdate > 60) {
    lastNoiseUpdate = now;
    const imgData = noiseCtx.createImageData(128, 128);
    const data = imgData.data;
    for (let i = 0; i < data.length; i += 4) {
      const val = Math.floor(Math.random() * 255);
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
  const { ctx: c, width, height, config, bassEnergy, beatStrength } = ctx;
  const bg = config.background;
  const baseGrainOpacity = bg.grainOpacity ?? 0.08;
  const alpha = Math.min(1.0, baseGrainOpacity + bassEnergy * 0.08 + beatStrength * 0.06);
  const pattern = getNoisePattern(c);
  if (pattern) {
    c.save();
    c.fillStyle = pattern;
    c.globalAlpha = alpha;
    c.fillRect(0, 0, width, height);
    c.restore();
  }
}
