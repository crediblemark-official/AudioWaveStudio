//! Export parity harness (Rust side).
//!
//! Synthesizes 1 second of audio, computes the same per-frame FFT pipeline as
//! `export_gpu`, dumps the shared input bins (freq + time) to
//! `/tmp/awcmp/inputs/`, and renders each visualizer style through the Rust
//! GPU renderer into PNG frames under `/tmp/awcmp/rust/<style>/`.
//!
//! The TS side (`src/services/compareExport.test.ts`) consumes the dumped
//! input bins so BOTH renderers see byte-identical frequency/time data, making
//! the pixel diff a pure measure of RENDERER parity (not audio/FFT drift).
//!
//! Run: `cargo test --test compare_export -- --ignored --nocapture`
//! (requires a Vulkan-capable GPU).
//!
//! Env overrides (used by the stress-config runs):
//!   COMPARE_CONFIG  path to the shared config JSON (default: scripts/compare-config.json)
//!   COMPARE_OUT     output directory            (default: /tmp/awcmp)

use audiowave_studio_lib::config::VisualizerConfig;
use audiowave_studio_lib::fft_analyzer::FftAnalyzer;
use audiowave_studio_lib::gpu2d::{GpuCanvas, GpuRenderer};
use audiowave_studio_lib::renderers::{draw_frame, RenderState};
use std::fs;

pub const W: u32 = 480;
pub const H: u32 = 270;
pub const FPS: u32 = 30;
pub const FRAMES: usize = 30;
const SAMPLE_RATE: u32 = 44100;
const FFT_SIZE: usize = 1024;

/// Style key strings MUST match the `VisualizerStyle` serde renames AND the
/// TS `VisualizerConfig.style` union — order is shared with the TS harness.
pub const STYLES: [(&str, &str); 19] = [
  ("spectrum", "spectrum"),
  ("radial", "radial"),
  ("oscilloscope", "oscilloscope"),
  ("equalizer", "equalizer"),
  ("minimal", "minimal"),
  ("waveformFill", "waveformFill"),
  ("circularBars", "circularBars"),
  ("smoothSpectrum", "smoothSpectrum"),
  ("pulseRings", "pulseRings"),
  ("vuMeter", "vuMeter"),
  ("auroraWave", "auroraWave"),
  ("flameFire", "flameFire"),
  ("spiralGalaxy", "spiralGalaxy"),
  ("threeD", "threeD"),
  ("api3D", "api3D"),
  ("neonCity3D", "neonCity3D"),
  ("speaker3D", "speaker3D"),
  ("speakerTrio", "speakerTrio"),
  ("speakerSplatter", "speakerSplatter"),
];

/// 1 second of synthetic "song": kick drum every 0.5s, sliding bass, AM
/// melody and a sparse hi-hat — produces evolving bass/mid/high content so
/// every style has something to react to.
fn synth_audio() -> Vec<f32> {
  let total = SAMPLE_RATE as usize;
  (0..total)
    .map(|i| {
      let t = i as f32 / SAMPLE_RATE as f32;
      let beat = (t * 2.0).floor();
      let phase = t - beat * 0.5;
      let kick = if phase < 0.12 {
        (1.0 - phase / 0.12) * (2.0 * std::f32::consts::PI * 85.0 * phase).sin()
      } else {
        0.0
      };
      let bass = 0.35 * (2.0 * std::f32::consts::PI * (55.0 + 12.0 * (t * 2.0).sin()) * t).sin();
      let melody = 0.2
        * (2.0 * std::f32::consts::PI * 440.0 * t).sin()
        * (0.5 + 0.5 * (2.0 * std::f32::consts::PI * 4.0 * t).sin());
      let hat = if (t % 0.25) < 0.008 {
        0.1 * (2.0 * std::f32::consts::PI * 8000.0 * t).sin()
      } else {
        0.0
      };
      (kick + bass + melody + hat).clamp(-1.0, 1.0)
    })
    .collect()
}

