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
// The text baseline ~ y=40 (positionY 15% of 270 = 40.5). Glyph body around y=15..40.
// Print rows 12..40 with threshold map to see glyph start/end precisely (x step 1).
const cls = (d, x, y) => {
  const o = (y * W + x) * 4;
  const mx = Math.max(d[o], d[o + 1], d[o + 2]);
  if (mx > 240) return '#';
  if (mx > 150) return '+';
  if (mx > 90) return '.';
  return ' ';
};
console.log('TS rows y=20..44 (x step 2, full 0..120):');
for (let y = 20; y <= 44; y += 3) {
  let row = '';
  for (let x = 0; x < 120; x += 2) row += cls(ts, x, y);
  console.log(String(y).padStart(3) + ' ' + row);
}
console.log('\nR rows y=20..44 (x step 2, full 0..120):');
for (let y = 20; y <= 44; y += 3) {
  let row = '';
  for (let x = 0; x < 120; x += 2) row += cls(rs, x, y);
  console.log(String(y).padStart(3) + ' ' + row);
}
// find first bright x at each row
for (let y = 20; y <= 44; y += 3) {
  let tFirst = -1, rFirst = -1;
  for (let x = 0; x < W; x++) {
    const to = (y * W + x) * 4;
    const ro = (y * W + x) * 4;
    if (tFirst < 0 && ts[to] > 150 && ts[to + 1] > 150 && ts[to + 2] > 150) tFirst = x;
    if (rFirst < 0 && rs[ro] > 150 && rs[ro + 1] > 150 && rs[ro + 2] > 150) rFirst = x;
  }
  console.log(`y=${y} first bright TS=${tFirst} R=${rFirst} (delta ${rFirst - tFirst})`);
}
