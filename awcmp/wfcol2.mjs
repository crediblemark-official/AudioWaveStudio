import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const px = (d, x, y) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o+1]},${d[o+2]}]`; };
const [ts, rs] = await Promise.all([
  load('/tmp/awcmp-stress/ts/waveformFill/frame_015.png'),
  load('/tmp/awcmp-stress/rust/waveformFill/frame_015.png'),
]);
// Sample a few columns at mid heights
for (const x of [60, 120, 240, 360, 440]) {
  console.log(`--- column x=${x} (frame 15) ---`);
  for (const y of [65, 80, 95, 110, 125, 140, 155, 170, 185, 200, 215, 230, 245]) {
    const t = px(ts, x, y), r = px(rs, x, y);
    if (t !== r) console.log(`  y=${y} TS=${t} RUST=${r}`);
  }
}
