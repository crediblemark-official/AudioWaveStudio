import { RenderContext } from './types';

export function renderSpectrumBars(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData, peakData } = ctx;
  const barCount = config.reactivity.barCount;
  const barGap = config.reactivity.barGap || 2;
  const configBarWidth = config.reactivity.barWidth || 0;
  const barRounding = config.reactivity.barRounding || 0;
  const sensitivity = config.reactivity.sensitivity;
  const mirror = config.reactivity.mirrorBars;
  const isPeaks = config.reactivity.showPeaks;

  const availableWidth = width * 0.85;
  const totalGap = barGap * (barCount - 1);
  const barWidth = configBarWidth > 0 ? configBarWidth : Math.max(3, (availableWidth - totalGap) / barCount);
  const startX = (width - (barCount * barWidth + totalGap)) / 2;
  const maxBarHeight = height * 0.45;
  const centerY = height * 0.55;

  const theme = config.theme;
  const barGradient = c.createLinearGradient(0, centerY, 0, centerY - maxBarHeight);
  barGradient.addColorStop(0, theme.secondaryColor);
  barGradient.addColorStop(0.6, theme.primaryColor);
  barGradient.addColorStop(1, theme.accentColor);

  c.shadowBlur = 15;
  c.shadowColor = theme.glowColor;
  c.fillStyle = barGradient;

  const step = Math.max(1, Math.floor(freqData.length / barCount));
  const doRound = barRounding > 0 && typeof c.roundRect === 'function';

  for (let i = 0; i < barCount; i++) {
    let val = 0;
    for (let j = 0; j < step; j++) {
      val += freqData[i * step + j] || 0;
    }
    val = (val / step / 255) * sensitivity;
    val = Math.min(1, Math.max(0, val));

    const barH = val * maxBarHeight;

    const prev = peakData[i] ?? 0;
    if (barH > prev) {
      peakData[i] = barH;
    } else {
      peakData[i] = Math.max(0, prev - 2);
    }

    const x = startX + i * (barWidth + barGap);

    if (doRound) {
      c.beginPath();
      c.roundRect(x, centerY - barH, barWidth, barH, [barRounding, barRounding, 0, 0]);
      c.fill();
    } else {
      c.fillRect(x, centerY - barH, barWidth, barH);
    }

    if (mirror) {
      c.globalAlpha = 0.4;
      c.fillRect(x, centerY + 2, barWidth, barH * 0.5);
      c.globalAlpha = 1;
    }

    if (isPeaks && peakData[i] > 2) {
      c.fillStyle = config.reactivity.peakColor || theme.accentColor;
      c.fillRect(x, centerY - peakData[i] - 4, barWidth, 3);
      c.fillStyle = barGradient;
    }
  }
}
