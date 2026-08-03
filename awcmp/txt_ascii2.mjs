import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const cls = (d, x, y) => {
  const o = (y * W + x) * 4; const r = d[o], g = d[o + 1], b = d[o + 2];
  if (r > 240 && g > 240 && b > 240) return '#';
  if (r > 150 && g > 150 && b > 150) return '+';
  if (r > 90 && g > 90 && b > 90) return '.';
  return ' ';
};
for (const f of ['000', '009']) {
  const d = await load(`/tmp/awcmp-stress/probe_spectrum_${f}.png`);
  console.log(`=== probe frame_${f} text region (y0-60, x step 8) ===`);
  for (let y = 0; y < 60; y += 3) {
    let row = '';
    for (let x = 0; x < W; x += 8) row += cls(d, x, y);
    console.log(String(y).padStart(3), row);
  }
}
