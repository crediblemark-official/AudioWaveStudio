import { createCanvas, loadImage } from '@napi-rs/canvas';
const load = async (p) => {
  const img = await loadImage(p);
  const c = createCanvas(img.width, img.height); const x = c.getContext('2d');
  x.drawImage(img, 0, 0);
  return { d: x.getImageData(0, 0, img.width, img.height).data, w: img.width, h: img.height };
};
for (const f of ['000', '009']) {
  const { d, w, h } = await load(`/tmp/awcmp-stress/probe_atlas_${f}.png`);
  console.log(`=== atlas ${f} (${w}x${h}) — y step 2, x step 8, alpha>10 = ink ===`);
  for (let y = 0; y < h; y += 2) {
    let row = '';
    for (let x = 0; x < w; x += 8) {
      const o = (y * w + x) * 4;
      row += d[o + 3] > 10 ? '#' : (d[o + 3] > 0 ? '.' : ' ');
    }
    console.log(String(y).padStart(2), row);
  }
}
