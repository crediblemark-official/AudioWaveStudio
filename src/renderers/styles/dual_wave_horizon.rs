//! Dual Wave Horizon style renderer (`dualWaveHorizon`).
//!
//! Complete overhaul: Dual Stereo Oscilloscope Waveform
//! - Smooth glowing red core waveform with power-curve audio tapering
//! - 6 clean, smooth, parallel stacked white contour lines (oscilloscope ridges)
//! - Clean horizontal red baseline running across center gap

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const WAVE_PTS: usize = 80;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let bright_red = Color::rgba(1.0, 0.05, 0.12, 0.98);

  let wave_w = (width * 0.42).clamp(160.0, 600.0);
  let gap_w = (width * 0.05).clamp(14.0, 48.0);

  let left_start_x = center_x - gap_w * 0.5 - wave_w;
  let right_start_x = center_x + gap_w * 0.5;

  let max_amp = (height * 0.22 * sensitivity).clamp(22.0, 240.0);
  let step = (freq.len() / (WAVE_PTS * 2)).max(1);

  // Red horizontal baseline running through center gap
  c.set_stroke(Fill::Solid(bright_red));
  c.set_line_width(2.5);
  c.set_shadow(bright_red, 10.0);
  c.stroke_line(left_start_x, center_y, right_start_x + wave_w, center_y);

  // -------------------------------------------------------------------------
  // DUAL STEREO WAVEFORM BLOCKS
  // -------------------------------------------------------------------------
  for &(start_x, is_right) in &[(left_start_x, false), (right_start_x, true)] {
    let mut top_pts = Vec::with_capacity(WAVE_PTS + 2);
    let mut bot_pts = Vec::with_capacity(WAVE_PTS + 2);

    top_pts.push((start_x, center_y));
    bot_pts.push((start_x, center_y));

    let mut smoothed_amps = [0.0f32; WAVE_PTS];

    for i in 0..WAVE_PTS {
      let x = start_x + (i as f32 / (WAVE_PTS - 1) as f32) * wave_w;

      let bin_idx = if is_right {
        (i * step).min(freq.len().saturating_sub(1))
      } else {
        ((WAVE_PTS - 1 - i) * step).min(freq.len().saturating_sub(1))
      };

      let raw_v = *freq.get(bin_idx).unwrap_or(&0) as f32 / 255.0;

      // Smooth gaussian-like envelope at ends to prevent sharp block edge cuts
      let t_env = (i as f32 / (WAVE_PTS - 1) as f32 * std::f32::consts::PI).sin();
      let amp = (raw_v.powf(1.5) * max_amp * t_env).clamp(1.5, max_amp);
      smoothed_amps[i] = amp;

      top_pts.push((x, center_y - amp));
      bot_pts.push((x, center_y + amp));
    }

    top_pts.push((start_x + wave_w, center_y));
    bot_pts.push((start_x + wave_w, center_y));

    // 1. Smooth Filled Glowing Red Core Waveform
    c.set_fill(Fill::Solid(bright_red));
    c.set_shadow(bright_red, 14.0 + bs * 6.0);
    c.fill_polyline_to_base(&top_pts, center_y);
    c.fill_polyline_to_base(&bot_pts, center_y);

    // 2. Multilayered Stacked Oscilloscope Contour Lines (6 Parallel Layers)
    let num_layers = 6;
    for layer in 1..=num_layers {
      let l_ratio = layer as f32 / num_layers as f32;
      let alpha = 0.95 - (l_ratio * 0.40);
      let stroke_w = 1.6 - (l_ratio * 0.3);

      let mut contour_top = Vec::with_capacity(WAVE_PTS);
      let mut contour_bot = Vec::with_capacity(WAVE_PTS);

      for i in 0..WAVE_PTS {
        let x = start_x + (i as f32 / (WAVE_PTS - 1) as f32) * wave_w;
        let amp = smoothed_amps[i];

        let ridge_offset = layer as f32 * 4.0 + amp * (0.8 + layer as f32 * 0.20);

        contour_top.push((x, center_y - ridge_offset));
        contour_bot.push((x, center_y + ridge_offset));
      }

      c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, alpha)));
      c.set_line_width(stroke_w);
      c.set_shadow(Color::TRANSPARENT, 0.0);
      c.stroke_polyline(&contour_top);
      c.stroke_polyline(&contour_bot);
    }
  }

  c.restore();
}
