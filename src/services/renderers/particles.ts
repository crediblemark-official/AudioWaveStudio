import { RenderContext, Particle } from './types';

function makeParticle(): Particle {
  const angle = Math.random() * Math.PI * 2;
  const speed = 0.0004 + Math.random() * 0.0004; // Smooth, slow ambient rolling speed
  return {
    x: 0.05 + Math.random() * 0.9,
    y: 0.05 + Math.random() * 0.9,
    radius: Math.random() * 3 + 1,
    vx: Math.cos(angle) * speed,
    vy: Math.sin(angle) * speed,
    alpha: Math.random() * 0.5 + 0.3,
    color: '#ffffff',
    phase: Math.random() * Math.PI * 2,
  };
}

export function initParticles(): Particle[] {
  return Array.from({ length: 60 }, makeParticle);
}

export function renderParticles(ctx: RenderContext) {
  const { ctx: c, width, height, config, beatStrength: bs, freqData, particles } = ctx;
  const bg = config.background;
  const color = bg.particleColor || config.theme?.accentColor || '#00f0ff';
  const style = bg.particleStyle || 'float';
  const speed = Math.max(0.1, bg.particleSpeed ?? 1.0);
  const size = Math.max(1, bg.particleSize ?? 4);
  const targetCount = Math.max(5, bg.particleCount ?? 60);

  // Dynamically scale array size to particleCount setting
  while (particles.length < targetCount) particles.push(makeParticle());
  while (particles.length > targetCount) particles.pop();

  const isPlaying = ctx.isPlaying;

  // Calculate percussive kick drum ("beat dug") & drum hit energy exclusively
  let kickSum = 0;
  const kickBins = Math.min(12, freqData ? freqData.length : 0);
  if (isPlaying && kickBins > 0) {
    for (let k = 0; k < kickBins; k++) {
      kickSum += freqData[k];
    }
  }
  const kickEnergy = (kickBins > 0 && isPlaying) ? (kickSum / (kickBins * 255)) : 0;

  // Percussive onset trigger (ONLY triggers on drum/percussion hits, NOT on vocals/melodies)
  const isPercussiveHit = isPlaying && (bs > 0.12 || (kickEnergy > 0.4 && bs > 0.08));
  const percussiveImpact = isPercussiveHit ? Math.max(bs * 1.2, kickEnergy * 0.8) : 0;

  const kagetForce = percussiveImpact * 0.025 * speed;
  const scatterForce = percussiveImpact * 0.03 * speed;

  for (let i = 0; i < particles.length; i++) {
    const p = particles[i];
    if (!p) continue;
    if (p.phase === undefined) p.phase = Math.random() * Math.PI * 2;

    // Advance phase smoothly for continuous slow rolling ambient motion
    p.phase += 0.012 * speed;

    // Smooth ambient rolling base velocities
    const rollAngle = p.phase + i * 0.3;
    const baseRollX = Math.cos(rollAngle) * 0.0003 * speed;
    const baseRollY = Math.sin(rollAngle) * 0.0003 * speed;

    // Apply vibration ONLY on percussive drum/kick hits ("beat dug")
    const drumVibX = isPercussiveHit ? (Math.random() - 0.5) * percussiveImpact * 0.018 * speed : 0;
    const drumVibY = isPercussiveHit ? (Math.random() - 0.5) * percussiveImpact * 0.018 * speed : 0;

    if (style === 'confined') {
      p.vx += baseRollX + drumVibX;
      p.vy += baseRollY + drumVibY;

      if (isPercussiveHit) {
        // Scatter outward from center ONLY on drum/kick hit ("beat dug")
        const dx = p.x - 0.5;
        const dy = p.y - 0.5;
        const dist = Math.hypot(dx, dy) || 1;
        const scatterX = (dx / dist) * scatterForce;
        const scatterY = (dy / dist) * scatterForce;

        const kagetDir = (i % 2 === 0 ? 1 : -1);
        p.vx += scatterX + (Math.random() - 0.5) * kagetForce * kagetDir;
        p.vy += scatterY + (Math.random() - 0.5) * kagetForce * kagetDir;
      }

      p.vx *= 0.94;
      p.vy *= 0.94;

    } else if (style === 'bounce') {
      p.vx += baseRollX * 0.6 + drumVibX;
      p.vy += baseRollY * 0.6 + drumVibY;

      if (isPercussiveHit) {
        const scatterAngle = Math.random() * Math.PI * 2;
        p.vx += Math.cos(scatterAngle) * scatterForce;
        p.vy += Math.sin(scatterAngle) * scatterForce;
      }

      p.vx *= 0.95;
      p.vy *= 0.95;

    } else if (style === 'wave') {
      const waveY = -0.0005 * speed + Math.sin(p.phase) * 0.0004 * speed;
      const waveX = Math.cos(p.phase * 0.7) * 0.0006 * speed;

      p.vx += (waveX - p.vx) * 0.1 + drumVibX;
      p.vy += (waveY - p.vy) * 0.1 + drumVibY;

      if (isPercussiveHit) {
        p.vy -= kagetForce * 1.5;
        p.vx += (Math.random() - 0.5) * scatterForce * 1.5;
      }

    } else if (style === 'static') {
      const hoverX = Math.cos(p.phase) * 0.00025 * speed;
      const hoverY = Math.sin(p.phase) * 0.00025 * speed;
      p.vx = hoverX + drumVibX;
      p.vy = hoverY + drumVibY;

      if (isPercussiveHit) {
        p.vx += (Math.random() - 0.5) * kagetForce * 1.2;
        p.vy += (Math.random() - 0.5) * kagetForce * 1.2;
      }

    } else { // 'float' default
      const floatUp = -0.0006 * speed;
      const floatSway = Math.sin(p.phase) * 0.0004 * speed;

      p.vy += (floatUp - p.vy) * 0.08 + drumVibY;
      p.vx += (floatSway - p.vx) * 0.08 + drumVibX;

      if (isPercussiveHit) {
        p.vy -= kagetForce * 1.8;
        p.vx += (Math.random() - 0.5) * scatterForce * 1.8;
      }
    }

    // Minimum ambient rolling speed so particles float smoothly during non-percussive audio
    const minRoll = 0.0003 * speed;
    const maxRoll = 0.018 * speed;
    const curSpeed = Math.hypot(p.vx, p.vy);
    if (curSpeed < minRoll) {
      const a = Math.random() * Math.PI * 2;
      p.vx += Math.cos(a) * minRoll;
      p.vy += Math.sin(a) * minRoll;
    } else if (curSpeed > maxRoll) {
      p.vx = (p.vx / curSpeed) * maxRoll;
      p.vy = (p.vy / curSpeed) * maxRoll;
    }

    // Update positions smoothly
    p.x += p.vx;
    p.y += p.vy;

    // Boundaries check per movement style
    if (style === 'confined' || style === 'bounce') {
      if (p.x < 0.03) { p.x = 0.03; p.vx = Math.abs(p.vx) * 0.9; }
      if (p.x > 0.97) { p.x = 0.97; p.vx = -Math.abs(p.vx) * 0.9; }
      if (p.y < 0.03) { p.y = 0.03; p.vy = Math.abs(p.vy) * 0.9; }
      if (p.y > 0.97) { p.y = 0.97; p.vy = -Math.abs(p.vy) * 0.9; }
    } else {
      // Screen wrap for float, wave, static
      if (p.y < -0.05) { p.y = 1.05; p.x = Math.random(); p.vy = -Math.random() * 0.0008 - 0.0003; }
      if (p.y > 1.05) { p.y = -0.05; p.x = Math.random(); }
      if (p.x < -0.05) { p.x = 1.05; }
      if (p.x > 1.05) { p.x = -0.05; }
    }

    // Render particle on canvas (size pulse & glow flash ONLY on drum/kick hits)
    const bx = p.x * width;
    const by = p.y * height;
    const baseRadius = (p.radius * 0.5 + 1.2) * (size / 4);
    
    // Size pulse ONLY on kick drum / percussive hits
    const beatPulse = isPercussiveHit ? (percussiveImpact * 10) * (size / 4) : 0;
    const r = Math.max(0.5, baseRadius + beatPulse);

    // Alpha glow flash ONLY on kick drum / percussive hits
    const alpha = Math.min(1, p.alpha + (isPercussiveHit ? percussiveImpact * 0.6 : 0));

    c.globalAlpha = alpha;
    c.fillStyle = color;
    c.beginPath();
    c.arc(bx, by, r, 0, Math.PI * 2);
    c.fill();
  }
  c.globalAlpha = 1;
}
