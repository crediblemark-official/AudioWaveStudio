import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const px = (d, x, y) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o+1]},${d[o+2]}]`; };
for (const style of ['spectrum', 'waveformFill', 'vuMeter', 'minimal', 'radial']) {
  const [ts, rs] = await Promise.all([
    load(`/tmp/awcmp-stress/ts/${style}/frame_015.png`),
    load(`/tmp/awcmp-stress/rust/${style}/frame_015.png`),
  ]);
  console.log(`${style.padEnd(14)} (2,2) TS=${px(ts,2,2)} R=${px(rs,2,2)}  (240,2) TS=${px(ts,240,2)} R=${px(rs,240,2)}  (2,30) TS=${px(ts,2,30)} R=${px(rs,2,30)}`);
}
