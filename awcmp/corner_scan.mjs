import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const px = (d, x, y) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o+1]},${d[o+2]}]`; };
console.log('RUST frames only (all 30 saved):');
for (let f = 0; f < 30; f++) {
  const fstr = String(f).padStart(3, '0');
  const rs = await load(`/tmp/awcmp-stress/rust/waveformFill/frame_${fstr}.png`);
  console.log(`frame_${fstr} (2,2)=${px(rs, 2, 2)} (240,2)=${px(rs, 240, 2)} (2,20)=${px(rs, 2, 20)} (240,20)=${px(rs, 240, 20)}`);
}
