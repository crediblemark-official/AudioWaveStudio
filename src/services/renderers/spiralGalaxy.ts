import { RenderContext } from './types';

interface GalaxyParticle {
  angle: number;
  radius: number;
  speed: number;
  size: number;
  hue: number;
  arm: number;
}

let galaxy: GalaxyParticle[] = [];
let galaxyInit = false;

function initGalaxy() {
  if (galaxyInit) return;
  galaxyInit = true;
  for (let i = 0; i < 400; i++) {
    const arm = Math.floor(Math.random() * 3);
    const r = Math.random();
    galaxy.push({
      angle: Math.random() * Math.PI * 2 + arm * 2.1,
      radius: r,
      speed: 0.002 + (1 - r) * 0.008,
      size: 0.5 + r * 2.5,
      hue: 200 + Math.random() * 60 + arm * 30,
      arm,
    });
  }
}

export function renderSpiralGalaxy(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy: be, beatStrength: bs } = ctx;
  const theme = config.theme;
  initGalaxy();

  const cx = width / 2;
  const cy = height / 2;
  const maxR = Math.min(width, height) * 0.45;
  const rotSpeed = 0.003 + be * 0.01 + bs * 0.02;
  const glowIntensity = 0.5 + be * 1.5;

  for (const p of galaxy) {
    p.angle += p.speed + rotSpeed;

    const dist = p.radius * maxR;
    const spiralOffset = p.radius * 0.5;
    const a = p.angle + p.arm * 2.1 + p.radius * 3;
    const x = cx + Math.cos(a) * (dist + Math.sin(p.angle * 3 + p.arm) * spiralOffset);
    const y = cy + Math.sin(a) * (dist + Math.cos(p.angle * 3 + p.arm) * spiralOffset);

    const alpha = (0.3 + p.radius * 0.4) * (0.5 + be * 0.5);
    const size = p.size * (1 + be * 0.5);

    c.globalAlpha = alpha;
    c.fillStyle = `hsl(${p.hue + be * 20}, 80%, ${50 + p.radius * 30}%)`;
    c.shadowBlur = size * 3 * glowIntensity;
    c.shadowColor = `hsl(${p.hue + be * 20}, 100%, 60%)`;

    c.beginPath();
    c.arc(x, y, size, 0, Math.PI * 2);
    c.fill();
  }

  c.beginPath();
  c.arc(cx, cy, 2 + be * 4, 0, Math.PI * 2);
  c.fillStyle = '#ffffff';
  c.shadowBlur = 20 * glowIntensity;
  c.shadowColor = theme.glowColor;
  c.globalAlpha = 0.8 + be * 0.2;
  c.fill();

  c.globalAlpha = 1;
  c.shadowBlur = 0;
}
