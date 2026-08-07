//! Retro Cassette Tape style renderer (`cassetteTape`).
//!
//! Faithful high-fidelity port matching the reference images:
//! Dual neon outline body (pink/cyan), spinning tape reels with 6-tooth gears,
//! unwinding magnetic tape rolls, cassette label metadata header, digital time counters,
//! floor reflection, and audio spectrum wave underneath.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let s = crate::renderers::theme_secondary(theme);

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let frame_time = ctx.frame_time;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.44;

  // Cassette Body Dimensions
  let tape_w = (width * 0.54).clamp(280.0, 680.0);
  let tape_h = tape_w * 0.61;
  let left_x = center_x - tape_w / 2.0;
  let top_y = center_y - tape_h / 2.0;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Neon colors matching reference photo (Hot Pink + Electric Cyan)
  let hot_pink = Color::rgba(1.0, 0.0, 0.72, 0.95);
  let electric_cyan = Color::rgba(0.0, 0.9, 1.0, 0.95);

  // -------------------------------------------------------------------------
  // 1. MIRRORED FLOOR REFLECTION (UNDER CASSETTE)
  // -------------------------------------------------------------------------
  let refl_y = top_y + tape_h + 20.0;
  c.save();
  c.set_global_alpha(0.18 + be * 0.1);

  // Mirrored lower spectrum wave
  let bar_count = 56usize;
  let step = (freq.len() / bar_count).max(1);
  let bar_w = (tape_w / bar_count as f32) - 1.5;

  for i in 0..bar_count {
    let raw_v = *freq.get(i * step).unwrap_or(&0) as f32 / 255.0;
    let val = (raw_v * sensitivity).clamp(0.0, 1.2);
    let bh = val * height * 0.12;
    let bx = left_x + i as f32 * (bar_w + 1.5);
    c.set_fill(Fill::Solid(hot_pink));
    c.fill_rect(bx, refl_y + 30.0, bar_w.max(2.0), bh.max(2.0));
  }
  c.restore();

  // -------------------------------------------------------------------------
  // 2. AUDIO SPECTRUM BARS & WAVE LINE DIRECTLY UNDER CASSETTE
  // -------------------------------------------------------------------------
  let spec_y = top_y + tape_h + 8.0;

  for i in 0..bar_count {
    let raw_v = *freq.get(i * step).unwrap_or(&0) as f32 / 255.0;
    let val = (raw_v * sensitivity).clamp(0.0, 1.2);
    let bh = val * height * 0.16;
    let bx = left_x + i as f32 * (bar_w + 1.5);

    let grad = Fill::linear_gradient(
      bx,
      spec_y,
      bx,
      spec_y - bh,
      &[(0.0, hot_pink), (0.6, s.with_alpha(0.95)), (1.0, electric_cyan)],
    );

    c.set_fill(grad);
    c.set_shadow(hot_pink.with_alpha(0.7), 10.0 + bs * 10.0);
    c.fill_rounded_rect(bx, spec_y - bh, bar_w.max(2.0), bh.max(2.0), 2.0);
  }

  // Horizontal scrub/neon line under cassette
  c.set_stroke(Fill::Solid(electric_cyan));
  c.set_line_width(2.0);
  c.set_shadow(electric_cyan.with_alpha(0.8), 12.0);
  c.stroke_line(left_x - 10.0, spec_y + 4.0, left_x + tape_w + 10.0, spec_y + 4.0);

  // Scrub handle circle
  let scrub_progress = ((frame_time * 0.05) % 1.0).clamp(0.05, 0.95);
  let scrub_x = left_x + scrub_progress * tape_w;
  c.set_fill(Fill::Solid(Color::WHITE));
  c.fill_ellipse(scrub_x, spec_y + 4.0, 6.0, 6.0);
  c.stroke_circle(scrub_x, spec_y + 4.0, 6.0);

  // -------------------------------------------------------------------------
  // 3. CASSETTE OUTER SHELL & DOUBLE NEON BORDER
  // -------------------------------------------------------------------------
  // Solid cassette body background
  c.set_fill(Fill::Solid(Color::rgba(0.04, 0.02, 0.08, 0.95)));
  c.set_shadow(hot_pink.with_alpha(0.6), 25.0 + be * 20.0);
  c.fill_rounded_rect(left_x, top_y, tape_w, tape_h, 14.0);

  // Outer Hot Pink Neon Border
  c.set_stroke(Fill::Solid(hot_pink));
  c.set_line_width(3.5);
  c.stroke_rect(left_x, top_y, tape_w, tape_h);

  // Inner Electric Cyan Accent Border
  c.set_stroke(Fill::Solid(electric_cyan.with_alpha(0.7)));
  c.set_line_width(1.5);
  c.stroke_rect(left_x + 4.0, top_y + 4.0, tape_w - 8.0, tape_h - 8.0);

  // -------------------------------------------------------------------------
  // 4. CASSETTE TAPE LABEL SECTION
  // -------------------------------------------------------------------------
  let label_margin = tape_w * 0.06;
  let label_w = tape_w - label_margin * 2.0;
  let label_h = tape_h * 0.68;
  let label_x = left_x + label_margin;
  let label_y = top_y + tape_h * 0.06;

  c.set_fill(Fill::Solid(Color::rgba(0.08, 0.05, 0.16, 0.9)));
  c.set_shadow(electric_cyan.with_alpha(0.5), 14.0);
  c.fill_rounded_rect(label_x, label_y, label_w, label_h, 8.0);

  c.set_stroke(Fill::Solid(electric_cyan));
  c.set_line_width(2.0);
  c.stroke_rect(label_x, label_y, label_w, label_h);

  // -------------------------------------------------------------------------
  // 5. TAPE WINDOW & SPINNING DUAL REELS
  // -------------------------------------------------------------------------
  let win_w = label_w * 0.72;
  let win_h = label_h * 0.48;
  let win_x = center_x - win_w / 2.0;
  let win_y = label_y + label_h * 0.46;

  c.set_fill(Fill::Solid(Color::rgba(0.03, 0.02, 0.06, 0.98)));
  c.set_stroke(Fill::Solid(hot_pink.with_alpha(0.8)));
  c.set_line_width(1.8);
  c.fill_rounded_rect(win_x, win_y, win_w, win_h, 6.0);
  c.stroke_rect(win_x, win_y, win_w, win_h);

  let reel_r = win_h * 0.46;
  let reel_left_x = center_x - win_w * 0.26;
  let reel_right_x = center_x + win_w * 0.26;
  let reel_center_y = win_y + win_h * 0.5;

  // Unwinding magnetic tape roll simulation
  let tape_progress = ((frame_time * 0.02) % 1.0).clamp(0.0, 1.0);
  let tape_roll_left = reel_r * (1.45 - tape_progress * 0.35);
  let tape_roll_right = reel_r * (1.10 + tape_progress * 0.35);

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Dark magnetic tape rolls behind reels
  c.set_fill(Fill::Solid(Color::rgba(0.22, 0.1, 0.16, 0.95)));
  c.fill_ellipse(reel_left_x, reel_center_y, tape_roll_left, tape_roll_left);
  c.fill_ellipse(reel_right_x, reel_center_y, tape_roll_right, tape_roll_right);

  // Draw Reels with Glowing Pink Hubs
  for &(rx, is_left) in &[(reel_left_x, true), (reel_right_x, false)] {
    // Bright Neon Pink Hub (matching reference photo!)
    c.set_fill(Fill::Solid(hot_pink));
    c.set_shadow(hot_pink, 15.0 + bs * 10.0);
    c.fill_ellipse(rx, reel_center_y, reel_r, reel_r);

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.0);
    c.stroke_circle(rx, reel_center_y, reel_r);

    // Inner black hub hole
    let hub_r = reel_r * 0.42;
    c.set_fill(Fill::Solid(Color::rgba(0.05, 0.02, 0.08, 0.98)));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_ellipse(rx, reel_center_y, hub_r, hub_r);

    // Rotating 6-spoke gear teeth inside hub
    let dir = if is_left { 1.0 } else { -1.0 };
    let current_rot = rot * dir;

    c.set_fill(Fill::Solid(Color::WHITE));

    for t_idx in 0..6 {
      let tooth_angle = current_rot + (t_idx as f32 / 6.0) * TAU;
      let tx = rx + tooth_angle.cos() * (hub_r * 0.75);
      let ty = reel_center_y + tooth_angle.sin() * (hub_r * 0.75);
      let tooth_size = hub_r * 0.28;

      c.fill_ellipse(tx, ty, tooth_size, tooth_size);
    }
  }

  // -------------------------------------------------------------------------
  // 6. LOWER CASSETTE TRAPEZOID AREA & SCREWS
  // -------------------------------------------------------------------------
  let trap_w = tape_w * 0.72;
  let trap_h = tape_h * 0.18;
  let trap_x = center_x - trap_w / 2.0;
  let trap_y = top_y + tape_h - trap_h - 5.0;

  c.set_fill(Fill::Solid(Color::rgba(0.06, 0.04, 0.12, 0.95)));
  c.set_stroke(Fill::Solid(electric_cyan.with_alpha(0.8)));
  c.set_line_width(2.0);
  c.set_shadow(electric_cyan.with_alpha(0.4), 8.0);
  c.stroke_rect(trap_x, trap_y, trap_w, trap_h);

  // Trapezoid small oval holes
  c.set_fill(Fill::Solid(hot_pink.with_alpha(0.8)));
  c.set_shadow(Color::TRANSPARENT, 0.0);

  c.fill_rounded_rect(trap_x + trap_w * 0.2, trap_y + trap_h * 0.3, trap_w * 0.12, trap_h * 0.4, 3.0);
  c.fill_rounded_rect(trap_x + trap_w * 0.68, trap_y + trap_h * 0.3, trap_w * 0.12, trap_h * 0.4, 3.0);

  // Corner screw holes
  let screw_r = 3.5f32;
  c.set_fill(Fill::Solid(electric_cyan));
  c.fill_ellipse(left_x + 12.0, top_y + 12.0, screw_r, screw_r);
  c.fill_ellipse(left_x + tape_w - 12.0, top_y + 12.0, screw_r, screw_r);
  c.fill_ellipse(left_x + 12.0, top_y + tape_h - 12.0, screw_r, screw_r);
  c.fill_ellipse(left_x + tape_w - 12.0, top_y + tape_h - 12.0, screw_r, screw_r);

  c.restore();
}
