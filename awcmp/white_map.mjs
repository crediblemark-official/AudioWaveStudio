import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const ts = await load('/tmp/awcmp-stress/ts/spectrum/frame_015.png');
// map: which pixels are ~pure white (all channels > 240)?
console.log('TS spectrum frame_015 pure-white map (x step 8, y step 6):');
for (let y = 0; y < H; y += 6) {
  let row = '';
  for (let x = 0; x < W; x += 8) {
    const o = (y * W + x) * 4;
    const r = ts[o], g = ts[o + 1], b = ts[o + 2];
    row += (r > 240 && g > 240 && b > 240) ? '#' : ((r > 200 && g > 200 && b > 200) ? '+' : '.');
  }
  console.log(String(y).padStart(3) + ' ' + row);
}
// find the bounding box of pure-white pixels
let minX = W, maxX = 0, minY = H, maxY = 0, n = 0;
for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
  const o = (y * W + x) * 4;
  if (ts[o] > 240 && ts[o + 1] > 240 && ts[o + 2] > 240) {
    if (x < minX) minX = x; if (x > maxX) maxX = x;
    if (y < minY) minY = y; if (y > maxY) maxY = y; n++;
  }
}
console.log(`\npure-white bbox: x[${minX}..${maxX}] y[${minY}..${maxY}] n=${n}`);
