//! Turntable style renderer (`turntable`).
//!
//! Hyper-realistic vintage DJ turntable featuring dark plinth deck, spinning vinyl record,
//! 360-degree radial equalizer, and a detailed S-shaped metallic tonearm assembly with
//! counterweight, gimbal housing, angled headshell cartridge, and real-time groove tracking.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const RADIAL_BARS: usize = 96;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let frame_time = ctx.frame_time;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  let disc_r = (width.min(height) * 0.28).clamp(80.0, 320.0);

  // Turntable Deck Plinth Dimensions
  let deck_w = disc_r * 2.4;
  let deck_h = disc_r * 2.0;
  let deck_x = center_x - deck_w * 0.5;
  let deck_y = center_y - deck_h * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.95);
  let electric_cyan = Color::rgba(0.0, 0.85, 1.0, 0.95);

  // -------------------------------------------------------------------------
  // 1. TURNTABLE BASE DECK (PLINTH)
  // -------------------------------------------------------------------------
  c.set_fill(Fill::Solid(Color::rgba(0.06, 0.05, 0.09, 0.94)));
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.75), 26.0);
  c.fill_rounded_rect(deck_x, deck_y, deck_w, deck_h, 20.0);

  c.set_stroke(Fill::Solid(Color::rgba(0.22, 0.2, 0.28, 0.6)));
  c.set_line_width(1.5);
  c.stroke_rect(deck_x + 2.0, deck_y + 2.0, deck_w - 4.0, deck_h - 4.0);

  // -------------------------------------------------------------------------
  // 2. 360-DEGREE RADIAL EQUALIZER SPECTRUM BARS (AROUND VINYL)
  // -------------------------------------------------------------------------
  let step = (freq.len() / (RADIAL_BARS / 2)).max(1);
  let max_bar_h = height * 0.14 * sensitivity;

  for i in 0..RADIAL_BARS {
    let angle = (i as f32 / RADIAL_BARS as f32) * TAU;

    let bin_i = if i <= RADIAL_BARS / 2 {
      (i * step).min(freq.len().saturating_sub(1))
    } else {
      ((RADIAL_BARS - i) * step).min(freq.len().saturating_sub(1))
    };

    let raw_v = *freq.get(bin_i).unwrap_or(&0) as f32 / 255.0;
    let bar_v = (raw_v * sensitivity).clamp(0.0, 1.3);
    let bh = bar_v * max_bar_h;

    let x1 = center_x + angle.cos() * (disc_r + 4.0);
    let y1 = center_y + angle.sin() * (disc_r + 4.0);
    let x2 = center_x + angle.cos() * (disc_r + 4.0 + bh);
    let y2 = center_y + angle.sin() * (disc_r + 4.0 + bh);

    let grad = Fill::linear_gradient(
      x1,
      y1,
      x2,
      y2,
      &[(0.0, hot_pink), (0.6, p.with_alpha(0.95)), (1.0, electric_cyan)],
    );

    c.set_stroke(grad);
    c.set_line_width(3.0);
    c.set_shadow(hot_pink.with_alpha(0.6), 8.0 + bs * 6.0);
    c.stroke_line(x1, y1, x2, y2);
  }

  // -------------------------------------------------------------------------
  // 3. SPINNING VINYL RECORD DISC
  // -------------------------------------------------------------------------
  c.set_fill(Fill::Solid(Color::rgba(0.08, 0.07, 0.11, 0.98)));
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.8), 18.0);
  c.fill_ellipse(center_x, center_y, disc_r, disc_r);

  // Concentric vinyl grooves
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_stroke(Fill::Solid(Color::rgba(0.28, 0.25, 0.35, 0.35)));
  c.set_line_width(1.0);

  for g_ratio in &[0.52f32, 0.60, 0.68, 0.76, 0.84, 0.92, 0.96] {
    let gr = disc_r * g_ratio;
    c.stroke_circle(center_x, center_y, gr);
  }

  // Glossy sheen light reflection wedges
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.07)));
  for wedge_offset in &[0.0f32, std::f32::consts::PI] {
    let w_angle = rot + wedge_offset;
    let mut wedge_pts = vec![(center_x, center_y)];
    for k in 0..10 {
      let a = w_angle - 0.22 + (k as f32 / 9.0) * 0.44;
      let wx = center_x + a.cos() * disc_r;
      let wy = center_y + a.sin() * disc_r;
      wedge_pts.push((wx, wy));
    }
    wedge_pts.push((center_x, center_y));
    c.stroke_polyline(&wedge_pts);
  }

  // Hot Pink Center Label
  let label_r = disc_r * 0.34;
  c.set_fill(Fill::Solid(hot_pink));
  c.set_shadow(hot_pink.with_alpha(0.6), 14.0);
  c.fill_ellipse(center_x, center_y, label_r, label_r);

  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(1.5);
  c.stroke_circle(center_x, center_y, label_r);

  // Spindle center hole
  let spindle_r = label_r * 0.18;
  c.set_fill(Fill::Solid(Color::rgba(0.04, 0.03, 0.06, 0.98)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_ellipse(center_x, center_y, spindle_r, spindle_r);

  // -------------------------------------------------------------------------
  // 4. HYPER-REALISTIC S-SHAPED METALLIC DJ TONEARM ASSEMBLY
  // -------------------------------------------------------------------------
  let pivot_x = deck_x + deck_w * 0.85;
  let pivot_y = deck_y + deck_h * 0.18;
  let pivot_r = disc_r * 0.14;

  // Real-time track progress sweep (moves tonearm gradually from outer edge to label)
  let track_progress = ((frame_time * 0.015) % 1.0).clamp(0.0, 1.0);
  let groove_r = disc_r * (0.86 - track_progress * 0.44);

  // Target stylus contact point on record (angle ~ 40 degrees)
  let contact_angle = std::f32::consts::FRAC_PI_4 + 0.15;
  let vib_x = (be * 1.5).sin() * 1.2;
  let vib_y = (bs * 1.5).cos() * 1.2;

  let stylus_x = center_x + contact_angle.cos() * groove_r + vib_x;
  let stylus_y = center_y + contact_angle.sin() * groove_r + vib_y;

  // Heavy Metallic Counterweight (Behind Pivot)
  let cw_angle = (pivot_y - stylus_y).atan2(pivot_x - stylus_x) + std::f32::consts::PI;
  let cw_x = pivot_x + cw_angle.cos() * (pivot_r * 1.8);
  let cw_y = pivot_y + cw_angle.sin() * (pivot_r * 1.8);

  c.set_fill(Fill::Solid(Color::rgba(0.3, 0.3, 0.35, 0.95)));
  c.set_stroke(Fill::Solid(Color::rgba(0.8, 0.8, 0.85, 0.9)));
  c.set_line_width(2.0);
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 10.0);
  c.fill_ellipse(cw_x, cw_y, pivot_r * 0.85, pivot_r * 0.85);
  c.stroke_circle(cw_x, cw_y, pivot_r * 0.85);

  // Counterweight stub rod
  c.set_stroke(Fill::Solid(Color::rgba(0.65, 0.65, 0.7, 0.9)));
  c.set_line_width(3.0);
  c.stroke_line(pivot_x, pivot_y, cw_x, cw_y);

  // Gimbal Base Housing (Concentric Chrome Rings)
  c.set_fill(Fill::Solid(Color::rgba(0.16, 0.15, 0.2, 0.98)));
  c.set_stroke(Fill::Solid(s));
  c.set_line_width(2.5);
  c.set_shadow(s.with_alpha(0.6), 12.0);
  c.fill_ellipse(pivot_x, pivot_y, pivot_r, pivot_r);
  c.stroke_circle(pivot_x, pivot_y, pivot_r);

  c.set_stroke(Fill::Solid(Color::rgba(0.9, 0.9, 0.95, 0.9)));
  c.set_line_width(1.5);
  c.stroke_circle(pivot_x, pivot_y, pivot_r * 0.55);

  // S-Shaped Curved Tonearm Pipe (2 Elbow Joints for Realistic S-Curve)
  let mid1_x = pivot_x * 0.65 + stylus_x * 0.35 + 18.0;
  let mid1_y = pivot_y * 0.65 + stylus_y * 0.35 - 12.0;

  let mid2_x = pivot_x * 0.35 + stylus_x * 0.65 - 10.0;
  let mid2_y = pivot_y * 0.35 + stylus_y * 0.65 + 8.0;

  let arm_pts = [(pivot_x, pivot_y), (mid1_x, mid1_y), (mid2_x, mid2_y), (stylus_x, stylus_y)];

  // Outer Chrome Shadow Line
  c.set_stroke(Fill::Solid(Color::rgba(0.15, 0.15, 0.2, 0.7)));
  c.set_line_width(5.5);
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.7), 8.0);
  c.stroke_polyline(&arm_pts);

  // Metallic Silver Chrome Pipe Body
  c.set_stroke(Fill::Solid(Color::rgba(0.88, 0.88, 0.94, 0.98)));
  c.set_line_width(3.5);
  c.stroke_polyline(&arm_pts);

  // Specular Highlight Line
  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(1.2);
  c.stroke_polyline(&arm_pts);

  // Angled DJ Headshell Cartridge (Ortofon Concorde Style)
  let head_angle = (stylus_y - mid2_y).atan2(stylus_x - mid2_x) + 0.25;

  let head_len = 22.0f32;
  let head_w = 10.0f32;

  let h_back_x = stylus_x - head_angle.cos() * head_len;
  let h_back_y = stylus_y - head_angle.sin() * head_len;

  c.set_stroke(Fill::Solid(hot_pink));
  c.set_line_width(head_w);
  c.set_shadow(hot_pink.with_alpha(0.8), 12.0);
  c.stroke_line(h_back_x, h_back_y, stylus_x, stylus_y);

  // Red/Pink Stylus Indicator Tip Light
  c.set_fill(Fill::Solid(Color::WHITE));
  c.set_shadow(hot_pink, 14.0);
  c.fill_ellipse(stylus_x, stylus_y, 4.0, 4.0);

  c.restore();
}
