// Measure Skia's fill shadowBlur=20 glow kernel: fill a solid shape with
// shadowColor and sample the interior + edge profile to get the effective
// interior alpha and the Gaussian falloff near edges.
import { createCanvas } from '@napi-rs/canvas';
const W = 480, H = 270;
const c = createCanvas(W, H);
const x = c.getContext('2d');
// bg
x.fillStyle = '#0b0c10';
x.fillRect(0, 0, W, H);
// big fill from y=100 to bottom edge, white fill, pink shadow
x.fillStyle = '#ffffff';
x.shadowColor = '#ff2d78';
x.shadowBlur = 20;
x.fillRect(0, 100, W, H - 100);
const d = x.getImageData(0, 0, W, H).data;
const px = (xx, yy) => {
  const o = (yy * W + xx) * 4;
  return [d[o], d[o + 1], d[o + 2]];
};
// interior point far from all edges (left=240, top edge at 100 -> dist 50)
console.log('interior (240,150):', px(240, 150));
console.log('interior (240,200):', px(240, 200));
console.log('interior (240,240):', px(240, 240));
// vertical profile at x=240 across the top edge (y=70..130)
let row = '';
for (let y = 60; y <= 130; y += 2) {
  const p = px(240, y);
  const l = Math.round((p[0] + p[1] + p[2]) / 3);
  row += `y${y}:${l} `;
}
console.log('profile top edge:', row);
// bottom edge (canvas bottom at 270): profile y=230..268
row = '';
for (let y = 230; y <= 268; y += 2) {
  const p = px(240, y);
  const l = Math.round((p[0] + p[1] + p[2]) / 3);
  row += `y${y}:${l} `;
}
console.log('profile bottom edge:', row);
// left edge at x=0..40, y=200
row = '';
for (let xx = 0; xx <= 40; xx += 2) {
  const p = px(xx, 200);
  const l = Math.round((p[0] + p[1] + p[2]) / 3);
  row += `x${xx}:${l} `;
}
console.log('profile left edge:', row);
