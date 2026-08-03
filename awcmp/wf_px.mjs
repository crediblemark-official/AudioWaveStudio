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
const px = (d, x, y) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o+1]},${d[o+2]},${d[o+3]}]`; };
// scan a vertical column in the middle of the fill region, print where fill starts/ends in both
console.log('=== column x=240 vertical scan (y 55..270) ===');
let tStart = -1, tEnd = -1, rStart = -1, rEnd = -1;
for (let y = 55; y < 270; y++) {
  const t = ts[(y * W + 240) * 4 + 0], tg = ts[(y * W + 240) * 4 + 1], tb = ts[(y * W + 240) * 4 + 2];
  const r = rs[(y * W + 240) * 4 + 0], rg = rs[(y * W + 240) * 4 + 1], rb = rs[(y * W + 240) * 4 + 2];
  const tFill = (t > 100 && tg > 100 && tb < 120); // yellowish
  const rFill = (r > 100 && rg > 100 && rb < 120);
  if (tFill && tStart < 0) tStart = y;
  if (rFill && rStart < 0) rStart = y;
  if (tFill) tEnd = y;
  if (rFill) rEnd = y;
  if (y % 10 === 0 || (tFill !== rFill)) {
    console.log(`y=${y} TS=${px(ts, 240, y)} R=${px(rs, 240, y)} ${tFill ? 'F' : ' '}${rFill ? 'F' : ' '}`);
  }
}
console.log(`\nTS fill y[${tStart}..${tEnd}]  RUST fill y[${rStart}..${rEnd}]`);
// horizontal scan at a fill row
console.log('\n=== row y=100 horizontal scan ===');
for (let x = 0; x < W; x += 40) {
  const t = ts[(100 * W + x) * 4], tg = ts[(100 * W + x) * 4 + 1], tb = ts[(100 * W + x) * 4 + 2];
  const r = rs[(100 * W + x) * 4], rg = rs[(100 * W + x) * 4 + 1], rb = rs[(100 * W + x) * 4 + 2];
  console.log(`x=${x} TS=[${t},${tg},${tb}] R=[${r},${rg},${rb}]`);
}
