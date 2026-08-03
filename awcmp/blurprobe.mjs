// Measure the Skia (@napi-rs/canvas) shadowBlur falloff profile directly.
// Usage: node awcmp/blurprobe.mjs
import { createCanvas } from '@napi-rs/canvas';

const W = 360, H = 80;
const c = createCanvas(W, H);
const x = c.getContext('2d');
x.fillStyle = '#000000';
x.fillRect(0, 0, W, H);

// Case 1: solid 30px-wide rect, shadowBlur=20, no offset
x.save();
x.shadowBlur = 20;
x.shadowColor = '#ffffff';
x.shadowOffsetX = 0;
x.shadowOffsetY = 0;
x.fillStyle = '#ffffff';
x.fillRect(140, 20, 30, 40);
x.restore();

// Case 2: thin 6px rect (glyph-stroke-like), shadowBlur=20
x.save();
x.shadowBlur = 20;
x.shadowColor = '#ffffff';
x.fillStyle = '#ff0000';
x.fillRect(210, 20, 6, 40);
x.restore();

const d = x.getImageData(0, 0, W, H).data;
const row = 40; // through the middle
function lum(o) { return Math.round(0.299 * d[o] + 0.587 * d[o + 1] + 0.114 * d[o + 2]); }

// Profile A: around the solid rect (x 130..190). Print every 3px.
let pa = 'SOLID30: ';
for (let px = 110; px <= 190; px += 3) {
  pa += `${px}:${String(lum((row * W + px) * 4)).padStart(3)} `;
}
console.log(pa);
// Profile B: around the thin rect (x 205..225), every 2px
let pb = 'THIN6:   ';
for (let px = 196; px <= 226; px += 2) {
  pb += `${px}:${String(lum((row * W + px) * 4)).padStart(3)} `;
}
console.log(pb);
