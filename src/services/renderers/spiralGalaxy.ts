import { RenderContext } from './types';

interface GalaxyParticle {
  angle: number;
  radius: number;
  speed: number;
  size: number;
  color: [number, number, number];
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
      color: [100, 150, 255],
      arm,
    });
  }
}

function hexToRgb(hex: string): [number, number, number] {
  const h = hex.replace('#', '');
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
}

export function renderSpiralGalaxy(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy: be, beatStrength: bs } = ctx;
  const theme = config.theme;
  const [pR, pG, pB] = hexToRgb(theme.primaryColor);
  const [sR, sG, sB] = hexToRgb(theme.secondaryColor);
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

    const mix = p.radius;
    const r = Math.round(pR + (sR - pR) * mix);
    const g = Math.round(pG + (sG - pG) * mix);
    const b = Math.round(pB + (sB - pB) * mix);
    p.color = [r, g, b];
    c.globalAlpha = alpha;
    c.fillStyle = `rgb(${r}, ${g}, ${b})`;
    c.shadowBlur = size * 3 * glowIntensity;
    c.shadowColor = theme.glowColor;

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
