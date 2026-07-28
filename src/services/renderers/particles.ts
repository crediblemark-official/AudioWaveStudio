import { RenderContext } from './types';

export function initParticles() {
  const particles: RenderContext['particles'] = [];
  for (let i = 0; i < 60; i++) {
    particles.push({
      x: Math.random(),
      y: Math.random(),
      radius: Math.random() * 3 + 1,
      vx: (Math.random() - 0.5) * 0.0005,
      vy: -Math.random() * 0.001 - 0.0002,
      alpha: Math.random() * 0.6 + 0.2,
      color: '#ffffff'
    });
  }
  return particles;
}

export function renderParticles(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy: be, particles } = ctx;
  const theme = config.theme;

  for (let i = 0; i < particles.length; i++) {
    const p = particles[i];
    p.x += p.vx;
    p.y += p.vy;
    if (p.y < -0.05) { p.y = 1.05; p.x = Math.random(); }

    const bx = p.x * width;
    const by = p.y * height;
    const r = p.radius + be * 3;
    const a = Math.min(1, p.alpha + be * 0.3);

    c.globalAlpha = a * 0.15;
    c.fillStyle = theme.accentColor;
    c.beginPath();
    c.arc(bx, by, r * 3, 0, Math.PI * 2);
    c.fill();

    c.globalAlpha = a;
    c.shadowBlur = 6;
    c.shadowColor = theme.accentColor;
    c.beginPath();
    c.arc(bx, by, r, 0, Math.PI * 2);
    c.fill();
    c.shadowBlur = 0;
  }
  c.globalAlpha = 1;
}
