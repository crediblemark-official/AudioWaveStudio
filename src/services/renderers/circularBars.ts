import { RenderContext } from './types';

export function renderCircularBars(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData } = ctx;
  const centerX = width / 2;
  const centerY = height / 2;
  const barCount = Math.min(64, config.reactivity.barCount);
  const sensitivity = config.reactivity.sensitivity;
  const theme = config.theme;

  const step = Math.max(1, Math.floor(freqData.length / barCount));
  const maxLen = Math.min(width, height) * 0.42;
  const minRadius = 20;

  c.save();
  for (let i = 0; i < barCount; i++) {
    let val = 0;
    for (let j = 0; j < step; j++) {
      val += freqData[i * step + j] || 0;
    }
    val = (val / step / 255) * sensitivity;
    const barLen = minRadius + val * maxLen;

    const angle = (i / barCount) * Math.PI * 2 - Math.PI / 2;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);

    const x1 = centerX + cos * minRadius;
    const y1 = centerY + sin * minRadius;
    const x2 = centerX + cos * barLen;
    const y2 = centerY + sin * barLen;

    c.beginPath();
    c.moveTo(x1, y1);
    c.lineTo(x2, y2);
    c.strokeStyle = i % 2 === 0 ? theme.primaryColor : theme.secondaryColor;
    c.lineWidth = 3;
    c.shadowBlur = 10;
    c.shadowColor = theme.glowColor;
    c.stroke();
  }
  c.restore();

  c.save();
  const glowGrad = c.createRadialGradient(centerX, centerY, 0, centerX, centerY, minRadius);
  glowGrad.addColorStop(0, theme.accentColor);
  glowGrad.addColorStop(1, 'transparent');
  c.fillStyle = glowGrad;
  c.shadowBlur = 30;
  c.shadowColor = theme.glowColor;
  c.beginPath();
  c.arc(centerX, centerY, minRadius, 0, Math.PI * 2);
  c.fill();
  c.restore();
}
