// Temporary isolation probe (not part of the suite): run with
//   npx vitest run --config vitest.compare.config.ts tests/isoProbe.test.ts
import { it } from 'vitest';
import { createCanvas, loadImage } from '@napi-rs/canvas';
import { readFileSync } from 'node:fs';
import { CanvasRenderer } from '../src/services/canvasRenderer';

const W = 480, H = 270, FPS = 30, FFT_HALF = 512;
const IN = '/tmp/awcmp-stress/inputs';
const baseConfig = JSON.parse(readFileSync('/media/rasyiqi/7653717A1C07B131/audiowave/scripts/compare-config-stress.json', 'utf8'));

function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
Math.random = mulberry32(0xc0ffee);

it('isolates waveformFill background', { timeout: 120_000 }, async () => {
  async function renderFrames(label: string, mutate: (c: any) => void) {
    const renderer = new CanvasRenderer();
    const canvas = createCanvas(W, H);
    renderer.init(canvas as unknown as HTMLCanvasElement);
    const ctx = canvas.getContext('2d');
    const config = JSON.parse(JSON.stringify(baseConfig));
    config.style = 'waveformFill';
    mutate(config);
    for (let f = 0; f <= 15; f++) {
      const bin = readFileSync(`${IN}/frame_${String(f).padStart(3, '0')}.bin`);
      const freq = new Uint8Array(bin.buffer, bin.byteOffset, FFT_HALF);
      const time = new Uint8Array(bin.buffer, bin.byteOffset + FFT_HALF, 1024);
      let bassSum = 0;
      for (let i = 0; i < Math.min(16, freq.length); i++) bassSum += freq[i];
      renderer.setExportData(freq, time, bassSum / (Math.min(16, freq.length) * 255));
      renderer.setFrameTime(f / FPS);
      renderer.drawFrame(config);
    }
    const d = ctx!.getImageData(0, 0, W, H).data;
    const px = (x: number, y: number) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o + 1]},${d[o + 2]}]`; };
    console.log(`${label.padEnd(34)} (2,100)=${px(2, 100)} (2,150)=${px(2, 150)} (240,100)=${px(240, 100)} (2,60)=${px(2, 60)}`);
  }
  await renderFrames('fx ON', () => {});
  await renderFrames('fx OFF', (c) => { c.screenEffects.enabled = false; });
  await renderFrames('fx OFF + text off', (c) => { c.screenEffects.enabled = false; c.text.blocks[0].enabled = false; });

  // Rust reference frame
  const img = await loadImage('/tmp/awcmp-stress/rust/waveformFill/frame_015.png');
  const c2 = createCanvas(W, H);
  const c2x = c2.getContext('2d');
  c2x.drawImage(img, 0, 0, W, H);
  const rd = c2x.getImageData(0, 0, W, H).data;
  const pxr = (x: number, y: number) => { const o = (y * W + x) * 4; return `[${rd[o]},${rd[o + 1]},${rd[o + 2]}]`; };
  console.log(`RUST frame_015                     (2,100)=${pxr(2, 100)} (2,150)=${pxr(2, 150)} (240,100)=${pxr(240, 100)} (2,60)=${pxr(2, 60)}`);
});
