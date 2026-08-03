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
const ts = await load('/tmp/awcmp-stress/ts/spectrum/frame_015.png');
const rs = await load('/tmp/awcmp-stress/rust/spectrum/frame_015.png');
for (let y = 0; y < 60; y += 3) {
  let tr = '', rr = '';
  for (let x = 0; x < W; x += 6) { tr += cls(ts, x, y); rr += cls(rs, x, y); }
  console.log(String(y).padStart(3), 'TS:', tr);
  console.log('     ', 'R :', rr);
}
// Sample some pixels
const px = (d, x, y) => { const o = (y*W+x)*4; return `[${d[o]},${d[o+1]},${d[o+2]}]`; };
console.log('--- samples ---');
for (const [x, y] of [[50, 15], [100, 20], [240, 20], [360, 15], [430, 20], [240, 5], [100, 45]]) {
  console.log(`(${x},${y}) TS=${px(ts, x, y)} RUST=${px(rs, x, y)}`);
}
