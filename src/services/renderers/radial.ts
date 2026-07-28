import { RenderContext } from './types';

export function renderRadialVisualizer(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy, rotationAngle, exportFreqData } = ctx;
  const centerX = width / 2;
  const centerY = height * 0.48;
  const baseRadius = Math.min(width, height) * 0.18 + bassEnergy * 18;
  const barCount = Math.min(96, config.reactivity.barCount);
  const sensitivity = config.reactivity.sensitivity;
  const theme = config.theme;

  if (!exportFreqData) {
    ctx.rotationAngle += 0.003;
  }

  c.save();
  c.beginPath();
  c.arc(centerX, centerY, baseRadius - 5, 0, Math.PI * 2);
  c.clip();

  const discGrad = c.createRadialGradient(centerX, centerY, 5, centerX, centerY, baseRadius);
  discGrad.addColorStop(0, theme.primaryColor);
  discGrad.addColorStop(1, theme.secondaryColor);
  c.fillStyle = discGrad;
  c.fill();
  c.restore();

  c.save();
  c.lineWidth = 4;
  c.strokeStyle = theme.accentColor;
  c.shadowBlur = 20;
  c.shadowColor = theme.glowColor;
  c.beginPath();
  c.arc(centerX, centerY, baseRadius, 0, Math.PI * 2);
  c.stroke();
  c.restore();

  const maxSpike = Math.min(width, height) * 0.25;
  const step = Math.floor(freqData.length / barCount);

  c.save();
  c.shadowBlur = 12;
  c.shadowColor = theme.glowColor;

  for (let i = 0; i < barCount; i++) {
    let val = 0;
    for (let j = 0; j < step; j++) {
      val += freqData[i * step + j] || 0;
    }
    val = (val / step / 255) * sensitivity;
    const spikeH = val * maxSpike;

    const angle = (i / barCount) * Math.PI * 2 + rotationAngle;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);

    const x1 = centerX + cos * baseRadius;
    const y1 = centerY + sin * baseRadius;
    const x2 = centerX + cos * (baseRadius + spikeH);
    const y2 = centerY + sin * (baseRadius + spikeH);

    const spikeGrad = c.createLinearGradient(x1, y1, x2, y2);
    spikeGrad.addColorStop(0, theme.primaryColor);
    spikeGrad.addColorStop(1, theme.accentColor);

    c.strokeStyle = spikeGrad;
    c.lineWidth = Math.max(2, (Math.PI * 2 * baseRadius) / barCount - 3);
    c.beginPath();
    c.moveTo(x1, y1);
    c.lineTo(x2, y2);
    c.stroke();
  }
  c.restore();
}
