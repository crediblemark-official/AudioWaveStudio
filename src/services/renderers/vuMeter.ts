import { RenderContext } from './types';

interface VuChannel {
  level: number;
  peak: number;
  peakHold: number;
}

const channels: VuChannel[] = [
  { level: 0, peak: 0, peakHold: 0 },
  { level: 0, peak: 0, peakHold: 0 },
];

export function renderVuMeter(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData } = ctx;
  const theme = config.theme;
  const sensitivity = config.reactivity.sensitivity;

  const cx = width / 2;
  const cy = height / 2;
  const radius = Math.min(width, height) * 0.38;
  const needleLen = radius * 0.7;

  for (let ch = 0; ch < 2; ch++) {
    const startBin = ch * 6;
    let sum = 0;
    for (let i = 0; i < 6 && startBin + i < freqData.length; i++) {
      sum += freqData[startBin + i] || 0;
    }
    const raw = (sum / (6 * 255)) * sensitivity;
    channels[ch].level += (Math.min(raw, 1) - channels[ch].level) * 0.3;
    channels[ch].peak = Math.max(channels[ch].peak, channels[ch].level);
    channels[ch].peak *= 0.92;
    channels[ch].peakHold = Math.max(channels[ch].peakHold, channels[ch].peak);
    channels[ch].peakHold -= 0.003;
    if (channels[ch].peakHold < 0) channels[ch].peakHold = 0;
  }

  const chX = (idx: number) => {
    const spacing = radius * 0.6;
    const baseX = cx;
    const gap = radius * 0.15;
    return baseX + (idx === 0 ? -spacing - gap : spacing + gap);
  };
  const chY = cy;

  for (let ch = 0; ch < 2; ch++) {
    const x = chX(ch);
    const y = chY;
    const { level, peakHold } = channels[ch];

    const greenAngle = Math.max(0, -0.75 + level * 2.5);

    c.save();
    c.translate(x, y);

    c.beginPath();
    c.arc(0, 0, radius, Math.PI * 0.8, Math.PI * 0.2);
    c.lineWidth = 6;
    c.strokeStyle = 'rgba(255,255,255,0.1)';
    c.stroke();

    c.beginPath();
    c.arc(0, 0, radius, Math.PI * 0.8, Math.PI * 0.8 + greenAngle);
    c.strokeStyle = ch === 0 ? theme.primaryColor : theme.secondaryColor;
    if (level > 0.7) {
      c.strokeStyle = ch === 0 ? theme.accentColor : '#ff3333';
    }
    c.lineWidth = 6;
    c.shadowBlur = 12;
    c.shadowColor = theme.glowColor;
    c.stroke();

    c.shadowBlur = 0;

    const needleAngle = Math.PI * 0.8 + level * 2.5;
    c.beginPath();
    c.moveTo(0, 0);
    c.lineTo(Math.cos(needleAngle) * needleLen, Math.sin(needleAngle) * needleLen);
    c.strokeStyle = theme.accentColor;
    c.lineWidth = 3;
    c.shadowBlur = 8;
    c.shadowColor = theme.glowColor;
    c.stroke();

    c.shadowBlur = 0;
    c.beginPath();
    c.arc(0, 0, 6, 0, Math.PI * 2);
    c.fillStyle = '#ffffff';
    c.fill();

    const holdAngle = Math.PI * 0.8 + peakHold * 2.5;
    c.beginPath();
    c.arc(
      Math.cos(holdAngle) * radius,
      Math.sin(holdAngle) * radius,
      4, 0, Math.PI * 2,
    );
    c.fillStyle = '#ffffff';
    c.fill();

    c.restore();
  }

  c.fillStyle = 'rgba(255,255,255,0.4)';
  c.font = `${Math.min(width * 0.025, 16)}px monospace`;
  c.textAlign = 'center';
  c.fillText('VU METER', cx, height - 15);
}
