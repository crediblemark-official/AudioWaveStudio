import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const [ts, rs] = await Promise.all([
  load('/tmp/awcmp-stress/ts/waveformFill/frame_015.png'),
  load('/tmp/awcmp-stress/rust/waveformFill/frame_015.png'),
]);
const px = (d, x, y) => {
  const o = (y * W + x) * 4;
  return `[${d[o]},${d[o + 1]},${d[o + 2]}]`;
};
// Sample the high-diff cells: y 100-170, x 130-230 and x 360-470
for (const [x0, x1] of [[130, 230], [360, 470]]) {
  console.log(`=== x ${x0}-${x1} ===`);
  for (let y = 100; y <= 170; y += 14) {
    let row = '';
    for (let x = x0; x <= x1; x += 20) {
      const t = px(ts, x, y), r = px(rs, x, y);
      const dr = Math.abs(ts[(y * W + x) * 4] - rs[(y * W + x) * 4]);
      const dg = Math.abs(ts[(y * W + x) * 4 + 1] - rs[(y * W + x) * 4 + 1]);
      const db = Math.abs(ts[(y * W + x) * 4 + 2] - rs[(y * W + x) * 4 + 2]);
      const d = ((dr + dg + db) / 3).toFixed(0);
      row += `x${x}: T${t} R${r} d=${d}  `;
    }
    console.log(`y${y}: ${row}`);
  }
}
