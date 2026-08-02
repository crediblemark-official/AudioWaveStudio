import { RenderContext } from './types';

interface AuroraLayer {
  offset: number;
  speed: number;
  freq: number;
  amp: number;
  color: string;
  alpha: number;
  yBase: number;
}

let t = 0;

export function renderAuroraWave(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy: be } = ctx;
  const theme = config.theme;
  t += 0.008;
  if (t > 10000) t -= 10000;

  const bassAmp = 0.1 + be * 0.4;

  const layers: AuroraLayer[] = [
    { offset: 0, speed: 0.5, freq: 0.003, amp: 0.06, color: theme.primaryColor, alpha: 0.25, yBase: height * 0.3 },
    { offset: 2, speed: 0.7, freq: 0.005, amp: 0.04, color: theme.secondaryColor, alpha: 0.2, yBase: height * 0.45 },
    { offset: 4, speed: 0.3, freq: 0.002, amp: 0.08, color: theme.accentColor, alpha: 0.15, yBase: height * 0.35 },
  ];

  for (let i = 0; i < layers.length; i++) {
    const layer = layers[i];
    const points: { x: number; y: number }[] = [];

    for (let x = 0; x <= width; x += 4) {
      const freqIdx = Math.floor((x / width) * freqData.length);
      const fVal = freqData[freqIdx] || 0;
      const wave = Math.sin(x * layer.freq + t * layer.speed + layer.offset);
      const wave2 = Math.sin(x * layer.freq * 2.3 + t * layer.speed * 1.7 + layer.offset + 1.5);
      const amp = layer.amp * (1 + bassAmp * 2) * (1 + (fVal / 255) * 0.5);
      const y = layer.yBase + wave * amp * height + wave2 * amp * height * 0.5;
      points.push({ x, y });
    }

    c.beginPath();
    c.moveTo(0, height);
    for (const p of points) c.lineTo(p.x, p.y);
    c.lineTo(width, height);
    c.closePath();

    const grad = c.createLinearGradient(0, 0, width, 0);
    grad.addColorStop(0, layer.color + '00');
    grad.addColorStop(0.3, layer.color + Math.round(layer.alpha * 255).toString(16).padStart(2, '0'));
    grad.addColorStop(0.7, layer.color + Math.round(layer.alpha * 255).toString(16).padStart(2, '0'));
    grad.addColorStop(1, layer.color + '00');
    c.fillStyle = grad;

    c.globalAlpha = layer.alpha * (0.5 + be * 0.5);
    c.fill();
  }

  c.globalAlpha = 1;

  for (let i = 0; i < layers.length; i++) {
    const layer = layers[i];
    c.beginPath();
    for (let x = 0; x <= width; x += 4) {
      const freqIdx = Math.floor((x / width) * freqData.length);
      const fVal = freqData[freqIdx] || 0;
      const wave = Math.sin(x * layer.freq + t * layer.speed + layer.offset);
      const wave2 = Math.sin(x * layer.freq * 2.3 + t * layer.speed * 1.7 + layer.offset + 1.5);
      const amp = layer.amp * (1 + bassAmp * 2) * (1 + (fVal / 255) * 0.5);
      const y = layer.yBase + wave * amp * height + wave2 * amp * height * 0.5;
      if (x === 0) c.moveTo(x, y);
      else c.lineTo(x, y);
    }
    c.strokeStyle = layer.color;
    c.lineWidth = 2;
    c.globalAlpha = layer.alpha * (0.8 + be * 0.2);
    c.shadowBlur = 20;
    c.shadowColor = layer.color;
    c.stroke();
  }

  c.globalAlpha = 1;
  c.shadowBlur = 0;
}
