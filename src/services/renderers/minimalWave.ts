import { RenderContext } from './types';

export function renderMinimalWaveVisualizer(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData } = ctx;
  const barCount = Math.min(64, config.reactivity.barCount);
  const availableWidth = width * 0.7;
  const barWidth = availableWidth / barCount - 3;
  const startX = (width - availableWidth) / 2;
  const centerY = height * 0.55;
  const theme = config.theme;

  const step = Math.max(1, Math.floor(freqData.length / barCount));

  c.save();
  c.fillStyle = theme.primaryColor;
  c.shadowBlur = 10;
  c.shadowColor = theme.glowColor;

  for (let i = 0; i < barCount; i++) {
    let val = 0;
    for (let j = 0; j < step; j++) {
      val += freqData[i * step + j] || 0;
    }
    val = (val / step / 255) * config.reactivity.sensitivity;
    const barH = Math.max(4, val * height * 0.35);

    const x = startX + i * (barWidth + 3);

    if (typeof c.roundRect === 'function') {
      c.beginPath();
      c.roundRect(x, centerY - barH / 2, barWidth, barH, barWidth / 2);
      c.fill();
    } else {
      c.fillRect(x, centerY - barH / 2, barWidth, barH);
    }
  }
  c.restore();
}
