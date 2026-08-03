import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const [ts, rs] = await Promise.all([
  load('/tmp/awcmp-stress/ts/spectrum/frame_015.png'),
  load('/tmp/awcmp-stress/rust/spectrum/frame_015.png'),
]);
// threshold scan: bright pixels per row band y=15..50, find leftmost/rightmost
const ext = (d, label) => {
  let minX = W, maxX = 0, n = 0;
  for (let y = 15; y < 50; y++) for (let x = 0; x < W; x++) {
    const o = (y * W + x) * 4;
    if (d[o] > 150 && d[o + 1] > 150 && d[o + 2] > 150) { if (x < minX) minX = x; if (x > maxX) maxX = x; n++; }
  }
  console.log(`${label}: bright(n=${n}) x[${minX}..${maxX}]`);
};
ext(ts, 'TS');
ext(rs, 'R ');
// count per-column distribution in the band
const colCount = (d) => {
  const cols = new Array(W).fill(0);
  for (let y = 15; y < 50; y++) for (let x = 0; x < W; x++) {
    const o = (y * W + x) * 4;
    if (d[o] > 150 && d[o + 1] > 150 && d[o + 2] > 150) cols[x]++;
  }
  return cols;
};
const tc = colCount(ts), rc = colCount(rs);
// print column histogram every 20px
let tl = '', rl = '';
for (let x = 0; x < W; x += 20) { tl += `${String(x).padStart(3)}:${String(tc[x]).padStart(2)} `; }
for (let x = 0; x < W; x += 20) { rl += `${String(x).padStart(3)}:${String(rc[x]).padStart(2)} `; }
console.log('TS cols:', tl);
console.log('R  cols:', rl);
