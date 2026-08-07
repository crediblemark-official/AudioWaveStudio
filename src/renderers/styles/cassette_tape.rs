//! Retro Cassette Tape style renderer (`cassetteTape`).
//!
//! Masterpiece 100% faithful port matching both reference photos:
//! Dual neon outline body (pink/cyan), glowing pink tape reels with 6-tooth gears,
//! magnetic tape unwinding rolls, side digital time counters, cassette label metadata header,
//! bottom trapezoid text, floor reflection, and audio spectrum wave with scrub handle.

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let frame_time = ctx.frame_time;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.44;

  // Cassette Body Dimensions
  let tape_w = (width * 0.52).clamp(280.0, 680.0);
  let tape_h = tape_w * 0.61;
  let left_x = center_x - tape_w / 2.0;
  let top_y = center_y - tape_h / 2.0;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Neon colors matching reference photos (Hot Pink + Electric Cyan)
  let hot_pink = Color::rgba(1.0, 0.0, 0.72, 0.95);
  let electric_cyan = Color::rgba(0.0, 0.9, 1.0, 0.95);

  // Calculate formatted digital timestamps for side displays
  let elapsed_sec = frame_time as u32;
  let cur_hrs = elapsed_sec / 3600;
  let cur_min = (elapsed_sec % 3600) / 60;
  let cur_s = elapsed_sec % 60;
  let cur_ms = ((frame_time - elapsed_sec as f32) * 100.0) as u32;

  let cur_time_str = format!("{:02}:{:02}:{:02}:{:02}", cur_hrs, cur_min, cur_s, cur_ms);
  let rem_time_str = format!("00:{:02}:{:02}:{:02}", (cur_min + 1) % 60, (59 - cur_s) % 60, (99 - cur_ms) % 100);
  let short_time_str = format!("{:02}:{:02}", cur_min, cur_s);

  // -------------------------------------------------------------------------
  // 1. MIRRORED FLOOR REFLECTION (UNDER CASSETTE)
  // -------------------------------------------------------------------------
  let refl_y = top_y + tape_h + 15.0;
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
    c.fill_rect(bx, refl_y + 25.0, bar_w.max(2.0), bh.max(2.0));
  }
  c.restore();

  // -------------------------------------------------------------------------
  // 2. SIDE DIGITAL TIMERS (MATCHING PHOTO 1)
  // -------------------------------------------------------------------------
  let timer_font_size = (tape_h * 0.085).clamp(12.0, 24.0);

  // Current time (Left Side)
  if left_x > 130.0 {
    c.draw_text(
      "current time",
      left_x - 110.0,
      center_y - 18.0,
      10.0,
      "monospace",
      400.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.45)),
      1.0,
      &Default::default(),
    );
    c.draw_text(
      &cur_time_str,
      left_x - 110.0,
      center_y + 4.0,
      timer_font_size,
      "monospace",
      700.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::WHITE),
      1.0,
      &Default::default(),
    );
  }

  // Time remaining (Right Side)
  if width - (left_x + tape_w) > 130.0 {
    c.draw_text(
      "time left",
      left_x + tape_w + 110.0,
      center_y - 18.0,
      10.0,
      "monospace",
      400.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.45)),
      1.0,
      &Default::default(),
    );
    c.draw_text(
      &rem_time_str,
      left_x + tape_w + 110.0,
      center_y + 4.0,
      timer_font_size,
      "monospace",
      700.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::WHITE),
      1.0,
      &Default::default(),
    );
  }

  // -------------------------------------------------------------------------
  // 3. AUDIO SPECTRUM BARS & WAVE LINE DIRECTLY UNDER CASSETTE
  // -------------------------------------------------------------------------
  let spec_y = top_y + tape_h + 6.0;

  for i in 0..bar_count {
    let raw_v = *freq.get(i * step).unwrap_or(&0) as f32 / 255.0;
    let val = (raw_v * sensitivity).clamp(0.0, 1.2);
    let bh = val * height * 0.15;
    let bx = left_x + i as f32 * (bar_w + 1.5);

    let grad = Fill::linear_gradient(
      bx,
      spec_y,
      bx,
      spec_y - bh,
      &[(0.0, hot_pink), (0.6, Color::rgba(0.7, 0.0, 0.9, 0.95)), (1.0, electric_cyan)],
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
  // 4. CASSETTE OUTER SHELL & DOUBLE NEON BORDER
  // -------------------------------------------------------------------------
  // Solid cassette body background
  c.set_fill(Fill::Solid(Color::rgba(0.04, 0.02, 0.08, 0.96)));
  c.set_shadow(hot_pink.with_alpha(0.6), 25.0 + be * 20.0);
  c.fill_rounded_rect(left_x, top_y, tape_w, tape_h, 14.0);

  // Outer Hot Pink Neon Border
  c.set_stroke(Fill::Solid(hot_pink));
  c.set_line_width(3.5);
  c.stroke_rect(left_x, top_y, tape_w, tape_h);

  // Inner Electric Cyan Accent Border
  c.set_stroke(Fill::Solid(electric_cyan.with_alpha(0.75)));
  c.set_line_width(1.5);
  c.stroke_rect(left_x + 4.0, top_y + 4.0, tape_w - 8.0, tape_h - 8.0);

  // -------------------------------------------------------------------------
  // 5. CASSETTE TAPE LABEL SECTION & HEADER METADATA STICKER
  // -------------------------------------------------------------------------
  let label_margin = tape_w * 0.05;
  let label_w = tape_w - label_margin * 2.0;
  let label_h = tape_h * 0.76;
  let label_x = left_x + label_margin;
  let label_y = top_y + tape_h * 0.05;

  c.set_fill(Fill::Solid(Color::rgba(0.07, 0.04, 0.14, 0.95)));
  c.set_shadow(electric_cyan.with_alpha(0.5), 14.0);
  c.fill_rounded_rect(label_x, label_y, label_w, label_h, 8.0);

  c.set_stroke(Fill::Solid(electric_cyan));
  c.set_line_width(2.0);
  c.stroke_rect(label_x, label_y, label_w, label_h);

  // Header Sticker Box in upper area of label ($Y \in [top + 0.04 \cdot H, top + 0.28 \cdot H]$)
  let hdr_w = label_w * 0.92;
  let hdr_h = label_h * 0.28;
  let hdr_x = center_x - hdr_w / 2.0;
  let hdr_y = label_y + label_h * 0.04;

  c.set_fill(Fill::Solid(Color::rgba(0.12, 0.06, 0.22, 0.9)));
  c.stroke_rect(hdr_x, hdr_y, hdr_w, hdr_h);

  // Cassette Label Title & Subtitle
  let title_str = if !ctx.config.text.song_title.trim().is_empty() {
    ctx.config.text.song_title.to_uppercase()
  } else {
    "NEON WAVE AUDIO VISUALIZER".to_string()
  };

  let title_size = (hdr_h * 0.38).clamp(10.0, 17.0);
  c.draw_text(
    &title_str,
    hdr_x + hdr_w * 0.04,
    hdr_y + hdr_h * 0.24,
    title_size,
    "sans-serif",
    700.0,
    false,
    TextAlign::Left,
    Fill::Solid(Color::WHITE),
    1.0,
    &Default::default(),
  );

  let artist_str = if !ctx.config.text.artist_name.trim().is_empty() {
    ctx.config.text.artist_name.clone()
  } else {
    "Produced by AudioWave Studio".to_string()
  };

  c.draw_text(
    &artist_str,
    hdr_x + hdr_w * 0.04,
    hdr_y + hdr_h * 0.70,
    title_size * 0.65,
    "sans-serif",
    400.0,
    false,
    TextAlign::Left,
    Fill::Solid(Color::rgba(0.8, 0.8, 0.9, 0.8)),
    1.0,
    &Default::default(),
  );

  // -------------------------------------------------------------------------
  // 6. TAPE WINDOW & DUAL SPINNING REELS (BELOW HEADER - NO OVERLAP!)
  // -------------------------------------------------------------------------
  let win_w = label_w * 0.74;
  let win_h = label_h * 0.42;
  let win_x = center_x - win_w / 2.0;
  let win_y = hdr_y + hdr_h + label_h * 0.03;

  c.set_fill(Fill::Solid(Color::rgba(0.03, 0.02, 0.06, 0.98)));
  c.set_stroke(Fill::Solid(hot_pink.with_alpha(0.85)));
  c.set_line_width(1.8);
  c.fill_rounded_rect(win_x, win_y, win_w, win_h, 6.0);
  c.stroke_rect(win_x, win_y, win_w, win_h);

  // Digital time counter inside tape window (matching Photo 2: "00:05")
  c.draw_text(
    &short_time_str,
    center_x,
    win_y + win_h * 0.82,
    (win_h * 0.25).clamp(9.0, 14.0),
    "monospace",
    600.0,
    false,
    TextAlign::Center,
    Fill::Solid(electric_cyan),
    1.0,
    &Default::default(),
  );

  let reel_r = win_h * 0.42;
  let reel_left_x = center_x - win_w * 0.26;
  let reel_right_x = center_x + win_w * 0.26;
  let reel_center_y = win_y + win_h * 0.46;

  // Unwinding magnetic tape roll simulation
  let tape_progress = ((frame_time * 0.02) % 1.0).clamp(0.0, 1.0);
  let tape_roll_left = reel_r * (1.30 - tape_progress * 0.25);
  let tape_roll_right = reel_r * (1.05 + tape_progress * 0.25);

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Translucent dark magnetic tape rolls behind reels
  c.set_fill(Fill::Solid(Color::rgba(0.18, 0.1, 0.25, 0.6)));
  c.fill_ellipse(reel_left_x, reel_center_y, tape_roll_left, tape_roll_left);
  c.fill_ellipse(reel_right_x, reel_center_y, tape_roll_right, tape_roll_right);

  // Draw Reels with Glowing Pink Hubs
  for &(rx, is_left) in &[(reel_left_x, true), (reel_right_x, false)] {
    // Bright Neon Pink Hub (matching reference photos!)
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
  // 7. LOWER CASSETTE TRAPEZOID AREA & SCREWS
  // -------------------------------------------------------------------------
  let trap_w = tape_w * 0.72;
  let trap_h = tape_h * 0.16;
  let trap_x = center_x - trap_w / 2.0;
  let trap_y = top_y + tape_h - trap_h - 4.0;

  c.set_fill(Fill::Solid(Color::rgba(0.06, 0.04, 0.12, 0.95)));
  c.set_stroke(Fill::Solid(electric_cyan.with_alpha(0.8)));
  c.set_line_width(2.0);
  c.set_shadow(electric_cyan.with_alpha(0.4), 8.0);
  c.stroke_rect(trap_x, trap_y, trap_w, trap_h);

  // Volume text inside trapezoid (matching Photo 1: "ENVATO VOLUME #2")
  c.draw_text(
    "AUDIOWAVE VOLUME #1",
    center_x,
    trap_y + trap_h * 0.65,
    (trap_h * 0.45).clamp(8.0, 13.0),
    "sans-serif",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(hot_pink),
    1.0,
    &Default::default(),
  );

  // Corner screw holes (matching Photo 2: "x" screws)
  let screw_r = 3.5f32;
  c.set_fill(Fill::Solid(electric_cyan));
  c.fill_ellipse(left_x + 12.0, top_y + 12.0, screw_r, screw_r);
  c.fill_ellipse(left_x + tape_w - 12.0, top_y + 12.0, screw_r, screw_r);
  c.fill_ellipse(left_x + 12.0, top_y + tape_h - 12.0, screw_r, screw_r);
  c.fill_ellipse(left_x + tape_w - 12.0, top_y + tape_h - 12.0, screw_r, screw_r);

  c.restore();
}
