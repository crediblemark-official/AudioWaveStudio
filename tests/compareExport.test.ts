// Export parity harness (TS side).
//
// Renders each visualizer style with the SAME canvas path the real canvas
// exporter uses (CanvasRenderer.drawFrame + setExportData), feeding it the
// byte-identical freq/time bins dumped by the Rust harness
// (src-tauri/tests/compare_export.rs), then diffs against the Rust GPU frames
// and reports the mean absolute pixel difference per style.
//
// NOTE: this file lives OUTSIDE src/ on purpose — it needs node built-ins
// (@types/node is not installed) and it is slow (~5 min), so it is excluded
// from `npm run build` (tsconfig includes only src) and `npm test` (vitest
// includes only src/**/*.test.ts).
//
// Parity assumptions:
// - Both sides are fed the SAME raw freq/time bins; each side then applies
//   its OWN bass envelope (TS canvasRenderer.ts smooths exportBassEnergy with
//   `+= (target - current) * 0.2`; Rust advance_envelope mirrors it), so raw
//   input is the correct shared unit.
// - TS particle styles draw from a SEEDED Math.random shim (mulberry32,
//   re-seeded per style) instead of the native RNG, so the harness is
//   reproducible run-to-run. The stream still differs from Rust's seeded Rng,
//   so particle MAD stays elevated (RNG-mismatch noise) — but it no longer
//   drifts between runs. Particle styles are excluded from the regression
//   threshold below.
//
// The report is committed at tests/parity-report.md (stress runs write
// tests/parity-report-stress.md instead), so parity drift shows up as an
// ordinary git diff.
//
// Run (from the repo root):
//   npm run compare:export
// or manually:
//   1. cargo test --manifest-path src-tauri/Cargo.toml --test compare_export -- --ignored
//   2. npx vitest run --config vitest.compare.config.ts

