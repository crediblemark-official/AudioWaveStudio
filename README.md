# AudioWave Studio

**AudioWave Studio** is a high-performance desktop audio visualizer built with **Tauri v2**, **React 19**, **TypeScript**, and **Rust**. It renders dynamic beat-synced visualizations and exports high-quality visualizer videos in real-time.

---

## Features

- **Beat-Synced Visuals** — Screen effects (shake, glitch, chromatic aberration, vignette, pulse) trigger on percussive beats, not smooth energy
- **Customizable Particles** — 8+ particle styles with beat-responsive bursts, size, and velocity
- **Multiple Visualizers** — Waveform, Spectrum Bars, Circular Wave, Radial Spectrum, Particles, Music Notes
- **Fullscreen Mode** — F11, double-click, or toggle button; hides chrome for distraction-free viewing
- **Keyboard Shortcuts** — Space (play/pause), S (stop), arrows (seek/volume), M (mute)
- **Custom Titlebar** — Hidden native decorations; drag via navbar with minimize/maximize/close buttons
- **Video Export** — Render to MP4 (H.264) via FFmpeg with configurable FPS and bitrate
- **Presets System** — Save and switch between themes
- **Drag & Drop** — Load MP3, WAV, FLAC, AAC, and more via drag-drop or native file dialog

---

## Tech Stack

- **Frontend**: React 19, TypeScript, Vite, HTML5 Canvas, Lucide React
- **Desktop**: Tauri v2 (Rust)
- **Audio**: Web Audio API + Rust FFT (symphonia, realfft)
- **Graphics**: GPU-accelerated Canvas 2D via wgpu
- **Export**: FFmpeg subprocess (system or bundled)

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [npm](https://www.npmjs.com/)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Tauri v2 system deps](https://v2.tauri.app/start/prerequisites/) for your OS
- **Linux**: `ffmpeg` for video export (`sudo apt install ffmpeg`)

### Install & Run

```bash
git clone https://github.com/crediblemark-official/AudioWaveStudio.git
cd AudioWaveStudio
npm install
npm run tauri dev       # desktop app
# or
npm run dev              # web-only preview
```

## Building

```bash
npm run tauri build
```

Output in `src-tauri/target/release/bundle/`. The `.deb` on Linux depends on system `ffmpeg` (auto-installed via apt).

---

## Directory Structure

```text
audiowave/
├── public/                 # Static web assets
├── src/                    # Frontend source code
│   ├── assets/             # Images and styles
│   ├── components/         # React components & visualizer tabs
│   ├── services/           # Audio engine, canvas renderer, & export services
│   ├── types/              # TypeScript interface definitions
│   └── utils/              # Presets and helper functions
├── src-tauri/              # Rust Tauri backend
│   ├── src/                # Audio decoding, FFT analyzer, & video encoding
│   ├── capabilities/       # Tauri permissions & security
│   └── tauri.conf.json     # Tauri configuration
├── index.html              # HTML entry point
├── package.json            # Node project configuration
└── vite.config.ts          # Vite bundler configuration
```

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
