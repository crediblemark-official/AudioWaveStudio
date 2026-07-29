import { RenderContext } from './types';

function makeParticle() {
  return {
    x: Math.random(),
    y: Math.random(),
    radius: Math.random() * 3 + 1,
    vx: (Math.random() - 0.5) * 0.0005,
    vy: -Math.random() * 0.001 - 0.0002,
    alpha: Math.random() * 0.6 + 0.2,
    color: '#ffffff'
  };
}

export function initParticles() {
  return Array.from({ length: 60 }, makeParticle);
}

export function renderParticles(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy: be, beatStrength: bs, particles } = ctx;
  const bg = config.background;
  const color = bg.particleColor || config.theme.accentColor;
  const style = bg.particleStyle || 'float';
  const speed = bg.particleSpeed ?? 1.0;
  const size = bg.particleSize ?? 4;
  const targetCount = bg.particleCount ?? 60;

  while (particles.length < targetCount) particles.push(makeParticle());
  while (particles.length > targetCount) particles.pop();

  for (let i = 0; i < particles.length; i++) {
    const p = particles[i];

    if (ctx.isPlaying) {
      if (style === 'bounce') {
        p.vy += (Math.random() - 0.5) * (be * 0.003 + bs * 0.025) * speed;
        p.vx += (Math.random() - 0.5) * (be * 0.003 + bs * 0.025) * speed;
        p.vx *= 1 - 0.03 * speed;
        p.vy *= 1 - 0.03 * speed;
      } else if (style === 'wave') {
        const wavePhase = (i / particles.length) * Math.PI * 2 + performance.now() * 0.001 * speed;
        p.vx += (Math.cos(wavePhase) * be * 0.001 + (Math.random() - 0.5) * bs * 0.02) * speed;
        p.vy += (Math.sin(wavePhase) * be * 0.001 + (Math.random() - 0.5) * bs * 0.02) * speed;
        p.vx *= 1 - 0.02 * speed;
        p.vy *= 1 - 0.02 * speed;
      } else if (style === 'static') {
        p.vx *= 0.9;
        p.vy *= 0.9;
      } else if (style === 'confined') {
        const beatKick = bs * 0.04 * speed;
        p.vx += (p.x - 0.5) * beatKick + (Math.random() - 0.5) * be * 0.004 * speed;
        p.vy += (p.y - 0.5) * beatKick + (Math.random() - 0.5) * be * 0.004 * speed - 0.0003;
        p.vx *= 1 - 0.02 * speed;
        p.vy *= 1 - 0.02 * speed;
      } else {
        const beatKick = bs * 0.015 * speed;
        p.vy += -beatKick + (Math.random() - 0.5) * be * 0.002 * speed;
        p.vx += (Math.random() - 0.5) * be * 0.002 * speed;
        p.vx *= 1 - 0.02 * speed;
        p.vy *= 1 - 0.02 * speed;
      }

      p.x += p.vx;
      p.y += p.vy;
    }

    if (style === 'confined') {
      if (p.x < 0.01) { p.x = 0.01; if (ctx.isPlaying) p.vx = Math.abs(p.vx) * 0.85; }
      if (p.x > 0.99) { p.x = 0.99; if (ctx.isPlaying) p.vx = -Math.abs(p.vx) * 0.85; }
      if (p.y < 0.01) { p.y = 0.01; if (ctx.isPlaying) p.vy = Math.abs(p.vy) * 0.85; }
      if (p.y > 0.99) { p.y = 0.99; if (ctx.isPlaying) p.vy = -Math.abs(p.vy) * 0.85; }
    } else {
      if (p.y < -0.05) { p.y = 1.05; p.x = Math.random(); p.vx = (Math.random() - 0.5) * 0.0005; p.vy = -Math.random() * 0.001 - 0.0002; }
      if (p.x < -0.05) { p.x = 1.05; }
      if (p.x > 1.05) { p.x = -0.05; }
    }

    const bx = p.x * width;
    const by = p.y * height;
    const r = (p.radius + be * 5 + bs * 8) * (size / 4);
    const a = Math.min(1, p.alpha + be * 0.5 + bs * 0.6);

    c.globalAlpha = a;
    c.fillStyle = color;
    c.beginPath();
    c.arc(bx, by, r, 0, Math.PI * 2);
    c.fill();
  }
  c.globalAlpha = 1;
}
