import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const madBand = (a, b, y0, y1) => {
  let sum = 0; let n = 0;
  for (let y = y0; y < y1; y++) for (let x = 0; x < W; x++) {
    const o = (y * W + x) * 4;
    sum += (Math.abs(a[o] - b[o]) + Math.abs(a[o + 1] - b[o + 1]) + Math.abs(a[o + 2] - b[o + 2])) / 3; n++;
  }
  return (sum / n).toFixed(2);
};
const bands = [
  ['text', 0, 60],
  ['mid', 60, 150],
  ['wave', 150, 220],
  ['low', 220, 270],
];
for (const f of ['000', '015', '029']) {
  const ts = await load(`/tmp/awcmp-stress/ts/waveformFill/frame_${f}.png`);
  const rs = await load(`/tmp/awcmp-stress/rust/waveformFill/frame_${f}.png`);
  const parts = bands.map(([n, y0, y1]) => `${n}=${madBand(ts, rs, y0, y1)}`).join(' ');
  console.log(`waveformFill frame_${f}: ${parts}`);
}
