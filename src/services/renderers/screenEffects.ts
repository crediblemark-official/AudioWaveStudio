import { ScreenEffectsSettings } from '../../types/visualizer';

let lastGlitchTime = 0;
let _glitchCanvas: HTMLCanvasElement | null = null;
let _snapshotCanvas: HTMLCanvasElement | null = null;
let _smallCanvas: HTMLCanvasElement | null = null;
let _shakeBucket = -1;
let _shakeX = 0;
let _shakeY = 0;
let _prevBeatHigh = false;
let _shockStart = -1e9;

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
    case 'strobe':
      applyStrobe(ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
    case 'scanline':
      applyScanline(ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
    case 'chromatic':
      applyChromatic(canvas, ctx, settings, useBe, beatStrength);
      break;
    case 'zoom':
      applyZoom(canvas, ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
    case 'invert':
      applyInvert(ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
    case 'bars':
      applyBars(canvas, ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
    case 'shockwave':
      applyShockwave(canvas, ctx, canvas.width, canvas.height, settings, beatStrength);
      break;
    case 'pixelate':
      applyPixelate(canvas, ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
    case 'tilt':
      applyTilt(canvas, ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
    case 'heatHaze':
      applyHeatHaze(canvas, ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
    case 'hueShift':
      applyHueShift(ctx, canvas.width, canvas.height, settings, useBe, beatStrength);
      break;
  }
}

function getSnapshot(source: HTMLCanvasElement): HTMLCanvasElement {
  const w = source.width;
  const h = source.height;
  if (!_snapshotCanvas || _snapshotCanvas.width !== w || _snapshotCanvas.height !== h) {
    _snapshotCanvas = document.createElement('canvas');
    _snapshotCanvas.width = w;
    _snapshotCanvas.height = h;
  }
  const sc = _snapshotCanvas.getContext('2d')!;
  sc.setTransform(1, 0, 0, 1, 0, 0);
  sc.globalCompositeOperation = 'source-over';
  sc.globalAlpha = 1;
  sc.filter = 'none';
  sc.clearRect(0, 0, w, h);
  sc.drawImage(source, 0, 0);
  return _snapshotCanvas;
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
  gc.setTransform(1, 0, 0, 1, 0, 0);
  gc.globalCompositeOperation = 'source-over';
  gc.globalAlpha = 1;
  gc.filter = 'none';
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

function applyStrobe(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? settings.strobeIntensity * 0.9 : 0;
  const smooth = settings.strobeIntensity * bassEnergy * 0.12;
  const alpha = smooth + beat;
  if (alpha < 0.02) return;

  const on = Math.floor(performance.now() / 100) % 2 === 0;
  if (!on) return;

  ctx.save();
  ctx.fillStyle = `rgba(255,255,255,${Math.min(0.9, alpha)})`;
  ctx.fillRect(0, 0, w, h);
  ctx.restore();
}

function applyScanline(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const opacity = settings.scanlineOpacity || 0;
  if (opacity <= 0.01) return;

  const beat = beatStrength > 0.15 ? beatStrength : 0;
  const darken = 0.08 * (bassEnergy * 0.5 + beat);

  ctx.save();
  ctx.fillStyle = `rgba(0,0,0,${Math.min(0.6, opacity)})`;
  for (let y = 0; y < h; y += 4) {
    ctx.fillRect(0, y, w, 1);
  }
  if (darken > 0.01) {
    ctx.fillStyle = `rgba(0,0,0,${Math.min(0.35, darken)})`;
    ctx.fillRect(0, 0, w, h);
  }
  ctx.restore();
}

function applyChromatic(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? settings.chromaticIntensity * beatStrength : 0;
  const smooth = settings.chromaticIntensity * bassEnergy * 0.5;
  const intensity = smooth + beat;
  if (intensity < 0.03) return;

  const snap = getSnapshot(canvas);
  const offset = Math.max(2, intensity * 14);

  ctx.save();
  ctx.globalCompositeOperation = 'screen';
  ctx.globalAlpha = Math.min(0.7, intensity);
  ctx.drawImage(snap, -offset, 0);
  ctx.drawImage(snap, offset, 0);
  ctx.restore();
}

function applyZoom(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? settings.zoomIntensity * beatStrength : 0;
  const smooth = settings.zoomIntensity * bassEnergy * 0.5;
  const amount = smooth + beat;
  if (amount < 0.01) return;

  const snap = getSnapshot(canvas);
  const scale = 1 + amount;

  ctx.save();
  ctx.setTransform(scale, 0, 0, scale, (w * (1 - scale)) / 2, (h * (1 - scale)) / 2);
  ctx.drawImage(snap, 0, 0);
  ctx.restore();
}

function applyInvert(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? settings.invertIntensity * beatStrength : 0;
  const smooth = settings.invertIntensity * bassEnergy * 0.4;
  const amount = Math.min(1, (smooth + beat) * 2);
  if (amount < 0.05) return;

  ctx.save();
  ctx.globalAlpha = amount;
  ctx.globalCompositeOperation = 'difference';
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, w, h);
  ctx.restore();
}

function applyBars(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? settings.barsAmount * beatStrength : 0;
  const smooth = settings.barsAmount * bassEnergy * 0.3;
  const amount = Math.min(0.5, smooth + beat);
  if (amount < 0.01) return;

  const snap = getSnapshot(canvas);
  const barH = Math.round(amount * h * 0.22);
  const zoom = 1 + amount * 0.12;

  ctx.save();
  ctx.clearRect(0, 0, w, h);
  ctx.translate(w / 2, h / 2);
  ctx.scale(zoom, zoom);
  ctx.translate(-w / 2, -h / 2);
  ctx.drawImage(snap, 0, 0);
  ctx.fillStyle = 'rgba(0,0,0,0.96)';
  ctx.fillRect(0, 0, w, barH);
  ctx.fillRect(0, h - barH, w, barH);
  ctx.restore();
}

function applyShockwave(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  beatStrength: number,
) {
  const beatHigh = beatStrength > 0.15;
  if (beatHigh && !_prevBeatHigh) {
    _shockStart = performance.now();
  }
  _prevBeatHigh = beatHigh;

  const elapsed = (performance.now() - _shockStart) / 650;
  if (elapsed < 0 || elapsed >= 1) return;

  const progress = Math.min(1, elapsed);
  const amount = settings.shockwaveIntensity * (1 - progress) * 1.1;
  if (amount < 0.02) return;

  const scale = 0.25;
  const sw = Math.max(2, Math.round(w * scale));
  const sh = Math.max(2, Math.round(h * scale));

  if (!_smallCanvas || _smallCanvas.width !== sw || _smallCanvas.height !== sh) {
    _smallCanvas = document.createElement('canvas');
    _smallCanvas.width = sw;
    _smallCanvas.height = sh;
  }
  const sc = _smallCanvas.getContext('2d')!;
  sc.setTransform(1, 0, 0, 1, 0, 0);
  sc.clearRect(0, 0, sw, sh);
  sc.drawImage(canvas, 0, 0, sw, sh);

  const src = sc.getImageData(0, 0, sw, sh);
  const dst = sc.createImageData(sw, sh);
  const srcPix = src.data;
  const dstPix = dst.data;
  const cx = sw / 2;
  const cy = sh / 2;
  const maxDist = Math.sqrt(cx * cx + cy * cy) || 1;
  const freq = 0.13;
  const time = performance.now() / 1000;

  for (let y = 0; y < sh; y++) {
    for (let x = 0; x < sw; x++) {
      const dx = x - cx;
      const dy = y - cy;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const phase = dist * freq - time * 7;
      const pull = amount * 5 * (dist / maxDist) * Math.sin(phase * Math.PI * 2);
      const sx = dist > 0 ? x + (dx / dist) * pull : x;
      const sy = dist > 0 ? y + (dy / dist) * pull : y;
      const si = (((Math.round(sy) % sh) + sh) % sh) * sw + (((Math.round(sx) % sw) + sw) % sw);
      const di = (y * sw + x) * 4;
      dstPix[di] = srcPix[si * 4];
      dstPix[di + 1] = srcPix[si * 4 + 1];
      dstPix[di + 2] = srcPix[si * 4 + 2];
      dstPix[di + 3] = srcPix[si * 4 + 3];
    }
  }
  sc.putImageData(dst, 0, 0);

  ctx.save();
  ctx.imageSmoothingEnabled = true;
  ctx.drawImage(_smallCanvas, 0, 0, w, h);
  ctx.restore();
}

function applyPixelate(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? settings.pixelateIntensity * beatStrength : 0;
  const smooth = settings.pixelateIntensity * bassEnergy * 0.4;
  const amount = smooth + beat;
  if (amount < 0.02) return;

  const block = Math.max(2, Math.round(4 + amount * 44));
  const sw = Math.max(1, Math.ceil(w / block));
  const sh = Math.max(1, Math.ceil(h / block));

  if (!_smallCanvas || _smallCanvas.width !== sw || _smallCanvas.height !== sh) {
    _smallCanvas = document.createElement('canvas');
    _smallCanvas.width = sw;
    _smallCanvas.height = sh;
  }
  const sc = _smallCanvas.getContext('2d')!;
  sc.setTransform(1, 0, 0, 1, 0, 0);
  sc.clearRect(0, 0, sw, sh);
  sc.drawImage(canvas, 0, 0, sw, sh);

  ctx.save();
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(_smallCanvas, 0, 0, w, h);
  ctx.restore();
}

function applyTilt(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? settings.tiltIntensity * beatStrength : 0;
  const smooth = settings.tiltIntensity * bassEnergy * 0.4;
  const amount = smooth + beat;
  if (amount < 0.02) return;

  const snap = getSnapshot(canvas);
  const angle = (Math.random() - 0.5) * amount * 0.08;

  ctx.save();
  ctx.clearRect(0, 0, w, h);
  ctx.translate(w / 2, h / 2);
  ctx.rotate(angle);
  ctx.translate(-w / 2, -h / 2);
  ctx.drawImage(snap, 0, 0);
  ctx.restore();
}

function applyHeatHaze(
  canvas: HTMLCanvasElement,
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? settings.heatHazeIntensity * beatStrength : 0;
  const smooth = settings.heatHazeIntensity * bassEnergy * 0.3;
  const amount = smooth + beat;
  if (amount < 0.02) return;

  const snap = getSnapshot(canvas);
  ctx.save();
  ctx.clearRect(0, 0, w, h);

  const stripH = 4;
  for (let y = 0; y < h; y += stripH) {
    const xOff = Math.sin((y + performance.now() / 28) * 0.05) * amount * 18;
    ctx.drawImage(snap, 0, y, w, stripH, xOff, y, w, stripH);
  }
  ctx.restore();
}

function applyHueShift(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  settings: ScreenEffectsSettings,
  bassEnergy: number,
  beatStrength: number,
) {
  const beat = beatStrength > 0.15 ? settings.hueShiftIntensity * beatStrength : 0;
  const smooth = settings.hueShiftIntensity * bassEnergy * 0.3;
  const amount = Math.min(0.9, smooth + beat);
  if (amount < 0.02) return;

  const hue = (performance.now() / 40) % 360;
  const grad = ctx.createLinearGradient(0, 0, w, h);
  grad.addColorStop(0, `hsla(${hue}, 85%, 50%, ${amount})`);
  grad.addColorStop(1, `hsla(${(hue + 180) % 360}, 85%, 50%, ${amount})`);

  ctx.save();
  ctx.globalCompositeOperation = 'hue';
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, w, h);
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
  let intensity = smooth + beat;
  if (settings.shakeOnBeat && !doesBeat) intensity = 0;
  if (intensity < 0.5) return { x: 0, y: 0 };

  const maxOffset = Math.max(1, settings.shakeMaxOffset || 40);
  const framesPerHold = Math.round((1 - (settings.shakeFrequency ?? 0.5)) * 8) + 1;
  const bucket = Math.floor(performance.now() / (framesPerHold * 16.67));

  if (bucket !== _shakeBucket) {
    _shakeBucket = bucket;
    const angle = doesBeat && beatStrength > 0.3 ? -Math.PI / 2 : Math.random() * Math.PI * 2;
    const dist = Math.min(intensity * (0.5 + Math.random() * 0.5), maxOffset);
    _shakeX = Math.cos(angle) * dist;
    _shakeY = Math.sin(angle) * dist;
  }

  return { x: _shakeX, y: _shakeY };
}
