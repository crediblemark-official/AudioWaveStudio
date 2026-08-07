//! 3D Audio Waterfall style renderer (`waterfall3D`).
//!
//! Recreates the iconic retro sci-fi soundpeek 3D spectrum waterfall landscape
//! with top oscilloscope waveform and Lissajous phase orbit ring.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const DEPTH_ROWS: usize = 36;
const BARS_PER_ROW: usize = 64;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let time = ctx.time_data;
  let frame_time = ctx.frame_time;
  let st = &mut ctx.state.advanced;

  // Maintain spectrum history for 3D waterfall depth
  if st.frame_history.first().map(|f| f.len()) != Some(freq.len()) {
    st.frame_history.clear();
  }
  st.frame_history.insert(0, freq.to_vec());
  if st.frame_history.len() > DEPTH_ROWS {
    st.frame_history.pop();
  }

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. TOP OSCILLOSCOPE WAVEFORM
  // -------------------------------------------------------------------------
  let wave_y = height * 0.22;
  let wave_amp = height * 0.12 * sensitivity;
  let mut wave_pts: Vec<(f32, f32)> = Vec::with_capacity(128);

  let step_t = (time.len() / 128).max(1);
  for i in 0..128 {
    let x = (i as f32 / 127.0) * width;
    let sample = *time.get(i * step_t).unwrap_or(&128) as f32;
    let norm = (sample - 128.0) / 128.0;
    let y = wave_y + norm * wave_amp;
    wave_pts.push((x, y));
  }

  c.set_stroke(Fill::Solid(Color::rgba(0.4, 0.6, 1.0, 0.95)));
  c.set_line_width(2.2);
  c.set_shadow(Color::rgba(0.3, 0.5, 1.0, 0.7), 10.0);
  c.stroke_polyline(&wave_pts);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 2. TOP RIGHT 3D LISSAJOUS ORBIT RING
  // -------------------------------------------------------------------------
  let orbit_cx = width * 0.82;
  let orbit_cy = height * 0.32;
  let orbit_r = (width.min(height) * 0.12).max(30.0);
  let mut orbit_pts: Vec<(f32, f32)> = Vec::with_capacity(72);

  let rot_angle = frame_time * 1.5;
  for i in 0..72 {
    let a = (i as f32 / 72.0) * TAU;
    let mod_r = orbit_r * (1.0 + (a * 3.0 + frame_time * 4.0).sin() * 0.08 * (1.0 + be));
    let x3d = mod_r * a.cos();
    let y3d = mod_r * a.sin() * 0.45; // Tilted ellipse angle
    let rot_x = x3d * rot_angle.cos() - y3d * rot_angle.sin();
    let rot_y = x3d * rot_angle.sin() + y3d * rot_angle.cos();
    orbit_pts.push((orbit_cx + rot_x, orbit_cy + rot_y));
  }
  if !orbit_pts.is_empty() {
    let first = orbit_pts[0];
    orbit_pts.push(first);
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.85, 0.2, 0.9)));
    c.set_line_width(2.0);
    c.set_shadow(Color::rgba(1.0, 0.8, 0.1, 0.6), 12.0);
    c.stroke_polyline(&orbit_pts);
    c.set_shadow(Color::TRANSPARENT, 0.0);
  }

  // -------------------------------------------------------------------------
  // 3. 3D WATERFALL SPECTRUM TERRAIN (GREEN PHOSPHOR)
  // -------------------------------------------------------------------------
  let center_x = width * 0.5;
  let floor_y = height * 0.88;
  let num_rows = st.frame_history.len();
  let step_f = (freq.len() / BARS_PER_ROW).max(1);

  // Render rows from back (depth Z_max) to front (depth Z_0)
  for r in (0..num_rows).rev() {
    let history_data = &st.frame_history[r];
    let depth_t = r as f32 / DEPTH_ROWS as f32; // 0.0 at front, 1.0 at back

    // Perspective depth projection parameters
    let z = depth_t * 500.0;
    let scale = 600.0 / (600.0 + z);
    let row_y = floor_y - z * 0.38 * scale;

    let total_w = width * 0.85 * scale;
    let start_x = center_x - total_w / 2.0;
    let bar_dx = total_w / (BARS_PER_ROW as f32 - 1.0);
    let max_h = height * 0.35 * scale * sensitivity;

    let mut row_pts: Vec<(f32, f32)> = Vec::with_capacity(BARS_PER_ROW);

    for i in 0..BARS_PER_ROW {
      let raw_v = *history_data.get(i * step_f).unwrap_or(&0) as f32 / 255.0;
      let val = (raw_v * sensitivity).clamp(0.0, 1.5);
      let bh = val * max_h;
      let x = start_x + i as f32 * bar_dx;
      let y = row_y - bh;
      row_pts.push((x, y));
    }

    if row_pts.len() > 2 {
      // CRT Matrix Phosphor Green spectrum gradient
      let alpha = (1.0 - depth_t * 0.75).clamp(0.15, 0.95);
      let green_val = (0.7 + (1.0 - depth_t) * 0.3).min(1.0);
      let col = Color::rgba(0.1 + (1.0 - depth_t) * 0.3 * bs, green_val, 0.25, alpha);

      c.set_stroke(Fill::Solid(col));
      c.set_line_width((1.5 * scale).clamp(0.8, 3.0));

      if r == 0 {
        // Front-most active row gets a strong phosphor glow
        c.set_shadow(Color::rgba(0.2, 1.0, 0.4, 0.8), 12.0);
      } else {
        c.set_shadow(Color::TRANSPARENT, 0.0);
      }

      c.stroke_polyline(&row_pts);
    }
  }

  c.restore();
}
