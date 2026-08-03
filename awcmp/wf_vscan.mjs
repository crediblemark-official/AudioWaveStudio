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
// Vertical scans at several x positions: print luminance + RGB where bright
for (const x of [60, 150, 260, 400]) {
  console.log(`=== vertical scan x=${x} ===`);
  let out = '';
  for (let y = 0; y < H; y += 2) {
    const ot = (y * W + x) * 4, or_ = ot;
    const lt = (ts[ot] + ts[ot + 1] + ts[ot + 2]) / 3;
    const lr = (rs[or_] + rs[or_ + 1] + rs[or_ + 2]) / 3;
    const mark = Math.abs(lt - lr) > 40 ? 'X' : Math.abs(lt - lr) > 15 ? '+' : Math.abs(lt - lr) > 5 ? '.' : ' ';
    out += `y${String(y).padStart(3)}:${mark}`;
    if (lt > 60 || lr > 60) out += ` T[${ts[ot]},${ts[ot + 1]},${ts[ot + 2]}] R[${rs[or_]},${rs[or_ + 1]},${rs[or_ + 2]}]`;
    out += '\n';
  }
  console.log(out);
}
