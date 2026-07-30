import { RenderContext } from './types';

interface EmberParticle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  size: number;
  life: number;
  maxLife: number;
  hue: number;
}

let embers: EmberParticle[] = [];
let time = 0;

export function renderApi3D(r: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy: be, beatStrength: bs } = r;
  const sensitivity = config.reactivity.sensitivity;

  const fireWidthRatio = config.reactivity.fireWidthRatio ?? 0.94;
  const fireHeightScale = config.reactivity.fireHeightScale ?? 1.0;

  time += 0.03 + be * 0.02;
  const centerY = height * 0.5;
  const halfMarginRatio = (1 - fireWidthRatio) / 2;
  const startX = width * Math.max(0.01, halfMarginRatio);
  const endX = width * Math.min(0.99, 1 - halfMarginRatio);
  const waveWidth = endX - startX;
  const pointCount = 220;

  // 1. Calculate smooth Gaussian bell-curve displacement for waveform
  const displacements = new Float32Array(pointCount);
  const subDisplacements1 = new Float32Array(pointCount);
  const subDisplacements2 = new Float32Array(pointCount);

  const binCount = Math.min(48, Math.floor(freqData.length / 4));
  const step = Math.floor(freqData.length / binCount);

  for (let b = 0; b < binCount; b++) {
    let sum = 0;
    for (let s = 0; s < step; s++) {
      sum += freqData[b * step + s] || 0;
    }
    const val = (sum / step / 255) * sensitivity;
    if (val < 0.04) continue;

    // Center x position for this frequency bin
    const binRatio = b / binCount;
    const peakX = startX + binRatio * waveWidth;

    // Peak direction: mostly upward for mids/highs, downward for bass/select bins
    const isDownward = (b % 5 === 2 || b % 7 === 4);
    const sign = isDownward ? 1 : -1;
    const peakH = val * height * 0.32 * fireHeightScale * sign;

    const sigma = 16 + val * 10; // Width of the Gaussian bell curve

    for (let i = 0; i < pointCount; i++) {
      const px = startX + (i / (pointCount - 1)) * waveWidth;
      const dist = px - peakX;
      const gaussian = Math.exp(-(dist * dist) / (2 * sigma * sigma));

      displacements[i] += peakH * gaussian;
      subDisplacements1[i] += peakH * 0.65 * Math.exp(-((dist - 12) * (dist - 12)) / (2 * sigma * sigma));
      subDisplacements2[i] += peakH * 0.45 * Math.exp(-((dist + 15) * (dist + 15)) / (2 * sigma * sigma));
    }

    // Spawn micro embers near peak tops
    if (val > 0.25 && Math.random() < 0.25 + bs * 0.3) {
      embers.push({
        x: peakX + (Math.random() - 0.5) * 12,
        y: centerY + peakH * 0.8 + (Math.random() - 0.5) * 10,
        vx: (Math.random() - 0.5) * 1.5,
        vy: (Math.random() - 0.5) * 1.5,
        size: 0.6 + Math.random() * 2.0,
        life: 0,
        maxLife: 25 + Math.random() * 35,
        hue: 20 + Math.random() * 25,
      });
    }
  }

  const theme = config.theme;
  function hexToRgb(hex: string): [number, number, number] {
    const h = hex.replace('#', '');
    return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
  }
  const [aR, aG, aB] = hexToRgb(theme.accentColor);

  c.save();

  // --- PASS 1: PLASMA SMOKE GLOW CLOUD ---
  c.globalCompositeOperation = 'lighter';
  c.shadowBlur = 40 + be * 20;
  c.shadowColor = theme.glowColor;

  const cloudGrad = c.createLinearGradient(0, centerY - height * 0.2, 0, centerY + height * 0.2);
  cloudGrad.addColorStop(0, 'rgba(0, 0, 0, 0)');
  cloudGrad.addColorStop(0.5, theme.secondaryColor + '33');
  cloudGrad.addColorStop(1, 'rgba(0, 0, 0, 0)');
  c.fillStyle = cloudGrad;
  c.fillRect(startX, centerY - height * 0.25, waveWidth, height * 0.5);

  // --- PASS 2: STRAIGHT CENTER LASER BASELINE ---
  c.shadowBlur = 14;
  c.shadowColor = theme.glowColor || theme.primaryColor;
  c.strokeStyle = theme.primaryColor;
  c.globalAlpha = 0.75;
  c.lineWidth = 1.6;
  c.beginPath();
  c.moveTo(startX, centerY);
  c.lineTo(endX, centerY);
  c.stroke();
  c.globalAlpha = 1.0;

  // --- PASS 3: SECONDARY WOVEN SUB-THREAD WAVE LINES ---
  // Sub-thread 1
  c.shadowBlur = 10;
  c.shadowColor = theme.glowColor;
  c.strokeStyle = theme.secondaryColor;
  c.globalAlpha = 0.45;
  c.lineWidth = 1.2;
  c.beginPath();
  for (let i = 0; i < pointCount; i++) {
    const px = startX + (i / (pointCount - 1)) * waveWidth;
    const py = centerY + subDisplacements1[i] + Math.sin(i * 0.1 + time * 4) * 4;
    if (i === 0) c.moveTo(px, py);
    else {
      const prevX = startX + ((i - 1) / (pointCount - 1)) * waveWidth;
      const prevY = centerY + subDisplacements1[i - 1] + Math.sin((i - 1) * 0.1 + time * 4) * 4;
      const xc = (prevX + px) / 2;
      const yc = (prevY + py) / 2;
      c.quadraticCurveTo(prevX, prevY, xc, yc);
    }
  }
  c.stroke();

  // Sub-thread 2
  c.strokeStyle = theme.accentColor;
  c.globalAlpha = 0.35;
  c.lineWidth = 1.0;
  c.beginPath();
  for (let i = 0; i < pointCount; i++) {
    const px = startX + (i / (pointCount - 1)) * waveWidth;
    const py = centerY + subDisplacements2[i] + Math.cos(i * 0.12 - time * 3.5) * 5;
    if (i === 0) c.moveTo(px, py);
    else {
      const prevX = startX + ((i - 1) / (pointCount - 1)) * waveWidth;
      const prevY = centerY + subDisplacements2[i - 1] + Math.cos((i - 1) * 0.12 - time * 3.5) * 5;
      const xc = (prevX + px) / 2;
      const yc = (prevY + py) / 2;
      c.quadraticCurveTo(prevX, prevY, xc, yc);
    }
  }
  c.stroke();
  c.globalAlpha = 1.0;

  // --- PASS 4: VERTICAL LIGHT NEEDLES ON HIGH PEAKS ---
  c.shadowBlur = 15;
  c.shadowColor = theme.glowColor;
  for (let i = 0; i < pointCount; i += 3) {
    const disp = displacements[i];
    const absDisp = Math.abs(disp);
    if (absDisp > 35) {
      const px = startX + (i / (pointCount - 1)) * waveWidth;
      const py = centerY + disp;
      const needleH = absDisp * 1.4;

      const needleGrad = c.createLinearGradient(0, py - needleH * 0.5, 0, py + needleH * 0.5);
      needleGrad.addColorStop(0, 'rgba(255, 255, 255, 0)');
      needleGrad.addColorStop(0.5, `rgba(${aR}, ${aG}, ${aB}, 0.8)`);
      needleGrad.addColorStop(1, 'rgba(255, 255, 255, 0)');

      c.strokeStyle = needleGrad;
      c.lineWidth = 1.2;
      c.beginPath();
      c.moveTo(px, centerY - (disp < 0 ? needleH : 10));
      c.lineTo(px, centerY + (disp > 0 ? needleH : 10));
      c.stroke();
    }
  }

  // --- PASS 5: MAIN GLOWING NEON WAVEFORM (HERO LINE) ---
  // 5A: Outer Neon Bloom
  c.shadowBlur = 28 + be * 18;
  c.shadowColor = theme.glowColor;
  c.strokeStyle = theme.secondaryColor;
  c.globalAlpha = 0.65;
  c.lineWidth = 7.5;
  c.beginPath();
  for (let i = 0; i < pointCount; i++) {
    const px = startX + (i / (pointCount - 1)) * waveWidth;
    const py = centerY + displacements[i];
    if (i === 0) c.moveTo(px, py);
    else {
      const prevX = startX + ((i - 1) / (pointCount - 1)) * waveWidth;
      const prevY = centerY + displacements[i - 1];
      const xc = (prevX + px) / 2;
      const yc = (prevY + py) / 2;
      c.quadraticCurveTo(prevX, prevY, xc, yc);
    }
  }
  c.stroke();
  c.globalAlpha = 1.0;

  // 5B: Inner Primary Line
  c.shadowBlur = 16;
  c.shadowColor = theme.glowColor;
  c.strokeStyle = theme.primaryColor;
  c.lineWidth = 3.6;
  c.stroke();

  // 5C: White-Hot / Accent Core
  c.shadowBlur = 8;
  c.shadowColor = theme.accentColor;
  c.strokeStyle = theme.accentColor;
  c.lineWidth = 1.8;
  c.stroke();

  // --- PASS 6: SUBTLE GLOSSY FLOOR REFLECTION ---
  c.shadowBlur = 12;
  c.shadowColor = theme.glowColor;
  c.strokeStyle = theme.secondaryColor;
  c.globalAlpha = 0.2;
  c.lineWidth = 2.2;
  c.beginPath();
  for (let i = 0; i < pointCount; i++) {
    const px = startX + (i / (pointCount - 1)) * waveWidth;
    const py = centerY - displacements[i] * 0.45;
    if (i === 0) c.moveTo(px, py);
    else {
      const prevX = startX + ((i - 1) / (pointCount - 1)) * waveWidth;
      const prevY = centerY - displacements[i - 1] * 0.45;
      const xc = (prevX + px) / 2;
      const yc = (prevY + py) / 2;
      c.quadraticCurveTo(prevX, prevY, xc, yc);
    }
  }
  c.stroke();
  c.globalAlpha = 1.0;

  // --- PASS 7: FLOATING MICRO EMBERS & DUST ---
  for (let i = embers.length - 1; i >= 0; i--) {
    const e = embers[i];
    e.life++;
    e.x += e.vx;
    e.y += e.vy;

    const progress = e.life / e.maxLife;
    if (progress >= 1) {
      embers.splice(i, 1);
      continue;
    }

    const alpha = (1 - progress) * 0.85;
    c.shadowBlur = 6;
    c.shadowColor = theme.glowColor;
    c.fillStyle = theme.primaryColor;
    c.globalAlpha = alpha;
    c.beginPath();
    c.arc(e.x, e.y, e.size * (1 - progress * 0.3), 0, Math.PI * 2);
    c.fill();
    c.globalAlpha = 1.0;
  }

  if (embers.length > 180) {
    embers.splice(0, embers.length - 180);
  }

  c.restore();
}





