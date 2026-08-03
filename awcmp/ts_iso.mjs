import { createCanvas } from '@napi-rs/canvas';
import { readFileSync } from 'node:fs';
import { CanvasRenderer } from '../src/services/canvasRenderer.ts';

const W = 480, H = 270, FPS = 30, FFT_HALF = 512;
const IN = '/tmp/awcmp-stress/inputs';
const baseConfig = JSON.parse(readFileSync('/media/rasyiqi/7653717A1C07B131/audiowave/scripts/compare-config-stress.json', 'utf8'));

function mulberry32(seed) { let a = seed >>> 0; return () => { a = (a + 0x6d2b79f5) >>> 0; let t = a; t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61); return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }; }
Math.random = mulberry32(0xc0ffee);

async function renderFrame(config, f, label) {
  const renderer = new CanvasRenderer();
  const canvas = createCanvas(W, H);
  renderer.init(canvas);
  const ctx = canvas.getContext('2d');
  const bin = readFileSync(`${IN}/frame_${String(f).padStart(3, '0')}.bin`);
  const freq = new Uint8Array(bin.buffer, bin.byteOffset, FFT_HALF);
  const time = new Uint8Array(bin.buffer, bin.byteOffset + FFT_HALF, 1024);
  let bassSum = 0;
  for (let i = 0; i < Math.min(16, freq.length); i++) bassSum += freq[i];
  renderer.setExportData(freq, time, bassSum / (Math.min(16, freq.length) * 255));
  renderer.setFrameTime(f / FPS);
  renderer.drawFrame(config);
  const d = ctx.getImageData(0, 0, W, H).data;
  const px = (x, y) => { const o = (y * W + x) * 4; return `[${d[o]},${d[o+1]},${d[o+2]}]`; };
  console.log(`${label.padEnd(36)} (2,2)=${px(2,2)} (240,2)=${px(240,2)} (2,15)=${px(2,15)} (240,15)=${px(240,15)} (2,40)=${px(2,40)}`);
}

const c = (over) => ({ ...baseConfig, screenEffects: { ...baseConfig.screenEffects, ...over } });
await renderFrame(c({}), 15, 'screen fx ON (as harness)');
await renderFrame(c({ enabled: false }), 15, 'screen fx DISABLED');
await renderFrame(c({ backgroundOnly: false }), 15, 'backgroundOnly=false');
