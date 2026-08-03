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
const px = (d, x, y) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o+1]},${d[o+2]}]`; };
// The glyph 'P' of PARITY STRESS is centered; text starts around x=7 (frame 15, 55px bold). Print rows 10..40
console.log('TS glyph pixels (rows 10-45, x step 2):');
for (let y = 10; y <= 45; y += 5) {
  let row = '';
  for (let x = 0; x < 240; x += 2) row += px(ts, x, y).padStart(14);
  console.log('y' + y + ': ' + row);
}
console.log('\nR glyph pixels:');
for (let y = 10; y <= 45; y += 5) {
  let row = '';
  for (let x = 0; x < 240; x += 2) row += px(rs, x, y).padStart(14);
  console.log('y' + y + ': ' + row);
}
