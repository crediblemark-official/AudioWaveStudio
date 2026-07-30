import { RenderContext } from './types';

interface Spark {
  x: number; y: number; vx: number; vy: number;
  life: number; maxLife: number; size: number; color: string;
  decay: number; trail: { x: number; y: number }[];
}

interface LightMote {
  x: number; y: number; vx: number; vy: number;
  size: number; alpha: number; phase: number;
  hue: number;
}

let sparks: Spark[] = [];
let motes: LightMote[] = [];
let prevBeat = 0;
let peaks: { x: number; y: number; alpha: number }[] = [];

export function renderThreeD(r: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy: be, beatStrength: bs } = r;
  const barCount = Math.min(48, config.reactivity.barCount);
  const sensitivity = config.reactivity.sensitivity;
  const theme = config.theme;

  const centerX = width / 2;
  const floorY = height * 0.78;
  const persp = 0.3;
  const maxBarW = 18;
  const gap = 2;
  const barStep = maxBarW + gap;

  const step = Math.floor(freqData.length / barCount);

  if (motes.length === 0) {
    for (let i = 0; i < 50; i++) {
      motes.push({
        x: Math.random() * width, y: Math.random() * height * 0.7,
        vx: (Math.random() - 0.5) * 0.2, vy: -0.05 - Math.random() * 0.15,
        size: 1 + Math.random() * 2.5, alpha: 0.15 + Math.random() * 0.35,
        phase: Math.random() * Math.PI * 2, hue: Math.random() * 60 + 200,
      });
    }
  }

  if (bs > 0.1 && bs > prevBeat) {
    const barHs: number[] = [];
    for (let i = 0; i < barCount; i++) {
      let val = 0;
      for (let j = 0; j < step; j++) val += freqData[i * step + j] || 0;
      barHs.push((val / step / 255) * sensitivity);
    }

    const sparkCount = Math.floor(8 + bs * 25);
    for (let s = 0; s < sparkCount; s++) {
      const bi = Math.floor(Math.random() * barCount);
      const bh = barHs[bi] * height * 0.38;
      const x = centerX - ((barCount * barStep) / 2) + bi * barStep + (Math.random() - 0.5) * 12;
      const cd = (x - centerX) / (centerX);
      const ps = 1 - Math.abs(cd) * persp;
      const py = floorY - bh * ps - 5;
      const hue = 200 + (bi / barCount) * 160;
      sparks.push({
        x, y: py, vx: (Math.random() - 0.5) * 4, vy: -3 - Math.random() * 5,
        life: 0, maxLife: 25 + Math.random() * 40, size: 1.5 + Math.random() * 3,
        color: `hsl(${hue}, 100%, ${60 + Math.random() * 30}%)`,
        decay: 0.96 + Math.random() * 0.03, trail: [],
      });
    }
  }
  prevBeat = bs;

  for (let i = sparks.length - 1; i >= 0; i--) {
    const sp = sparks[i];
    sp.trail.push({ x: sp.x, y: sp.y });
    if (sp.trail.length > 6) sp.trail.shift();
    sp.life++;
    sp.x += sp.vx;
    sp.vy *= sp.decay;
    sp.vy += 0.04;
    sp.y += sp.vy;
    sp.vx *= 0.99;
    if (sp.life > sp.maxLife || sp.y > floorY || sp.y < -30) {
      sparks.splice(i, 1);
    }
  }

  for (const m of motes) {
    m.x += m.vx + Math.sin(r.rotationAngle + m.phase) * 0.15;
    m.y += m.vy + Math.cos(r.rotationAngle * 0.5 + m.phase) * 0.05;
    if (m.y < -15) { m.y = height * 0.7; m.x = Math.random() * width; }
    if (m.x < -15) m.x = width + 15;
    if (m.x > width + 15) m.x = -15;
  }

  c.save();

  c.shadowBlur = 0;
  c.globalAlpha = 1;

  const floorGrad = c.createLinearGradient(0, floorY, 0, height);
  floorGrad.addColorStop(0, `rgba(25, 15, 45, 0.4)`);
  floorGrad.addColorStop(0.4, `rgba(15, 10, 35, 0.15)`);
  floorGrad.addColorStop(1, `rgba(5, 5, 15, 0)`);
  c.fillStyle = floorGrad;
  c.fillRect(0, floorY, width, height - floorY);

  c.strokeStyle = `rgba(120, 80, 200, 0.05)`;
  c.lineWidth = 1;
  for (let i = -35; i <= 35; i++) {
    const gx = centerX + i * 12;
    if (gx < 0 || gx > width) continue;
    c.beginPath();
    c.moveTo(gx, floorY + Math.abs(i) * 0.25);
    c.lineTo(gx, height);
    c.stroke();
  }

  const totalW = (maxBarW + gap) * barCount;
  const startX = centerX - totalW / 2;

  r.rotationAngle += 0.002 + be * 0.004;

  const bars: { x: number; by: number; bh: number; bw: number; dx: number; dy: number; ps: number; val: number }[] = [];

  for (let i = 0; i < barCount; i++) {
    let val = 0;
    for (let j = 0; j < step; j++) val += freqData[i * step + j] || 0;
    val = (val / step / 255) * sensitivity;
    const barH = Math.max(2, val * height * 0.38);

    const x = startX + i * (maxBarW + gap);
    const cd = (x - centerX) / (totalW / 2);
    const ps = 1 - Math.abs(cd) * persp;

    const bw = Math.max(2, maxBarW * ps);
    const bh = barH * ps;
    const bx = x;
    const by = floorY - bh;
    const depth = Math.max(1, bw * 0.4 * ps);
    const dx = depth * 0.7;
    const dy = depth * 0.5;
    const pulse = 1 + be * 0.2 + Math.sin(r.rotationAngle + i * 0.3) * be * 0.05;

    bars.push({ x: bx, by, bh, bw, dx, dy, ps, val });

    const freqRatio = i / barCount;
    const hue = 200 + freqRatio * 140;
    const sat = 85 + be * 15;
    const lit = 40 + val * 40;
    const barColor = `hsl(${hue}, ${sat}%, ${lit}%)`;
    const topColor = `hsl(${hue + 20}, ${sat}%, ${lit + 15}%)`;
    const sideColor = `hsl(${hue - 10}, ${sat}%, ${lit - 10}%)`;

    const brightBoost = 0.3 + be * 0.3 + (bs > 0.12 ? bs * 0.5 : 0);

    c.shadowBlur = 0;

    c.fillStyle = barColor;
    c.fillRect(bx, by, bw, bh);

    c.beginPath();
    c.moveTo(bx, by);
    c.lineTo(bx + dx, by - dy * pulse);
    c.lineTo(bx + bw + dx, by - dy * pulse);
    c.lineTo(bx + bw, by);
    c.closePath();
    c.fillStyle = topColor;
    c.globalAlpha = 0.6 + be * 0.15;
    c.fill();

    c.beginPath();
    c.moveTo(bx + bw, by);
    c.lineTo(bx + bw + dx, by - dy * pulse);
    c.lineTo(bx + bw + dx, floorY - dy * pulse);
    c.lineTo(bx + bw, floorY);
    c.closePath();
    c.fillStyle = sideColor;
    c.globalAlpha = 0.4 + be * 0.15;
    c.fill();

    c.shadowBlur = 12 + be * 18;
    c.shadowColor = theme.glowColor;
    c.fillStyle = barColor;
    c.globalAlpha = brightBoost * 0.4;
    c.fillRect(bx, by, bw, Math.min(bh, 5));
    c.shadowBlur = 6 + be * 8;
    c.fillRect(bx, by, bw, bh);
    c.shadowBlur = 0;
    c.globalAlpha = 1;

    if (be > 0.25) {
      c.fillStyle = `rgba(255, 255, 255, ${(be - 0.25) * 0.35})`;
      c.fillRect(bx, by, bw, Math.min(bh, 2));
    }
  }

  peaks = peaks.filter(p => p.alpha > 0.01);
  for (const b of bars) {
    if (b.val * 0.35 > 0.8) {
      const existing = peaks.find(p => Math.abs(p.x - (b.x + b.bw / 2)) < 5);
      if (!existing) {
        peaks.push({ x: b.x + b.bw / 2, y: b.by, alpha: 1 });
      }
    }
  }
  for (const p of peaks) {
    p.alpha *= 0.94;
    p.y -= 0.3;
  }

  c.shadowBlur = 0;
  c.globalAlpha = 1;

  c.beginPath();
  for (let i = 0; i < bars.length; i++) {
    const b = bars[i];
    const tipX = b.x + b.bw / 2;
    const tipY = b.by - b.dy * (1 + be * 0.2 + Math.sin(r.rotationAngle + i * 0.3) * be * 0.05);
    if (i === 0) c.moveTo(tipX, tipY);
    else c.lineTo(tipX, tipY);
  }
  c.strokeStyle = theme.primaryColor;
  c.globalAlpha = 0.2 + be * 0.2;
  c.lineWidth = 2;
  c.shadowBlur = 10;
  c.shadowColor = theme.glowColor;
  c.stroke();
  c.shadowBlur = 0;
  c.globalAlpha = 1;

  c.shadowBlur = 0;
  c.globalAlpha = 1;

  for (let i = 0; i < bars.length; i++) {
    const b = bars[i];
    if (b.val < 0.05) continue;
    const tipX = b.x + b.bw / 2;
    const tipY = b.by - b.dy * (1 + be * 0.2 + Math.sin(r.rotationAngle + i * 0.3) * be * 0.05);

    c.fillStyle = theme.glowColor;
    c.globalAlpha = b.val * 0.15 + be * 0.1;
    c.shadowBlur = 20;
    c.shadowColor = theme.glowColor;
    c.beginPath();
    c.arc(tipX, tipY, 2 + b.val * 3, 0, Math.PI * 2);
    c.fill();
    c.shadowBlur = 0;
  }
  c.globalAlpha = 1;

  for (const p of peaks) {
    c.fillStyle = theme.accentColor;
    c.globalAlpha = p.alpha * 0.6;
    c.shadowBlur = 8;
    c.shadowColor = theme.accentColor;
    c.beginPath();
    c.arc(p.x, p.y, 2.5, 0, Math.PI * 2);
    c.fill();
  }
  c.shadowBlur = 0;
  c.globalAlpha = 1;

  for (const sp of sparks) {
    const progress = sp.life / sp.maxLife;
    c.globalAlpha = (1 - progress) * 0.7;
    for (let t = 0; t < sp.trail.length; t++) {
      const tp = t / sp.trail.length;
      c.fillStyle = sp.color;
      c.globalAlpha = (1 - progress) * tp * 0.4;
      const ts = sp.size * tp * 0.5;
      c.beginPath();
      c.arc(sp.trail[t].x, sp.trail[t].y, ts, 0, Math.PI * 2);
      c.fill();
    }
    c.globalAlpha = 1 - progress;
    c.fillStyle = sp.color;
    c.shadowBlur = 10;
    c.shadowColor = sp.color;
    c.beginPath();
    c.arc(sp.x, sp.y, sp.size * (1 - progress * 0.4), 0, Math.PI * 2);
    c.fill();
  }

  c.shadowBlur = 0;
  c.globalAlpha = 0.5;

  for (let i = 0; i < bars.length; i++) {
    const b = bars[i];
    if (b.val < 0.03) continue;
    const freqRatio = i / barCount;
    const hue = 200 + freqRatio * 140;
    const refY = floorY + 4;
    const refH = b.bh * 0.25;
    const refAlpha = Math.max(0, 0.08 - Math.abs(i - barCount / 2) * 0.003);

    c.fillStyle = `hsl(${hue}, 80%, 40%)`;
    c.globalAlpha = refAlpha;
    c.fillRect(b.x, refY, b.bw, refH);
  }
  c.globalAlpha = 1;

  for (const m of motes) {
    c.globalAlpha = m.alpha * (0.4 + Math.sin(r.rotationAngle * 2 + m.phase) * 0.25);
    c.fillStyle = `hsl(${m.hue + be * 20}, 80%, 70%)`;
    c.shadowBlur = 4;
    c.shadowColor = c.fillStyle;
    c.beginPath();
    c.arc(m.x, m.y, m.size * (0.8 + be * 0.3), 0, Math.PI * 2);
    c.fill();
  }

  c.shadowBlur = 0;
  c.globalAlpha = 1;
  c.restore();
}
