import { RenderContext } from './types';

export function renderWaveformFill(ctx: RenderContext) {
  const { ctx: c, width, height, config, timeData } = ctx;
  const centerY = height * 0.55;
  const theme = config.theme;
  const len = timeData.length;
  const sliceWidth = width / (len - 1);
  const mirror = config.reactivity.mirrorBars;

  const fillGrad = c.createLinearGradient(0, 0, 0, height);
  fillGrad.addColorStop(0, theme.primaryColor);
  fillGrad.addColorStop(0.5, theme.secondaryColor);
  fillGrad.addColorStop(1, 'transparent');

  // Build wave points
  const pts: { x: number; y: number }[] = [];
  for (let i = 0; i < len; i++) {
    const v = (timeData[i] / 128.0) - 1.0;
    const y = centerY + v * (height * 0.28) * config.reactivity.sensitivity;
    pts.push({ x: i * sliceWidth, y });
  }

  // Fill upper waveform
  c.save();
  c.beginPath();
  for (let i = 0; i < pts.length; i++) {
    if (i === 0) c.moveTo(pts[i].x, pts[i].y);
    else c.lineTo(pts[i].x, pts[i].y);
  }
  c.lineTo(width, height);
  c.lineTo(0, height);
  c.closePath();
  c.fillStyle = fillGrad;
  c.shadowBlur = 20;
  c.shadowColor = theme.glowColor;
  c.fill();

  // Mirrored fill
  if (mirror) {
    c.globalAlpha = 0.5;
    c.beginPath();
    for (let i = 0; i < pts.length; i++) {
      const my = centerY - (pts[i].y - centerY);
      if (i === 0) c.moveTo(pts[i].x, my);
      else c.lineTo(pts[i].x, my);
    }
    c.lineTo(width, 0);
    c.lineTo(0, 0);
    c.closePath();
    c.fill();
    c.globalAlpha = 1;
  }

  c.restore();

  // Stroke upper waveform
  c.save();
  c.beginPath();
  for (let i = 0; i < pts.length; i++) {
    if (i === 0) c.moveTo(pts[i].x, pts[i].y);
    else c.lineTo(pts[i].x, pts[i].y);
  }
  c.strokeStyle = theme.accentColor;
  c.lineWidth = 2;
  c.shadowBlur = 10;
  c.shadowColor = theme.glowColor;
  c.stroke();

  // Mirrored stroke
  if (mirror) {
    c.globalAlpha = 0.6;
    c.beginPath();
    for (let i = 0; i < pts.length; i++) {
      const my = centerY - (pts[i].y - centerY);
      if (i === 0) c.moveTo(pts[i].x, my);
      else c.lineTo(pts[i].x, my);
    }
    c.stroke();
    c.globalAlpha = 1;
  }

  c.restore();
}
