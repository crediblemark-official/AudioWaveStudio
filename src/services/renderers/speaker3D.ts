import { RenderContext } from './types';

let _cachedH = 0;
let _barGrad: CanvasGradient | null = null;

export function renderSpeaker3D(r: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy: be, beatStrength: bs } = r;
  const sensitivity = config.reactivity.sensitivity;
  const theme = config.theme;

  const centerX = width / 2;
  const centerY = height / 2;

  const baseRadius = Math.min(width, height) * 0.27;
  const bassPulse = 1.0 + be * 0.12 + bs * 0.08;
  const speakerR = baseRadius * bassPulse;

  function hexToRgb(hex: string): [number, number, number] {
    const h = hex.replace('#', '');
    return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
  }
  const [pR, pG, pB] = hexToRgb(theme.primaryColor);
  const [sR, sG, sB] = hexToRgb(theme.secondaryColor);
  const [aR, aG, aB] = hexToRgb(theme.accentColor);

  if (!_barGrad || _cachedH !== height) {
    _cachedH = height;
    _barGrad = c.createLinearGradient(0, 0, 0, height);
    _barGrad.addColorStop(0, `rgba(${pR}, ${pG}, ${pB}, 0.85)`);
    _barGrad.addColorStop(0.3, `rgba(${sR}, ${sG}, ${sB}, 0.95)`);
    _barGrad.addColorStop(0.6, `rgba(${aR}, ${aG}, ${aB}, 0.98)`);
    _barGrad.addColorStop(0.85, `rgba(${sR}, ${sG}, ${sB}, 0.95)`);
    _barGrad.addColorStop(1, `rgba(${pR}, ${pG}, ${pB}, 0.85)`);
  }

  c.save();

  // --- PASS 1: BACKGROUND HIGH-DENSITY FIERY SPECTRUM BARS (LEFT & RIGHT WINGS) ---
  const halfBars = 48;
  const step = Math.max(1, Math.floor((freqData.length * 0.7) / halfBars));

  const leftStart = width * 0.02;
  const leftEnd = Math.max(leftStart + 20, centerX - speakerR * 0.85);
  const leftWidth = leftEnd - leftStart;

  const rightStart = Math.min(width * 0.98 - 20, centerX + speakerR * 0.85);
  const rightEnd = width * 0.98;
  const rightWidth = rightEnd - rightStart;

  const barW = Math.max(2.5, (leftWidth / halfBars) - 2.5);

  // Laser baseline passing behind speaker
  c.shadowBlur = 20 + be * 20;
  c.shadowColor = theme.glowColor || '#FF4400';
  c.strokeStyle = theme.primaryColor || '#FF7700';
  c.lineWidth = 2.2;
  c.beginPath();
  c.moveTo(0, centerY);
  c.lineTo(width, centerY);
  c.stroke();

  c.shadowBlur = 15;
  c.shadowColor = theme.glowColor || '#FF5500';

  for (let i = 0; i < halfBars; i++) {
    let sum = 0;
    for (let j = 0; j < step; j++) {
      sum += freqData[i * step + j] || 0;
    }
    const val = (sum / step / 255) * sensitivity;
    if (val < 0.01) continue;

    const barH = val * height * 0.36;
    const topY = centerY - barH;
    const botY = centerY + barH * 0.82;

    const xLeft = leftEnd - (i / (halfBars - 1)) * leftWidth - barW;
    const xRight = rightStart + (i / (halfBars - 1)) * rightWidth;

    c.fillStyle = _barGrad;
    c.fillRect(xLeft, topY, barW, barH * 1.82);
    c.fillRect(xRight, topY, barW, barH * 1.82);

    c.fillStyle = theme.accentColor;
    c.fillRect(xLeft - 0.5, topY - 1.5, barW + 1, 1.5);
    c.fillRect(xLeft - 0.5, botY, barW + 1, 1.5);
    c.fillRect(xRight - 0.5, topY - 1.5, barW + 1, 1.5);
    c.fillRect(xRight - 0.5, botY, barW + 1, 1.5);
  }

  // --- PASS 2: SPEAKER AURA & LENS FLARES ---
  const glowR = speakerR * 1.4;
  const backGlow = c.createRadialGradient(centerX, centerY, speakerR * 0.5, centerX, centerY, glowR);
  backGlow.addColorStop(0, `rgba(${aR}, ${aG}, ${aB}, 0.85)`);
  backGlow.addColorStop(0.5, `rgba(${sR}, ${sG}, ${sB}, 0.4)`);
  backGlow.addColorStop(1, 'rgba(0, 0, 0, 0)');

  c.shadowBlur = 45 + be * 35;
  c.shadowColor = theme.glowColor;
  c.fillStyle = backGlow;
  c.beginPath();
  c.arc(centerX, centerY, glowR, 0, Math.PI * 2);
  c.fill();

  const flareX = (dx: number) => {
    const fx = centerX + dx;
    const fg = c.createRadialGradient(fx, centerY, 0, fx, centerY, speakerR * 0.45);
    fg.addColorStop(0, 'rgba(255, 255, 255, 0.95)');
    fg.addColorStop(0.2, `rgba(${pR}, ${pG}, ${pB}, 0.85)`);
    fg.addColorStop(0.6, `rgba(${sR}, ${sG}, ${sB}, 0.3)`);
    fg.addColorStop(1, 'rgba(0, 0, 0, 0)');
    c.fillStyle = fg;
    c.beginPath();
    c.arc(fx, centerY, speakerR * 0.45, 0, Math.PI * 2);
    c.fill();
  };
  flareX(-speakerR * 0.96);
  flareX(speakerR * 0.96);

  // --- PASS 3: OUTSIDE CHROME METALLIC RIM ---
  const outerRimR = speakerR;
  const innerRimR = speakerR * 0.88;

  // 3D Metallic Gradient
  const metallicGrad = c.createLinearGradient(
    centerX - outerRimR, centerY - outerRimR,
    centerX + outerRimR, centerY + outerRimR
  );
  metallicGrad.addColorStop(0.0, '#FFFFFF');
  metallicGrad.addColorStop(0.15, '#8E8E93');
  metallicGrad.addColorStop(0.35, '#2C2C2E');
  metallicGrad.addColorStop(0.55, '#D1D1D6');
  metallicGrad.addColorStop(0.75, '#48484A');
  metallicGrad.addColorStop(1.0, '#E5E5EA');

  c.shadowBlur = 18;
  c.shadowColor = '#000000';
  c.fillStyle = metallicGrad;
  c.beginPath();
  c.arc(centerX, centerY, outerRimR, 0, Math.PI * 2);
  c.arc(centerX, centerY, innerRimR, 0, Math.PI * 2, true);
  c.closePath();
  c.fill();

  // Highlight Ring Edges
  c.strokeStyle = 'rgba(255, 255, 255, 0.5)';
  c.lineWidth = 1.5;
  c.beginPath();
  c.arc(centerX, centerY, outerRimR - 1, 0, Math.PI * 2);
  c.stroke();

  c.strokeStyle = 'rgba(0, 0, 0, 0.6)';
  c.lineWidth = 1.5;
  c.beginPath();
  c.arc(centerX, centerY, innerRimR + 1, 0, Math.PI * 2);
  c.stroke();

  // Screws at 12, 3, 6, 9 o'clock
  const boltRadius = (outerRimR + innerRimR) / 2;
  for (let a = 0; a < 4; a++) {
    const angle = a * (Math.PI / 2);
    const bx = centerX + Math.cos(angle) * boltRadius;
    const by = centerY + Math.sin(angle) * boltRadius;

    c.shadowBlur = 4;
    c.shadowColor = '#000000';
    c.fillStyle = '#E5E5EA';
    c.beginPath();
    c.arc(bx, by, 3.8, 0, Math.PI * 2);
    c.fill();

    // Screw center dot & slot line
    c.strokeStyle = '#1C1C1E';
    c.lineWidth = 1.2;
    c.beginPath();
    c.moveTo(bx - 2, by);
    c.lineTo(bx + 2, by);
    c.stroke();
  }

  // --- PASS 4: FLEXIBLE RUBBER SURROUND RING ---
  const surroundOuterR = innerRimR;
  const surroundInnerR = speakerR * 0.74;

  const rubberGrad = c.createRadialGradient(
    centerX, centerY, surroundInnerR,
    centerX, centerY, surroundOuterR
  );
  rubberGrad.addColorStop(0, '#1C1C1E');
  rubberGrad.addColorStop(0.5, '#3A3A3C');
  rubberGrad.addColorStop(1, '#0C0C0E');

  c.fillStyle = rubberGrad;
  c.beginPath();
  c.arc(centerX, centerY, surroundOuterR, 0, Math.PI * 2);
  c.arc(centerX, centerY, surroundInnerR, 0, Math.PI * 2, true);
  c.closePath();
  c.fill();

  // --- PASS 5: POLKA-DOT MESH CONE DIAPHRAGM ---
  const coneOuterR = surroundInnerR;
  const coneInnerR = speakerR * 0.30;

  const coneGrad = c.createRadialGradient(
    centerX - coneOuterR * 0.25, centerY - coneOuterR * 0.25, coneInnerR * 0.4,
    centerX, centerY, coneOuterR
  );
  coneGrad.addColorStop(0, '#48484A');
  coneGrad.addColorStop(0.5, '#2C2C2E');
  coneGrad.addColorStop(1, '#1C1C1E');

  c.fillStyle = coneGrad;
  c.beginPath();
  c.arc(centerX, centerY, coneOuterR, 0, Math.PI * 2);
  c.fill();

  c.fillStyle = 'rgba(255, 255, 255, 0.08)';
  const gridSpacing = 12;
  const dotR = 1.8;
  const minD = coneInnerR * 0.9;
  const maxD = coneOuterR * 0.98;
  const minD2 = minD * minD;
  const maxD2 = maxD * maxD;
  for (let gy = centerY - coneOuterR; gy <= centerY + coneOuterR; gy += gridSpacing) {
    const rowOffset = Math.floor((gy - centerY) / gridSpacing) % 2 === 0 ? 0 : gridSpacing * 0.5;
    for (let gx = centerX - coneOuterR; gx <= centerX + coneOuterR; gx += gridSpacing) {
      const xPos = gx + rowOffset;
      const dx = xPos - centerX;
      const dy = gy - centerY;
      const dist2 = dx * dx + dy * dy;
      if (dist2 >= minD2 && dist2 <= maxD2) {
        c.beginPath();
        c.arc(xPos, gy, dotR, 0, Math.PI * 2);
        c.fill();
      }
    }
  }

  // --- PASS 6: 3D DUST CAP CONE WITH GLOSSY CRESCENT GLARE ---
  const dustCapR = coneInnerR * (1.0 + be * 0.06);

  const dustCapGrad = c.createRadialGradient(
    centerX - dustCapR * 0.3, centerY - dustCapR * 0.3, 0,
    centerX, centerY, dustCapR
  );
  dustCapGrad.addColorStop(0, '#636366');
  dustCapGrad.addColorStop(0.4, '#3A3A3C');
  dustCapGrad.addColorStop(0.85, '#1C1C1E');
  dustCapGrad.addColorStop(1, '#0C0C0E');

  c.shadowBlur = 14 + be * 10;
  c.shadowColor = '#000000';
  c.fillStyle = dustCapGrad;
  c.beginPath();
  c.arc(centerX, centerY, dustCapR, 0, Math.PI * 2);
  c.fill();

  // Glossy Crescent Reflection Arc
  c.fillStyle = 'rgba(255, 255, 255, 0.35)';
  c.beginPath();
  c.arc(centerX - dustCapR * 0.15, centerY - dustCapR * 0.15, dustCapR * 0.65, Math.PI * 1.0, Math.PI * 1.85);
  c.arc(centerX - dustCapR * 0.15, centerY - dustCapR * 0.15, dustCapR * 0.45, Math.PI * 1.85, Math.PI * 1.0, true);
  c.closePath();
  c.fill();

  c.restore();
}

