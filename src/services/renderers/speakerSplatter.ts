import { RenderContext } from './types';

interface DotParticle {
  x: number;
  y: number;
  r: number;
  color: string;
}

let staticDotsLeft: DotParticle[] = [];
let staticDotsRight: DotParticle[] = [];
let staticDotsTop: DotParticle[] = [];
let staticSplatterDots: DotParticle[] = [];

function initSplatterElements(centerX: number, centerY: number, baseR: number) {
  if (staticDotsLeft.length > 0) return;

  // 1. Blue Halftone Arc Array (Top-Left)
  for (let ring = 1; ring <= 6; ring++) {
    const radius = baseR * (0.8 + ring * 0.35);
    const dotsInRing = 6 + ring * 2;
    const startAngle = Math.PI * 0.95;
    const endAngle = Math.PI * 1.45;
    const dotR = 1.8 + ring * 1.3;

    for (let i = 0; i < dotsInRing; i++) {
      const angle = startAngle + (i / (dotsInRing - 1)) * (endAngle - startAngle);
      const x = centerX + Math.cos(angle) * radius - ring * 3;
      const y = centerY + Math.sin(angle) * radius - ring * 2;
      staticDotsLeft.push({ x, y, r: dotR, color: i % 2 === 0 ? '#00B0FF' : '#00E5FF' });
    }
  }

  // 2. Magenta Halftone Arc Array (Bottom-Right)
  for (let ring = 1; ring <= 6; ring++) {
    const radius = baseR * (0.8 + ring * 0.35);
    const dotsInRing = 6 + ring * 2;
    const startAngle = Math.PI * 0.05;
    const endAngle = Math.PI * 0.55;
    const dotR = 1.8 + ring * 1.3;

    for (let i = 0; i < dotsInRing; i++) {
      const angle = startAngle + (i / (dotsInRing - 1)) * (endAngle - startAngle);
      const x = centerX + Math.cos(angle) * radius + ring * 3;
      const y = centerY + Math.sin(angle) * radius + ring * 3;
      staticDotsRight.push({ x, y, r: dotR, color: i % 2 === 0 ? '#FF007F' : '#FF40A0' });
    }
  }

  // 3. White & Silver Halftone Array (Top-Right)
  for (let ring = 1; ring <= 5; ring++) {
    const radius = baseR * (0.9 + ring * 0.38);
    const dotsInRing = 5 + ring * 2;
    const startAngle = -Math.PI * 0.45;
    const endAngle = -Math.PI * 0.05;
    const dotR = 2.2 + ring * 1.3;

    for (let i = 0; i < dotsInRing; i++) {
      const angle = startAngle + (i / (dotsInRing - 1)) * (endAngle - startAngle);
      const x = centerX + Math.cos(angle) * radius;
      const y = centerY + Math.sin(angle) * radius;
      const color = i % 2 === 0 ? '#FFFFFF' : '#888899';
      staticDotsTop.push({ x, y, r: dotR, color });
    }
  }

  // 4. Random Splatter Dots
  for (let i = 0; i < 45; i++) {
    const angle = Math.random() * Math.PI * 2;
    const dist = baseR * (0.4 + Math.random() * 1.3);
    const x = centerX + Math.cos(angle) * dist;
    const y = centerY + Math.sin(angle) * dist + baseR * 0.2;
    const r = 1.2 + Math.random() * 4.5;
    staticSplatterDots.push({ x, y, r, color: Math.random() > 0.4 ? '#FFFFFF' : '#444455' });
  }
}

