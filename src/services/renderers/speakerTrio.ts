import { RenderContext } from './types';

interface FloatingNote {
  x: number;
  y: number;
  vx: number;
  vy: number;
  symbol: string;
  size: number;
  alpha: number;
  rotation: number;
  rotSpeed: number;
}

let floatingNotes: FloatingNote[] = [];
const NOTE_SYMBOLS = ['♪', '♫', '♬', '♩', '∮', '🎼'];

function initNotes(width: number, height: number) {
  if (floatingNotes.length > 0) return;
  for (let i = 0; i < 18; i++) {
    floatingNotes.push({
      x: Math.random() * width,
      y: Math.random() * height,
      vx: (Math.random() - 0.5) * 0.8,
      vy: -0.5 - Math.random() * 1.2,
      symbol: NOTE_SYMBOLS[Math.floor(Math.random() * NOTE_SYMBOLS.length)],
      size: 14 + Math.random() * 18,
      alpha: 0.3 + Math.random() * 0.5,
      rotation: (Math.random() - 0.5) * 0.5,
      rotSpeed: (Math.random() - 0.5) * 0.02,
    });
  }
}

export function renderSpeakerTrio(r: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy: be, beatStrength: bs } = r;
  const sensitivity = config.reactivity.sensitivity;
  const theme = config.theme;

  initNotes(width, height);

  const centerX = width / 2;
  const centerY = height / 2;
  const baseR = Math.min(width, height) * 0.14;

  c.save();

  // --- PASS 1: BACKGROUND HALFTONE / PIXEL MATRIX EQUALIZER (SYMMETRICAL MIRRORED BARS) ---
  const halfCount = 36;
  const step = Math.max(1, Math.floor((freqData.length * 0.5) / halfCount));
  const startX = width * 0.03;
  const halfW = (centerX - startX);

  // Horizontal central baseline
  c.shadowBlur = 15 + be * 15;
  c.shadowColor = theme.glowColor || '#FF0055';
  c.strokeStyle = theme.primaryColor || '#FF3366';
  c.lineWidth = 2;
  c.beginPath();
  c.moveTo(0, centerY);
  c.lineTo(width, centerY);
  c.stroke();

  c.shadowBlur = 6;
  c.shadowColor = theme.primaryColor || '#FF0055';

  for (let i = 0; i < halfCount; i++) {
    let sum = 0;
    for (let j = 0; j < step; j++) {
      sum += freqData[i * step + j] || 0;
    }
    const val = (sum / step / 255) * sensitivity;
    if (val < 0.01) continue;

    const barH = val * height * 0.35;
    const dotSize = 4;
    const dotGap = 6;
    const dotsCount = Math.min(Math.floor(barH / dotGap), 8);

    const xLeft = centerX - (i / (halfCount - 1)) * halfW;
    const xRight = centerX + (i / (halfCount - 1)) * halfW;

    for (let d = 0; d < dotsCount; d++) {
      const yUp = centerY - d * dotGap;
      const yDown = centerY + d * dotGap;
      const ratio = d / dotsCount;

      let dotColor = '#FF0055';
      if (ratio > 0.75) dotColor = '#FFFFFF';
      else if (ratio > 0.45) dotColor = '#FF6600';
      else dotColor = theme.primaryColor || '#FF0055';

      c.fillStyle = dotColor;
      c.fillRect(xLeft, yUp, dotSize, dotSize);
      c.fillRect(xLeft, yDown, dotSize, dotSize);
      if (i > 0) {
        c.fillRect(xRight, yUp, dotSize, dotSize);
        c.fillRect(xRight, yDown, dotSize, dotSize);
      }
    }
  }

  c.shadowBlur = 0;

  // --- PASS 2: FLOATING ANIMATED MUSICAL NOTES ---
  c.font = '24px sans-serif';
  c.textAlign = 'center';
  c.textBaseline = 'middle';
  c.shadowBlur = 4;
  c.shadowColor = 'rgba(0, 0, 0, 0.4)';

  for (let i = 0; i < floatingNotes.length; i++) {
    const n = floatingNotes[i];
    n.y += n.vy - be * 1.5;
    n.x += n.vx + Math.sin(n.y * 0.02) * 0.5;
    n.rotation += n.rotSpeed;

    if (n.y < -30) {
      n.y = height + 20;
      n.x = Math.random() * width;
    }

    c.save();
    c.translate(n.x, n.y);
    c.rotate(n.rotation);
    c.fillStyle = `rgba(30, 30, 40, ${n.alpha})`;
    c.fillText(n.symbol, 0, 0);
    c.restore();
  }

  c.textAlign = 'start';
  c.textBaseline = 'alphabetic';

  // --- PASS 3: TRIPLE SPEAKER ASSEMBLY (LEFT, RIGHT, CENTER) ---
  const leftX = centerX - baseR * 1.25;
  const rightX = centerX + baseR * 1.25;

  const leftR = baseR * 0.82 * (1 + be * 0.08);
  const rightR = baseR * 0.82 * (1 + be * 0.08);
  const centerR = baseR * 1.12 * (1 + be * 0.14 + bs * 0.08);

  const drawWoofer = (x: number, y: number, r: number, isCenter: boolean) => {
    c.save();

    // Soft drop shadow under speaker
    c.shadowBlur = isCenter ? 25 : 18;
    c.shadowColor = 'rgba(0, 0, 0, 0.6)';

    // 1. Metallic Rim
    const outerR = r;
    const innerR = r * 0.86;
    const metallicGrad = c.createLinearGradient(x - r, y - r, x + r, y + r);
    metallicGrad.addColorStop(0.0, '#FFFFFF');
    metallicGrad.addColorStop(0.2, '#999999');
    metallicGrad.addColorStop(0.5, '#222222');
    metallicGrad.addColorStop(0.8, '#CCCCCC');
    metallicGrad.addColorStop(1.0, '#444444');

    c.fillStyle = metallicGrad;
    c.beginPath();
    c.arc(x, y, outerR, 0, Math.PI * 2);
    c.arc(x, y, innerR, 0, Math.PI * 2, true);
    c.closePath();
    c.fill();

    // Metallic Screws
    const boltR = (outerR + innerR) / 2;
    for (let a = 0; a < 4; a++) {
      const angle = a * (Math.PI / 2);
      const bx = x + Math.cos(angle) * boltR;
      const by = y + Math.sin(angle) * boltR;
      c.fillStyle = '#DDDDDD';
      c.beginPath();
      c.arc(bx, by, 3, 0, Math.PI * 2);
      c.fill();
    }

    // 2. Rubber Surround
    const surroundInnerR = r * 0.72;
    const rubberGrad = c.createRadialGradient(x, y, surroundInnerR, x, y, innerR);
    rubberGrad.addColorStop(0, '#1A1A1E');
    rubberGrad.addColorStop(0.5, '#3A3A40');
    rubberGrad.addColorStop(1, '#101014');

    c.fillStyle = rubberGrad;
    c.beginPath();
    c.arc(x, y, innerR, 0, Math.PI * 2);
    c.arc(x, y, surroundInnerR, 0, Math.PI * 2, true);
    c.closePath();
    c.fill();

    // 3. Concentric Ring Diaphragm Cone
    const coneInnerR = r * 0.32;
    const coneGrad = c.createRadialGradient(x - r * 0.2, y - r * 0.2, coneInnerR * 0.5, x, y, surroundInnerR);
    coneGrad.addColorStop(0, '#444855');
    coneGrad.addColorStop(0.6, '#22242C');
    coneGrad.addColorStop(1, '#111216');

    c.fillStyle = coneGrad;
    c.beginPath();
    c.arc(x, y, surroundInnerR, 0, Math.PI * 2);
    c.fill();

    // Concentric Ridges / Ribbed Ring Lines on Cone
    c.strokeStyle = 'rgba(255, 255, 255, 0.08)';
    c.lineWidth = 1.2;
    for (let ring = coneInnerR + 6; ring < surroundInnerR - 4; ring += 10) {
      c.beginPath();
      c.arc(x, y, ring, 0, Math.PI * 2);
      c.stroke();
    }

    // 4. Center Dust Cap
    const dustR = coneInnerR * (1 + be * 0.06);
    const dustGrad = c.createRadialGradient(x - dustR * 0.3, y - dustR * 0.3, 0, x, y, dustR);
    dustGrad.addColorStop(0, '#666A78');
    dustGrad.addColorStop(0.4, '#30333D');
    dustGrad.addColorStop(1, '#0C0D10');

    c.shadowBlur = 10;
    c.shadowColor = '#000000';
    c.fillStyle = dustGrad;
    c.beginPath();
    c.arc(x, y, dustR, 0, Math.PI * 2);
    c.fill();

    // Crescent Specular Highlight
    c.fillStyle = 'rgba(255, 255, 255, 0.35)';
    c.beginPath();
    c.arc(x - dustR * 0.15, y - dustR * 0.15, dustR * 0.65, Math.PI * 1.0, Math.PI * 1.85);
    c.arc(x - dustR * 0.15, y - dustR * 0.15, dustR * 0.45, Math.PI * 1.85, Math.PI * 1.0, true);
    c.closePath();
    c.fill();

    c.restore();
  };

  // Render Left & Right Woofers first (behind center)
  drawWoofer(leftX, centerY, leftR, false);
  drawWoofer(rightX, centerY, rightR, false);

  // Render Center Main Subwoofer (in front)
  drawWoofer(centerX, centerY, centerR, true);

  c.restore();
}
