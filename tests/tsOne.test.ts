// Helper: render ONE TS frame through the real export path so we can inspect
// fill/mirror pixels without the full 7-min harness. Usage:
//   COMPARE_CONFIG=scripts/compare-config-stress-notext.json COMPARE_OUT=/tmp/awcmp-notext \
//     STYLE=waveformFill FRAME=15 npx vitest run --config vitest.compare.config.ts tests/tsOne.test.ts
import { it } from 'vitest';
import { createCanvas, GlobalFonts } from '@napi-rs/canvas';
import { readFileSync, existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { CanvasRenderer } from '../src/services/canvasRenderer';

const OUT = process.env.COMPARE_OUT ?? '/tmp/awcmp-notext';
const STYLE = process.env.STYLE ?? 'waveformFill';
const FRAME = Number(process.env.FRAME ?? 15);
const LABEL = process.env.LABEL ?? ''; // e.g. '_fx' to avoid clobbering frames
const W = 480;
const H = 270;
const FPS = 30;
const FFT_HALF = 512;

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
const originalRandom = Math.random.bind(Math);
let nextRandom = mulberry32(0xc0ffee);
Math.random = () => nextRandom();

const GENERIC_FONT_ALIASES: Record<string, string[]> = {
  monospace: ['/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf', '/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf'],
  'sans-serif': ['/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', '/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf'],
  serif: ['/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf', '/usr/share/fonts/truetype/dejavu/DejaVuSerif-Bold.ttf'],
};
for (const [family, paths] of Object.entries(GENERIC_FONT_ALIASES)) {
  for (const p of paths) {
    if (existsSync(p)) {
      try {
        GlobalFonts.registerFromPath(p, family);
      } catch {}
    }
  }
}

// Only run when TS_ONE=1 — this is a debug helper, not part of the suite.
it.skipIf(!process.env.TS_ONE)('renders one TS frame', () => {
  const env = process.env.COMPARE_CONFIG;
  const config = JSON.parse(
    readFileSync(env ? resolve(process.cwd(), env) : 'scripts/compare-config-stress-notext.json', 'utf8'),
  );
  const styleConfig = { ...config, style: STYLE };
  const renderer = new CanvasRenderer();
  const canvas = createCanvas(W, H);
  renderer.init(canvas as unknown as HTMLCanvasElement);

  // Warmup frames 0..FRAME so the renderer state (bassEnergy / bassFloor /
  // beatStrength envelopes) matches the full harness at frame FRAME — a fresh
  // renderer's first frame has a huge bass onset that fires beatStrength to
  // ~10, which saturates the pulse screen effect to full white (an artifact).
  for (let f = 0; f <= FRAME; f++) {
    const b = readFileSync(`${OUT}/inputs/frame_${String(f).padStart(3, '0')}.bin`);
    const fr = new Uint8Array(b.buffer, b.byteOffset, FFT_HALF);
    const tm = new Uint8Array(b.buffer, b.byteOffset + FFT_HALF, 1024);
    let bs = 0;
    const bb = Math.min(16, fr.length);
    for (let i = 0; i < bb; i++) bs += fr[i];
    const be = bb > 0 ? bs / (bb * 255) : 0;
    nextRandom = mulberry32(0xc0ffee);
    renderer.setExportData(fr, tm, be);
    renderer.setFrameTime(f / FPS);
    renderer.drawFrame(styleConfig);
    if (f === FRAME) {
      mkdirSync(`${OUT}/ts/${STYLE}`, { recursive: true });
      const p = `${OUT}/ts/${STYLE}/frame_${String(FRAME).padStart(3, '0')}${LABEL}.png`;
      writeFileSync(p, canvas.toBuffer('image/png'));
      console.log(`wrote ${p}`);
    }
  }
  Math.random = originalRandom;
  expect(existsSync(`${OUT}/ts/${STYLE}/frame_${String(FRAME).padStart(3, '0')}${LABEL}.png`)).toBe(true);
});
