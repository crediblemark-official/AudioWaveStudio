import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const extents = (d, y0, y1, thresh) => {
  let minX = W, maxX = 0, n = 0;
  for (let y = y0; y < y1; y++) for (let x = 0; x < W; x++) {
    const o = (y * W + x) * 4;
    const l = Math.max(d[o], d[o + 1], d[o + 2]);
    if (l > thresh) { if (x < minX) minX = x; if (x > maxX) maxX = x; n++; }
  }
  return n ? `n=${n} x[${minX}..${maxX}] (w=${maxX - minX})` : 'none';
};
for (const f of ['015']) {
  const ts = await load(`/tmp/awcmp-stress/ts/spectrum/frame_${f}.png`);
  const rs = await load(`/tmp/awcmp-stress/rust/spectrum/frame_${f}.png`);
  console.log(`frame ${f} y8-45:`);
  console.log('  TS  >200:', extents(ts, 8, 45, 200));
  console.log('  RUST>200:', extents(rs, 8, 45, 200));
  console.log('  TS  >150:', extents(ts, 8, 45, 150));
  console.log('  RUST>150:', extents(rs, 8, 45, 150));
}
