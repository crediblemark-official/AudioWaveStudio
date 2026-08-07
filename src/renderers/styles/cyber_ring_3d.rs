//! Cyber Ring 3D style renderer (`cyberRing3D`).
//!
//! Renders a 3D holographic HUD stage podium complete with tilted concentric dashed/dotted
//! HUD rings, 360-degree radial spectrum rim, floating translucent dome, and a glowing
//! 3D music badge with floor reflection.

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const RADIAL_BARS: usize = 96;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.52;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.95);
  let electric_cyan = Color::rgba(0.0, 0.85, 1.0, 0.95);
  let neon_purple = Color::rgba(0.7, 0.1, 1.0, 0.95);

  let tilt_y = 0.40f32; // 3D Perspective vertical tilt factor
  let base_rx = (width.min(height) * 0.30).clamp(100.0, 340.0);
  let base_ry = base_rx * tilt_y;

  // -------------------------------------------------------------------------
  // 1. 3D PERSPECTIVE FLOOR GRID LINES
  // -------------------------------------------------------------------------
  let grid_y_top = center_y - height * 0.25;
  let grid_y_bot = center_y + height * 0.35;
  let num_grid_v = 16usize;

  c.set_stroke(Fill::Solid(Color::rgba(0.2, 0.1, 0.3, 0.25)));
  c.set_line_width(1.0);

  for i in 0..num_grid_v {
    let t_val = i as f32 / (num_grid_v - 1) as f32;
    let x_top = center_x + (t_val - 0.5) * (width * 0.6);
    let x_bot = center_x + (t_val - 0.5) * (width * 1.4);
    c.stroke_line(x_top, grid_y_top, x_bot, grid_y_bot);
  }

  // -------------------------------------------------------------------------
  // 2. 360-DEGREE RADIAL SPECTRUM BARS (3D TILTED ELLIPSE RIM)
  // -------------------------------------------------------------------------
  let step = (freq.len() / (RADIAL_BARS / 2)).max(1);
  let max_bar_len = height * 0.14 * sensitivity;

  for i in 0..RADIAL_BARS {
    let angle = (i as f32 / RADIAL_BARS as f32) * TAU;

    let bin_i = if i <= RADIAL_BARS / 2 {
      (i * step).min(freq.len().saturating_sub(1))
    } else {
      ((RADIAL_BARS - i) * step).min(freq.len().saturating_sub(1))
    };

    let raw_v = *freq.get(bin_i).unwrap_or(&0) as f32 / 255.0;
    let bar_h = (raw_v * sensitivity * max_bar_len).clamp(4.0, max_bar_len * 1.3);

    let rx_start = base_rx * 0.95;
    let ry_start = base_ry * 0.95;
    let rx_end = rx_start + bar_h;
    let ry_end = ry_start + bar_h * tilt_y;

    let x1 = center_x + angle.cos() * rx_start;
    let y1 = center_y + angle.sin() * ry_start;
    let x2 = center_x + angle.cos() * rx_end;
    let y2 = center_y + angle.sin() * ry_end;

    let bar_col = if i % 3 == 0 {
      hot_pink
    } else if i % 3 == 1 {
      electric_cyan
    } else {
      neon_purple
    };

    c.set_stroke(Fill::Solid(bar_col.with_alpha(0.85)));
    c.set_line_width(2.2);
    c.set_shadow(bar_col.with_alpha(0.6), 8.0 + bs * 6.0);
    c.stroke_line(x1, y1, x2, y2);
  }

  // -------------------------------------------------------------------------
  // 3. DASHED & DOTTED 3D CONCENTRIC HUD RINGS
  // -------------------------------------------------------------------------
  // Ring A: Outer Cyan Dashed HUD Ring (Rotating CCW)
  let rx_a = base_rx * 0.88;
  let ry_a = rx_a * tilt_y;
  let rot_a = -rot * 1.4;

  let dash_segments = 24usize;
  for d in 0..dash_segments {
    if d % 2 == 0 {
      let a1 = rot_a + (d as f32 / dash_segments as f32) * TAU;
      let a2 = rot_a + ((d as f32 + 0.7) / dash_segments as f32) * TAU;

      let mut arc_pts = Vec::with_capacity(10);
      for k in 0..10 {
        let a = a1 + (k as f32 / 9.0) * (a2 - a1);
        arc_pts.push((center_x + a.cos() * rx_a, center_y + a.sin() * ry_a));
      }

      c.set_stroke(Fill::Solid(electric_cyan));
      c.set_line_width(3.0);
      c.set_shadow(electric_cyan.with_alpha(0.8), 12.0);
      c.stroke_polyline(&arc_pts);
    }
  }

  // Ring B: Middle Pink Dotted Ring (Rotating CW)
  let rx_b = base_rx * 0.74 + (be * 12.0);
  let ry_b = rx_b * tilt_y;
  let rot_b = rot * 1.1;

  let num_dots = 36usize;
  for d in 0..num_dots {
    let a = rot_b + (d as f32 / num_dots as f32) * TAU;
    let dx = center_x + a.cos() * rx_b;
    let dy = center_y + a.sin() * ry_b;

    c.set_fill(Fill::Solid(hot_pink));
    c.set_shadow(hot_pink, 8.0);
    c.fill_ellipse(dx, dy, 3.0, 2.0);
  }

  // Ring C: Inner Magenta Wave Spectrum Ring
  let rx_c = base_rx * 0.58;
  let ry_c = rx_c * tilt_y;
  let num_wave_pts = 80usize;
  let mut c_pts = Vec::with_capacity(num_wave_pts + 1);

  for k in 0..=num_wave_pts {
    let a = (k as f32 / num_wave_pts as f32) * TAU;
    let bin = (k * freq.len() / num_wave_pts).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

    let w_r_x = rx_c + fv * 16.0 * sensitivity;
    let w_r_y = ry_c + fv * 16.0 * tilt_y * sensitivity;

    c_pts.push((center_x + a.cos() * w_r_x, center_y + a.sin() * w_r_y));
  }

  c.set_stroke(Fill::Solid(neon_purple));
  c.set_line_width(2.5);
  c.set_shadow(neon_purple.with_alpha(0.9), 14.0);
  c.stroke_polyline(&c_pts);

  // -------------------------------------------------------------------------
  // 4. HOLOGRAPHIC DOME & GLOWING 3D MUSIC BADGE CORE
  // -------------------------------------------------------------------------
  let dome_rx = base_rx * 0.42;
  let dome_ry = dome_rx * 0.55;
  let dome_cy = center_y - dome_ry * 0.4;

  // Glowing Translucent Dome Surface
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.65, 0.22)));
  c.set_shadow(hot_pink.with_alpha(0.8), 24.0);
  c.fill_ellipse(center_x, dome_cy, dome_rx, dome_ry);

  c.set_stroke(Fill::Solid(Color::WHITE.with_alpha(0.9)));
  c.set_line_width(1.5);
  let mut dome_rim = Vec::with_capacity(36);
  for k in 0..=36 {
    let a = (k as f32 / 36.0) * TAU;
    dome_rim.push((center_x + a.cos() * dome_rx, dome_cy + a.sin() * dome_ry));
  }
  c.stroke_polyline(&dome_rim);

  // Glowing 🎵 Music Note Icon Badge in Hologram Dome
  c.draw_text(
    "🎵",
    center_x,
    dome_cy - 6.0,
    (dome_rx * 0.45).clamp(20.0, 50.0),
    "sans-serif",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::WHITE),
    1.0,
    &Default::default(),
  );

  // Floor Reflection Text Shadow (Matching Photo 1: "TSIXOM / Reflection")
  c.draw_text(
    "🎵",
    center_x,
    dome_cy + dome_ry * 0.8,
    (dome_rx * 0.40).clamp(16.0, 44.0),
    "sans-serif",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(hot_pink.with_alpha(0.35)),
    1.0,
    &Default::default(),
  );

  c.restore();
}
