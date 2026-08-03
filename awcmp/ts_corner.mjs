import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const px = (d, x, y) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o+1]},${d[o+2]}]`; };
for (const f of ['000', '015', '029']) {
  const [ts, rs] = await Promise.all([
    load(`/tmp/awcmp-stress/ts/waveformFill/frame_${f}.png`),
    load(`/tmp/awcmp-stress/rust/waveformFill/frame_${f}.png`),
  ]);
  console.log(`=== frame_${f} ===`);
  for (const [x, y] of [[2,2],[240,2],[477,2],[2,20],[240,20],[2,40],[240,40],[2,60],[2,100],[2,150]]) {
    console.log(`(${x},${y}) TS=${px(ts,x,y)} R=${px(rs,x,y)}`);
  }
}
