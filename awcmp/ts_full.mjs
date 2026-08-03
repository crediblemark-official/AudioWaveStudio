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
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b);
  if (mx > 240 && mn > 200) return 'W';  // white
  if (r > 200 && g < 130 && b > 100) return 'P'; // pink
  if (r > 170 && g > 140 && b < 90) return 'Y';  // yellow
  if (b > 140 && g > 90 && r < 110) return 'C';  // cyan/blue
  if (mx > 140) return '+';
  if (mx > 90) return '.';
  if (mx > 50) return ',';
  return ' ';
};
const [ts, rs] = await Promise.all([
  load('/tmp/awcmp-stress/ts/waveformFill/frame_015.png'),
  load('/tmp/awcmp-stress/rust/waveformFill/frame_015.png'),
]);
for (const [name, d] of [['TS', ts], ['R ', rs]]) {
  console.log(`=== ${name} waveformFill frame_015 (x step 8, y step 6) ===`);
  for (let y = 0; y < H; y += 6) {
    let row = '';
    for (let x = 0; x < W; x += 8) row += cls(d, x, y);
    console.log(String(y).padStart(3) + ' ' + row);
  }
}
