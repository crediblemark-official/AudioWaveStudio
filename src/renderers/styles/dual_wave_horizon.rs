//! Dual Wave Horizon style renderer (`dualWaveHorizon`).
//!
//! Renders dual stereo audio waveform peaks with filled glowing red core waveforms,
//! multilayered white contour lines, and a symmetrical horizontal mirror reflection.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const WAVE_POINTS: usize = 64;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let _be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let bright_red = Color::rgba(1.0, 0.05, 0.15, 0.98);

  // -------------------------------------------------------------------------
  // 1. DUAL LEFT & RIGHT WAVEFORM PEAKS (PHOTO 2)
  // -------------------------------------------------------------------------
  let wave_w = (width * 0.38).clamp(140.0, 560.0);
  let gap_w = (width * 0.06).clamp(16.0, 60.0);

  let left_start_x = center_x - gap_w * 0.5 - wave_w;
  let right_start_x = center_x + gap_w * 0.5;

  let max_amp = (height * 0.22 * sensitivity).clamp(20.0, 240.0);
  let step = (freq.len() / (WAVE_POINTS * 2)).max(1);

  // Render left and right stereo waveform blocks
  for &(start_x, is_right) in &[(left_start_x, false), (right_start_x, true)] {
    let mut top_pts = Vec::with_capacity(WAVE_POINTS + 2);
    let mut bot_pts = Vec::with_capacity(WAVE_POINTS + 2);

    top_pts.push((start_x, center_y));
    bot_pts.push((start_x, center_y));

    for i in 0..WAVE_POINTS {
      let x = start_x + (i as f32 / (WAVE_POINTS - 1) as f32) * wave_w;

      let bin_idx = if is_right {
        (i * step).min(freq.len().saturating_sub(1))
      } else {
        ((WAVE_POINTS - 1 - i) * step).min(freq.len().saturating_sub(1))
      };

      let raw_v = *freq.get(bin_idx).unwrap_or(&0) as f32 / 255.0;
      let amp = (raw_v * sensitivity * max_amp).clamp(2.0, max_amp);

      top_pts.push((x, center_y - amp));
      bot_pts.push((x, center_y + amp));
    }

    top_pts.push((start_x + wave_w, center_y));
    bot_pts.push((start_x + wave_w, center_y));

    // Filled Glowing Red Waveform Core
    c.set_fill(Fill::Solid(bright_red));
    c.set_shadow(bright_red, 14.0 + bs * 8.0);
    c.fill_polyline_to_base(&top_pts, center_y);
    c.fill_polyline_to_base(&bot_pts, center_y);

    // -----------------------------------------------------------------------
    // 2. MULTILAYERED WHITE CONTOUR LINES (PHOTO 2)
    // -----------------------------------------------------------------------
    c.set_stroke(Fill::Solid(Color::WHITE.with_alpha(0.9)));
    c.set_line_width(1.8);
    c.set_shadow(Color::TRANSPARENT, 0.0);

    for layer in 1..=4 {
      let l_scale = 1.0 + (layer as f32 * 0.18);
      let mut layer_top_pts = Vec::with_capacity(WAVE_POINTS);
      let mut layer_bot_pts = Vec::with_capacity(WAVE_POINTS);

      for i in 0..WAVE_POINTS {
        let x = start_x + (i as f32 / (WAVE_POINTS - 1) as f32) * wave_w;
        let bin_idx = if is_right {
          (i * step).min(freq.len().saturating_sub(1))
        } else {
          ((WAVE_POINTS - 1 - i) * step).min(freq.len().saturating_sub(1))
        };
        let raw_v = *freq.get(bin_idx).unwrap_or(&0) as f32 / 255.0;
        let amp = (raw_v * sensitivity * max_amp * l_scale).clamp(2.0, max_amp * 1.5);

        layer_top_pts.push((x, center_y - amp));
        layer_bot_pts.push((x, center_y + amp));
      }

      c.stroke_polyline(&layer_top_pts);
      c.stroke_polyline(&layer_bot_pts);
    }
  }

  // -------------------------------------------------------------------------
  // 3. CENTER AXIS LINE
  // -------------------------------------------------------------------------
  c.set_stroke(Fill::Solid(bright_red.with_alpha(0.8)));
  c.set_line_width(2.0);
  c.set_shadow(bright_red, 10.0);
  c.stroke_line(left_start_x, center_y, right_start_x + wave_w, center_y);

  c.restore();
}
