# AudioWave Studio

**AudioWave Studio** is a high-performance desktop audio visualizer built as a **Pure Rust Native Desktop Application** using **Slint UI 1.9** and **wgpu Hardware Acceleration**. It renders dynamic beat-synced visualizations and exports high-quality visualizer videos in real-time.

The app was migrated from a Tauri v2 (React 18 + TypeScript + Vite) webview architecture to a native Rust + Slint + wgpu stack for performance: eliminating WebView/Chromium IPC overhead, reducing RAM usage, and lowering rendering latency for 60 FPS real-time visualizers. See [MIGRATION_PLAN.md](MIGRATION_PLAN.md) for the full architecture document.

---

## Features

- **Beat-Synced Visuals** — Screen effects (shake, glitch, chromatic aberration, vignette, pulse) trigger on percussive beats, not smooth energy
- **Customizable Particles** — 8+ particle styles with beat-responsive bursts, size, and velocity
- **Multiple Visualizers** — Waveform, Spectrum Bars, Circular Wave, Radial Spectrum, Particles, Music Notes (14 renderer styles)
- **60 FPS Real-Time Preview** — GPU-accelerated live preview viewport rendered via wgpu
- **Video Export** — Hardware-accelerated wgpu MP4 (H.264) export via FFmpeg pipeline with configurable FPS and bitrate
- **Theme Presets** — Switch between ready-made color themes (cyberpunk, synthwave, emerald, violet, gold) from the Colors tab, or tune custom colors manually
- **Custom Image Backgrounds** — Load your own image via the native file dialog
- **Multi-Format Audio** — Load MP3, WAV, FLAC, OGG, and AAC via the native file dialog, or drag & drop a song directly onto the canvas (with an onboarding empty state when no track is loaded)
- **Hardware Info Modal** — System RAM, GPU adapter, and FFmpeg encoder capability detection

---

## Tech Stack

- **UI**: Slint 1.9 (native, compiled markup) with winit/x11/wayland backends
- **Language**: Pure Rust (edition 2021), no JavaScript runtime
- **Audio**: symphonia decoding + realfft spectrum analysis
- **Graphics**: wgpu GPU-accelerated 2D rendering engine (custom shaders) + femtovg fallback
- **Export**: Full-Rust wgpu MP4 video exporter with FFmpeg subprocess pipeline

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
│   ├── control_panel.slint    # Right Control Panel (5 Tabs: Style, Colors, Bg, FX, Text)
│   ├── export_modal.slint     # Export MP4 Video modal component
│   ├── hardware_modal.slint   # System Hardware info modal component
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
│   ├── gpu2d/                 # 2D GPU rendering engine & shaders (WGSL)
│   ├── gpu_export.rs          # Full-Rust wgpu MP4 video exporter
│   ├── hardware.rs            # Hardware detection (RAM, GPU, FFmpeg encoders)
│   └── renderers/             # 14 Visualizer styles, background effects, screen FX, & text overlay
├── legacy_tauri_backup/       # Archive of the legacy React/TS and Tauri config files
├── MIGRATION_PLAN.md          # Migration architecture document
├── build.rs                   # Slint UI build script
└── Cargo.toml                 # Root Cargo crate manifest
```

---

## License

This project is licensed under the MIT License.
