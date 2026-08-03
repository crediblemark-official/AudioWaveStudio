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
const px = (d, x, y) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o+1]},${d[o+2]}]`; };
// mirror fill region is above centerY=148; scan rows 50-140 at several x
for (const y of [50, 60, 70, 80, 90, 100, 110, 120, 130, 140]) {
  console.log(`y=${y} x40 TS=${px(ts,40,y)} R=${px(rs,40,y)}  x200 TS=${px(ts,200,y)} R=${px(rs,200,y)}  x360 TS=${px(ts,360,y)} R=${px(rs,360,y)}`);
}
