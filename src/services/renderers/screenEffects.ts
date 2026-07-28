import { ScreenEffectsSettings } from '../../types/visualizer';

let lastGlitchTime = 0;

export function applyScreenEffects(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
) {
  if (!settings.enabled) return;

  const effect = settings.mainEffect;
  if (effect === 'none') return;

  switch (effect) {
    case 'glitch':
      applyGlitch(canvas, ctx, settings, bassEnergy);
      break;
    case 'chromatic':
      applyChromaticAberration(canvas, ctx, settings, bassEnergy);
      break;
    case 'vignette':
      applyVignette(ctx, canvas.width, canvas.height, settings, bassEnergy);
      break;
    case 'pulse':
      applyPulse(ctx, settings, bassEnergy);
      break;
  }
}

function applyGlitch(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
) {
  const now = performance.now();
  const intensity = settings.glitchIntensity * bassEnergy * 2;
  if (intensity < 0.05) return;

  const w = canvas.width;
  const h = canvas.height;

  ctx.save();
  ctx.globalAlpha = 0.6;

  const sliceCount = Math.floor(3 + intensity * 12);
  for (let i = 0; i < sliceCount; i++) {
    const sliceY = Math.random() * h;
    const sliceH = 2 + Math.random() * 8 * intensity;
    const offsetX = (Math.random() - 0.5) * 40 * intensity;
    ctx.drawImage(canvas, 0, sliceY, w, sliceH, offsetX, sliceY, w, sliceH);
  }
  ctx.restore();

  if (intensity > 0.3 && now - lastGlitchTime > 200) {
    lastGlitchTime = now;
    const gH = 1 + Math.random() * 4 * intensity;
    const gY = Math.random() * h;
    const gX = Math.random() * w * 0.3;
    const gW = w * (0.3 + Math.random() * 0.7);

    ctx.fillStyle = `rgb(${Math.random() > 0.5 ? 0 : 255}, ${Math.random() > 0.5 ? 0 : 255}, ${Math.random() > 0.5 ? 0 : 255})`;
    ctx.globalAlpha = 0.3 + Math.random() * 0.4;
    ctx.fillRect(gX, gY, gW, gH);
    ctx.globalAlpha = 1;
  }
}

function applyChromaticAberration(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
) {
  const offset = settings.chromaticIntensity * bassEnergy * 15;
  if (offset < 1) return;

  const w = canvas.width;
  const h = canvas.height;

  const imageData = ctx.getImageData(0, 0, w, h);
  const data = imageData.data;
  const output = ctx.createImageData(w, h);
  const outData = output.data;

  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const srcIdx = (y * w + x) * 4;

      const rIdx = (y * w + Math.max(0, Math.min(w - 1, x + offset))) * 4;
      const bIdx = (y * w + Math.max(0, Math.min(w - 1, x - offset))) * 4;

      const outIdx = srcIdx;
      outData[outIdx] = data[rIdx];
      outData[outIdx + 1] = data[srcIdx + 1];
      outData[outIdx + 2] = data[bIdx + 2];
      outData[outIdx + 3] = 255;
    }
  }

  ctx.putImageData(output, 0, 0);
}

function applyVignette(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
) {
  const pulse = bassEnergy * settings.pulseIntensity;
  const maxRadius = Math.sqrt(w * w + h * h) / 2;
  const radius = maxRadius * (0.5 + pulse * 0.3);

  const gradient = ctx.createRadialGradient(w / 2, h / 2, radius * 0.3, w / 2, h / 2, radius);
  gradient.addColorStop(0, 'rgba(0,0,0,0)');
  gradient.addColorStop(0.6, 'rgba(0,0,0,0)');
  gradient.addColorStop(1, `rgba(0,0,0,${0.4 + pulse * 0.4})`);

  ctx.save();
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, w, h);
  ctx.restore();
}

function applyPulse(
  ctx: CanvasRenderingContext2D,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
) {
  const alpha = bassEnergy * settings.pulseIntensity * 0.4;
  if (alpha < 0.01) return;

  ctx.save();
  ctx.fillStyle = `rgba(255,255,255,${alpha})`;
  ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  ctx.restore();
}

export function getShakeOffset(
  settings: ScreenEffectsSettings,
  bassEnergy: number,
): { x: number; y: number } {
  if (!settings.enabled || settings.mainEffect !== 'shake') return { x: 0, y: 0 };

  const intensity = settings.shakeIntensity * bassEnergy * 20;
  if (intensity < 0.5) return { x: 0, y: 0 };

  const angle = Math.random() * Math.PI * 2;
  const dist = intensity * (0.5 + Math.random() * 0.5);
  return {
    x: Math.cos(angle) * dist,
    y: Math.sin(angle) * dist,
  };
}
