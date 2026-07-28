import { RenderContext } from './types';

export function renderWaveformFill(ctx: RenderContext) {
  const { ctx: c, width, height, config, timeData } = ctx;
  const centerY = height * 0.55;
  const theme = config.theme;
  const len = timeData.length;
  const sliceWidth = width / (len - 1);

  const fillGrad = c.createLinearGradient(0, 0, 0, height);
  fillGrad.addColorStop(0, theme.primaryColor);
  fillGrad.addColorStop(0.5, theme.secondaryColor);
  fillGrad.addColorStop(1, 'transparent');

  c.save();
  c.beginPath();
  let x = 0;
  for (let i = 0; i < len; i++) {
    const v = (timeData[i] / 128.0) - 1.0;
    const y = centerY + v * (height * 0.28) * config.reactivity.sensitivity;
    if (i === 0) c.moveTo(x, y);
    else c.lineTo(x, y);
    x += sliceWidth;
  }
  c.lineTo(width, height);
  c.lineTo(0, height);
  c.closePath();
  c.fillStyle = fillGrad;
  c.shadowBlur = 20;
  c.shadowColor = theme.glowColor;
  c.fill();
  c.restore();

  c.save();
  c.beginPath();
  x = 0;
  for (let i = 0; i < len; i++) {
    const v = (timeData[i] / 128.0) - 1.0;
    const y = centerY + v * (height * 0.28) * config.reactivity.sensitivity;
    if (i === 0) c.moveTo(x, y);
    else c.lineTo(x, y);
    x += sliceWidth;
  }
  c.strokeStyle = theme.accentColor;
  c.lineWidth = 2;
  c.shadowBlur = 10;
  c.shadowColor = theme.glowColor;
  c.stroke();
  c.restore();
}
