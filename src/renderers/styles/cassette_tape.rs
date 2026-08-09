//! Retro Cassette Tape style renderer (`cassetteTape`).
//!
//! Renders an ultra-realistic 3D/2D vintage audio cassette tape complete with
//! realistic plastic shell shading, paper label sticker with Type II/Metal bias marks,
//! clear acrylic tape window with glass sheen reflections, authentic magnetic tape
//! transfer physics (conservation of tape volume between spools), visible tape path
//! over guide rollers & read heads, corner cross screws, digital LED counters,
//! audio-reactive spectrum display, and full UI settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;

  let p = theme_primary(theme);
  let s = theme_secondary(theme);
  let accent = theme_accent(theme);
  let glow = theme_glow(theme);

  // Settings integration
  let _sensitivity = ctx.config.reactivity.sensitivity;
  let user_scale = ctx.config.scale.clamp(0.1, 5.0);
  let pos_offset_x = ctx.config.position_x * width * 0.5;
  let pos_offset_y = -ctx.config.position_y * height * 0.5;
  let _bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

  let be = ctx.bass_energy;
  let _bs = ctx.beat_strength;
  let _freq = ctx.freq_data;
  let frame_time = ctx.frame_time;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5 + pos_offset_x;
  let center_y = height * 0.44 + pos_offset_y;

  // Cassette Body Dimensions (Scale applied internally)
  let tape_w = ((width * 0.54 * user_scale).clamp(300.0, 720.0)).clamp(160.0, width * 0.95);
  let tape_h = tape_w * 0.62;
  let left_x = center_x - tape_w / 2.0;
  let top_y = center_y - tape_h / 2.0;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Time calculations for digital LED displays
  let elapsed_sec = frame_time as u32;
  let cur_hrs = elapsed_sec / 3600;
  let cur_min = (elapsed_sec % 3600) / 60;
  let cur_s = elapsed_sec % 60;
  let cur_ms = ((frame_time - elapsed_sec as f32) * 100.0) as u32;

  let cur_time_str = format!("{:02}:{:02}:{:02}:{:02}", cur_hrs, cur_min, cur_s, cur_ms);
  let rem_time_str = format!("00:{:02}:{:02}:{:02}", (cur_min + 1) % 60, (59 - cur_s) % 60, (99 - cur_ms) % 100);
  let short_time_str = format!("{:02}:{:02}", cur_min, cur_s);

  // -------------------------------------------------------------------------
  // 1. ATMOSPHERIC BACKLIGHT & AMBIENT GLOW
  // -------------------------------------------------------------------------
  let haze = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    tape_w * 0.85,
    &[
      (0.0, glow.with_alpha(0.20 + be * 0.15)),
      (0.40, p.with_alpha(0.12)),
      (0.75, Color::rgba(0.04, 0.02, 0.10, 0.06)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(haze);
//   c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 2. SIDE DIGITAL TIMERS (VINTAGE LED FOIL DISPLAYS)
  // -------------------------------------------------------------------------
  let timer_font_size = (tape_h * 0.080).clamp(10.0, 22.0);

  // Current time (Left Side)
  if left_x > 110.0 {
    c.draw_text(
      "CURRENT TIME",
      left_x - 95.0f32.clamp(50.0, 120.0),
      center_y - 18.0,
      9.0f32.clamp(7.0, 11.0),
      "monospace",
      500.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.45)),
      1.0,
      &Default::default(),
    );
    c.draw_text(
      &cur_time_str,
      left_x - 95.0f32.clamp(50.0, 120.0),
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
  if width - (left_x + tape_w) > 110.0 {
    c.draw_text(
      "TIME LEFT",
      left_x + tape_w + 95.0f32.clamp(50.0, 120.0),
      center_y - 18.0,
      9.0f32.clamp(7.0, 11.0),
      "monospace",
      500.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.45)),
      1.0,
      &Default::default(),
    );
    c.draw_text(
      &rem_time_str,
      left_x + tape_w + 95.0f32.clamp(50.0, 120.0),
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
  // 3. CASSETTE SHELL SHADING & MOLDED PLASTIC BODY
  // -------------------------------------------------------------------------
  // Drop shadow behind cassette
  c.save();
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.8), 24.0);
  c.set_fill(Fill::Solid(Color::rgba(0.08, 0.09, 0.13, 0.98)));
  c.fill_rounded_rect(left_x, top_y, tape_w, tape_h, 14.0);
  c.restore();

  // Plastic shell subtle gradient texture (smoked matte black / dark slate)
  let shell_grad = Fill::linear_gradient(
    left_x,
    top_y,
    left_x + tape_w,
    top_y + tape_h,
    &[
      (0.0, Color::rgba(0.18, 0.19, 0.25, 0.98)),
      (0.3, Color::rgba(0.11, 0.12, 0.17, 0.98)),
      (0.7, Color::rgba(0.07, 0.08, 0.12, 0.98)),
      (1.0, Color::rgba(0.14, 0.15, 0.20, 0.98)),
    ],
  );
  c.set_fill(shell_grad);
  c.fill_rounded_rect(left_x, top_y, tape_w, tape_h, 14.0);

  // Outer bevel rim highlight
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.15)));
  c.set_line_width(1.5);
  c.stroke_rect(left_x, top_y, tape_w, tape_h);

  // Recessed top & side grip ribs (tactile cassette ridges)
  let _rib_w = tape_w * 0.12;
  let rib_y = top_y + 8.0;
  for rib_i in 0..4 {
    let rx1 = left_x + 18.0 + rib_i as f32 * 6.0;
    let rx2 = left_x + tape_w - 18.0 - rib_i as f32 * 6.0;
    c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, 0.4)));
    c.set_line_width(1.2);
    c.stroke_line(rx1, rib_y, rx1, rib_y + 12.0);
    c.stroke_line(rx2, rib_y, rx2, rib_y + 12.0);
  }

  // -------------------------------------------------------------------------
  // 4. PAPER CASSETTE LABEL STICKER (VINTAGE CHROME / TYPE II)
  // -------------------------------------------------------------------------
  let label_margin_x = tape_w * 0.05;
  let label_margin_y = tape_h * 0.05;
  let label_w = tape_w - label_margin_x * 2.0;
  let label_h = tape_h * 0.76;
  let label_x = left_x + label_margin_x;
  let label_y = top_y + label_margin_y;

  // Creamy vintage paper label background
  let label_grad = Fill::linear_gradient(
    label_x,
    label_y,
    label_x,
    label_y + label_h,
    &[
      (0.0, Color::rgba(0.92, 0.90, 0.85, 0.98)),
      (0.18, Color::rgba(0.96, 0.94, 0.90, 0.98)),
      (0.85, Color::rgba(0.88, 0.86, 0.82, 0.98)),
      (1.0, Color::rgba(0.80, 0.78, 0.74, 0.98)),
    ],
  );
  c.set_fill(label_grad);
  c.fill_rounded_rect(label_x, label_y, label_w, label_h, 6.0);

  // Label paper border
  c.set_stroke(Fill::Solid(Color::rgba(0.2, 0.2, 0.2, 0.3)));
  c.set_line_width(1.0);
  c.stroke_rect(label_x, label_y, label_w, label_h);

  // Top accent stripe (Type II / Chrome red & cyan racing stripes)
  let stripe_h = label_h * 0.14;
  let stripe_grad = Fill::linear_gradient(
    label_x,
    label_y + 4.0,
    label_x + label_w,
    label_y + 4.0,
    &[(0.0, p), (0.5, accent), (1.0, s)],
  );
  c.set_fill(stripe_grad);
  c.fill_rect(label_x + 6.0, label_y + 6.0, label_w - 12.0, stripe_h);

  // Side A / Stereo badges on label header
  let badge_sz = (label_h * 0.09).clamp(9.0, 14.0);
  c.draw_text(
    "A",
    label_x + 16.0,
    label_y + stripe_h * 0.85,
    badge_sz * 1.3,
    "sans-serif",
    800.0,
    false,
    TextAlign::Left,
    Fill::Solid(Color::WHITE),
    1.0,
    &Default::default(),
  );

  c.draw_text(
    "HIGH BIAS / TYPE II  •  STEREO  •  NR",
    label_x + label_w - 12.0,
    label_y + stripe_h * 0.80,
    badge_sz * 0.75,
    "monospace",
    600.0,
    false,
    TextAlign::Right,
    Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.90)),
    1.0,
    &Default::default(),
  );

  // Track Title / Cassette Label Text
  let title_y = label_y + label_h * 0.23;
  if !ctx.config.text.cassette_label.trim().is_empty() {
    let track_title = ctx.config.text.cassette_label.to_uppercase();
    c.draw_text(
      &track_title,
      center_x,
      title_y,
      (label_h * 0.080).clamp(9.0, 15.0),
      "monospace",
      700.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(0.12, 0.12, 0.15, 0.90)),
      1.0,
      &Default::default(),
    );
  }

  // Dotted write-in title lines below title
  c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, 0.15)));
  c.set_line_width(1.0);
  c.stroke_line(label_x + 20.0, title_y + 4.0, label_x + label_w - 20.0, title_y + 4.0);

  // -------------------------------------------------------------------------
  // 5. TRANSPARENT ACRYLIC TAPE WINDOW (RECESSED CLEAR WINDOW)
  // -------------------------------------------------------------------------
  let win_w = label_w * 0.72;
  let win_h = label_h * 0.44;
  let win_x = center_x - win_w / 2.0;
  let win_y = label_y + label_h * 0.32;

  // Dark window interior chamber
  c.set_fill(Fill::Solid(Color::rgba(0.04, 0.03, 0.06, 0.98)));
  c.fill_rounded_rect(win_x, win_y, win_w, win_h, 6.0);

  // Recessed inner shadow
  c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, 0.8)));
  c.set_line_width(2.0);
  c.stroke_rect(win_x, win_y, win_w, win_h);

  // -------------------------------------------------------------------------
  // 6. AUTHENTIC DUAL TAPE REELS & MAGNETIC TAPE VOLUME CONSERVATION
  // -------------------------------------------------------------------------
  let reel_r_min = win_h * 0.22;
  let reel_r_max = win_h * 0.42;

  let reel_left_x = center_x - win_w * 0.26;
  let reel_right_x = center_x + win_w * 0.26;
  let reel_center_y = win_y + win_h * 0.48;

  // Real tape unwinding physics: conservation of tape cross-sectional area
  // Left reel starts full of tape, transfers to right reel as song plays!
  let tape_progress = ((frame_time * 0.02) % 1.0).clamp(0.0, 1.0);
  let area_min = reel_r_min * reel_r_min;
  let area_max = reel_r_max * reel_r_max;

  let r_left = (area_max * (1.0 - tape_progress) + area_min * tape_progress).sqrt();
  let r_right = (area_min * (1.0 - tape_progress) + area_max * tape_progress).sqrt();

  // Dark brown magnetic oxide tape wound rolls on spools
  let tape_color = Color::rgba(0.18, 0.11, 0.07, 0.98);
  let tape_sheen = Color::rgba(0.32, 0.20, 0.12, 0.98);

  // Left tape roll
  let left_roll_grad = Fill::radial_gradient(
    reel_left_x - r_left * 0.3,
    reel_center_y - r_left * 0.3,
    0.0,
    reel_left_x,
    reel_center_y,
    r_left,
    &[(0.0, tape_sheen), (0.7, tape_color), (1.0, Color::rgba(0.10, 0.06, 0.04, 0.98))],
  );
  c.set_fill(left_roll_grad);
  c.fill_circle(reel_left_x, reel_center_y, r_left);

  // Right tape roll
  let right_roll_grad = Fill::radial_gradient(
    reel_right_x - r_right * 0.3,
    reel_center_y - r_right * 0.3,
    0.0,
    reel_right_x,
    reel_center_y,
    r_right,
    &[(0.0, tape_sheen), (0.7, tape_color), (1.0, Color::rgba(0.10, 0.06, 0.04, 0.98))],
  );
  c.set_fill(right_roll_grad);
  c.fill_circle(reel_right_x, reel_center_y, r_right);

  // Draw White Plastic Hub Spools & Rotating Drive Gear Teeth
  for &(rx, is_left) in &[(reel_left_x, true), (reel_right_x, false)] {
    // White plastic reel hub flange
    let hub_outer_r = reel_r_min;
    c.set_fill(Fill::Solid(Color::rgba(0.95, 0.95, 0.92, 0.98)));
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.4), 4.0);
    c.fill_circle(rx, reel_center_y, hub_outer_r);

    c.set_stroke(Fill::Solid(Color::rgba(0.4, 0.4, 0.4, 0.5)));
    c.set_line_width(1.0);
    c.stroke_circle(rx, reel_center_y, hub_outer_r);

    // Inner center drive hole
    let hub_inner_r = hub_outer_r * 0.45;
    c.set_fill(Fill::Solid(Color::rgba(0.04, 0.03, 0.06, 0.98)));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(rx, reel_center_y, hub_inner_r);

    // 3-Spoke / 6-Teeth rotating drive gear teeth
    let dir = if is_left { 1.0 } else { -1.0 };
    let current_rot = rot * dir * 1.5;

    c.set_fill(Fill::Solid(Color::rgba(0.92, 0.92, 0.90, 0.98)));

    for t_idx in 0..6 {
      let tooth_a = current_rot + (t_idx as f32 / 6.0) * TAU;
      let tx = rx + tooth_a.cos() * (hub_inner_r * 0.85);
      let ty = reel_center_y + tooth_a.sin() * (hub_inner_r * 0.85);
      let tooth_r = hub_inner_r * 0.28;

      c.fill_circle(tx, ty, tooth_r);
    }
  }

  // Digital LED Time Counter inside Window
  let led_bg_w = win_w * 0.25;
  let led_bg_h = win_h * 0.22;
  let led_bg_x = center_x - led_bg_w / 2.0;
  let led_bg_y = win_y + win_h * 0.70;

  c.set_fill(Fill::Solid(Color::rgba(0.02, 0.05, 0.04, 0.90)));
  c.fill_rounded_rect(led_bg_x, led_bg_y, led_bg_w, led_bg_h, 3.0);

  c.draw_text(
    &short_time_str,
    center_x,
    led_bg_y + led_bg_h * 0.72,
    (led_bg_h * 0.65).clamp(8.0, 13.0),
    "monospace",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::hex("#00ff66")),
    1.0,
    &Default::default(),
  );

  // Glass Sheen Diagonal Reflection across acrylic window
  let window_glare = Fill::linear_gradient(
    win_x,
    win_y,
    win_x + win_w,
    win_y + win_h,
    &[
      (0.0, Color::rgba(1.0, 1.0, 1.0, 0.12)),
      (0.35, Color::rgba(1.0, 1.0, 1.0, 0.04)),
      (0.40, Color::TRANSPARENT),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(window_glare);
  c.fill_rounded_rect(win_x, win_y, win_w, win_h, 6.0);

  // -------------------------------------------------------------------------
  // 7. BOTTOM TRAPEZOID TAPE HEAD CHAMBER & VISIBLE TAPE PATH
  // -------------------------------------------------------------------------
  let trap_w = tape_w * 0.68;
  let trap_h = tape_h * 0.18;
  let trap_top_w = trap_w * 0.88;

  let trap_x0 = center_x - trap_top_w / 2.0;
  let trap_x1 = center_x + trap_top_w / 2.0;
  let trap_x2 = center_x + trap_w / 2.0;
  let trap_x3 = center_x - trap_w / 2.0;

  let trap_y0 = top_y + tape_h - trap_h - 4.0;
  let trap_y1 = top_y + tape_h - 4.0;

  // Recessed trapezoid head bay opening
  c.set_fill(Fill::Solid(Color::rgba(0.06, 0.06, 0.09, 0.98)));
  c.fill_polygon(&[
    (trap_x0, trap_y0),
    (trap_x1, trap_y0),
    (trap_x2, trap_y1),
    (trap_x3, trap_y1),
  ]);

  c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.15)));
  c.set_line_width(1.2);
  c.stroke_line(trap_x0, trap_y0, trap_x1, trap_y0);
  c.stroke_line(trap_x2, trap_y1, trap_x3, trap_y1);

  // Visible magnetic tape ribbon running across bottom read heads
  let tape_path_y = trap_y0 + trap_h * 0.45;
  c.set_stroke(Fill::Solid(Color::rgba(0.24, 0.15, 0.10, 0.95)));
  c.set_line_width(5.0);
  c.stroke_line(trap_x3 + 12.0, tape_path_y, trap_x2 - 12.0, tape_path_y);

  // Read Head Metallic Blocks & Guide Rollers inside Chamber
  let head_cx = center_x;
  let head_y = trap_y0 + trap_h * 0.40;

  // Main metallic play/record head block
  c.set_fill(Fill::Solid(Color::rgba(0.70, 0.72, 0.78, 0.95)));
  c.fill_rounded_rect(head_cx - 12.0, head_y - 4.0, 24.0, 12.0, 2.0);
  c.set_fill(Fill::Solid(Color::rgba(0.30, 0.32, 0.38, 0.95)));
  c.fill_rect(head_cx - 4.0, head_y - 4.0, 8.0, 12.0);

  // Left & Right Capstan Roller Pins
  let pin_left_x = trap_x3 + trap_w * 0.18;
  let pin_right_x = trap_x2 - trap_w * 0.18;

  c.set_fill(Fill::Solid(Color::rgba(0.85, 0.88, 0.92, 0.95)));
  c.fill_circle(pin_left_x, head_y + 2.0, 4.0);
  c.fill_circle(pin_right_x, head_y + 2.0, 4.0);


  // -------------------------------------------------------------------------
  // 9. METALLIC CORNER CROSS SCREWS (4 CORNERS)
  // -------------------------------------------------------------------------
  let screw_r = (tape_w * 0.016).clamp(3.0, 7.0);
  let screw_margin = 14.0;

  let corners = [
    (left_x + screw_margin, top_y + screw_margin),
    (left_x + tape_w - screw_margin, top_y + screw_margin),
    (left_x + screw_margin, top_y + tape_h - screw_margin),
    (left_x + tape_w - screw_margin, top_y + tape_h - screw_margin),
  ];

  for &(sx, sy) in &corners {
    // Silver metallic screw body
    let screw_grad = Fill::radial_gradient(
      sx - screw_r * 0.3,
      sy - screw_r * 0.3,
      0.0,
      sx,
      sy,
      screw_r,
      &[
        (0.0, Color::rgba(0.95, 0.95, 0.98, 0.98)),
        (0.6, Color::rgba(0.65, 0.68, 0.72, 0.98)),
        (1.0, Color::rgba(0.35, 0.38, 0.42, 0.98)),
      ],
    );
    c.set_fill(screw_grad);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 3.0);
    c.fill_circle(sx, sy, screw_r);

    // Cross-head screw slot line ("X")
    c.set_stroke(Fill::Solid(Color::rgba(0.20, 0.22, 0.26, 0.95)));
    c.set_line_width(1.0);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.stroke_line(sx - screw_r * 0.6, sy, sx + screw_r * 0.6, sy);
    c.stroke_line(sx, sy - screw_r * 0.6, sx, sy + screw_r * 0.6);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);
  c.restore();
}
