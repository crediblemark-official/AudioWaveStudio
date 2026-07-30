import { RenderContext } from '../types';

function hash(n: number): number {
  const x = Math.sin(n * 12.9898 + 78.233) * 43758.5453;
  return x - Math.floor(x);
}

const MAX_STARS = 300;
const starData = Array.from({ length: MAX_STARS }, (_, i) => ({
  x: hash(i + 1),
  y: hash(i + 1000),
  size: 1.2 + hash(i + 2000) * 2.8,
  phase: hash(i + 3000) * Math.PI * 2,
  speed: 0.01 + hash(i + 4000) * 0.03,
}));

export function renderStarfield(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy, beatStrength } = ctx;
  const bg = config.background;
  const targetCount = Math.min(MAX_STARS, Math.max(20, bg.starCount ?? 160));
  const speedMult = bg.starSpeed ?? 1.0;
  const brightnessMult = bg.starBrightness ?? 1.0;

  const t = (Date.now() / 1000) * speedMult;
  const pulse = 0.7 + bassEnergy * 0.4;

  c.save();
  for (let i = 0; i < targetCount; i++) {
    const s = starData[i];
    const rawX = s.x * width + Math.sin(t * s.speed + s.phase) * 12;
    const rawY = s.y * height + Math.cos(t * s.speed * 0.7 + s.phase) * 12;
    const x = ((rawX % width) + width) % width;
    const y = ((rawY % height) + height) % height;
    const twinkle = 0.4 + Math.sin(t * (1.5 + s.speed * 4) + s.phase) * 0.6;
    const alpha = Math.min(1.0, twinkle * pulse * (0.6 + beatStrength * 0.4) * brightnessMult);

    c.beginPath();
    c.arc(x, y, s.size * pulse, 0, Math.PI * 2);
    c.fillStyle = `rgba(255, 255, 255, ${alpha})`;
    c.fill();
  }
  c.restore();
}
