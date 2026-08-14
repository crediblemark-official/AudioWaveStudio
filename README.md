# AudioWave Studio

**AudioWave Studio** is a high-performance desktop audio visualizer built as a **Pure Rust Native Desktop Application** using **Slint UI 1.17** and **wgpu Hardware Acceleration**. It renders dynamic beat-synced visualizations and exports high-quality visualizer videos in real-time.

---

## Features

- **134+ Visualizer Styles** — Spectrum Bars, 3D Synthwave Landscapes, 3D Laser Equalizer Wall, 3D Orbit Spike Rainbow Wheel, Hologram Stage, 10+ Glass Box 3D Chambers, 10+ Waveform Renderers (3D Seismograph, Neon Dual Tube, Voxel Terrain, Spring Comb, Harmonic Web, etc.), Audio Prism 3D, Cyber Black Hole, and many more.
- **Beat-Synced Visuals** — Screen effects (shake, glitch, chromatic aberration, vignette, pulse) trigger on percussive beats, not smooth energy.
- **Customizable Particles** — 8+ particle styles with beat-responsive bursts, size, and velocity.
- **60 FPS Real-Time Preview** — GPU-accelerated live preview viewport rendered via wgpu with seamless CPU software fallback.
- **Video Export (MP4 & WebM)** — Hardware-accelerated wgpu MP4 (H.264, HEVC, AV1) and WebM (VP9 + Opus) export via FFmpeg pipeline with configurable FPS, resolution (720p, 1080p, 4K), aspect ratios (16:9, 9:16, 1:1), and bitrate.
- **Theme Presets & Custom Colors** — Switch between ready-made color themes (cyberpunk, synthwave, emerald, violet, gold) or tune custom primary, secondary, accent, and glow colors.
- **Custom Image Backgrounds** — Load your own image via the native file dialog or drag & drop.
- **Multi-Format Audio** — Load MP3, WAV, FLAC, OGG, and AAC via the native file dialog, or drag & drop a song directly onto the canvas.
- **Hardware Info Modal** — System RAM, GPU adapter, and FFmpeg hardware encoder capability detection (NVENC, QSV, VAAPI, AMF, VideoToolbox).

---

## Tech Stack

- **UI**: Slint 1.17 (native, compiled markup) with winit/x11/wayland backends
- **Language**: Pure Rust (edition 2021), no JavaScript runtime
- **Audio**: symphonia decoding + realfft spectrum analysis
- **Graphics**: wgpu GPU-accelerated 2D/3D rendering engine (custom WGSL shaders)
- **Export**: Full-Rust wgpu MP4/WebM video exporter with FFmpeg subprocess pipeline

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- **Linux**: `ffmpeg` for video export (`sudo apt install ffmpeg`)

### Build & Run

```bash
cargo build --release     # optimized desktop build
cargo run                 # run in debug mode
```

The release binary is produced at `target/release/audiowave-studio`.

## Testing

```bash
cargo test
```

The Slint UI files under `ui/` are compiled at build time via `build.rs` (`slint-build`).

---

## Directory Structure

```text
audiowave/
├── ui/                        # Modular Slint UI Markup Components
│   ├── app_window.slint       # Main Window layout composing all subcomponents
│   ├── navbar.slint           # Top Navigation Bar component
│   ├── audio_bar.slint        # Bottom Audio Player Bar component
│   ├── control_panel.slint    # Right Control Panel (Style cards, Colors, Bg, FX, Text)
│   ├── export_modal.slint     # Export MP4/WebM Video modal component
│   ├── hardware_modal.slint   # System Hardware info modal component
│   ├── custom_text.slint      # Custom Dynamic Text Overlays data model
│   └── about_modal.slint      # About modal component
├── src/                       # Rust application source
│   ├── lib.rs                 # Application entry point & 60 FPS live timer loop
│   ├── main.rs                # Binary main entry point
│   ├── app_state.rs           # SlintAppState, GpuPreviewEngine, & image buffer converters
│   ├── audio_decoder.rs       # Symphonia multi-format audio decoder (MP3, WAV, FLAC, OGG, AAC)
│   ├── audio_player.rs        # High-precision native audio playback controller
│   ├── callbacks.rs           # Event handlers binding Slint UI callbacks to Rust state
│   ├── config.rs              # VisualizerConfig, themes, background, & reactivity structs
│   ├── ffmpeg.rs              # FFmpeg resolution & installation helper
│   ├── fft_analyzer.rs        # Real-time RealFFT spectrum analyzer
│   ├── gpu2d/                 # 2D GPU rendering engine & WGSL shaders
│   ├── gpu_export.rs          # Full-Rust wgpu MP4/WebM video exporter
│   ├── hardware.rs            # Hardware detection (RAM, GPU, FFmpeg encoders)
│   └── renderers/             # 134+ Visualizer styles, background effects, screen FX, & text overlay
├── docs/                      # Documentation & parity references
├── build.rs                   # Slint UI build script
└── Cargo.toml                 # Root Cargo crate manifest
```

---

## License

This project is licensed under the MIT License.
