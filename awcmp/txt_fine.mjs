import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
// Fine rows of RUST frame 9 text band (every row y=2..50, every 12 x)
const rs = await load('/tmp/awcmp-stress/probe_spectrum_009.png');
console.log('=== RUST frame 9 rows (luma, x step 12) ===');
for (let y = 2; y <= 50; y += 3) {
  let row = '';
  for (let x = 0; x < W; x += 12) {
    const o = (y * W + x) * 4;
    const l = Math.max(rs[o], rs[o + 1], rs[o + 2]);
    row += l > 200 ? '#' : l > 140 ? '+' : l > 90 ? '.' : ' ';
  }
  console.log(String(y).padStart(2), row);
}
// TS frame 15 same rows
const ts = await load('/tmp/awcmp-stress/ts/spectrum/frame_015.png');
console.log('=== TS frame 15 rows ===');
for (let y = 2; y <= 50; y += 3) {
  let row = '';
  for (let x = 0; x < W; x += 12) {
    const o = (y * W + x) * 4;
    const l = Math.max(ts[o], ts[o + 1], ts[o + 2]);
    row += l > 200 ? '#' : l > 140 ? '+' : l > 90 ? '.' : ' ';
  }
  console.log(String(y).padStart(2), row);
}
