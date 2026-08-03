// Usage: node awcmp/profile.mjs <style> <frame> <row> [step]
// Prints R channel (or grayscale of RGB) at x=0..W step for TS and Rust side by side.
import { createCanvas, loadImage } from '@napi-rs/canvas';

const OUT = process.env.COMPARE_OUT || '/tmp/awcmp-stress';
const style = process.argv[2];
const frame = process.argv[3] || '015';
const row = parseInt(process.argv[4] || '30', 10);
const step = parseInt(process.argv[5] || '2', 10);
const W = 480, H = 270;

const name = `frame_${frame}.png`;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H);
  const x = c.getContext('2d');
  x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};

const ts = await load(`${OUT}/ts/${style}/${name}`);
const rs = await load(`${OUT}/rust/${style}/${name}`);

const lum = (d, o) => Math.round(0.299 * d[o] + 0.587 * d[o + 1] + 0.114 * d[o + 2]);
let line = `y=${row} `;
for (let x = 0; x < W; x += step) {
  const o = (row * W + x) * 4;
  line += `${String(x).padStart(3)}:${String(lum(ts, o)).padStart(3)}/${String(lum(rs, o)).padStart(3)} `;
}
console.log(line);