export function renderSpeakerSplatter(r: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy: be, beatStrength: bs } = r;
  const sensitivity = config.reactivity.sensitivity;

  const centerX = width / 2;
  const centerY = height / 2;
  const maxDim = Math.min(width, height);
  // Compact scaled-down radius
  const baseR = maxDim * 0.13;

  initSplatterElements(centerX, centerY, baseR);

  let freqAvg = 0;
  for (let i = 0; i < freqData.length; i++) {
    freqAvg += freqData[i] || 0;
  }
  freqAvg = (freqAvg / freqData.length / 255) * sensitivity;

  c.save();

  // --- PASS 1: HALFTONE DOT ARRAYS (NEON GLOW ON BLACK BG) ---
  const pulseScale = 1.0 + be * 0.10 + bs * 0.06;

  // Blue Halftone Dots (Top-Left)
  c.shadowBlur = 8;
  for (let i = 0; i < staticDotsLeft.length; i++) {
    const dot = staticDotsLeft[i];
    const dx = centerX + (dot.x - centerX) * pulseScale;
    const dy = centerY + (dot.y - centerY) * pulseScale;
    const rSize = dot.r * (1 + be * 0.25 + freqAvg * 0.2);

    c.shadowColor = dot.color;
    c.fillStyle = dot.color;
    c.beginPath();
    c.arc(dx, dy, Math.max(1, rSize), 0, Math.PI * 2);
    c.fill();
  }

  // Magenta Halftone Dots (Bottom-Right)
  for (let i = 0; i < staticDotsRight.length; i++) {
    const dot = staticDotsRight[i];
    const dx = centerX + (dot.x - centerX) * pulseScale;
    const dy = centerY + (dot.y - centerY) * pulseScale;
    const rSize = dot.r * (1 + be * 0.25 + freqAvg * 0.2);

    c.shadowColor = dot.color;
    c.fillStyle = dot.color;
    c.beginPath();
    c.arc(dx, dy, Math.max(1, rSize), 0, Math.PI * 2);
    c.fill();
  }

  // White & Silver Halftone Dots (Top-Right)
  c.shadowBlur = 6;
  for (let i = 0; i < staticDotsTop.length; i++) {
    const dot = staticDotsTop[i];
    const dx = centerX + (dot.x - centerX) * pulseScale;
    const dy = centerY + (dot.y - centerY) * pulseScale;
    const rSize = dot.r * (1 + be * 0.2 + freqAvg * 0.2);

    c.shadowColor = dot.color;
    c.fillStyle = dot.color;
    c.beginPath();
    c.arc(dx, dy, Math.max(1, rSize), 0, Math.PI * 2);
    c.fill();
  }

  // --- PASS 2: GREY BRUSH SMEAR (LOWER-LEFT) ---
  c.save();
  c.shadowBlur = 0;
  c.fillStyle = 'rgba(80, 85, 100, 0.45)';
  c.translate(centerX - baseR * 1.3, centerY + baseR * 0.9);
  c.rotate(-Math.PI * 0.22);
  c.beginPath();
  c.ellipse(0, 0, baseR * 1.4, baseR * 0.45, 0, 0, Math.PI * 2);
  c.fill();
  c.restore();

  // --- PASS 3: DARK INK SPLATTER POOL & PAINT DRIPS UNDER SPEAKERS ---
  const inkY = centerY + baseR * 0.25;
  c.fillStyle = '#14141A';
  c.beginPath();
  c.ellipse(centerX, inkY, baseR * 1.4, baseR * 0.75, 0, 0, Math.PI * 2);
  c.fill();

  // Scattered Splatter Dots around central splash
  for (let i = 0; i < staticSplatterDots.length; i++) {
    const s = staticSplatterDots[i];
    const sx = centerX + (s.x - centerX) * (1 + be * 0.15);
    const sy = centerY + (s.y - centerY) * (1 + be * 0.15);
    c.fillStyle = s.color;
    c.beginPath();
    c.arc(sx, sy, s.r * (1 + bs * 0.3), 0, Math.PI * 2);
    c.fill();
  }

  // Vertical Paint Drips extending down
  c.strokeStyle = '#FFFFFF';
  c.lineCap = 'round';
  const dripXs = [-0.85, -0.55, -0.15, 0.15, 0.5, 0.8];
  const dripLens = [30, 55, 75, 45, 65, 25];

  for (let d = 0; d < dripXs.length; d++) {
    const dx = centerX + dripXs[d] * baseR;
    const dy = inkY + baseR * 0.2;
    const len = dripLens[d] * (1 + be * 0.3);
    const thick = 2.5 + (d % 3);

    c.lineWidth = thick;
    c.shadowBlur = 6;
    c.shadowColor = '#FFFFFF';
    c.beginPath();
    c.moveTo(dx, dy);
    c.lineTo(dx, dy + len);
    c.stroke();

    // Round drop tip
    c.fillStyle = '#FFFFFF';
    c.beginPath();
    c.arc(dx, dy + len + thick * 0.5, thick * 1.2, 0, Math.PI * 2);
    c.fill();
  }

  // --- PASS 4: TRIPLE CLUSTERED CHROME SPEAKERS (SCALED DOWN) ---
  const centerR = baseR * 1.15 * (1 + be * 0.10);
  const leftR = baseR * 0.88 * (1 + be * 0.07);
  const rightR = baseR * 0.88 * (1 + be * 0.07);

  const leftX = centerX - baseR * 0.92;
  const leftY = centerY + baseR * 0.06;

  const rightX = centerX + baseR * 0.92;
  const rightY = centerY + baseR * 0.06;

  const drawSplatterWoofer = (x: number, y: number, r: number, isCenter: boolean) => {
    c.save();

    // 1. Drop Shadow
    c.shadowBlur = isCenter ? 25 : 18;
    c.shadowColor = 'rgba(0, 0, 0, 0.95)';
    c.shadowOffsetY = 6;

    // 2. Metallic Chrome Outer Rim
    const outerR = r;
    const innerR = r * 0.86;
    const metallicGrad = c.createLinearGradient(x - r, y - r, x + r, y + r);
    metallicGrad.addColorStop(0.0, '#FFFFFF');
    metallicGrad.addColorStop(0.2, '#AAAAAA');
    metallicGrad.addColorStop(0.45, '#222226');
    metallicGrad.addColorStop(0.75, '#DDDDDD');
    metallicGrad.addColorStop(1.0, '#55555A');

    c.fillStyle = metallicGrad;
    c.beginPath();
    c.arc(x, y, outerR, 0, Math.PI * 2);
    c.arc(x, y, innerR, 0, Math.PI * 2, true);
    c.closePath();
    c.fill();

    // 3. Metallic Bolts
    const boltR = (outerR + innerR) / 2;
    for (let a = 0; a < 4; a++) {
      const angle = a * (Math.PI / 2);
      const bx = x + Math.cos(angle) * boltR;
      const by = y + Math.sin(angle) * boltR;
      c.fillStyle = '#E5E5EA';
      c.beginPath();
      c.arc(bx, by, 2.5, 0, Math.PI * 2);
      c.fill();
    }

    // 4. Rubber Surround Ring
    const surroundInnerR = r * 0.72;
    const rubberGrad = c.createRadialGradient(x, y, surroundInnerR, x, y, innerR);
    rubberGrad.addColorStop(0, '#1C1C20');
    rubberGrad.addColorStop(0.5, '#3C3C44');
    rubberGrad.addColorStop(1, '#0F0F12');

    c.fillStyle = rubberGrad;
    c.beginPath();
    c.arc(x, y, innerR, 0, Math.PI * 2);
    c.arc(x, y, surroundInnerR, 0, Math.PI * 2, true);
    c.closePath();
    c.fill();

    // 5. Concentric Ribbed Cone Diaphragm
    const coneInnerR = r * 0.30;
    const coneGrad = c.createRadialGradient(x - r * 0.2, y - r * 0.2, coneInnerR * 0.5, x, y, surroundInnerR);
    coneGrad.addColorStop(0, '#444856');
    coneGrad.addColorStop(0.6, '#22242D');
    coneGrad.addColorStop(1, '#0E0F14');

    c.fillStyle = coneGrad;
    c.beginPath();
    c.arc(x, y, surroundInnerR, 0, Math.PI * 2);
    c.fill();

    // Ribbed Ring Grooves
    c.strokeStyle = 'rgba(255, 255, 255, 0.15)';
    c.lineWidth = 1.0;
    for (let ring = coneInnerR + 4; ring < surroundInnerR - 2; ring += 8) {
      c.beginPath();
      c.arc(x, y, ring, 0, Math.PI * 2);
      c.stroke();
    }

    // 6. Center Dust Cap Cone
    const dustR = coneInnerR * (1 + be * 0.05);
    const dustGrad = c.createRadialGradient(x - dustR * 0.3, y - dustR * 0.3, 0, x, y, dustR);
    dustGrad.addColorStop(0, '#666A78');
    dustGrad.addColorStop(0.4, '#30333E');
    dustGrad.addColorStop(1, '#0C0D10');

    c.shadowBlur = 8;
    c.shadowColor = '#000000';
    c.fillStyle = dustGrad;
    c.beginPath();
    c.arc(x, y, dustR, 0, Math.PI * 2);
    c.fill();

    // 7. Glossy Crescent Glare Spot
    c.fillStyle = 'rgba(255, 255, 255, 0.45)';
    c.beginPath();
    c.arc(x - dustR * 0.15, y - dustR * 0.15, dustR * 0.65, Math.PI * 1.0, Math.PI * 1.85);
    c.arc(x - dustR * 0.15, y - dustR * 0.15, dustR * 0.45, Math.PI * 1.85, Math.PI * 1.0, true);
    c.closePath();
    c.fill();

    c.restore();
  };

  // Render Left & Right Woofers first (behind)
  drawSplatterWoofer(leftX, leftY, leftR, false);
  drawSplatterWoofer(rightX, rightY, rightR, false);

  // Render Center Main Woofer (in front)
  drawSplatterWoofer(centerX, centerY, centerR, true);

  c.restore();
}
