import { RenderContext } from '../types';

export function renderBokeh(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy, beatStrength } = ctx;
  const bg = config.background;
  const count = Math.min(40, bg.bokehCount ?? 18);
  const scaleFactor = Math.min(width, height) / 1080;
  const baseSize = (bg.bokehSize ?? 30) * scaleFactor;
  const baseOpacity = bg.bokehOpacity ?? 0.3;
  const t = ctx.frameTime / 5;

  c.save();
  c.globalCompositeOperation = 'screen';

  for (let i = 0; i < count; i++) {
    const seed = i * 137.5;
    const x = (Math.sin(seed + t * (0.2 + i * 0.03)) * 0.5 + 0.5) * width;
    const y = (Math.cos(seed * 0.7 + t * (0.15 + i * 0.02)) * 0.5 + 0.5) * height;
    const radius = Math.max(4 * scaleFactor, baseSize + Math.sin(seed * 0.3 + t) * (baseSize * 0.4) + beatStrength * 15 * scaleFactor);
    const hue = (seed + t * 30) % 360;
    const alpha = (baseOpacity + bassEnergy * 0.15);

    c.beginPath();
    c.arc(x, y, radius, 0, Math.PI * 2);
    c.fillStyle = `hsla(${hue}, 80%, 65%, ${alpha})`;
    c.fill();
  }

  c.restore();
}
