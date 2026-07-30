import { RenderContext } from '../types';

export function renderAurora(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy, beatStrength } = ctx;
  const bg = config.background;
  const speedMult = bg.auroraSpeed ?? 1.0;
  const baseAmp = bg.auroraAmplitude ?? 50;
  const baseOpacity = bg.auroraOpacity ?? 0.25;

  const t = (Date.now() / 1000) * speedMult;
  const speed = (0.3 + bassEnergy * 0.6) * speedMult;
  const amp = baseAmp + beatStrength * 60;

  c.save();
  c.globalCompositeOperation = 'screen';
  for (let i = 0; i < 4; i++) {
    const hue = (i * 60 + t * 25) % 360;
    c.beginPath();
    for (let x = 0; x <= width; x += 6) {
      const y =
        height * 0.45 +
        Math.sin(x * 0.006 + t * speed + i * 1.5) * amp +
        Math.sin(x * 0.012 + t * speed * 0.7 + i * 2) * (amp * 0.5);
      if (x === 0) c.moveTo(x, y);
      else c.lineTo(x, y);
    }
    c.lineTo(width, height);
    c.lineTo(0, height);
    c.closePath();
    const alpha = Math.min(1.0, (baseOpacity * 0.6) + bassEnergy * 0.1);
    c.fillStyle = `hsla(${hue}, 85%, 60%, ${alpha})`;
    c.fill();
  }
  c.restore();
}
