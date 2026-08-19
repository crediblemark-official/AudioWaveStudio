//! 3D Radial Ripple style renderer (`radialRipple3D`).
//!
//! Renders a 3D topographic audio landscape of concentric ripple rings
//! tilted in 3D perspective, matching the vibrant neon spectrum terrain reference.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use super::radial_common;

const RING_COUNT: usize = 32;
const RING_POINTS: usize = 128;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let theme = &ctx.config.theme;
  let p_col = theme_primary(theme);
  let s_col = theme_secondary(theme);
  let accent = theme_accent(theme);
  let glow = theme_glow(theme);

  let center_x = width * 0.48;
  let center_y = height * 0.62;
  let pitch = 1.08f32; // ~62 degrees tilt angle for 3D perspective
  let cos_p = pitch.cos();
  let sin_p = pitch.sin();
  let cam_dist = 600.0f32;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let st = &mut ctx.state.advanced;

  // Maintain spectrum history for outward ripple propagation
  if st.frame_history.first().map(|f| f.len()) != Some(freq.len()) {
    st.frame_history.clear();
  }
  st.frame_history.insert(0, freq.to_vec());
  if st.frame_history.len() > RING_COUNT {
    st.frame_history.pop();
  }

  let num_rings = st.frame_history.len();
  let step = (freq.len() / (RING_POINTS / 2)).max(1);

  // Render rings from outer (back) to inner (front) for proper depth blending
  for r_idx in (0..num_rings).rev() {
    let history_data = &st.frame_history[r_idx];
    let ring_ratio = r_idx as f32 / RING_COUNT as f32; // 0.0 at center, 1.0 at outer rim

    // Base radius grows non-linearly outward for perspective depth feeling
    let base_radius = 18.0 + (r_idx as f32) * 11.0 + (r_idx as f32).powf(1.4) * 2.2;
    let max_height = 35.0 + (1.0 - ring_ratio) * 65.0 + be * 30.0;

    // Calculate ring color: theme-mixed gradient from accent core -> p_col -> glow rim
    let col = compute_ripple_color(ring_ratio, p_col, s_col, accent, glow);

    let mut points_2d: Vec<(f32, f32)> = Vec::with_capacity(RING_POINTS + 1);

    for p_idx in 0..RING_POINTS {
      let angle = (p_idx as f32 / RING_POINTS as f32) * TAU;
      let cos_a = angle.cos();
      let sin_a = angle.sin();

      // Mirror frequency indexing around the circle so ripples are symmetrical left/right.
      // Rotate the sample start by a beat-driven pseudo-random sweep so the
      // spectral shape (and its bass prominences) scatters across the ring.
      let sweep_off = ((radial_common::sweep_angle(ctx.beat_count) / TAU)
        * (RING_POINTS as f32)) as usize
        % RING_POINTS.max(1);
      let p_rot = (p_idx + RING_POINTS - sweep_off) % RING_POINTS.max(1);
      let bin_i = if p_rot <= RING_POINTS / 2 {
        (p_rot * step).min(history_data.len().saturating_sub(1))
      } else {
        ((RING_POINTS - p_rot) * step).min(history_data.len().saturating_sub(1))
      };

      let raw_v = *history_data.get(bin_i).unwrap_or(&0) as f32 / 255.0;
      let wave_v = (raw_v * sensitivity).clamp(0.0, 1.5);
      let disp = wave_v * max_height;

      // 3D coordinates in world space
      let radius = base_radius + disp * 0.4;
      let x3d = radius * cos_a;
      let z3d = radius * sin_a;
      let y3d = -disp * 0.95; // Vertical displacement

      // Apply 3D perspective rotation around X-axis (pitch tilt)
      let y_rot = y3d * cos_p - z3d * sin_p;
      let z_rot = y3d * sin_p + z3d * cos_p;

      // 3D Projection to 2D Screen Space
      let scale = cam_dist / (cam_dist + z_rot + 300.0);
      let sx = center_x + x3d * scale;
      let sy = center_y + y_rot * scale;

      points_2d.push((sx, sy));
    }

    if points_2d.len() > 3 {
      // Close the loop
      let first_pt = points_2d[0];
      points_2d.push(first_pt);

      let line_width = (1.2 + (1.0 - ring_ratio) * 1.8 + if r_idx < 4 { 1.0 } else { 0.0 }).clamp(1.0, 4.0);
      let glow_radius = if r_idx < 8 { 12.0 + (8 - r_idx) as f32 * 2.0 } else { 4.0 };

      c.set_stroke(Fill::Solid(col));
      c.set_line_width(line_width);
      c.set_shadow(col.with_alpha(0.6), glow_radius);

      // Draw continuous closed 3D ring contour
      c.stroke_polyline(&points_2d);
    }
  }

  // Render high-intensity core flash on beat
  if bs > 0.2 {
    let core_radius = 25.0 + bs * 20.0;
    let flash = mix(accent, s_col, 0.5);
    c.set_shadow(flash, 25.0);
    c.set_fill(Fill::radial_gradient(
      center_x,
      center_y,
      0.0,
      center_x,
      center_y,
      core_radius,
      &[
        (0.0, mix(Color::WHITE, flash, 0.5).with_alpha(0.8 * bs)),
        (0.4, flash.with_alpha(0.5 * bs)),
        (1.0, Color::TRANSPARENT),
      ],
    ));
    c.fill_ellipse(center_x, center_y, core_radius, core_radius * 0.5);
  }

  c.restore();
}

/// Dynamic spectrum color interpolation: theme-driven gradient that sweeps from
/// the accent color at the core through `p_col` out to `glow` at the rim, with
/// the secondary colour mixed in at the mid range for the classic neon banding.
fn compute_ripple_color(ratio: f32, p_col: Color, s_col: Color, accent: Color, glow: Color) -> Color {
  if ratio < 0.2 {
    // Center Core: Accent -> Accent/Secondary blend (fiery edge)
    let t = ratio / 0.2;
    mix(accent, mix(accent, s_col, 0.6), t).with_alpha(0.95)
  } else if ratio < 0.6 {
    // Mid Range: Accent/Secondary -> Primary (gold band to cool)
    let t = (ratio - 0.2) / 0.4;
    mix(mix(accent, s_col, 0.6), mix(s_col, p_col, 0.5), t).with_alpha(0.9)
  } else {
    // Outer Rim: Primary -> Glow (cool electric rim)
    let t = (ratio - 0.6) / 0.4;
    mix(mix(s_col, p_col, 0.5), glow, t).with_alpha(0.85)
  }
}
