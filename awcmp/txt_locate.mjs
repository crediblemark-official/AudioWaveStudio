import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const d = await load('/tmp/awcmp-stress/probe_spectrum_009.png');
// Print coordinates of bright pixels (>150)
let list = [];
for (let y = 0; y < 60; y++) for (let x = 0; x < W; x++) {
  const o = (y * W + x) * 4;
  const l = Math.max(d[o], d[o + 1], d[o + 2]);
  if (l > 150) list.push(`(${x},${y})=[${d[o]},${d[o+1]},${d[o+2]}]`);
}
console.log('bright px count:', list.length);
console.log('first 30:', list.slice(0, 30).join(' '));
// Sample a horizontal strip at y=20 across the text region
let row = '';
for (let x = 40; x < 440; x += 16) {
  const o = (20 * W + x) * 4;
  row += `x${x}[${d[o]},${d[o+1]},${d[o+2]}] `;
}
console.log('row y=20:', row);
// Full-frame bright bbox
let minX = W, maxX = 0, minY = H, maxY = 0, n = 0;
for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
  const o = (y * W + x) * 4;
  const l = Math.max(d[o], d[o + 1], d[o + 2]);
  if (l > 150) { if (x < minX) minX = x; if (x > maxX) maxX = x; if (y < minY) minY = y; if (y > maxY) maxY = y; n++; }
}
console.log(`full-frame bright bbox: n=${n} x[${minX},${maxX}] y[${minY},${maxY}]`);