import { afterAll, describe, expect, it } from 'vitest';
import { createCanvas, GlobalFonts, loadImage } from '@napi-rs/canvas';
import { readFileSync, existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { basename, resolve } from 'node:path';
import { CanvasRenderer } from '../src/services/canvasRenderer';
import type { VisualizerConfig } from '../src/types/visualizer';

// Env overrides (see header): COMPARE_CONFIG / COMPARE_OUT select a shared
// config and output dir, letting the stress-config variant run side-by-side
// with the default baseline without overwriting its frames or report.
const OUT = process.env.COMPARE_OUT ?? '/tmp/awcmp';
const RUST = `${OUT}/rust`;
const TS = `${OUT}/ts`;
const IN = `${OUT}/inputs`;

const W = 480;
const H = 270;
const FPS = 30;
const FRAMES = 30;
const FFT_HALF = 512;

// Must match the Rust harness STYLES list (serde renames / TS style union).
const STYLES: string[] = [
  'spectrum', 'radial', 'oscilloscope', 'equalizer', 'minimal',
  'waveformFill', 'circularBars', 'smoothSpectrum', 'pulseRings',
  'vuMeter', 'auroraWave', 'flameFire', 'spiralGalaxy', 'threeD',
  'api3D', 'neonCity3D', 'speaker3D', 'speakerTrio', 'speakerSplatter',
];

// --- Seeded Math.random shim -----------------------------------------------
//
// The TS renderers call Math.random() directly for particles/music-notes,
// which made the particle styles' MAD drift between harness runs. Overriding
// it with a seeded mulberry32 and re-seeding per style makes the whole run
// deterministic: the stream still differs from Rust's seeded Rng (positions
// stay different — this harness measures renderer parity, not RNG parity),
// but the noise floor no longer wanders run-to-run, so two runs of the same
// config produce byte-identical TS frames.

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

// Shared with the Rust harness's RenderState seed (0xC0FFEE) for symmetry;
// the algorithms/order differ, so positions still won't match Rust.
const TS_RNG_SEED = 0xc0ffee;
const originalRandom = Math.random.bind(Math);
let nextRandom = mulberry32(TS_RNG_SEED);
Math.random = () => nextRandom();

// ---------------------------------------------------------------------------
// Generic-family font aliasing (Skia vs fontconfig)
//
// @napi-rs/canvas runs on Skia, which does NOT consult fontconfig for generic
// CSS families: `c.font = "700 55px monospace"` silently resolves to Skia's
// default sans-serif fallback, making the TS reference measure ~10% wider text
// than the real preview (Chromium/WebView resolve `monospace` via fontconfig
// to e.g. Cousine / DejaVu Sans Mono). The Rust GPU renderer mirrors the
// browser: it picks the actual mono face (DejaVuSansMono-Bold here), so the
// harness must alias the same family or every style's text band diverges by a
// constant ~40px offset. `registerFromPath` lets us pin `monospace` (and the
// generic sans/serif aliases) to the SAME fonts the Rust side loads.
//
// Registration is idempotent and cheap; paths that don't exist are skipped so
// the harness still runs on machines without the exact font files (the
// reference then falls back to Skia's default and the text-band MAD just stays
// elevated — it does not crash).

const GENERIC_FONT_ALIASES: Record<string, string[]> = {
  // Order matters: first existing file wins. Keep mono ahead of the generic
  // sans/serif so `monospace` never resolves to a proportional face.
  monospace: [
    '/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf',
    '/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf',
    '/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Oblique.ttf',
    '/usr/share/fonts/truetype/dejavu/DejaVuSansMono-BoldOblique.ttf',
    '/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf',
    '/usr/share/fonts/truetype/liberation/LiberationMono-Bold.ttf',
    '/usr/share/fonts/truetype/croscore/Cousine-Regular.ttf',
    '/usr/share/fonts/truetype/croscore/Cousine-Bold.ttf',
    'C:\\Windows\\Fonts\\consola.ttf',
    'C:\\Windows\\Fonts\\consolab.ttf',
    '/System/Library/Fonts/Supplemental/Courier New.ttf',
    '/System/Library/Fonts/Supplemental/Courier New Bold.ttf',
  ],
  'sans-serif': [
    '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf',
    '/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf',
    '/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf',
    '/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf',
    'C:\\Windows\\Fonts\\segoeui.ttf',
    'C:\\Windows\\Fonts\\segoeuib.ttf',
    '/System/Library/Fonts/Supplemental/Arial.ttf',
    '/System/Library/Fonts/Supplemental/Arial Bold.ttf',
  ],
  serif: [
    '/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf',
    '/usr/share/fonts/truetype/dejavu/DejaVuSerif-Bold.ttf',
    '/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf',
    '/usr/share/fonts/truetype/liberation/LiberationSerif-Bold.ttf',
    'C:\\Windows\\Fonts\\times.ttf',
    'C:\\Windows\\Fonts\\timesbd.ttf',
    '/System/Library/Fonts/Supplemental/Times New Roman.ttf',
    '/System/Library/Fonts/Supplemental/Times New Roman Bold.ttf',
  ],
};

function registerGenericFontAliases() {
  for (const [family, paths] of Object.entries(GENERIC_FONT_ALIASES)) {
    for (const p of paths) {
      if (!existsSync(p)) continue;
      try {
        GlobalFonts.registerFromPath(p, family);
      } catch {
        // ignore per-file failures; fall back to the next candidate
      }
    }
  }
}
registerGenericFontAliases();

afterAll(() => {
  Math.random = originalRandom;
});

function loadConfig(): VisualizerConfig {
  const env = process.env.COMPARE_CONFIG;
  const p = env
    ? resolve(process.cwd(), env)
    : fileURLToPath(new URL('../scripts/compare-config.json', import.meta.url));
  return JSON.parse(readFileSync(p, 'utf8')) as VisualizerConfig;
}

function meanAbsDiff(a: Uint8ClampedArray, b: Uint8ClampedArray): { mad: number; maxMad: number; gt32: number; close: number } {
  let sum = 0;
  let max = 0;
  let gt32 = 0;
  let close = 0;
  const n = a.length / 4;
  for (let i = 0; i < n; i++) {
    const o = i * 4;
    const dr = Math.abs(a[o] - b[o]);
    const dg = Math.abs(a[o + 1] - b[o + 1]);
    const db = Math.abs(a[o + 2] - b[o + 2]);
    const d = (dr + dg + db) / 3;
    sum += d;
    if (d > max) max = d;
    if (d > 32) gt32++;
    if (d <= 2) close++;
  }
  return { mad: sum / n, maxMad: max, gt32: gt32 / n, close: close / n };
}

describe('export parity: TS canvas vs Rust GPU', () => {
  // 19 styles x 30 frames with async PNG decode — bump the 5s default timeout.
  // Skip (not pass) when the Rust dump hasn't been produced yet.
  const hasInputs = existsSync(`${IN}/frame_000.bin`);
  it.skipIf(!hasInputs)('renders 1s per style and reports pixel diff', { timeout: 600_000 }, async () => {
    // Sanity: the shim must reproduce the same stream when re-seeded, or
    // run-to-run reproducibility silently breaks.
    nextRandom = mulberry32(12345);
    const r1 = [Math.random(), Math.random(), Math.random(), Math.random()];
    nextRandom = mulberry32(12345);
    const r2 = [Math.random(), Math.random(), Math.random(), Math.random()];
    expect(r1).toEqual(r2);
    expect(new Set(r1).size).toBeGreaterThan(1); // not constant

    const baseConfig = loadConfig();
    mkdirSync(TS, { recursive: true });

    const rows: { style: string; mad: number; maxMad: number; gt32: number; close: number; frames: number }[] = [];

    for (const style of STYLES) {
      const config: VisualizerConfig = { ...baseConfig, style: style as VisualizerConfig['style'] };
      const styleTsDir = `${TS}/${style}`;
      mkdirSync(styleTsDir, { recursive: true });

      // Fresh renderer per style (mirrors a fresh export session).
      const renderer = new CanvasRenderer();
      const canvas = createCanvas(W, H);
      renderer.init(canvas as unknown as HTMLCanvasElement);
      const ctx = canvas.getContext('2d');

      // Re-seed right before the frame loop so EVERY style consumes the same
      // deterministic stream, independent of how many draws init/particle-setup
      // consumed earlier.
      nextRandom = mulberry32(TS_RNG_SEED);

      let styleMad = 0;
      let styleMax = 0;
      let styleGt32 = 0;
      let styleClose = 0;
      let framesDone = 0;

      for (let f = 0; f < FRAMES; f++) {
        const bin = readFileSync(`${IN}/frame_${String(f).padStart(3, '0')}.bin`);
        const freq = new Uint8Array(bin.buffer, bin.byteOffset, FFT_HALF);
        const time = new Uint8Array(bin.buffer, bin.byteOffset + FFT_HALF, 1024);

        // Rust advance_envelope derives raw bass from the same freq bytes.
        let bassSum = 0;
        const bassBins = Math.min(16, freq.length);
        for (let i = 0; i < bassBins; i++) bassSum += freq[i];
        const bassEnergy = bassBins > 0 ? bassSum / (bassBins * 255) : 0;

        renderer.setExportData(freq, time, bassEnergy);
        renderer.setFrameTime(f / FPS);
        renderer.drawFrame(config);

        const tsData = ctx!.getImageData(0, 0, W, H).data;
        // Keep only a few TS frames for visual inspection (PNG encode is slow).
        if (f === 0 || f === 15 || f === FRAMES - 1) {
          writeFileSync(`${styleTsDir}/frame_${String(f).padStart(3, '0')}.png`, canvas.toBuffer('image/png'));
        }

        const rustPng = `${RUST}/${style}/frame_${String(f).padStart(3, '0')}.png`;
        if (!existsSync(rustPng)) {
          console.log(`  [warn] missing ${rustPng}`);
          continue;
        }
        const img = await loadImage(rustPng);
        const c2 = createCanvas(W, H);
        const c2ctx = c2.getContext('2d');
        c2ctx.drawImage(img, 0, 0, W, H);
        const rustData = c2ctx.getImageData(0, 0, W, H).data;

        const m = meanAbsDiff(tsData, rustData);
        styleMad += m.mad;
        styleMax = Math.max(styleMax, m.maxMad);
        styleGt32 += m.gt32;
        styleClose += m.close;
        framesDone++;
      }

      if (framesDone > 0) {
        const row = {
          style,
          mad: styleMad / framesDone,
          maxMad: styleMax,
          gt32: styleGt32 / framesDone,
          close: styleClose / framesDone,
          frames: framesDone,
        };
        rows.push(row);
        console.log(
          `  [ts] ${style.padEnd(16)} MAD=${row.mad.toFixed(2)}  max=${row.maxMad.toFixed(1)}  >32px=${(row.gt32 * 100).toFixed(1)}%  match<=2=${(row.close * 100).toFixed(1)}%`,
        );
      }
    }

    // Report is written BEFORE the assertions so a failing guard never eats
    // the measurement (the whole point of the harness is the numbers).
    rows.sort((a, b) => a.mad - b.mad);
    const cfgLabel = process.env.COMPARE_CONFIG ?? 'scripts/compare-config.json';
    const lines: string[] = [
      '# Export parity: TS canvas vs Rust GPU (480x270, 30 fps, 1s, shared input bins)',
      `config: ${cfgLabel} — regenerate with \`npm run compare:export\``,
      '',
      '| style | avg MAD | max MAD | %px diff>32 | %px diff<=2 |',
      '|---|---|---|---|---|',
      ...rows.map((r) => `| ${r.style} | ${r.mad.toFixed(2)} | ${r.maxMad.toFixed(1)} | ${(r.gt32 * 100).toFixed(1)} | ${(r.close * 100).toFixed(1)} |`),
      '',
      'MAD = mean over R/G/B of |a-b| averaged across all pixels and frames (0-255 scale).',
      'TS particle styles use a seeded Math.random shim; Rust uses a seeded RNG (different seeds) — expect high MAD there.',
      'The TS reference here is @napi-rs/canvas (Skia); its shadowBlur/AA can differ slightly from Chrome.',
      '',
    ];
    const reportBody = lines.join('\n');
    const reportTmp = `${OUT}/report.md`;
    writeFileSync(reportTmp, reportBody);
    // Committed repo copy so parity drift shows up as a git diff. The
    // stress-config variant writes a distinct file instead of clobbering the
    // baseline report.
    const cfgStem = (process.env.COMPARE_CONFIG ? basename(process.env.COMPARE_CONFIG) : 'compare-config.json').replace(/\.json$/i, '');
    const stem = cfgStem.replace(/^compare-config-?/, '');
    const reportRepo = fileURLToPath(new URL(stem ? `parity-report-${stem}.md` : 'parity-report.md', import.meta.url));
    writeFileSync(reportRepo, reportBody);
    console.log(`\nReport written to ${reportTmp} (repo copy: ${reportRepo})\n`);
    console.log(lines.join('\n'));

    // Every style must have produced comparable frames — a silently missing
    // style would otherwise pass the harness without measuring anything.
    expect(rows.length).toBe(STYLES.length);

    // Loose regression guard for deterministic (non-particle) styles. MAD > 18
    // previously indicated real renderer bugs (waveformFill's fan-anchor
    // overflow measured ~21). Particle styles are excluded because their MAD
    // is RNG-mismatch noise (seeded TS shim vs seeded Rust Rng). Headroom over
    // the current worst deterministic style (13.24) tolerates Skia-vs-Chrome
    // shadowBlur/AA differences.
    const DETERMINISTIC = new Set([
      'spectrum', 'radial', 'oscilloscope', 'equalizer', 'minimal',
      'waveformFill', 'circularBars', 'smoothSpectrum', 'vuMeter',
      'neonCity3D', 'speakerTrio',
    ]);
    for (const r of rows) {
      if (DETERMINISTIC.has(r.style)) {
        expect(r.mad, `${r.style} parity regression (MAD ${r.mad.toFixed(2)} > 18)`).toBeLessThan(18);
      }
    }
  });
});
