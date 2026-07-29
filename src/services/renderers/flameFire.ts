import { RenderContext } from './types';

interface FireParticle {
  x: number;
  y: number;
  vy: number;
  vx: number;
  size: number;
  alpha: number;
  life: number;
  maxLife: number;
  hue: number;
}

let fireParticles: FireParticle[] = [];

export function renderFlameFire(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy: be } = ctx;
  const sensitivity = config.reactivity.sensitivity;

  if (fireParticles.length > 0 || be > 0.02) {
    const spawnCount = Math.floor(2 + be * 8 * sensitivity);
    for (let s = 0; s < spawnCount; s++) {
      const maxLife = 40 + Math.random() * 30 + be * 30;
      fireParticles.push({
        x: Math.random() * width,
        y: height - Math.random() * height * 0.05,
        vy: -(0.5 + Math.random() * 1 + be * 3),
        vx: (Math.random() - 0.5) * 0.5,
        size: 2 + Math.random() * 4 + be * 4,
        alpha: 0.5 + Math.random() * 0.5,
        life: 0,
        maxLife,
        hue: 15 + Math.random() * 30,
      });
    }
  }

  const maxParticles = 300;
  while (fireParticles.length > maxParticles) fireParticles.shift();

  for (let i = fireParticles.length - 1; i >= 0; i--) {
    const p = fireParticles[i];
    p.life++;
    if (p.life >= p.maxLife) {
      fireParticles.splice(i, 1);
      continue;
    }

    p.x += p.vx + (Math.random() - 0.5) * 0.3;
    p.y += p.vy;
    p.vy += 0.02;
    p.alpha *= 0.99;

    const t = p.life / p.maxLife;
    const size = p.size * (1 - t * 0.7);
    const alpha = p.alpha * (1 - t);

    const lightness = 100 - t * 80;
    const sat = 100 - t * 30;
    c.fillStyle = `hsl(${p.hue - t * 10}, ${sat}%, ${lightness}%)`;
    c.globalAlpha = alpha;

    c.beginPath();
    c.arc(p.x, p.y, size, 0, Math.PI * 2);
    c.shadowBlur = 15;
    c.shadowColor = `hsl(${p.hue}, 100%, 50%)`;
    c.fill();
  }

  const highSum = freqData.slice(24, 48).reduce((a, b) => a + b, 0) / (24 * 255);
  if (highSum > 0.2) {
    for (let s = 0; s < Math.floor(highSum * 5 * sensitivity); s++) {
      fireParticles.push({
        x: Math.random() * width,
        y: height - 10,
        vy: -(1 + Math.random() * 2 + highSum * 4),
        vx: (Math.random() - 0.5) * 1.5,
        size: 1 + Math.random() * 2,
        alpha: 1,
        life: 0,
        maxLife: 15 + Math.random() * 10,
        hue: 40 + Math.random() * 20,
      });
    }
  }

  c.globalAlpha = 1;
  c.shadowBlur = 0;
}
