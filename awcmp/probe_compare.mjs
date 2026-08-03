// Compare probe-rendered frames vs dumped frames (text presence).
import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const bandStats = (d, y0, y1) => {
  let nBright = 0, maxL = 0;
  for (let y = y0; y < y1; y++) for (let x = 0; x < W; x++) {
    const o = (y * W + x) * 4;
    const l = Math.max(d[o], d[o + 1], d[o + 2]);
    if (l > 150) nBright++;
    if (l > maxL) maxL = l;
  }
  return `bright(${nBright}) max(${maxL})`;
};
for (const f of ['000', '009', '015', '029']) {
  const probe = `/tmp/awcmp-stress/probe_spectrum_${f}.png`;
  const dumpP = `/tmp/awcmp-stress/rust/spectrum/frame_${f}.png`;
  let probeStr = 'n/a', dumpStr = 'n/a';
  try { probeStr = bandStats(await load(probe), 0, 60); } catch {}
  try { dumpStr = bandStats(await load(dumpP), 0, 60); } catch {}
  console.log(`frame ${f}: probe=${probeStr}  dump=${dumpStr}`);
}
