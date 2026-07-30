import { ScreenEffectsSettings } from '../../types/visualizer';

let lastGlitchTime = 0;
let _glitchCanvas: HTMLCanvasElement | null = null;

export function applyScreenEffects(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  settings: ScreenEffectsSettings,
  beatStrength: number,
  aboveFloor: number,
) {
  if (!settings.enabled) return;

  const effect = settings.mainEffect;
  if (effect === 'none') return;

  const useBe = Math.max(0, aboveFloor);

  switch (effect) {
    case 'glitch':
      applyGlitch(canvas, ctx, settings, useBe, beatStrength);
      break;

    case 'vignette':
      applyVignette(ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
    case 'pulse':
      applyPulse(ctx, settings, useBe, beatStrength);
      break;
    case 'spotlight':
      applySpotlight(ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
  }
}

function applyGlitch(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const now = performance.now();
  const beat = beatStrength > 0.15 ? settings.glitchIntensity * beatStrength * 8 : 0;
  const smooth = settings.glitchIntensity * bassEnergy * 0.8;
  const intensity = smooth + beat;
  if (intensity < 0.05) return;

  const w = canvas.width;
  const h = canvas.height;

  if (!_glitchCanvas || _glitchCanvas.width !== w || _glitchCanvas.height !== h) {
    _glitchCanvas = document.createElement('canvas');
    _glitchCanvas.width = w;
    _glitchCanvas.height = h;
  }
  const gc = _glitchCanvas.getContext('2d')!;
  gc.clearRect(0, 0, w, h);
  gc.drawImage(canvas, 0, 0);

  ctx.save();
  ctx.globalAlpha = 0.6;

  const sliceCount = Math.floor(3 + intensity * 12);
  for (let i = 0; i < sliceCount; i++) {
    const sliceY = Math.random() * h;
    const sliceH = 2 + Math.random() * 8 * intensity;
    const offsetX = (Math.random() - 0.5) * 40 * intensity;
    ctx.drawImage(_glitchCanvas, 0, sliceY, w, sliceH, offsetX, sliceY, w, sliceH);
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

function applyVignette(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beatPulse = beatStrength > 0.15 ? beatStrength * settings.pulseIntensity * 2.5 : 0;
  const pulse = bassEnergy * settings.pulseIntensity * 0.5 + beatPulse;
  const maxRadius = Math.sqrt(w * w + h * h) / 2;
  const radius = maxRadius * Math.max(0.2, 0.5 + pulse * 0.3);

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
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? beatStrength * settings.pulseIntensity * 1.0 : 0;
  const smooth = bassEnergy * settings.pulseIntensity * 0.15;
  const alpha = smooth + beat;
  if (alpha < 0.01) return;

  ctx.save();
  ctx.fillStyle = `rgba(255,255,255,${alpha})`;
  ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  ctx.restore();
}

function hexToRgb(hex: string) {
  const h = hex.replace('#', '');
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
}

function applySpotlight(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const pulse = 0.5 + bassEnergy * 0.3 + (beatStrength > 0.15 ? beatStrength * 0.3 : 0);
  const alpha = Math.min(1, 0.6 * pulse);
  const mainRgb = hexToRgb(settings.spotlightColor || '#FFD700');

  ctx.save();
  ctx.globalCompositeOperation = 'screen';

  const [r0, g0, b0] = mainRgb;
  const maxDim = Math.max(w, h);
  const corners = [
    { x: 0, y: 0, color: mainRgb },
    { x: w, y: 0, color: [g0, b0, r0] },
    { x: w, y: h, color: [b0, r0, g0] },
    { x: 0, y: h, color: [Math.min(255, g0 + 40), Math.min(255, r0 + 20), Math.min(255, b0 + 30)] },
  ];

  for (const corner of corners) {
    const grad = ctx.createRadialGradient(corner.x, corner.y, 0, corner.x, corner.y, maxDim * 1.1);
    const [r, g, b] = corner.color;
    grad.addColorStop(0, `rgba(${r},${g},${b},${alpha * 0.3})`);
    grad.addColorStop(0.35, `rgba(${r},${g},${b},${alpha * 0.08})`);
    grad.addColorStop(1, `rgba(${r},${g},${b},0)`);

    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, w, h);
  }

  ctx.restore();
}

export function getShakeOffset(
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
  aboveFloor: number = 0,
): { x: number; y: number } {
  if (!settings.enabled || settings.mainEffect !== 'shake') return { x: 0, y: 0 };

  const useBe = Math.max(0, aboveFloor || bassEnergy);
  const doesBeat = beatStrength > 0.15;
  const smooth = settings.shakeIntensity * useBe * 8;
  const beat = doesBeat ? settings.shakeIntensity * beatStrength * 50 : 0;
  const intensity = smooth + beat;
  if (intensity < 0.5) return { x: 0, y: 0 };

  const angle = doesBeat && beatStrength > 0.3 ? -Math.PI / 2 : Math.random() * Math.PI * 2;
  const dist = intensity * (0.5 + Math.random() * 0.5);
  return {
    x: Math.cos(angle) * dist,
    y: Math.sin(angle) * dist,
  };
}
