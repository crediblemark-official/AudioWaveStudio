import { RenderContext } from '../types';

export function renderBokeh(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy, beatStrength } = ctx;
  const bg = config.background;
  const count = bg.bokehCount ?? 18;
  const baseSize = bg.bokehSize ?? 30;
  const baseOpacity = bg.bokehOpacity ?? 0.3;
  const t = Date.now() / 5000;

  c.save();
  c.globalCompositeOperation = 'screen';

  for (let i = 0; i < count; i++) {
    const seed = i * 137.5;
    const x = (Math.sin(seed + t * (0.2 + i * 0.03)) * 0.5 + 0.5) * width;
    const y = (Math.cos(seed * 0.7 + t * (0.15 + i * 0.02)) * 0.5 + 0.5) * height;
    const radius = Math.max(1, baseSize + Math.sin(seed * 0.3 + t) * (baseSize * 0.4) + beatStrength * 40);
    const hue = (seed + t * 30) % 360;

    c.beginPath();
    c.arc(x, y, radius, 0, Math.PI * 2);
    c.fillStyle = `hsla(${hue}, 80%, 65%, ${baseOpacity + bassEnergy * 0.15})`;
    c.fill();

    c.beginPath();
    c.arc(x - radius * 0.25, y - radius * 0.25, radius * 0.45, 0, Math.PI * 2);
    c.fillStyle = `hsla(${hue}, 90%, 85%, ${baseOpacity * 1.3 + bassEnergy * 0.2})`;
    c.fill();
  }

  c.restore();
}
