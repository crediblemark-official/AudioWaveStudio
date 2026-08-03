import { RenderContext } from './types';
import { TextAlign, TextBlock } from '../../types/visualizer';

const FADE_MS = 0.8;

let playStartFrame = 0;
let wasPlaying = false;

/**
 * Reset the fade-in state so a fresh session (e.g. export) starts the fade
 * from frame 0 instead of inheriting a stale `playStartFrame` from live
 * playback (which would leave fadeIn text invisible for a long stretch).
 */
export function resetTextFadeState() {
  playStartFrame = 0;
  wasPlaying = false;
}

function fadeFactor(isPlaying: boolean, frameTime: number): number {
  if (isPlaying && !wasPlaying) {
    playStartFrame = frameTime;
  }
  wasPlaying = isPlaying;
  if (!isPlaying) return 1;
  return Math.min(1, Math.max(0, (frameTime - playStartFrame) / FADE_MS));
}

function applyTransform(text: string, transform: TextBlock['transform']): string {
  switch (transform) {
    case 'uppercase':
      return text.toUpperCase();
    case 'lowercase':
      return text.toLowerCase();
    case 'capitalize':
      return text.replace(/\b\p{L}/gu, (ch) => ch.toUpperCase());
    default:
      return text;
  }
}

function wrapText(c: CanvasRenderingContext2D, text: string, maxWidthPx: number): string[] {
  const paragraphs = text.split('\n');
  const lines: string[] = [];
  for (const paragraph of paragraphs) {
    if (paragraph === '') {
      lines.push('');
      continue;
    }
    if (maxWidthPx <= 0) {
      lines.push(paragraph);
      continue;
    }
    const words = paragraph.split(/\s+/);
    let line = '';
    for (const word of words) {
      const candidate = line ? `${line} ${word}` : word;
      if (!line || c.measureText(candidate).width <= maxWidthPx) {
        line = candidate;
      } else {
        lines.push(line);
        line = word;
      }
    }
    lines.push(line);
  }
  return lines;
}

interface DrawLineOptions {
  fill: string | CanvasGradient;
  outline?: { color: string; width: number };
  letterSpacing: number;
  wave: boolean;
  waveAmp: number;
  charIndexStart: number;
  now: number;
  bass: number;
}

function drawLine(
  c: CanvasRenderingContext2D,
  text: string,
  anchorX: number,
  y: number,
  align: TextAlign,
  o: DrawLineOptions,
) {
  const needsChars = o.letterSpacing > 0 || o.wave;
  c.textAlign = needsChars ? 'left' : align;
  if (!needsChars) {
    if (o.outline) {
      c.lineWidth = o.outline.width;
      c.strokeStyle = o.outline.color;
      c.lineJoin = 'round';
      c.strokeText(text, anchorX, y);
    }
    c.fillStyle = o.fill;
    c.fillText(text, anchorX, y);
    return;
  }

  const chars = Array.from(text);
  const widths = chars.map((ch) => c.measureText(ch).width);
  const spacing = o.letterSpacing;
  const totalWidth = widths.reduce((a, b) => a + b, 0) + spacing * Math.max(0, chars.length - 1);

  let startX = anchorX;
  if (align === 'right') startX = anchorX - totalWidth;
  else if (align === 'center') startX = anchorX - totalWidth / 2;

  let cx = startX;
  for (let i = 0; i < chars.length; i++) {
    if (i > 0) cx += spacing;
    let cy = y;
    if (o.wave) {
      cy += Math.sin(o.now * 5 + (o.charIndexStart + i) * 0.6) * o.waveAmp * (0.4 + o.bass * 0.6);
    }
    if (o.outline) {
      c.lineWidth = o.outline.width;
      c.strokeStyle = o.outline.color;
      c.lineJoin = 'round';
      c.strokeText(chars[i], cx, cy);
    }
    c.fillStyle = o.fill;
    c.fillText(chars[i], cx, cy);
    cx += widths[i];
  }
}

