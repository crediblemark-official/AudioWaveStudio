import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const bandStats = (d, y0, y1) => {
  let nB = 0, maxL = 0, sum = 0, n = 0;
  for (let y = y0; y < y1; y++) for (let x = 0; x < W; x++) {
    const o = (y * W + x) * 4;
    const l = Math.max(d[o], d[o + 1], d[o + 2]);
    if (l > 150) nB++;
    if (l > maxL) maxL = l;
    sum += l; n++;
  }
  return `bright(${nB}) max(${maxL}) avg(${(sum / n).toFixed(0)})`;
};
for (const f of ['000', '015', '029']) {
  let ts, rs;
  try { ts = await load(`/tmp/awcmp-stress/ts/spectrum/frame_${f}.png`); } catch { console.log(`frame ${f}: TS missing`); continue; }
  rs = await load(`/tmp/awcmp-stress/rust/spectrum/frame_${f}.png`);
  console.log(`frame ${f}: TS=${bandStats(ts, 0, 60)}  RUST=${bandStats(rs, 0, 60)}`);
}
