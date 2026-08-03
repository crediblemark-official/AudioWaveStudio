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
// band stats
const band = (y0, y1) => {
  let s = 0, n = 0;
  for (let y = y0; y < y1; y++) for (let x = 0; x < W; x++) {
    const o = (y * W + x) * 4;
    s += (Math.abs(ts[o] - rs[o]) + Math.abs(ts[o + 1] - rs[o + 1]) + Math.abs(ts[o + 2] - rs[o + 2])) / 3; n++;
  }
  return (s / n).toFixed(1);
};
console.log('bands:', [0,30,60,90,120,150,180,210,240].map((y0,i,a)=>i<a.length-1?`y${y0}-${a[i+1]}=${band(y0,a[i+1])}`:'').join(' '));
// cell grid 8x6 (60x45 cells)
console.log('=== 8x6 cell MAD grid ===');
for (let cy = 0; cy < 6; cy++) {
  const cells = [];
  for (let cx = 0; cx < 8; cx++) {
    let s = 0, n = 0;
    for (let y = cy * 45; y < (cy + 1) * 45; y++) for (let x = cx * 60; x < (cx + 1) * 60; x++) {
      const o = (y * W + x) * 4;
      s += (Math.abs(ts[o] - rs[o]) + Math.abs(ts[o + 1] - rs[o + 1]) + Math.abs(ts[o + 2] - rs[o + 2])) / 3; n++;
    }
    cells.push((s / n).toFixed(0));
  }
  console.log(`  row${cy}: ${cells.join('  ')}`);
}
