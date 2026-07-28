# AudioWave Studio 🎵⚡

**AudioWave Studio** is a high-performance desktop audio visualizer application built with **Tauri v2**, **React 19**, **TypeScript**, and **Vite**. It enables creators, musicians, and video producers to render dynamic audio spectrum visualizations and export high-quality visualizer videos in real-time.

---

## ✨ Features

- 🎨 **Dynamic Audio Visualizer Renderers**
  - Multiple visualizer styles: Waveform, Spectrum Bars, Circular Wave, Radial Spectrum, and Dynamic Particle systems.
- 🎛️ **Extensive Customization Options**
  - **Backgrounds**: Custom colors, smooth gradients, background images, and video loops.
  - **Color Themes**: Customizable primary, secondary, and accent colors with glow effects.
  - **Audio Reactivity**: Adjust sensitivity for Bass, Mid, and Treble frequency bands.
  - **Typography & Text Overlays**: Add song titles, artist names, custom fonts, size, position, and shadows.
  - **Presets System**: Save and quickly switch between custom visualization themes.
- ⚡ **Native Performance with Rust & Tauri v2**
  - Fast audio FFT analysis and GPU-accelerated canvas rendering.
  - Low memory footprint compared to traditional Electron apps.
- 🎬 **Video Export & Recording**
  - Integrated recording studio for rendering visualizers to video formats (WebM/MP4) with custom framerate and bitrate settings.
- 📁 **Native File System Integration**
  - Drag & drop local audio files (MP3, WAV, FLAC, AAC, etc.) or select them via native OS file dialogs.
- 🌓 **Modern Glassmorphic UI**
  - Sleek, intuitive dark mode user interface designed for maximum productivity.

---

## 🛠️ Tech Stack

- **Frontend**: React 19, TypeScript, Vite, HTML5 Canvas API, Lucide React Icons
- **Desktop Framework**: Tauri v2 (Rust)
- **Audio Processing**: Web Audio API & Rust FFT analysis engine
- **Video Exporting**: Web MediaRecorder API / Native FFmpeg integration

---

## 🚀 Getting Started

### Prerequisites

Ensure you have the following installed on your machine:
- [Node.js](https://nodejs.org/) (v18 or higher)
- [npm](https://www.npmjs.com/) or [pnpm](https://pnpm.io/)
- [Rust & Cargo](https://www.rust-lang.org/tools/install)
- [Tauri v2 Prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system (Linux / macOS / Windows)

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/crediblemark-official/AudioWaveStudio.git
   cd AudioWaveStudio
   ```

2. **Install frontend dependencies:**
   ```bash
   npm install
   ```

---

## 💻 Development & Usage

### Run Web Development Server
To run only the web frontend in the browser:
```bash
npm run dev
```

### Run Tauri Desktop Application
To launch the full desktop app with Rust backend integration:
```bash
npm run tauri dev
```

---

## 📦 Building for Production

To build the desktop application bundle for your operating system:

```bash
npm run tauri build
```

The output installer/executable will be generated in `src-tauri/target/release/bundle/`.

---

## 📂 Directory Structure

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

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
