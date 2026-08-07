//! Retro Cassette Tape style renderer (`cassetteTape`).
//!
//! Renders a vector synthwave neon cassette tape with continuously spinning reels
//! (speed reactive to music beat), tape window, label text, and audio spectrum wave.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let glow = crate::renderers::theme_glow(theme);

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.48;

  // Cassette Body Dimensions
  let tape_w = (width * 0.55).clamp(240.0, 640.0);
  let tape_h = tape_w * 0.62;
  let left_x = center_x - tape_w / 2.0;
  let top_y = center_y - tape_h / 2.0;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. AUDIO SPECTRUM BARS UNDER CASSETTE
  // -------------------------------------------------------------------------
  let bar_count = 48usize;
  let step = (freq.len() / bar_count).max(1);
  let bar_w = (tape_w / bar_count as f32) - 2.0;
  let bar_base_y = top_y + tape_h + 35.0;

  for i in 0..bar_count {
    let raw_v = *freq.get(i * step).unwrap_or(&0) as f32 / 255.0;
    let val = (raw_v * sensitivity).clamp(0.0, 1.2);
    let bh = val * height * 0.18;
    let bx = left_x + i as f32 * (bar_w + 2.0);

    let grad = Fill::linear_gradient(
      bx,
      bar_base_y,
      bx,
      bar_base_y - bh,
      &[(0.0, p.with_alpha(0.85)), (0.6, s.with_alpha(0.95)), (1.0, a)],
    );

    c.set_fill(grad);
    c.set_shadow(glow.with_alpha(0.6), 10.0);
    c.fill_rounded_rect(bx, bar_base_y - bh, bar_w.max(2.0), bh.max(2.0), 2.0);
  }

  // -------------------------------------------------------------------------
  // 2. CASSETTE OUTER SHELL & NEON BORDER
  // -------------------------------------------------------------------------
  c.set_fill(Fill::Solid(Color::rgba(0.05, 0.03, 0.1, 0.92)));
  c.set_shadow(glow, 22.0 + be * 15.0 + bs * 12.0);
  c.fill_rounded_rect(left_x, top_y, tape_w, tape_h, 12.0);

  c.set_fill(Fill::Solid(p));
  c.stroke_rect(left_x, top_y, tape_w, tape_h);

  // -------------------------------------------------------------------------
  // 3. TAPE LABEL SECTION
  // -------------------------------------------------------------------------
  let label_margin = tape_w * 0.08;
  let label_w = tape_w - label_margin * 2.0;
  let label_h = tape_h * 0.65;
  let label_x = left_x + label_margin;
  let label_y = top_y + tape_h * 0.08;

  c.set_fill(Fill::Solid(Color::rgba(0.12, 0.08, 0.22, 0.85)));
  c.set_shadow(s.with_alpha(0.5), 10.0);
  c.fill_rounded_rect(label_x, label_y, label_w, label_h, 8.0);

  c.set_fill(Fill::Solid(s));
  c.stroke_rect(label_x, label_y, label_w, label_h);

  // -------------------------------------------------------------------------
  // 4. SPINNING TAPE REELS (LEFT & RIGHT)
  // -------------------------------------------------------------------------
  let reel_r = tape_h * 0.22;
  let reel_y = label_y + label_h * 0.5;
  let reel_left_x = center_x - tape_w * 0.22;
  let reel_right_x = center_x + tape_w * 0.22;

  // Tape roll background circles (unwinding simulation)
  let tape_roll_left = reel_r * 1.35;
  let tape_roll_right = reel_r * 1.18;

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Left tape roll (dark brown/purple magnetic tape)
  c.set_fill(Fill::Solid(Color::rgba(0.2, 0.12, 0.18, 0.95)));
  c.fill_ellipse(reel_left_x, reel_y, tape_roll_left, tape_roll_left);

  // Right tape roll
  c.fill_ellipse(reel_right_x, reel_y, tape_roll_right, tape_roll_right);

  // Draw Reels (Left & Right)
  for &(rx, is_left) in &[(reel_left_x, true), (reel_right_x, false)] {
    c.set_fill(Fill::Solid(Color::rgba(0.95, 0.92, 0.98, 0.95)));
    c.set_shadow(a.with_alpha(0.7), 12.0);
    c.fill_ellipse(rx, reel_y, reel_r, reel_r);

    c.set_stroke(Fill::Solid(a));
    c.set_line_width(2.5);
    c.stroke_circle(rx, reel_y, reel_r);

    // Inner hub hole
    let hub_r = reel_r * 0.45;
    c.set_fill(Fill::Solid(Color::rgba(0.08, 0.05, 0.14, 0.98)));
    c.fill_ellipse(rx, reel_y, hub_r, hub_r);

    // Rotating teeth / spokes (6 teeth around each reel)
    let dir = if is_left { 1.0 } else { -1.0 };
    let current_rot = rot * dir;

    c.set_fill(Fill::Solid(a));
    c.set_shadow(Color::TRANSPARENT, 0.0);

    for t_idx in 0..6 {
      let tooth_angle = current_rot + (t_idx as f32 / 6.0) * TAU;
      let tx = rx + tooth_angle.cos() * (hub_r * 0.72);
      let ty = reel_y + tooth_angle.sin() * (hub_r * 0.72);
      let tooth_size = hub_r * 0.28;

      c.fill_ellipse(tx, ty, tooth_size, tooth_size);
    }
  }

  // -------------------------------------------------------------------------
  // 5. CASSETTE LOWER TRAPEZOID & SCREWS
  // -------------------------------------------------------------------------
  let trap_w = tape_w * 0.75;
  let trap_h = tape_h * 0.22;
  let trap_x = center_x - trap_w / 2.0;
  let trap_y = top_y + tape_h - trap_h - 4.0;

  c.set_fill(Fill::Solid(s.with_alpha(0.8)));
  c.set_shadow(s.with_alpha(0.4), 8.0);
  c.stroke_rect(trap_x, trap_y, trap_w, trap_h);

  // Corner screw holes
  let screw_r = 3.5f32;
  c.set_fill(Fill::Solid(Color::rgba(0.6, 0.6, 0.7, 0.8)));
  c.set_shadow(Color::TRANSPARENT, 0.0);

  c.fill_ellipse(left_x + 12.0, top_y + 12.0, screw_r, screw_r);
  c.fill_ellipse(left_x + tape_w - 12.0, top_y + 12.0, screw_r, screw_r);
  c.fill_ellipse(left_x + 12.0, top_y + tape_h - 12.0, screw_r, screw_r);
  c.fill_ellipse(left_x + tape_w - 12.0, top_y + tape_h - 12.0, screw_r, screw_r);

  c.restore();
}