function drawBlock(
  c: CanvasRenderingContext2D,
  width: number,
  height: number,
  block: TextBlock,
  font: string,
  now: number,
  bassEnergy: number,
  globalFade: number,
) {
  if (!block.text.trim() || block.opacity <= 0) return;

  const react = Math.min(1, Math.max(0, bassEnergy)) * (block.reactiveScale || 0);
  const fontSize = block.fontSize * (1 + react * 0.5);
  const lineHeight = fontSize * (block.lineHeight || 1.2);
  const maxWidthPx = block.maxWidth > 0 ? (block.maxWidth / 100) * width : 0;
  const fontFamily = block.fontFamily || font;

  c.font = `${block.italic ? 'italic ' : ''}${block.fontWeight || 700} ${fontSize}px ${fontFamily}`;

  const text = applyTransform(block.text, block.transform);
  const lines = wrapText(c, text, maxWidthPx);

  const anchorX = (block.positionX / 100) * width;
  const anchorY = (block.positionY / 100) * height;

  c.save();
  c.globalAlpha = block.opacity * (block.fadeIn ? globalFade : 1);

  const shadowColor = block.useGradient ? block.gradientEnd : block.color;
  if (block.shadow) {
    c.shadowBlur = block.shadowBlur + (block.glowIntensity || 0);
    c.shadowColor = shadowColor;
    c.shadowOffsetX = block.shadowOffsetX || 0;
    c.shadowOffsetY = block.shadowOffsetY || 0;
  } else if ((block.glowIntensity || 0) > 0) {
    c.shadowBlur = block.glowIntensity;
    c.shadowColor = shadowColor;
    c.shadowOffsetX = 0;
    c.shadowOffsetY = 0;
  }

  let charIndex = 0;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const y = anchorY + i * lineHeight;
    if (line === '') {
      charIndex += 1;
      continue;
    }

    const lineWidth = c.measureText(line).width;
    let fill: string | CanvasGradient = block.color;
    if (block.useGradient) {
      const lineStartX = block.align === 'left' ? anchorX : block.align === 'right' ? anchorX - lineWidth : anchorX - lineWidth / 2;
      const centerX = lineStartX + lineWidth / 2;
      const angle = ((block.gradientAngle || 0) * Math.PI) / 180;
      const dx = Math.cos(angle);
      const dy = Math.sin(angle);
      const span = Math.max(lineWidth, 8);
      const g = c.createLinearGradient(
        centerX - (dx * span) / 2,
        anchorY - (dy * span) / 2,
        centerX + (dx * span) / 2,
        anchorY + (dy * span) / 2,
      );
      g.addColorStop(0, block.gradientStart);
      g.addColorStop(1, block.gradientEnd);
      fill = g;
    }

    drawLine(c, line, anchorX, y, block.align, {
      fill,
      outline: block.outline ? { color: block.outlineColor, width: block.outlineWidth } : undefined,
      letterSpacing: block.letterSpacing || 0,
      wave: block.waveEffect,
      waveAmp: fontSize * 0.12,
      charIndexStart: charIndex,
      now,
      bass: Math.min(1, Math.max(0, bassEnergy)),
    });
    charIndex += Array.from(line).length;
  }

  c.restore();
}

export function renderTextOverlay(ctx: RenderContext) {
  const { ctx: c, width, height, config } = ctx;
  const txt = config.text;
  const font = txt.fontFamily || '"Outfit", "Inter", sans-serif';

  const blocks: TextBlock[] = [];
  if (txt.showTitle && txt.songTitle?.trim()) {
    blocks.push({ ...txt.title, text: txt.songTitle });
  }
  if (txt.showArtist && txt.artistName?.trim()) {
    blocks.push({ ...txt.artist, text: txt.artistName });
  }
  for (const b of txt.blocks) {
    if (b.enabled && b.text?.trim()) blocks.push(b);
  }
  if (blocks.length === 0) return;

  const globalFade = fadeFactor(ctx.isPlaying, ctx.frameTime);
  const now = ctx.frameTime;

  for (const block of blocks) {
    drawBlock(c, width, height, block, font, now, ctx.bassEnergy, globalFade);
  }
}
