import { RenderContext } from './types';

const SYMBOLS = ['♩', '♪', '♫', '♬'];

export function renderMusicNotes(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData, beatStrength: bs, musicNotes } = ctx;
  const bg = config.background;
  const density = bg.musicNoteDensity ?? 1.0;
  const noteSize = bg.musicNoteSize ?? 60;
  const maxNotes = Math.min(bg.musicNoteCount ?? 80, 80);
  const sensitivity = bg.musicNoteSensitivity ?? 1.0;
  const color = bg.musicNoteColor || config.theme.accentColor;
  const style = bg.musicNoteStyle ?? 'float';

  let highSum = 0;
  const flen = freqData.length;
  const highBins = Math.min(64, flen);
  for (let i = 24; i < highBins; i++) highSum += freqData[i];
  const highEnergy = highSum / (Math.max(1, highBins - 24) * 255);
  const wobbleAmp = highEnergy * 3;

  const isConfined = style === 'confined';
  if (bs > 0.05 && Math.random() < Math.min(1, density * 0.5 + bs * 0.5)) {
    const count = Math.min(
      Math.floor(1 + bs * 3),
      maxNotes - musicNotes.length,
      isConfined ? 1 : 3,
    );
    const phaseStep = (Math.PI * 2) / Math.max(1, count);
    const baseVY = -(3 + Math.random() * 3 + bs * 12);
    for (let n = 0; n < count; n++) {
      musicNotes.push({
        x: Math.random() * width,
        y: isConfined ? Math.random() * height : height,
        vx: isConfined ? (Math.random() - 0.5) * 3 : (Math.random() - 0.5) * 2,
        vy: isConfined ? (Math.random() - 0.5) * 3 : baseVY * (1 + Math.random() * 0.3),
        size: noteSize * (0.5 + Math.random() * 0.7 + bs * 0.5),
        alpha: 0.5 + Math.random() * 0.5,
        rotation: (Math.random() - 0.5) * 0.3,
        symbol: SYMBOLS[Math.floor(Math.random() * 4)],
        life: 0,
        maxLife: isConfined ? 100 + Math.random() * 50 : 60 + Math.random() * 30,
        baseX: Math.random() * width,
        phase: n * phaseStep,
      });
    }
  }

  const speedBoost = 1.5 + bs * 4 * sensitivity;

  c.textAlign = 'center';
  c.textBaseline = 'middle';
  c.shadowBlur = 0;

  const isHex = color.startsWith('#');
  const hexNum = isHex ? parseInt(color.replace('#', ''), 16) : 0;
  const cr = isHex ? (hexNum >> 16) & 255 : 255;
  const cg = isHex ? (hexNum >> 8) & 255 : 230;
  const cb = isHex ? hexNum & 255 : 0;

  let alive = 0;
  let lastFontSz = 0;

  for (let i = 0; i < musicNotes.length; i++) {
    const n = musicNotes[i];
    n.life++;
    if (n.life >= n.maxLife) continue;

    const t = n.life / n.maxLife;
    const fadeOut = Math.min(1, (n.maxLife - n.life) / 10);
    const alpha = n.alpha * fadeOut;

    switch (style) {
      case 'bounce':
        n.y += n.vy * speedBoost * 0.6;
        n.x += Math.sin(n.life * 0.06 + n.phase) * wobbleAmp * 0.5;
        n.vy += 0.3;
        break;
      case 'spiral': {
        const r = t * Math.min(width, height) * 0.4;
        const a = n.life * 0.04 + n.phase;
        n.x = n.baseX + Math.cos(a) * r * 0.3;
        n.y = height * 0.5 - t * height * 0.3 + Math.sin(a) * r * 0.1;
        break;
      }
      case 'wave':
        n.y += n.vy * speedBoost * 0.5;
        n.x = n.baseX + Math.sin(n.life * 0.05 + n.phase) * width * 0.2;
        break;
      case 'burst':
        n.x += (n.vx + Math.sin(n.life * 0.1) * 0.5) * speedBoost;
        n.y += n.vy * speedBoost + 0.5;
        break;
      case 'confined': {
        n.vx += (Math.random() - 0.5) * 0.2 + (Math.random() - 0.5) * bs * 2;
        n.vy += (Math.random() - 0.5) * 0.2 + (Math.random() - 0.5) * bs * 2;
        const maxV = 4;
        n.vx = Math.max(-maxV, Math.min(maxV, n.vx));
        n.vy = Math.max(-maxV, Math.min(maxV, n.vy));
        n.x += n.vx * speedBoost;
        n.y += n.vy * speedBoost;
        if (n.x < 0) { n.x = 0; n.vx = Math.abs(n.vx); }
        else if (n.x > width) { n.x = width; n.vx = -Math.abs(n.vx); }
        if (n.y < 0) { n.y = 0; n.vy = Math.abs(n.vy); }
        else if (n.y > height) { n.y = height; n.vy = -Math.abs(n.vy); }
        break;
      }
      default:
        n.x += n.vx * speedBoost;
        n.y += n.vy * speedBoost;
        break;
    }

    n.rotation += 0.02;
    const pulse = 1 + Math.sin(n.life * 0.1) * 0.1;
    const sz = n.size * pulse;

    const cos = Math.cos(n.rotation);
    const sin = Math.sin(n.rotation);
    c.setTransform(cos, sin, -sin, cos, n.x, n.y);

    const fontSz = Math.round(sz);
    if (fontSz !== lastFontSz) {
      c.font = `${fontSz}px serif`;
      lastFontSz = fontSz;
    }

    c.fillStyle = `rgba(${cr},${cg},${cb},${alpha})`;
    c.fillText(n.symbol, 0, 0);

    if (alive !== i) musicNotes[alive] = n;
    alive++;
  }

  musicNotes.length = alive;

  c.setTransform(1, 0, 0, 1, 0, 0);
}
