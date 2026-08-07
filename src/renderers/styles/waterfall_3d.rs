//! 3D Audio Waterfall style renderer (`waterfall3D`).
//!
//! Recreates the iconic retro sci-fi soundpeek 3D spectrum waterfall landscape enclosed
//! inside a vintage CRT hardware monitor frame with oscilloscope waveform and Lissajous orbit.

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
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

  let center_x = width * 0.5;
  let center_y = height * 0.48;

  // -------------------------------------------------------------------------
  // 1. RETRO HARDWARE CRT MONITOR OUTER FRAME & GOLD BEZEL
  // -------------------------------------------------------------------------
  let frame_w = (width * 0.90).clamp(320.0, 1100.0);
  let frame_h = (height * 0.82).clamp(240.0, 750.0);
  let frame_x = center_x - frame_w / 2.0;
  let frame_y = center_y - frame_h / 2.0;

  // Heavy Outer Metallic Chassis Shell
  c.set_fill(Fill::Solid(Color::rgba(0.07, 0.07, 0.1, 0.96)));
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.8), 28.0);
  c.fill_rounded_rect(frame_x, frame_y, frame_w, frame_h, 18.0);

  c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.22, 0.32, 0.8)));
  c.set_line_width(2.0);
  c.stroke_rect(frame_x, frame_y, frame_w, frame_h);

  // Copper / Gold Inner Bezel Trim
  let scr_w = frame_w * 0.94;
  let scr_h = frame_h * 0.85;
  let scr_x = center_x - scr_w / 2.0;
  let scr_y = frame_y + frame_h * 0.11;

  c.set_fill(Fill::Solid(Color::rgba(0.72, 0.52, 0.28, 0.92)));
  c.set_shadow(Color::rgba(0.8, 0.5, 0.2, 0.35), 10.0);
  c.fill_rounded_rect(scr_x - 3.0, scr_y - 3.0, scr_w + 6.0, scr_h + 6.0, 6.0);

  // Inner Dark CRT Glass Screen Panel
  c.set_fill(Fill::Solid(Color::rgba(0.02, 0.02, 0.04, 0.98)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_rounded_rect(scr_x, scr_y, scr_w, scr_h, 4.0);

  // Header Title Bar inside Monitor Chassis
  c.draw_text(
    "SOUNDPEEK 3D SPECTRUM ANALYZER",
    frame_x + 18.0,
    frame_y + 14.0,
    10.0,
    "monospace",
    700.0,
    false,
    TextAlign::Left,
    Fill::Solid(Color::rgba(0.75, 0.68, 0.55, 0.85)),
    1.0,
    &Default::default(),
  );

  // Green Power LED Indicator Dot
  c.set_fill(Fill::Solid(Color::rgba(0.0, 0.95, 0.4, 0.95)));
  c.set_shadow(Color::rgba(0.0, 0.95, 0.4, 0.8), 8.0);
  c.fill_ellipse(frame_x + frame_w - 24.0, frame_y + 18.0, 5.0, 5.0);

  // -------------------------------------------------------------------------
  // 2. TOP OSCILLOSCOPE WAVEFORM (INSIDE CRT SCREEN)
  // -------------------------------------------------------------------------
  let wave_y = scr_y + scr_h * 0.18;
  let wave_amp = scr_h * 0.14 * sensitivity;
  let mut wave_pts: Vec<(f32, f32)> = Vec::with_capacity(128);

  let step_t = (time.len() / 128).max(1);
  for i in 0..128 {
    let x = scr_x + (i as f32 / 127.0) * scr_w;
    let sample = *time.get(i * step_t).unwrap_or(&128) as f32;
    let norm = (sample - 128.0) / 128.0;
    let y = wave_y + norm * wave_amp;
    wave_pts.push((x, y));
  }

  c.set_stroke(Fill::Solid(Color::rgba(0.4, 0.6, 1.0, 0.95)));
  c.set_line_width(2.0);
  c.set_shadow(Color::rgba(0.3, 0.5, 1.0, 0.7), 10.0);
  c.stroke_polyline(&wave_pts);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 3. TOP RIGHT 3D LISSAJOUS ORBIT RING (INSIDE CRT SCREEN)
  // -------------------------------------------------------------------------
  let orbit_cx = scr_x + scr_w * 0.82;
  let orbit_cy = scr_y + scr_h * 0.28;
  let orbit_r = (scr_w.min(scr_h) * 0.12).max(25.0);
  let mut orbit_pts: Vec<(f32, f32)> = Vec::with_capacity(72);

  let rot_angle = frame_time * 1.5;
  for i in 0..72 {
    let a = (i as f32 / 72.0) * TAU;
    let mod_r = orbit_r * (1.0 + (a * 3.0 + frame_time * 4.0).sin() * 0.08 * (1.0 + be));
    let x3d = mod_r * a.cos();
    let y3d = mod_r * a.sin() * 0.45;
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
  // 4. 3D WATERFALL SPECTRUM TERRAIN (INSIDE CRT SCREEN)
  // -------------------------------------------------------------------------
  let floor_y = scr_y + scr_h * 0.92;
  let num_rows = st.frame_history.len();
  let step_f = (freq.len() / BARS_PER_ROW).max(1);

  for r in (0..num_rows).rev() {
    let history_data = &st.frame_history[r];
    let depth_t = r as f32 / DEPTH_ROWS as f32;

    let z = depth_t * 500.0;
    let scale = 600.0 / (600.0 + z);
    let row_y = floor_y - z * 0.38 * scale;

    let total_w = scr_w * 0.85 * scale;
    let start_x = center_x - total_w / 2.0;
    let bar_dx = total_w / (BARS_PER_ROW as f32 - 1.0);
    let max_h = scr_h * 0.35 * scale * sensitivity;

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
      let alpha = (1.0 - depth_t * 0.75).clamp(0.15, 0.95);
      let green_val = (0.7 + (1.0 - depth_t) * 0.3).min(1.0);
      let col = Color::rgba(0.1 + (1.0 - depth_t) * 0.3 * bs, green_val, 0.25, alpha);

      c.set_stroke(Fill::Solid(col));
      c.set_line_width((1.5 * scale).clamp(0.8, 3.0));

      if r == 0 {
        c.set_shadow(Color::rgba(0.2, 1.0, 0.4, 0.8), 12.0);
      } else {
        c.set_shadow(Color::TRANSPARENT, 0.0);
      }

      c.stroke_polyline(&row_pts);
    }
  }

  c.restore();
}
