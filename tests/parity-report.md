# Export parity: TS canvas vs Rust GPU (480x270, 30 fps, 1s, shared input bins)
config: scripts/compare-config.json — regenerate with `npm run compare:export`

| style | avg MAD | max MAD | %px diff>32 | %px diff<=2 |
|---|---|---|---|---|
| minimal | 0.96 | 72.7 | 0.5 | 93.0 |
| vuMeter | 0.99 | 186.0 | 0.6 | 92.2 |
| equalizer | 1.62 | 86.0 | 0.2 | 72.5 |
| radial | 1.67 | 137.7 | 0.7 | 84.7 |
| smoothSpectrum | 2.02 | 172.0 | 1.1 | 85.4 |
| neonCity3D | 2.41 | 95.3 | 0.8 | 82.7 |
| oscilloscope | 2.53 | 122.7 | 1.7 | 81.2 |
| circularBars | 2.59 | 159.3 | 2.3 | 93.7 |
| spectrum | 2.98 | 112.3 | 2.2 | 82.2 |
| speakerTrio | 5.00 | 162.0 | 5.5 | 75.0 |
| auroraWave | 5.11 | 50.3 | 0.0 | 38.0 |
| flameFire | 6.00 | 118.7 | 7.2 | 81.3 |
| threeD | 7.36 | 189.0 | 3.3 | 54.4 |
| pulseRings | 7.60 | 88.0 | 8.8 | 72.6 |
| spiralGalaxy | 9.05 | 129.3 | 9.6 | 61.7 |
| speaker3D | 10.80 | 167.0 | 11.4 | 44.2 |
| waveformFill | 13.24 | 177.0 | 9.0 | 46.6 |
| api3D | 15.52 | 237.7 | 14.4 | 24.0 |
| speakerSplatter | 24.06 | 200.3 | 26.9 | 58.7 |

MAD = mean over R/G/B of |a-b| averaged across all pixels and frames (0-255 scale).
TS particle styles use a seeded Math.random shim; Rust uses a seeded RNG (different seeds) — expect high MAD there.
The TS reference here is @napi-rs/canvas (Skia); its shadowBlur/AA can differ slightly from Chrome.
