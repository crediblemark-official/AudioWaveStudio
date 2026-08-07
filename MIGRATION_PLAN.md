# Migration Plan & Architecture Document: AudioWave Studio (Tauri -> Slint UI)

## 📋 Overview

AudioWave Studio has been migrated from a **Tauri v2 (React 18 + TypeScript + Vite)** webview architecture to a **Pure Rust Native Desktop Application** using **Slint UI 1.9** and **wgpu Hardware Acceleration**.

### Rationale for Migration
1. **Performance & Low Latency**: WebView/Chromium introduces significant IPC overhead, higher RAM usage, and rendering delay for 60 FPS real-time audio visualizers.
2. **Native Audio Processing**: Direct integration between Rust audio decoding (`symphonia`), FFT analysis (`rustfft`), and native canvas rendering (`slint` + `wgpu`) eliminates web audio buffer detachment risks.
3. **Cross-Platform Portability**: Eliminates C-library system dependencies (`libasound2-dev` ALSA headers) on Linux environments by utilizing `slint` with `femtovg` and `winit` backends.

---

## 🏛️ Architecture & Component Mapping

| Legacy Tauri / React Component | Native Slint UI + Rust Replacement | Description |
| :--- | :--- | :--- |
| `src/App.tsx` | [lib.rs](file:///media/rasyiqi/7653717A1C07B131/audiowave/src/lib.rs) & [app_window.slint](file:///media/rasyiqi/7653717A1C07B131/audiowave/ui/app_window.slint) | Main app entry point, 60 FPS live timer loop, and workspace layout. |
| `src/components/Navbar.tsx` | `app_window.slint` (`Navbar Rectangle`) | Top bar with logo, track loader, custom image picker, hardware modal, and export triggers. |
| `src/components/VisualizerCanvas.tsx` | `app_window.slint` (`Image preview-frame`) | 60 FPS GPU-accelerated live preview viewport. |
| `src/components/AudioPlayerBar.tsx` | `app_window.slint` (`Audio Player Bar`) & [audio_player.rs](file:///media/rasyiqi/7653717A1C07B131/audiowave/src/audio_player.rs) | Native audio play, pause, stop, seek slider, timecode, and volume controls. |
| `src/components/ControlPanel.tsx` & `src/components/tabs/*` | `app_window.slint` (`Control Panel ScrollView`) & [callbacks.rs](file:///media/rasyiqi/7653717A1C07B131/audiowave/src/callbacks.rs) | 5 interactive tabs (Style, Colors, Background, Screen FX, Text Overlay) with full real-time config binding. |
| `src/components/ExportModal.tsx` | `app_window.slint` (`Export Modal`) & [gpu_export.rs](file:///media/rasyiqi/7653717A1C07B131/audiowave/src/gpu_export.rs) | Hardware-accelerated GPU MP4 video exporter with FFmpeg pipeline. |
| `src/components/HardwareModal.tsx` | `app_window.slint` (`Hardware Modal`) & [hardware.rs](file:///media/rasyiqi/7653717A1C07B131/audiowave/src/hardware.rs) | System RAM, GPU adapter, and FFmpeg encoder capability detection. |

---

## 📁 Modular Rust Code Base Structure

```
audiowave/
├── legacy_tauri_backup/            # Archive of legacy React/TS and Tauri config files
├── MIGRATION_PLAN.md               # Migration architecture document
├── README.md
├── Cargo.toml                      # Root Cargo crate manifest
├── build.rs                    # Slint UI build script
├── ui/                             # Modular Slint UI Markup Components
│   ├── about_modal.slint           # About modal component
│   ├── app_window.slint            # Main Window layout composing all subcomponents
│   ├── audio_bar.slint             # Bottom Audio Player Bar component
│   ├── control_panel.slint         # Right Control Panel (5 Tabs: Style, Colors, Bg, FX, Text)
│   ├── export_modal.slint          # Export MP4 Video modal component
│   ├── hardware_modal.slint        # System Hardware info modal component
│   └── navbar.slint                # Top Navigation Bar component
└── src/
        ├── app_state.rs            # SlintAppState, GpuPreviewEngine, & image buffer converters
        ├── audio_decoder.rs        # Symphonia multi-format audio decoder (MP3, WAV, FLAC, OGG, AAC)
        ├── audio_player.rs         # High-precision native audio playback controller
        ├── callbacks.rs            # Event handlers binding Slint UI callbacks to Rust state
        ├── config.rs               # VisualizerConfig, themes, background, & reactivity structs
        ├── ffmpeg.rs               # FFmpeg resolution & installation helper
        ├── fft_analyzer.rs         # Real-time RealFFT spectrum analyzer
        ├── gpu2d/                  # 2D GPU rendering engine & shaders
        ├── gpu_export.rs           # Full-Rust wgpu MP4 video exporter
        ├── hardware.rs             # Hardware detection (RAM, GPU, FFmpeg encoders)
        ├── lib.rs                  # Application entry point & 60 FPS live timer loop
        ├── main.rs                 # Binary main entry point
        └── renderers/              # 14 Visualizer styles, background effects, screen FX, & text overlay
```

---

## 🗄️ Archived & Purged Legacy Files

1. **Archived in `legacy_tauri_backup/`**:
   - `legacy_tauri_backup/src/` (React TSX components, hooks, services)
   - `legacy_tauri_backup/index.html`
   - `legacy_tauri_backup/tauri.conf.json`

2. **Purged (Zero Web Overhead)**:
   - `node_modules/` (Purged, saving storage space and removing JavaScript runtime overhead)
   - `package.json` / `package-lock.json`
   - `vite.config.ts` / `tsconfig.json`

---

## ⚙️ Building & Running

To build & run the native Slint UI application (Cargo manifest at project root):
```bash
cargo build --release   # optimized build -> target/release/audiowave-studio
cargo run               # debug build & run
```

To run unit tests:
```bash
cargo test
```
