import { createCanvas } from '@napi-rs/canvas';
const W = 480, H = 270;
const c = createCanvas(W, H);
const x = c.getContext('2d');
x.fillStyle = '#0b0c10';
x.fillRect(0, 0, W, H);
// Semi-transparent fill so the shadow shows through the interior.
x.fillStyle = 'rgba(255,255,255,0.5)';
x.shadowColor = '#ff2d78';
x.shadowBlur = 20;
x.fillRect(0, 100, W, H - 100);
const d = x.getImageData(0, 0, W, H).data;
const px = (xx, yy) => {
  const o = (yy * W + xx) * 4;
  return [d[o], d[o + 1], d[o + 2]];
};
const lum = (p) => Math.round((p[0] + p[1] + p[2]) / 3);
console.log('interior (240,150):', px(240, 150), 'lum', lum(px(240, 150)));
console.log('interior (240,200):', px(240, 200), 'lum', lum(px(240, 200)));
console.log('interior (240,240):', px(240, 240), 'lum', lum(px(240, 240)));
// top edge profile (shadow cast above the fill)
let row = '';
for (let y = 60; y <= 105; y += 3) row += `y${y}:${lum(px(240, y))} `;
console.log('top edge:', row);
// bottom edge profile (shadow cast would be below 270, but sample interior above it)
row = '';
for (let y = 200; y <= 268; y += 4) row += `y${y}:${lum(px(240, y))} `;
console.log('lower interior:', row);
// left edge: shadow cast to the left of x=0 doesn't exist; sample near-left interior
row = '';
for (let xx = 0; xx <= 40; xx += 4) row += `x${xx}:${lum(px(xx, 200))} `;
console.log('left interior:', row);
