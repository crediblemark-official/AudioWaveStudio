import { RenderContext } from './types';

const SYMBOLS = ['♩', '♪', '♫', '♬'];

export function renderMusicNotes(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy: be, musicNotes } = ctx;
  const bg = config.background;
  const density = bg.musicNoteDensity ?? 0.5;
  const noteSize = bg.musicNoteSize ?? 24;
  const noteSpeed = bg.musicNoteSpeed ?? 1.0;
  const maxNotes = bg.musicNoteCount ?? 80;
  const color = bg.musicNoteColor || config.theme.accentColor;
  const style = bg.musicNoteStyle ?? 'float';
  const bassMult = config.reactivity.bassMultiplier;

  let midSum = 0, highSum = 0;
  const flen = freqData.length;
  const midBins = Math.min(32, flen);
  const highBins = Math.min(64, flen);
  for (let i = 8; i < midBins; i++) midSum += freqData[i];
  for (let i = 24; i < highBins; i++) highSum += freqData[i];
  const midEnergy = midSum / (midBins * 255);
  const highEnergy = highSum / (Math.max(1, highBins - 24) * 255);
  const wobbleAmp = highEnergy * 3;

  const spawnChance = density * 0.12 + be * 0.4 * bassMult;
  if (Math.random() < spawnChance) {
    const count = Math.min(
      Math.floor(1 + midEnergy * 2 + be * 2),
      maxNotes - musicNotes.length,
    );
    const phaseStep = (Math.PI * 2) / Math.max(1, count);
    const baseVY = -(0.8 + Math.random() * 1.5 + be * 3 * bassMult) * noteSpeed;
    for (let n = 0; n < count; n++) {
      musicNotes.push({
        x: Math.random() * width,
        y: height + 30,
        vx: (Math.random() - 0.5) * 2,
        vy: baseVY * (1 + Math.random() * 0.3),
        size: noteSize * (0.5 + Math.random() * 0.7 + be * 0.4),
        alpha: 0.5 + Math.random() * 0.5,
        rotation: (Math.random() - 0.5) * 0.3,
        symbol: SYMBOLS[Math.floor(Math.random() * 4)],
        life: 0,
        maxLife: 100 + Math.random() * 80,
        baseX: Math.random() * width,
        phase: n * phaseStep,
      });
    }
  }

  const speedBoost = 1 + be * 1.2 * bassMult;

  c.textAlign = 'center';
  c.textBaseline = 'middle';
  c.fillStyle = color;

  const useGlow = be > 0.15 && musicNotes.length > 0;
  if (useGlow) {
    c.shadowColor = color;
    c.shadowBlur = 12;
  }

  let alive = 0;
  for (let i = 0; i < musicNotes.length; i++) {
    const n = musicNotes[i];
    n.life++;
    if (n.life >= n.maxLife) continue;

    const t = n.life / n.maxLife;
    const fadeOut = Math.min(1, (n.maxLife - n.life) / 30);
    const alpha = n.alpha * fadeOut;

    switch (style) {
      case 'bounce':
        n.y += n.vy * speedBoost * 0.6;
        n.x += Math.sin(n.life * 0.06 + n.phase) * wobbleAmp * speedBoost;
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
        n.x += (n.vx + Math.sin(n.life * 0.1) * 0.5) * speedBoost * (1 + be);
        n.y += n.vy * speedBoost * (1 + be * 0.5) + 0.5;
        break;
      default:
        n.x += (n.vx + Math.sin(n.life * 0.08) * wobbleAmp) * speedBoost;
        n.y += n.vy * speedBoost;
        break;
    }

    n.rotation += 0.01 + be * 0.04 * bassMult;
    const pulse = 1 + Math.sin(n.life * 0.1) * 0.1 + be * 0.2;
    const sz = n.size * pulse;

    c.globalAlpha = alpha;
    c.font = `${sz}px serif`;

    c.save();
    c.translate(n.x, n.y);
    c.rotate(n.rotation);
    c.fillText(n.symbol, 0, 0);
    if (useGlow) {
      c.shadowBlur = 0;
      c.globalAlpha = alpha * 0.35;
      c.fillText(n.symbol, 0, 0);
      c.shadowBlur = 12;
    }
    c.restore();

    if (alive !== i) musicNotes[alive] = n;
    alive++;
  }

  musicNotes.length = alive;

  c.setTransform(1, 0, 0, 1, 0, 0);
  c.globalAlpha = 1;
  c.shadowBlur = 0;
}
