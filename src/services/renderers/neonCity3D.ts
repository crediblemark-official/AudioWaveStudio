import { RenderContext } from './types';

let frameHistory: Uint8Array[] = [];
const HISTORY_DEPTH = 12;

function hexToRgb(hex: string): [number, number, number] {
  const h = hex.replace('#', '');
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
}

export function renderNeonCity3D(r: RenderContext) {
  const { ctx: c, width, height, config, freqData } = r;
  const barCount = Math.min(64, Math.max(36, config.reactivity.barCount));
  const sensitivity = config.reactivity.sensitivity;
  const theme = config.theme;
  const [pR, pG, pB] = hexToRgb(theme.primaryColor);
  const [sR, sG, sB] = hexToRgb(theme.secondaryColor);

  const centerX = width / 2;
  const floorY = height * 0.58;
  const step = Math.floor(freqData.length / barCount);

  if (frameHistory.length > 0 && frameHistory[0].length !== freqData.length) {
    frameHistory = [];
  }
  frameHistory.unshift(new Uint8Array(freqData));
  if (frameHistory.length > HISTORY_DEPTH) frameHistory.pop();

  c.save();

  const rows = frameHistory.length;
  const cols = barCount;

  const totalAvailableW = width * 0.88;
  const gap = 2;
  const maxBarW = Math.max(4, Math.min(18, (totalAvailableW - cols * gap) / cols));
  const totalW = cols * (maxBarW + gap);
  const startX = centerX - totalW / 2;

  const vals2D: number[][] = [];
  for (let row = 0; row < rows; row++) {
    const data = frameHistory[row];
    const rowVals: number[] = [];
    for (let i = 0; i < cols; i++) {
      let v = 0;
      for (let j = 0; j < step; j++) v += data[i * step + j] || 0;
      const normalized = (v / step / 255) * sensitivity;
      rowVals.push(normalized);
    }
    vals2D.push(rowVals);
  }

  const getColor = (ratio: number, bright: number): [number, number, number] => {
    const t = ratio;
    const cr = Math.round(pR + (sR - pR) * t);
    const cg = Math.round(pG + (sG - pG) * t);
    const cb = Math.round(pB + (sB - pB) * t);
    const f = 0.5 + bright * 0.5;
    return [Math.min(255, Math.round(cr * f)), Math.min(255, Math.round(cg * f)), Math.min(255, Math.round(cb * f))];
  };

  // PASS 1: RENDER MAIN 3D BUILDINGS / COLUMNS (Back to Front)
  for (let row = rows - 1; row >= 0; row--) {
    const depthRatio = row / rows;
    const zOffset = (rows - 1 - row) * 8;
    const rowY = floorY - zOffset * 0.5;
    const scale = 1 - depthRatio * 0.35;
    const rowAlpha = 1 - depthRatio * 0.45;

    for (let i = 0; i < cols; i++) {
      const val = vals2D[row][i];
      if (val < 0.005) continue;

      const freqRatio = i / cols;
      const [cr, cg, cb] = getColor(freqRatio, val);

      const maxH = height * 0.45 * scale;
      const bh = Math.max(2, val * maxH);
      const bw = Math.max(2, maxBarW * scale);

      const x = startX + i * (maxBarW + gap) * scale + (1 - scale) * (totalW / 2);
      const by = rowY - bh;

      const dx = Math.max(1, bw * 0.4);
      const dy = Math.max(1, bw * 0.3);

      const frontCol = `rgba(${cr}, ${cg}, ${cb}, ${rowAlpha * 0.85})`;
      const topCol = `rgba(${Math.min(255, cr + 40)}, ${Math.min(255, cg + 40)}, ${Math.min(255, cb + 40)}, ${rowAlpha})`;
      const sideCol = `rgba(${Math.round(cr * 0.5)}, ${Math.round(cg * 0.5)}, ${Math.round(cb * 0.5)}, ${rowAlpha * 0.75})`;
      const strokeCol = `rgba(${Math.min(255, cr + 60)}, ${Math.min(255, cg + 60)}, ${Math.min(255, cb + 60)}, ${rowAlpha * 0.6})`;

      // Front Face
      c.fillStyle = frontCol;
      c.fillRect(x, by, bw, bh);
      c.strokeStyle = strokeCol;
      c.lineWidth = 0.7;
      c.strokeRect(x, by, bw, bh);

      // Top Cap Face (Glowing peak)
      c.beginPath();
      c.moveTo(x, by);
      c.lineTo(x + dx, by - dy);
      c.lineTo(x + bw + dx, by - dy);
      c.lineTo(x + bw, by);
      c.closePath();
      c.fillStyle = topCol;
      c.fill();
      c.stroke();

      // Side Face (Depth)
      c.beginPath();
      c.moveTo(x + bw, by);
      c.lineTo(x + bw + dx, by - dy);
      c.lineTo(x + bw + dx, rowY - dy);
      c.lineTo(x + bw, rowY);
      c.closePath();
      c.fillStyle = sideCol;
      c.fill();
      c.stroke();

      // Volumetric Vertical Light Beams for high peaks
      if (val > 0.45 && row === 0) {
        const beamGrad = c.createLinearGradient(0, by, 0, 0);
        beamGrad.addColorStop(0, `rgba(${cr}, ${cg}, ${cb}, ${val * 0.35})`);
        beamGrad.addColorStop(1, `rgba(${cr}, ${cg}, ${cb}, 0)`);
        c.fillStyle = beamGrad;
        c.fillRect(x - 1, 0, bw + 2, by);
      }
    }
  }

  // PASS 2: RENDER HIGH-FIDELITY GLOSSY FLOOR REFLECTION (With transparent fade)
  for (let row = 0; row < rows; row++) {
    const depthRatio = row / rows;
    const zOffset = (rows - 1 - row) * 8;
    const rowY = floorY + zOffset * 0.3;
    const scale = 1 - depthRatio * 0.35;
    // Transparent reflection alpha fading out smoothly
    const refAlpha = Math.max(0.05, (0.35 - depthRatio * 0.2) * (1 - (rowY - floorY) / (height - floorY)));

    for (let i = 0; i < cols; i++) {
      const val = vals2D[row][i];
      if (val < 0.01) continue;

      const freqRatio = i / cols;
      const [cr, cg, cb] = getColor(freqRatio, val);

      const maxH = height * 0.38 * scale;
      const bh = Math.max(2, val * maxH * 0.8);
      const bw = Math.max(2, maxBarW * scale);

      const x = startX + i * (maxBarW + gap) * scale + (1 - scale) * (totalW / 2);
      const refBy = rowY;

      const dx = Math.max(1, bw * 0.4);
      const dy = Math.max(1, bw * 0.3);

      const frontRefCol = `rgba(${cr}, ${cg}, ${cb}, ${refAlpha * 0.6})`;
      const bottomRefCol = `rgba(${Math.round(cr * 0.7)}, ${Math.round(cg * 0.7)}, ${Math.round(cb * 0.7)}, ${refAlpha * 0.4})`;
      const sideRefCol = `rgba(${Math.round(cr * 0.3)}, ${Math.round(cg * 0.3)}, ${Math.round(cb * 0.3)}, ${refAlpha * 0.4})`;

      // Mirrored Front Face
      c.fillStyle = frontRefCol;
      c.fillRect(x, refBy, bw, bh);

      // Mirrored Bottom Cap Face
      c.beginPath();
      c.moveTo(x, refBy + bh);
      c.lineTo(x + dx, refBy + bh + dy);
      c.lineTo(x + bw + dx, refBy + bh + dy);
      c.lineTo(x + bw, refBy + bh);
      c.closePath();
      c.fillStyle = bottomRefCol;
      c.fill();

      // Mirrored Side Face
      c.beginPath();
      c.moveTo(x + bw, refBy);
      c.lineTo(x + bw + dx, refBy + dy);
      c.lineTo(x + bw + dx, refBy + bh + dy);
      c.lineTo(x + bw, refBy + bh);
      c.closePath();
      c.fillStyle = sideRefCol;
      c.fill();
    }
  }

  c.restore();
}

