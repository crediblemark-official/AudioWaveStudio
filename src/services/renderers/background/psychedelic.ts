import { RenderContext } from '../types';

export function renderPsychedelic(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy, beatStrength } = ctx;
  if (!width || !height || width <= 0 || height <= 0) return;

  const bg = config.background;
  const speedMult = Math.max(0.01, bg.psychedelicSpeed ?? 1.0);
  const targetBands = Math.max(1, bg.psychedelicBands ?? 24);
  const baseLineWidth = Math.max(0.5, bg.psychedelicLineWidth ?? 4);

  const t = (ctx.frameTime / 2) * speedMult;
  const cx = width / 2;
  const cy = height / 2;
  const maxR = Math.sqrt(width * width + height * height) / 2;
  const bands = Math.max(1, targetBands + Math.floor((beatStrength || 0) * 20));

  if (!isFinite(cx) || !isFinite(cy) || !isFinite(maxR) || maxR <= 0) return;

  c.save();
  for (let i = 0; i < bands; i++) {
    const r = (i / bands) * maxR;
    const angle = r * 0.05 + t * (0.3 + (bassEnergy || 0) * 0.4) + i * 0.5;
    const hue = ((angle * 40 + t * 50) % 360 + 360) % 360;
    const alpha = Math.min(1.0, Math.max(0, 0.20 + (bassEnergy || 0) * 0.15));
    const ringRadius = r + Math.sin(t * 2 + i) * 12;

    if (!isFinite(ringRadius) || ringRadius <= 0) continue;

    c.beginPath();
    c.arc(cx, cy, ringRadius, 0, Math.PI * 2);
    c.strokeStyle = `hsla(${hue}, 95%, 60%, ${alpha})`;
    c.lineWidth = Math.max(0.1, baseLineWidth + (beatStrength || 0) * 6);
    c.stroke();
  }
  c.restore();
}
