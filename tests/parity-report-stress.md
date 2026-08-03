# Export parity: TS canvas vs Rust GPU (480x270, 30 fps, 1s, shared input bins)
config: /media/rasyiqi/7653717A1C07B131/audiowave/scripts/compare-config-stress.json — regenerate with `npm run compare:export`

| style | avg MAD | max MAD | %px diff>32 | %px diff<=2 |
|---|---|---|---|---|
| vuMeter | 2.79 | 132.7 | 2.0 | 81.9 |
| minimal | 3.40 | 128.7 | 2.4 | 78.7 |
| oscilloscope | 4.12 | 136.0 | 2.8 | 71.7 |
| radial | 4.25 | 151.0 | 2.5 | 66.7 |
| neonCity3D | 4.34 | 128.7 | 1.9 | 69.9 |
| smoothSpectrum | 4.75 | 173.7 | 2.9 | 67.3 |
| circularBars | 5.02 | 160.0 | 4.2 | 79.2 |
| equalizer | 5.17 | 128.7 | 3.7 | 58.3 |
| auroraWave | 5.93 | 128.0 | 1.6 | 37.0 |
| pulseRings | 6.82 | 128.7 | 6.4 | 72.1 |
| spectrum | 8.28 | 196.3 | 5.5 | 47.0 |
| speakerTrio | 9.82 | 161.3 | 10.3 | 54.2 |
| waveformFill | 10.86 | 173.3 | 3.3 | 10.5 |
| flameFire | 11.61 | 143.7 | 14.7 | 59.2 |
| threeD | 12.58 | 161.0 | 9.1 | 31.9 |
| api3D | 20.31 | 199.0 | 21.5 | 22.2 |
| speaker3D | 20.95 | 203.3 | 23.5 | 9.8 |
| speakerSplatter | 23.25 | 208.3 | 32.7 | 42.5 |
| spiralGalaxy | 30.94 | 172.7 | 32.5 | 35.8 |

MAD = mean over R/G/B of |a-b| averaged across all pixels and frames (0-255 scale).
TS particle styles use a seeded Math.random shim; Rust uses a seeded RNG (different seeds) — expect high MAD there.
The TS reference here is @napi-rs/canvas (Skia); its shadowBlur/AA can differ slightly from Chrome.
