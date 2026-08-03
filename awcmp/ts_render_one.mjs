// Render a single TS frame through the real export path (CanvasRenderer +
// setExportData + drawFrame) so we can compare fill/mirror pixels without
// the text overlay muddying the region.
import { createCanvas, GlobalFonts, loadImage } from '@napi-rs/canvas';
import { readFileSync, existsSync } from 'node:fs';
import { CanvasRenderer } from '../src/services/canvasRenderer.ts';

const W = 480, H = 270, FPS = 30, FFT_HALF = 512;
const OUT = process.argv[2] ?? '/tmp/awcmp-notext';
const STYLE = process.argv[3] ?? 'waveformFill';
const FRAME = Number(process.argv[4] ?? 15);

const config = JSON.parse(readFileSync('/media/rasyiqi/7653717A1C07B131/audiowave/scripts/compare-config-stress-notext.json', 'utf8'));

function mulberry32(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const originalRandom = Math.random.bind(Math);
let nextRandom = mulberry32(0xc0ffee);
Math.random = () => nextRandom();

// Generic-family font aliases (mirrors tests/compareExport.test.ts)
const GENERIC_FONT_ALIASES = {
  monospace: ['/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf', '/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf'],
  'sans-serif': ['/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', '/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf'],
  serif: ['/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf', '/usr/share/fonts/truetype/dejavu/DejaVuSerif-Bold.ttf'],
};
for (const [family, paths] of Object.entries(GENERIC_FONT_ALIASES)) {
  for (const p of paths) if (existsSync(p)) { try { GlobalFonts.registerFromPath(p, family); } catch {} }
}

const styleConfig = { ...config, style: STYLE };
const renderer = new CanvasRenderer();
const canvas = createCanvas(W, H);
renderer.init(canvas);
const ctx = canvas.getContext('2d');

const bin = readFileSync(`${OUT}/inputs/frame_${String(FRAME).padStart(3, '0')}.bin`);
const freq = new Uint8Array(bin.buffer, bin.byteOffset, FFT_HALF);
const time = new Uint8Array(bin.buffer, bin.byteOffset + FFT_HALF, 1024);
let bassSum = 0;
const bassBins = Math.min(16, freq.length);
for (let i = 0; i < bassBins; i++) bassSum += freq[i];
const bassEnergy = bassBins > 0 ? bassSum / (bassBins * 255) : 0;

renderer.setExportData(freq, time, bassEnergy);
renderer.setFrameTime(FRAME / FPS);
renderer.drawFrame(styleConfig);

const buf = canvas.toBuffer('image/png');
const fs = await import('node:fs');
fs.mkdirSync(`${OUT}/ts/${STYLE}`, { recursive: true });
fs.writeFileSync(`${OUT}/ts/${STYLE}/frame_${String(FRAME).padStart(3, '0')}.png`, buf);
console.log(`wrote ${OUT}/ts/${STYLE}/frame_${String(FRAME).padStart(3, '0')}.png`);
Math.random = originalRandom;