#[test]
#[ignore = "comparison harness: run cargo test --test compare_export -- --ignored"]
fn dump_compare_frames() {
  let manifest = env!("CARGO_MANIFEST_DIR");
  let out = std::env::var("COMPARE_OUT").unwrap_or_else(|_| "/tmp/awcmp".to_string());
  let cfg_path = std::env::var("COMPARE_CONFIG")
    .unwrap_or_else(|_| format!("{}/../scripts/compare-config.json", manifest));
  let cfg_json = fs::read_to_string(&cfg_path)
    .unwrap_or_else(|e| panic!("cannot read {}: {e}", cfg_path));
  let base_cfg: VisualizerConfig =
    serde_json::from_str(&cfg_json).expect("compare config JSON must deserialize");
  eprintln!("[rust] config={cfg_path} out={out}");

  fs::create_dir_all(format!("{out}/inputs")).unwrap();
  fs::create_dir_all(format!("{out}/rust")).unwrap();

  let samples = synth_audio();
  let analyzer = FftAnalyzer::new(FFT_SIZE);
  let smoothing = 0.8f32;
  let mut prev_smoothed: Option<Vec<f32>> = None;

  // Per-frame shared inputs, computed ONCE and reused by every style.
  let mut frame_inputs: Vec<(Vec<u8>, Vec<u8>)> = Vec::with_capacity(FRAMES);
  for frame in 0..FRAMES {
    let t = frame as f64 / FPS as f64;
    let start = (t * SAMPLE_RATE as f64) as usize;
    let window: Vec<f32> = samples[start..start + FFT_SIZE].to_vec();

    let (mag, _bass) = analyzer
      .compute_full_spectrum(&window)
      .expect("fft failed");
    let smoothed = match &mut prev_smoothed {
      Some(prev) if prev.len() == mag.len() => {
        for (p, &m) in prev.iter_mut().zip(mag.iter()) {
          *p = *p * smoothing + m * (1.0 - smoothing);
        }
        prev.clone()
      }
      _ => {
        prev_smoothed = Some(mag.clone());
        mag
      }
    };
    let freq_u8: Vec<u8> = smoothed
      .iter()
      .map(|m| (m.clamp(0.0, 1.0) * 255.0).round() as u8)
      .collect();
    let time_u8: Vec<u8> = window
      .iter()
      .map(|s| ((s + 1.0) * 128.0).clamp(0.0, 255.0) as u8)
      .collect();

    // inputs/frame_XXX.bin = [freq_u8 (512 bytes) | time_u8 (1024 bytes)]
    let mut bin = Vec::with_capacity(1536);
    bin.extend_from_slice(&freq_u8);
    bin.extend_from_slice(&time_u8);
    fs::write(format!("{out}/inputs/frame_{frame:03}.bin"), &bin).unwrap();

    frame_inputs.push((freq_u8, time_u8));
  }

  // Render every style with the SAME input bins.
  let mut gpu = pollster::block_on(GpuRenderer::new(W, H)).expect("GPU init failed");
  for (style_key, style_name) in STYLES {
    let mut config = base_cfg.clone();
    config.style = serde_json::from_value(serde_json::json!(style_key)).expect("style key");
    let dir = format!("{out}/rust/{style_name}");
    fs::create_dir_all(&dir).unwrap();

    let mut rstate = RenderState::new(config.reactivity.bar_count, 0xC0FFEE);
    for (frame, (freq_u8, time_u8)) in frame_inputs.iter().enumerate() {
      let mut canvas = GpuCanvas::new(W, H);
      draw_frame(
        &mut canvas,
        &mut rstate,
        &config,
        freq_u8,
        time_u8,
        frame as f32 / FPS as f32,
        true,
      );
      let mesh = canvas.finish();
      let rgba = gpu.render(&mesh);
      let path = format!("{dir}/frame_{frame:03}.png");
      image::save_buffer(&path, &rgba, W, H, image::ExtendedColorType::Rgba8)
        .unwrap_or_else(|e| panic!("save {path}: {e}"));
    }
    eprintln!("[rust] rendered {style_name}: {} frames", FRAMES);
  }

  // Dump the base config so the TS side can confirm it loaded the same file.
  fs::write(
    format!("{out}/inputs/config_sha.txt"),
    format!("{:?}", base_cfg.style),
  )
  .unwrap();
  eprintln!("[rust] done. inputs -> {out}/inputs, frames -> {out}/rust/<style>/");
}
