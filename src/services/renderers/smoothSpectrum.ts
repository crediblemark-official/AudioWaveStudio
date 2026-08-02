import { RenderContext } from './types';

export function renderSmoothSpectrum(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData } = ctx;
  const barCount = config.reactivity.barCount;
  const sensitivity = config.reactivity.sensitivity;
  const theme = config.theme;

  const availableWidth = width * 0.92;
  const startX = (width - availableWidth) / 2;
  const step = Math.max(1, Math.floor(freqData.length / barCount));
  const bottomY = height * 0.85;
  const maxH = height * 0.65;

  if (barCount < 2) return;

  const points: { x: number; y: number }[] = [];
  const xStep = availableWidth / (barCount - 1);

  for (let i = 0; i < barCount; i++) {
    let val = 0;
    for (let j = 0; j < step; j++) {
      val += freqData[i * step + j] || 0;
    }
    val = (val / step / 255) * sensitivity;
    val = Math.min(1, Math.max(0, val));
    const barH = val * maxH;
    const x = startX + i * xStep;
    points.push({ x, y: bottomY - barH });
  }

  const mirror = config.reactivity.mirrorBars;

  c.save();
  c.beginPath();
  c.moveTo(points[0].x, bottomY);
  for (let i = 0; i < points.length - 1; i++) {
    const xc = (points[i].x + points[i + 1].x) / 2;
    const yc = (points[i].y + points[i + 1].y) / 2;
    c.quadraticCurveTo(points[i].x, points[i].y, xc, yc);
  }
  const last = points[points.length - 1];
  c.lineTo(last.x, last.y);
  c.lineTo(last.x, bottomY);
  c.closePath();

  const fillGrad = c.createLinearGradient(0, bottomY - maxH, 0, bottomY);
  fillGrad.addColorStop(0, theme.primaryColor);
  fillGrad.addColorStop(0.5, theme.secondaryColor);
  fillGrad.addColorStop(1, 'transparent');
  c.fillStyle = fillGrad;
  c.shadowBlur = 20;
  c.shadowColor = theme.glowColor;
  c.fill();

  if (mirror) {
    c.globalAlpha = 0.5;
    c.beginPath();
    c.moveTo(points[0].x, bottomY);
    for (let i = 0; i < points.length - 1; i++) {
      const xc = (points[i].x + points[i + 1].x) / 2;
      const yc = bottomY + (bottomY - (points[i].y + points[i + 1].y) / 2);
      const my = bottomY + (bottomY - points[i].y);
      c.quadraticCurveTo(points[i].x, my, xc, yc);
    }
    const myLast = bottomY + (bottomY - last.y);
    c.lineTo(last.x, myLast);
    c.lineTo(last.x, bottomY);
    c.closePath();
    c.fill();
    c.globalAlpha = 1;
  }

  c.restore();

  c.save();
  c.beginPath();
  c.moveTo(points[0].x, points[0].y);
  for (let i = 0; i < points.length - 1; i++) {
    const xc = (points[i].x + points[i + 1].x) / 2;
    const yc = (points[i].y + points[i + 1].y) / 2;
    c.quadraticCurveTo(points[i].x, points[i].y, xc, yc);
  }
  c.lineTo(last.x, last.y);

  if (mirror) {
    c.moveTo(points[0].x, bottomY + (bottomY - points[0].y));
    for (let i = 0; i < points.length - 1; i++) {
      const xc = (points[i].x + points[i + 1].x) / 2;
      const yc = bottomY + (bottomY - (points[i].y + points[i + 1].y) / 2);
      const my = bottomY + (bottomY - points[i].y);
      c.quadraticCurveTo(points[i].x, my, xc, yc);
    }
    const myLast = bottomY + (bottomY - last.y);
    c.lineTo(last.x, myLast);
  }

  c.strokeStyle = theme.accentColor;
  c.lineWidth = 2;
  c.shadowBlur = 10;
  c.shadowColor = theme.glowColor;
  c.stroke();
  c.restore();
}
