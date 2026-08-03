// Usage: node awcmp/analyze.mjs <style> [frames]
// Prints per-frame MAD and horizontal-band MAD for TS vs Rust frames.
import { createCanvas, loadImage } from '@napi-rs/canvas';
import { readdirSync } from 'fs';

const OUT = process.env.COMPARE_OUT || '/tmp/awcmp-stress';
const style = process.argv[2];
const maxFrames = process.argv[3] ? parseInt(process.argv[3], 10) : 30;
const W = 480, H = 270;

if (!style) { console.error('usage: node awcmp/analyze.mjs <style> [frames]'); process.exit(1); }

const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H);
  const x = c.getContext('2d');
  x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};

// TS side only renders a few sampled frames (0, 15, 29); compare on those.
const tsFiles = readdirSync(`${OUT}/ts/${style}`).filter((f) => f.endsWith('.png')).sort();
const files = tsFiles.slice(0, maxFrames);
const frames = files.length;

const bands = [
  ['y0-60 (text)', 0, 60],
  ['y60-120', 60, 120],
  ['y120-200', 120, 200],
  ['y200-270', 200, 270],
];

function madIn(a, b, y0, y1) {
  let s = 0, cnt = 0;
  for (let y = y0; y < y1; y++) {
    for (let x = 0; x < W; x++) {
      const o = (y * W + x) * 4;
      s += Math.abs(a[o] - b[o]) + Math.abs(a[o + 1] - b[o + 1]) + Math.abs(a[o + 2] - b[o + 2]);
      cnt += 3;
    }
  }
  return s / cnt;
}

let total = 0, bandSum = bands.map(() => 0), maxMad = 0, worstFrame = 0;
for (let f = 0; f < files.length; f++) {
  const name = files[f];
  const ts = await load(`${OUT}/ts/${style}/${name}`);
  const rs = await load(`${OUT}/rust/${style}/${name}`);
  const m = madIn(ts, rs, 0, H);
  total += m;
  if (m > maxMad) { maxMad = m; worstFrame = f; }
  bands.forEach((b, i) => { bandSum[i] += madIn(ts, rs, b[1], b[2]); });
}

console.log(`\n=== ${style} (${frames} frames, avg MAD ${(total / frames).toFixed(2)}, max ${maxMad.toFixed(2)} @ f${worstFrame}) ===`);
bands.forEach((b, i) => {
  console.log(`  ${b[0].padEnd(14)} MAD=${(bandSum[i] / frames).toFixed(2)}`);
});
