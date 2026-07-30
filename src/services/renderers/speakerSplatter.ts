import { RenderContext } from './types';

let staticSplatterDots: { x: number; y: number; r: number }[] = [];
let arcRotation = 0;

function initSplatterElements(centerX: number, centerY: number, baseR: number) {
  if (staticSplatterDots.length > 0) return;

  for (let i = 0; i < 45; i++) {
    const angle = Math.random() * Math.PI * 2;
    const dist = baseR * (0.4 + Math.random() * 1.3);
    const x = centerX + Math.cos(angle) * dist;
    const y = centerY + Math.sin(angle) * dist + baseR * 0.2;
    const r = 1.2 + Math.random() * 4.5;
    staticSplatterDots.push({ x, y, r });
  }
}

function hexToRgb(hex: string): [number, number, number] {
  const h = hex.replace('#', '');
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
}

export function renderSpeakerSplatter(r: RenderContext) {
  const { ctx: c, width, height, config, freqData, bassEnergy: be, beatStrength: bs } = r;
  const sensitivity = config.reactivity.sensitivity;
  const theme = config.theme;
  const [pR, pG, pB] = hexToRgb(theme.primaryColor);
  const [sR, sG, sB] = hexToRgb(theme.secondaryColor);
  const [aR, aG, aB] = hexToRgb(theme.accentColor);
  const [gR, gG, gB] = hexToRgb(theme.glowColor);

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

  // --- PASS 1: NEON GRADIENT ARCS (SOFT FEATHERED) ---
  const pulse = 1 + be * 0.18 + bs * 0.12;
  const arcAlpha = 0.25 + be * 0.25 + freqAvg * 0.15;
  const wide = 1 + be * 0.15;

  arcRotation += 0.02;

  const softStroke = (
    cx: number, cy: number, radius: number,
    startAngle: number, endAngle: number, color: string, alpha: number,
  ) => {
    const layers = [
      { w: 10, a: 0.08 },
      { w: 6, a: 0.15 },
      { w: 2, a: 0.4 },
    ];
    for (const layer of layers) {
      c.lineWidth = layer.w;
      c.strokeStyle = `rgba(${color}, ${alpha * layer.a})`;
      c.beginPath();
      c.arc(cx, cy, radius, startAngle, endAngle);
      c.stroke();
    }
  };

  c.save();
  c.shadowBlur = 30;
  c.lineCap = 'round';

  // Primary arc (top-left)
  c.shadowColor = theme.primaryColor;
  for (let r = 1; r <= 4; r++) {
    const radius = baseR * (1.0 + r * 0.35) * pulse;
    const spread = (Math.PI * 0.60) * wide;
    const baseAngle = Math.PI * 1.18;
    const fade = Math.max(0.08, 1 - (r - 1) * 0.28);
    softStroke(centerX - 6, centerY - 4, radius,
      baseAngle - spread * 0.5 + arcRotation,
      baseAngle + spread * 0.5 + arcRotation,
      `${pR}, ${pG}, ${pB}`, arcAlpha * fade);
  }

  // Secondary arc (bottom-right)
  c.shadowColor = theme.secondaryColor;
  for (let r = 1; r <= 4; r++) {
    const radius = baseR * (1.0 + r * 0.35) * pulse;
    const spread = (Math.PI * 0.56) * wide;
    const baseAngle = Math.PI * 0.30;
    const fade = Math.max(0.08, 1 - (r - 1) * 0.28);
    softStroke(centerX + 6, centerY + 4, radius,
      baseAngle - spread * 0.5 - arcRotation,
      baseAngle + spread * 0.5 - arcRotation,
      `${sR}, ${sG}, ${sB}`, arcAlpha * fade);
  }

  // Accent arc (top-right)
  c.shadowColor = theme.accentColor;
  for (let r = 1; r <= 3; r++) {
    const radius = baseR * (1.1 + r * 0.38) * pulse;
    const spread = (Math.PI * 0.46) * wide;
    const baseAngle = -Math.PI * 0.25;
    const fade = Math.max(0.08, 1 - (r - 1) * 0.4);
    softStroke(centerX + 2, centerY - 2, radius,
      baseAngle - spread * 0.5 + arcRotation,
      baseAngle + spread * 0.5 + arcRotation,
      `${aR}, ${aG}, ${aB}`, arcAlpha * fade * 0.7);
  }

  c.restore();

  // --- PASS 2: LIGHT BURST FROM BEHIND SPEAKERS ---
  const glowIntensity = 0.3 + be * 0.25 + bs * 0.15;
  const glowRadius = baseR * 2.8 * pulse;

  for (const pos of [
    { x: centerX, y: centerY },
    { x: centerX - baseR * 0.92, y: centerY + baseR * 0.06 },
    { x: centerX + baseR * 0.92, y: centerY + baseR * 0.06 },
  ]) {
    const grad = c.createRadialGradient(pos.x, pos.y, 0, pos.x, pos.y, glowRadius);
    grad.addColorStop(0, `rgba(${gR}, ${gG}, ${gB}, ${glowIntensity * 0.5})`);
    grad.addColorStop(0.15, `rgba(${pR}, ${pG}, ${pB}, ${glowIntensity * 0.2})`);
    grad.addColorStop(0.4, `rgba(${sR}, ${sG}, ${sB}, ${glowIntensity * 0.08})`);
    grad.addColorStop(1, 'rgba(0, 0, 0, 0)');
    c.fillStyle = grad;
    c.globalCompositeOperation = 'screen';
    c.beginPath();
    c.arc(pos.x, pos.y, glowRadius, 0, Math.PI * 2);
    c.fill();
  }
  c.globalCompositeOperation = 'source-over';

  // --- PASS 3: DARK INK SPLATTER POOL & PAINT DRIPS UNDER SPEAKERS ---
  const inkY = centerY + baseR * 0.25;
  c.fillStyle = '#14141A';
  c.beginPath();
  c.ellipse(centerX, inkY, baseR * 1.4, baseR * 0.75, 0, 0, Math.PI * 2);
  c.fill();

  // Scattered Splatter Dots around central splash
  const splatterColors = [theme.primaryColor, theme.secondaryColor, theme.accentColor];
  for (let i = 0; i < staticSplatterDots.length; i++) {
    const s = staticSplatterDots[i];
    const sx = centerX + (s.x - centerX) * (1 + be * 0.15);
    const sy = centerY + (s.y - centerY) * (1 + be * 0.15);
    c.fillStyle = splatterColors[i % 3];
    c.beginPath();
    c.arc(sx, sy, s.r * (1 + bs * 0.3), 0, Math.PI * 2);
    c.fill();
  }

  // Vertical Paint Drips extending down
  c.strokeStyle = theme.accentColor;
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
    c.shadowColor = theme.glowColor;
    c.beginPath();
    c.moveTo(dx, dy);
    c.lineTo(dx, dy + len);
    c.stroke();

    // Round drop tip
    c.fillStyle = theme.accentColor;
    c.beginPath();
    c.arc(dx, dy + len + thick * 0.5, thick * 1.2, 0, Math.PI * 2);
    c.fill();
  }

  // --- PASS 3: TRIPLE CLUSTERED CHROME SPEAKERS (SCALED DOWN) ---
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
