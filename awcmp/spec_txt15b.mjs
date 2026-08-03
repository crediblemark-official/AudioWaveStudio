import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const cls = (d, x, y) => {
  const o = (y * W + x) * 4;
  const r = d[o], g = d[o + 1], b = d[o + 2];
  const mx = Math.max(r, g, b);
  if (mx > 235) return '#';
  if (mx > 180) return '+';
  if (mx > 120) return '.';
  if (mx > 70) return ',';
  return ' ';
};
const [ts, rs] = await Promise.all([
  load('/tmp/awcmp-stress/ts/spectrum/frame_015.png'),
  load('/tmp/awcmp-stress/rust/spectrum/frame_015.png'),
]);
console.log('=== spectrum frame_015 text band (y0-70, x step 2) ===');
for (let y = 0; y < 70; y += 3) {
  let a = '', b = '';
  for (let x = 0; x < W; x += 2) { a += cls(ts, x, y); b += cls(rs, x, y); }
  console.log('TS y' + String(y).padStart(2) + ': ' + a);
  console.log('R  y' + String(y).padStart(2) + ': ' + b);
}
// sample pixels at a few text glyph positions
const px = (d, x, y) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o+1]},${d[o+2]}]`; };
console.log('\n=== samples ===');
for (const [x, y] of [[100, 20], [150, 20], [200, 20], [250, 20], [300, 20], [100, 35], [200, 35], [300, 35], [150, 10], [250, 10]]) {
  console.log(`(${x},${y}) TS=${px(ts, x, y)} R=${px(rs, x, y)}`);
}
