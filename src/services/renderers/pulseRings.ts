import { RenderContext } from './types';

interface PulseRing {
  radius: number;
  maxRadius: number;
  alpha: number;
  speed: number;
  thickness: number;
  color: string;
}

let rings: PulseRing[] = [];
let prevBeat = 0;

export function renderPulseRings(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy: be, beatStrength: bs } = ctx;
  const centerX = width / 2;
  const centerY = height / 2;
  const theme = config.theme;
  const maxDim = Math.max(width, height) * 0.8;

  if (bs > 0.15 && bs > prevBeat) {
    const count = 1 + Math.floor(bs * 2);
    for (let i = 0; i < count; i++) {
      rings.push({
        radius: 10 + i * 20,
        maxRadius: maxDim * (0.5 + Math.random() * 0.5),
        alpha: 0.4 + be * 0.3,
        speed: 2 + bs * 3 + Math.random() * 2,
        thickness: 2 + be * 4 + bs * 3,
        color: i % 2 === 0 ? theme.primaryColor : theme.secondaryColor,
      });
    }
  }
  prevBeat = bs;

  for (let i = rings.length - 1; i >= 0; i--) {
    const r = rings[i];
    r.radius += r.speed;
    r.alpha *= 0.985;

    if (r.radius > r.maxRadius || r.alpha < 0.01) {
      rings.splice(i, 1);
      continue;
    }

    c.beginPath();
    c.arc(centerX, centerY, r.radius, 0, Math.PI * 2);
    c.strokeStyle = r.color;
    c.lineWidth = r.thickness * (r.alpha / (0.4 + be * 0.3));
    c.globalAlpha = r.alpha;
    c.shadowBlur = 15;
    c.shadowColor = theme.glowColor;
    c.stroke();
  }

  c.globalAlpha = 1;
  c.shadowBlur = 0;
}
