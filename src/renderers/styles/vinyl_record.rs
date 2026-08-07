//! Vinyl Record style renderer (`vinylRecord`).
//!
//! Renders a spinning vinyl record disc with synthwave center label, glossy sheen
//! reflections, and a surrounding 360-degree radial spectrum wave in magenta/cyan glow.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const RADIAL_POINTS: usize = 144;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let a = crate::renderers::theme_accent(theme);

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;

  // Continuous vinyl spin angle
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  let base_disc_r = (width.min(height) * 0.28).clamp(80.0, 320.0);
  let disc_r = base_disc_r * (1.0 + be * 0.05 + bs * 0.04);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. SURROUNDING RADIAL AUDIO SPECTRUM WAVE (MAGENTA/CYAN NEON GLOW)
  // -------------------------------------------------------------------------
  let step = (freq.len() / (RADIAL_POINTS / 2)).max(1);
  let max_wave_h = height * 0.18 * sensitivity;
  let mut wave_pts: Vec<(f32, f32)> = Vec::with_capacity(RADIAL_POINTS + 1);

  for i in 0..RADIAL_POINTS {
    let angle = (i as f32 / RADIAL_POINTS as f32) * TAU;

    // Symmetrical frequency sampling around 360 degrees
    let bin_i = if i <= RADIAL_POINTS / 2 {
      (i * step).min(freq.len().saturating_sub(1))
    } else {
      ((RADIAL_POINTS - i) * step).min(freq.len().saturating_sub(1))
    };

    let raw_v = *freq.get(bin_i).unwrap_or(&0) as f32 / 255.0;
    let wave_v = (raw_v * sensitivity).clamp(0.0, 1.4);
    let r = disc_r + wave_v * max_wave_h;

    let x = center_x + angle.cos() * r;
    let y = center_y + angle.sin() * r;
    wave_pts.push((x, y));
  }

  if wave_pts.len() > 3 {
    let first = wave_pts[0];
    wave_pts.push(first);

    // Spectrum glow fill & outline
    let wave_grad = Fill::linear_gradient(
      center_x - disc_r,
      center_y - disc_r,
      center_x + disc_r,
      center_y + disc_r,
      &[
        (0.0, Color::rgba(1.0, 0.0, 0.85, 0.95)),
        (0.5, p.with_alpha(0.95)),
        (1.0, Color::rgba(0.0, 0.85, 1.0, 0.95)),
      ],
    );

    c.set_stroke(wave_grad);
    c.set_line_width(2.5 + be * 2.0);
    c.set_shadow(Color::rgba(0.9, 0.0, 0.8, 0.8), 20.0 + bs * 15.0);
    c.stroke_polyline(&wave_pts);
  }

  // -------------------------------------------------------------------------
  // 2. BLACK VINYL RECORD DISC
  // -------------------------------------------------------------------------
  c.set_fill(Fill::Solid(Color::rgba(0.07, 0.06, 0.1, 0.98)));
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 15.0);
  c.fill_ellipse(center_x, center_y, disc_r, disc_r);

  // Concentric vinyl grooves
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.22, 0.3, 0.4)));
  c.set_line_width(1.0);

  for g_ratio in &[0.55f32, 0.65, 0.72, 0.80, 0.88, 0.94] {
    let gr = disc_r * g_ratio;
    c.stroke_circle(center_x, center_y, gr);
  }

  // -------------------------------------------------------------------------
  // 3. GLOSSY SHEEN REFLECTIONS (ROTATING LIGHT WEDGES)
  // -------------------------------------------------------------------------
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.06)));

  for wedge_offset in &[0.0f32, std::f32::consts::PI] {
    let w_angle = rot + wedge_offset;
    let mut wedge_pts = vec![(center_x, center_y)];

    for k in 0..12 {
      let a = w_angle - 0.25 + (k as f32 / 11.0) * 0.5;
      let wx = center_x + a.cos() * disc_r;
      let wy = center_y + a.sin() * disc_r;
      wedge_pts.push((wx, wy));
    }
    wedge_pts.push((center_x, center_y));
    c.stroke_polyline(&wedge_pts);
  }

  // -------------------------------------------------------------------------
  // 4. SYNTHWAVE CENTER SUN LABEL
  // -------------------------------------------------------------------------
  let label_r = disc_r * 0.36;

  let label_grad = Fill::linear_gradient(
    center_x,
    center_y - label_r,
    center_x,
    center_y + label_r,
    &[
      (0.0, Color::rgba(0.95, 0.0, 0.75, 0.95)),
      (0.6, Color::rgba(0.7, 0.0, 0.9, 0.95)),
      (1.0, Color::rgba(0.0, 0.75, 0.95, 0.95)),
    ],
  );

  c.set_fill(label_grad);
  c.set_shadow(a.with_alpha(0.6), 12.0);
  c.fill_ellipse(center_x, center_y, label_r, label_r);

  // Synthwave horizontal stripes across the center label
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::rgba(0.07, 0.06, 0.1, 0.9)));

  let stripe_count = 5usize;
  for s_i in 0..stripe_count {
    let sy = center_y + (s_i as f32 - 1.5) * (label_r * 0.22);
    let stripe_h = 1.5 + (s_i as f32) * 0.8;
    let sw = (label_r * label_r - (sy - center_y).powi(2)).max(0.0).sqrt() * 1.8;
    if sw > 0.0 {
      c.fill_rect(center_x - sw / 2.0, sy, sw, stripe_h);
    }
  }

  // Spindle center hole
  let spindle_r = label_r * 0.16;
  c.set_fill(Fill::Solid(Color::rgba(0.04, 0.03, 0.06, 0.98)));
  c.fill_ellipse(center_x, center_y, spindle_r, spindle_r);

  c.set_stroke(Fill::Solid(Color::rgba(0.9, 0.9, 1.0, 0.8)));
  c.set_line_width(1.5);
  c.stroke_circle(center_x, center_y, spindle_r);

  c.restore();
}
