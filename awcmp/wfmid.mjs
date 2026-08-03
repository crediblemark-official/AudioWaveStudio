import { createCanvas, loadImage } from '@napi-rs/canvas';
const W = 480, H = 270;
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(W, H); const x = c.getContext('2d'); x.drawImage(img, 0, 0, W, H);
  return x.getImageData(0, 0, W, H).data;
};
const cls = (d, x, y) => {
  const o = (y * W + x) * 4; const r = d[o], g = d[o + 1], b = d[o + 2];
  if (r > 240 && g > 220 && b < 60) return 'Y';   // pure accent yellow (stroke/fill crest)
  if (r > 200 && g > 150 && b < 100) return 'y';  // dimmer yellow
  if (r > 180 && g > 180) return 'W';             // whitish
  if (r < 60 && g < 60 && b < 90) return '.';     // dark bg
  if (b > 150 && g > 120 && r < 120) return 'c';  // cyan
  if (r > 200 && g < 120 && b < 200) return 'p';  // pink/magenta
  if (r > 100 && g > 80) return 'm';              // mauve/mixed
  return 'o';
};
for (const [side, p] of [['TS', '/tmp/awcmp-stress/ts/waveformFill/frame_015.png'], ['RUST', '/tmp/awcmp-stress/rust/waveformFill/frame_015.png']]) {
  const d = await load(p);
  console.log(`=== ${side} waveformFill frame_015 mid band (y60-165, x step 8) ===`);
  for (let y = 60; y < 165; y += 6) {
    let row = '';
    for (let x = 0; x < W; x += 8) row += cls(d, x, y);
    console.log(String(y).padStart(3), row);
  }
}
