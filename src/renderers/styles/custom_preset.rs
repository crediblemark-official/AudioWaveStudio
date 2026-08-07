//! Custom Preset JSON style renderer (`customPreset`).
//!
//! Scans and renders user-created JSON visualizer presets from `./custom_styles/*.json`.
//! Allows users to add, edit, and share their own custom visualizer styles without modifying code.

use std::f32::consts::TAU;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LayerConfig {
  SpectrumRadial {
    #[serde(default = "default_bar_count")]
    bar_count: usize,
    #[serde(default = "default_inner_ratio")]
    inner_ratio: f32,
    #[serde(default = "default_bar_height")]
    max_bar_height: f32,
    #[serde(default = "default_pink")]
    color_primary: String,
    #[serde(default = "default_cyan")]
    color_secondary: String,
  },
  SpectrumBars {
    #[serde(default = "default_bar_count")]
    bar_count: usize,
    #[serde(default = "default_cyan")]
    color: String,
  },
  GlowingRing {
    #[serde(default = "default_inner_ratio")]
    radius_ratio: f32,
    #[serde(default = "default_cyan")]
    color: String,
    #[serde(default = "default_glow")]
    glow_intensity: f32,
  },
  WaveformLine {
    #[serde(default = "default_pink")]
    color: String,
  },
}

fn default_bar_count() -> usize {
  64
}
fn default_inner_ratio() -> f32 {
  0.25
}
fn default_bar_height() -> f32 {
  80.0
}
fn default_pink() -> String {
  "#ff007f".to_string()
}
fn default_cyan() -> String {
  "#00f0ff".to_string()
}
fn default_glow() -> f32 {
  14.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomStylePreset {
  pub id: String,
  pub name: String,
  pub icon: Option<String>,
  pub description: Option<String>,
  pub layers: Vec<LayerConfig>,
}

/// Helper to load a preset from JSON file
pub fn load_preset_from_file(path: &Path) -> Option<CustomStylePreset> {
  if let Ok(content) = fs::read_to_string(path) {
    serde_json::from_str(&content).ok()
  } else {
    None
  }
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  // Try to load preset from `./custom_styles/custom_cyber_ring.json` or first `.json` file found
  let custom_dir = Path::new("./custom_styles");
  let mut preset = None;

  if custom_dir.exists() {
    if let Ok(entries) = fs::read_dir(custom_dir) {
      for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("json") {
          if let Some(loaded) = load_preset_from_file(&p) {
            preset = Some(loaded);
            break;
          }
        }
      }
    }
  }

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  if let Some(p_data) = preset {
    for layer in &p_data.layers {
      match layer {
        LayerConfig::SpectrumRadial {
          bar_count,
          inner_ratio,
          max_bar_height,
          color_primary,
          color_secondary,
        } => {
          let num_bars = (*bar_count).clamp(12, 180);
          let base_r = (width.min(height) * inner_ratio).clamp(40.0, 300.0);
          let step = (freq.len() / (num_bars / 2)).max(1);

          let col1 = Color::hex(color_primary);
          let col2 = Color::hex(color_secondary);

          for i in 0..num_bars {
            let angle = (i as f32 / num_bars as f32) * TAU;
            let bin_i = (i * step).min(freq.len().saturating_sub(1));
            let raw_v = *freq.get(bin_i).unwrap_or(&0) as f32 / 255.0;

            let bh = (raw_v * sensitivity * max_bar_height).clamp(2.0, max_bar_height * 1.5);
            let r1 = base_r + be * 10.0;
            let r2 = r1 + bh;

            let x1 = center_x + angle.cos() * r1;
            let y1 = center_y + angle.sin() * r1;
            let x2 = center_x + angle.cos() * r2;
            let y2 = center_y + angle.sin() * r2;

            let bar_col = if i % 2 == 0 { col1 } else { col2 };
            c.set_stroke(Fill::Solid(bar_col));
            c.set_line_width(3.0);
            c.set_shadow(bar_col, 8.0 + bs * 6.0);
            c.stroke_line(x1, y1, x2, y2);
          }
        }
        LayerConfig::GlowingRing {
          radius_ratio,
          color,
          glow_intensity,
        } => {
          let ring_r = (width.min(height) * radius_ratio).clamp(20.0, 350.0) + (be * 15.0);
          let ring_col = Color::hex(color);

          c.set_stroke(Fill::Solid(ring_col));
          c.set_line_width(3.0);
          c.set_shadow(ring_col, *glow_intensity + bs * 10.0);
          c.stroke_circle(center_x, center_y, ring_r);
        }
        LayerConfig::SpectrumBars { bar_count, color } => {
          let num_bars = (*bar_count).clamp(16, 128);
          let bar_w = width / num_bars as f32;
          let bar_col = Color::hex(color);

          c.set_fill(Fill::Solid(bar_col));
          c.set_shadow(bar_col, 10.0);

          for i in 0..num_bars {
            let bin = (i * freq.len() / num_bars).min(freq.len().saturating_sub(1));
            let val = (*freq.get(bin).unwrap_or(&0) as f32 / 255.0 * sensitivity).clamp(0.0, 1.2);

            let bh = val * height * 0.3;
            let bx = i as f32 * bar_w;
            let by = height - bh;

            c.fill_rect(bx + 1.0, by, bar_w - 2.0, bh);
          }
        }
        LayerConfig::WaveformLine { color } => {
          let wave_col = Color::hex(color);
          let steps = 80usize;
          let mut pts = Vec::with_capacity(steps + 1);

          for i in 0..=steps {
            let x = (i as f32 / steps as f32) * width;
            let bin = (i * freq.len() / steps).min(freq.len().saturating_sub(1));
            let val = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

            let y = center_y + (val - 0.5) * height * 0.3 * sensitivity + (rot.sin() * 10.0);
            pts.push((x, y));
          }

          c.set_stroke(Fill::Solid(wave_col));
          c.set_line_width(3.0);
          c.set_shadow(wave_col, 12.0);
          c.stroke_polyline(&pts);
        }
      }
    }
  } else {
    // Fallback indicator when no custom preset file exists
    let cyan = Color::rgba(0.0, 0.85, 1.0, 0.9);
    c.set_stroke(Fill::Solid(cyan));
    c.set_line_width(2.0);
    c.stroke_circle(center_x, center_y, width.min(height) * 0.2);
  }

  c.restore();
}
