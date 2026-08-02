import { RenderContext } from '../types';

export function renderNebula(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy, beatStrength } = ctx;
  const bg = config.background;
  const speedMult = bg.nebulaSpeed ?? 1.0;
  const intensityMult = bg.nebulaIntensity ?? 0.6;

  const t = (ctx.frameTime / 7) * speedMult;
  const intensity = (0.5 + bassEnergy * 0.5) * intensityMult;

  c.save();
  c.globalCompositeOperation = 'screen';

  for (let i = 0; i < 5; i++) {
    const seed = i * 73;
    const cx = (Math.sin(seed * 0.1 + t * (0.1 + i * 0.02)) * 0.5 + 0.5) * width;
    const cy = (Math.cos(seed * 0.13 + t * (0.08 + i * 0.03)) * 0.5 + 0.5) * height;
    const r = 180 + Math.sin(seed + t * 0.05) * 80 + beatStrength * 100;
    const hue = (seed * 0.7 + t * 20 + i * 50) % 360;

    const grad = c.createRadialGradient(cx, cy, 0, cx, cy, r);
    grad.addColorStop(0, `hsla(${hue}, 85%, 65%, ${0.50 * intensity})`);
    grad.addColorStop(0.5, `hsla(${hue + 30}, 75%, 45%, ${0.25 * intensity})`);
    grad.addColorStop(1, `hsla(${hue + 60}, 65%, 25%, 0)`);
    c.fillStyle = grad;
    c.fillRect(0, 0, width, height);
  }

  c.restore();
}
