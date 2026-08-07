//! Custom Preset JSON style renderer (`customPreset`).
//!
//! Scans and renders user-created JSON visualizer presets from `./custom_styles/*.json`.
//! Supports complex multi-layered compositions: radial spectrums, equalizer bars,
//! glowing neon rings, oscilloscope waveforms, particle bursts, 3D wireframe tunnels,
//! dual studio speakers, and raw SVG vector code paths (`svg_path`)!

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
  GlowingDisc {
    #[serde(default = "default_inner_ratio")]
    radius_ratio: f32,
    #[serde(default = "default_pink")]
    color: String,
  },
  WaveformLine {
    #[serde(default = "default_pink")]
    color: String,
  },
  ParticlesBurst {
    #[serde(default = "default_bar_count")]
    count: usize,
    #[serde(default = "default_cyan")]
    color: String,
  },
  WireframeTunnel {
    #[serde(default = "default_pink")]
    color: String,
  },
  DualSpeakers {
    #[serde(default = "default_pink")]
    color: String,
  },
  SvgPath {
    path_data: String,
    #[serde(default = "default_cyan")]
    color: String,
    #[serde(default = "default_scale")]
    scale: f32,
    #[serde(default = "default_motion")]
    motion: String,
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
fn default_scale() -> f32 {
  1.0
}
fn default_motion() -> String {
  "pulse".to_string()
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
        LayerConfig::SvgPath {
          path_data,
          color,
          scale,
          motion,
        } => {
          let svg_col = Color::hex(color);
          c.set_stroke(Fill::Solid(svg_col));
          c.set_line_width(2.5);
          c.set_shadow(svg_col, 12.0);

          let m_scale = match motion.as_str() {
            "pulse" | "bounce" => scale * (1.0 + be * 0.2),
            _ => *scale,
          };
          let m_angle = if motion == "spin" { rot } else { 0.0 };

          // Parse SVG path coordinates M x y L x y Z
          let tokens: Vec<&str> = path_data.split_whitespace().collect();
          let mut pts: Vec<(f32, f32)> = Vec::new();
          let mut idx = 0;

          while idx < tokens.len() {
            let cmd = tokens[idx];
            if (cmd == "M" || cmd == "L") && idx + 2 < tokens.len() {
              if let (Ok(px), Ok(py)) = (tokens[idx + 1].parse::<f32>(), tokens[idx + 2].parse::<f32>()) {
                let sx = (px - 50.0) * m_scale;
                let sy = (py - 50.0) * m_scale;
                let rx = sx * m_angle.cos() - sy * m_angle.sin();
                let ry = sx * m_angle.sin() + sy * m_angle.cos();

                pts.push((center_x + rx, center_y + ry));
              }
              idx += 3;
            } else {
              idx += 1;
            }
          }

          if pts.len() > 1 {
            c.stroke_polyline(&pts);
          }
        }
        LayerConfig::WireframeTunnel { color } => {
          let wire_col = Color::hex(color).with_alpha(0.4);
          c.set_stroke(Fill::Solid(wire_col));
          c.set_line_width(1.5);

          let num_frames = 12usize;
          for f in 0..num_frames {
            let z_t = ((f as f32 / num_frames as f32) + rot * 0.2) % 1.0;
            let fw = width * (0.15 + z_t * 0.85);
            let fh = height * (0.15 + z_t * 0.85);
            let fx = center_x - fw / 2.0;
            let fy = center_y - fh / 2.0;

            c.stroke_rect(fx, fy, fw, fh);
          }

          // Corner vanishing lines to center
          for &(cx_sign, cy_sign) in &[(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            c.stroke_line(center_x, center_y, center_x + cx_sign * width * 0.5, center_y + cy_sign * height * 0.5);
          }
        }
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
        LayerConfig::GlowingDisc { radius_ratio, color } => {
          let disc_r = (width.min(height) * radius_ratio).clamp(10.0, 350.0) + (be * 12.0);
          let disc_col = Color::hex(color);

          c.set_fill(Fill::Solid(disc_col));
          c.set_shadow(disc_col, 16.0);
          c.fill_ellipse(center_x, center_y, disc_r, disc_r);
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
        LayerConfig::ParticlesBurst { count, color } => {
          let p_count = (*count).clamp(8, 120);
          let p_col = Color::hex(color);

          c.set_shadow(p_col, 10.0);
          for i in 0..p_count {
            let seed = i as f32 * 71.3;
            let angle = (seed * 0.3 + rot * 0.5) % TAU;
            let dist = (width.min(height) * 0.15) + (seed % (height * 0.25)) + (be * 30.0);

            let px = center_x + angle.cos() * dist;
            let py = center_y + angle.sin() * dist;
            let pr = (3.0 + (seed % 4.0) + bs * 3.0).clamp(1.5, 10.0);

            c.set_fill(Fill::Solid(p_col));
            c.fill_ellipse(px, py, pr, pr);
          }
        }
        LayerConfig::DualSpeakers { color } => {
          let spk_col = Color::hex(color);
          let spk_w = (width * 0.22).clamp(120.0, 320.0);
          let spk_h = spk_w * 1.35;
          let spk_y = center_y - spk_h * 0.45;

          let left_spk_x = center_x - spk_w * 1.15;
          let right_spk_x = center_x + spk_w * 0.15;

          for &sx in &[left_spk_x, right_spk_x] {
            // Speaker Box Cabinet Body
            c.set_fill(Fill::Solid(Color::rgba(0.08, 0.07, 0.11, 0.98)));
            c.set_stroke(Fill::Solid(spk_col.with_alpha(0.85)));
            c.set_line_width(2.5);
            c.set_shadow(spk_col.with_alpha(0.6), 18.0);
            c.fill_rounded_rect(sx, spk_y, spk_w, spk_h, 8.0);
            c.stroke_rect(sx, spk_y, spk_w, spk_h);

            // Upper Tweeter Circle
            let tw_cx = sx + spk_w * 0.5;
            let tw_cy = spk_y + spk_h * 0.25;
            let tw_r = spk_w * 0.15;
            c.set_fill(Fill::Solid(Color::rgba(0.03, 0.02, 0.04, 0.98)));
            c.set_stroke(Fill::Solid(Color::rgba(0.8, 0.8, 0.9, 0.9)));
            c.set_line_width(1.5);
            c.fill_ellipse(tw_cx, tw_cy, tw_r, tw_r);
            c.stroke_circle(tw_cx, tw_cy, tw_r);

            // Main Lower Woofer Cone (Pumping on Bass!)
            let woo_cx = sx + spk_w * 0.5;
            let woo_cy = spk_y + spk_h * 0.68;
            let woo_r = spk_w * (0.34 + be * 0.05);

            c.set_fill(Fill::Solid(Color::rgba(0.02, 0.02, 0.04, 0.98)));
            c.set_stroke(Fill::Solid(spk_col));
            c.set_line_width(3.0);
            c.set_shadow(spk_col, 14.0);
            c.fill_ellipse(woo_cx, woo_cy, woo_r, woo_r);
            c.stroke_circle(woo_cx, woo_cy, woo_r);

            // Gold Dust Cap
            let cap_r = woo_r * 0.35;
            c.set_fill(Fill::Solid(Color::rgba(1.0, 0.7, 0.2, 0.98)));
            c.fill_ellipse(woo_cx, woo_cy, cap_r, cap_r);
          }
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
