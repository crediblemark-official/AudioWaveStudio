//! VU Meter style renderer (`vuMeter`).
//!
//! Renders dual high-precision analog/cyber VU meters with dynamic ballistic
//! needles, glowing multi-zone arc gauges, peak-hold indicators, overload LEDs,
//! glassmorphic meter housings, and frequency spectrum audio reactivity across
//! both channels.

use std::f32::consts::PI;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas, LineCap};
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let sensitivity = ctx.config.reactivity.sensitivity;
  let num_bins = ctx.freq_data.len();

  // -------------------------------------------------------------------------
  // 1. DYNAMIC FREQUENCY & AUDIO LEVEL ANALYSIS (STEREO SPAN)
  // -------------------------------------------------------------------------
  for ch in 0..2 {
    let raw = if num_bins > 0 {
      // Left channel (ch 0): focus on low-to-mid range (0% to 65% of spectrum)
      // Right channel (ch 1): focus on mid-to-high range (20% to 95% of spectrum)
      let (start_pct, end_pct) = if ch == 0 { (0.00, 0.65) } else { (0.20, 0.95) };
      let start_i = ((start_pct * num_bins as f32) as usize).min(num_bins - 1);
      let end_i = ((end_pct * num_bins as f32) as usize).clamp(start_i + 1, num_bins);

      let mut sum = 0usize;
      let mut count = 0usize;
      for k in start_i..end_i {
        sum += ctx.freq_data[k] as usize;
        count += 1;
      }

      let spectrum_avg = if count > 0 {
        sum as f32 / (count as f32 * 255.0)
      } else {
        0.0
      };

      // Add energy boost from bass/beat strength for responsive needle bouncing
      let energy_boost = if ch == 0 {
        ctx.bass_energy * 0.35
      } else {
        ctx.beat_strength * 0.30
      };

      ((spectrum_avg + energy_boost) * sensitivity).clamp(0.0, 1.2)
    } else {
      0.0
    };

    let ch_state = &mut ctx.state.vu[ch];
    let target = raw.clamp(0.0, 1.0);

    // Ballistic physics for realistic analog needle response:
    // Fast attack on transients, smooth physical decay on drop
    let speed = if target > ch_state.level { 0.35 } else { 0.16 };
    ch_state.level += (target - ch_state.level) * speed;
    ch_state.level = ch_state.level.clamp(0.0, 1.0);

    // Peak & Peak Hold decay logic:
    ch_state.peak = ch_state.peak.max(ch_state.level);
    ch_state.peak_hold = ch_state.peak_hold.max(ch_state.level);
    ch_state.peak *= 0.93;
    ch_state.peak_hold -= 0.004;
    if ch_state.peak_hold < ch_state.level {
      ch_state.peak_hold = ch_state.level;
    }
  }

  // -------------------------------------------------------------------------
  // 2. LAYOUT & METER GAUGES GEOMETRY
  // -------------------------------------------------------------------------
  let user_scale = ctx.config.scale.clamp(0.1, 5.0);
  let pos_offset_x = ctx.config.position_x * ctx.width * 0.5;
  let pos_offset_y = -ctx.config.position_y * ctx.height * 0.5;
  let cx = ctx.width / 2.0 + pos_offset_x;
  let cy = ctx.height / 2.0 + pos_offset_y;

  let res_scale = ctx.height / 1080.0;
  // Compute meter housing dimensions so both meters sit side-by-side cleanly
  let card_w = ctx.width * 0.44 * user_scale;
  let card_h = card_w * 0.75;
  let card_gap = ctx.width * 0.03 * user_scale;
  let center_spacing = (card_w + card_gap) / 2.0;

  let gauge_r = (card_w * 0.40).min(card_h * 0.45);
  let pivot_rel_y = gauge_r * 0.40;

  // Arc angles (212.4° to 327.6° = 115.2° fan sweep)
  let start_angle = PI * 1.18;
  let end_angle = PI * 1.82;
  let sweep_angle = end_angle - start_angle;

  // -------------------------------------------------------------------------
  // 3. RENDER METERS
  // -------------------------------------------------------------------------
  c.save();

  for ch in 0..2 {
    let card_cx = if ch == 0 { cx - center_spacing } else { cx + center_spacing };
    let card_cy = cy - ctx.height * 0.02;
    let card_x = card_cx - card_w / 2.0;
    let card_y = card_cy - card_h / 2.0;

    let ch_state = &ctx.state.vu[ch];
    let level = ch_state.level;
    let peak_hold = ch_state.peak_hold;

    // --- A. Meter Housing Card (Glassmorphic dark chassis) ---
    c.save();
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 20.0);
    c.set_fill(Fill::Solid(Color::rgba(0.07, 0.08, 0.12, 0.90)));
    c.fill_rounded_rect(card_x, card_y, card_w, card_h, 16.0);
    c.restore();

    // Dial Window Background Plate
    let plate_w = card_w - 24.0;
    let plate_h = card_h - 24.0;
    let plate_x = card_cx - plate_w / 2.0;
    let plate_y = card_cy - plate_h / 2.0;

    c.save();
    c.set_fill(Fill::Solid(Color::rgba(0.04, 0.05, 0.08, 0.95)));
    c.fill_rounded_rect(plate_x, plate_y, plate_w, plate_h, 10.0);

    // Subtle warm dial plate backlight glow when level rises
    let glow_col = if ch == 0 { theme_primary(theme) } else { theme_secondary(theme) };
    c.set_shadow(glow_col.with_alpha(0.3 + level * 0.4), 16.0 + level * 14.0);
    c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.02)));
    c.fill_circle(card_cx, card_cy + pivot_rel_y, gauge_r * 0.6);
    c.restore();

    // --- B. Gauge Dial Coordinates ---
    let pivot_x = card_cx;
    let pivot_y = card_cy + pivot_rel_y;

    c.save();

    // Base Gray Scale Arc Track
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.12)));
    c.set_line_width(6.0);
    c.set_line_cap(LineCap::Round);
    c.stroke_arc(pivot_x, pivot_y, gauge_r, start_angle, end_angle);

    // Active Level Arc
    let active_angle = (level * sweep_angle).clamp(0.001, sweep_angle);
    let active_color = if level > 0.82 {
      Color::hex("#ff3344")
    } else if level > 0.65 {
      theme_accent(theme)
    } else if ch == 0 {
      theme_primary(theme)
    } else {
      theme_secondary(theme)
    };

    c.save();
    c.set_stroke(Fill::Solid(active_color));
    c.set_line_width(6.0);
    c.set_line_cap(LineCap::Round);
    c.set_shadow(active_color, 10.0 + level * 10.0);
    c.stroke_arc(pivot_x, pivot_y, gauge_r, start_angle, start_angle + active_angle);
    c.restore();

    // Peak Zone Red Arc Segment (0dB to +3dB area)
    let red_zone_start = start_angle + sweep_angle * 0.78;
    c.save();
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.2, 0.2, 0.35)));
    c.set_line_width(3.0);
    c.stroke_arc(pivot_x, pivot_y, gauge_r + 6.0, red_zone_start, end_angle);
    c.restore();

    // --- C. Scale Ticks & dB Markings ---
    // Ticks at -20, -10, -5, -3, 0, +3 dB
    let ticks = [
      (-20.0f32, 0.00f32, "-20"),
      (-10.0, 0.30, "-10"),
      (-5.0, 0.55, "-5"),
      (-3.0, 0.68, "-3"),
      (0.0, 0.82, "0"),
      (3.0, 1.00, "+3"),
    ];

    let font_sz = card_w * 0.038;

    for &(db, rel_pos, label) in &ticks {
      let tick_a = start_angle + rel_pos * sweep_angle;
      let is_overload = db >= 0.0;

      let tick_col = if is_overload {
        Color::hex("#ff4444")
      } else {
        Color::rgba(1.0, 1.0, 1.0, 0.50)
      };

      let inner_r = gauge_r - 8.0 * res_scale;
      let outer_r = gauge_r + 4.0 * res_scale;

      let x1 = pivot_x + tick_a.cos() * inner_r;
      let y1 = pivot_y + tick_a.sin() * inner_r;
      let x2 = pivot_x + tick_a.cos() * outer_r;
      let y2 = pivot_y + tick_a.sin() * outer_r;

      c.set_stroke(Fill::Solid(tick_col));
      c.set_line_width(if is_overload { 2.0 * res_scale } else { 1.2 * res_scale });
      c.stroke_line(x1, y1, x2, y2);

      // Label text
      let text_r = gauge_r - 20.0 * res_scale;
      let tx = pivot_x + tick_a.cos() * text_r;
      let ty = pivot_y + tick_a.sin() * text_r + font_sz * 0.35;

      c.draw_text(
        label,
        tx,
        ty,
        font_sz,
        "monospace",
        500.0,
        false,
        TextAlign::Center,
        Fill::Solid(tick_col),
        1.0,
        &Default::default(),
      );
    }

    // --- D. Peak Hold Dot Indicator ---
    let hold_a = start_angle + (peak_hold * sweep_angle).clamp(0.0, sweep_angle);
    let hold_x = pivot_x + hold_a.cos() * gauge_r;
    let hold_y = pivot_y + hold_a.sin() * gauge_r;

    c.save();
    c.set_fill(Fill::Solid(Color::WHITE));
    c.set_shadow(Color::WHITE, 8.0 * res_scale);
    c.fill_circle(hold_x, hold_y, 4.0 * res_scale);
    c.restore();

    // Overload Peak LED Indicator
    let led_x = card_cx + card_w * 0.34;
    let led_y = card_cy - card_h * 0.34;
    let is_clipping = level > 0.85;
    let led_color = if is_clipping { Color::hex("#ff1122") } else { Color::rgba(0.2, 0.05, 0.05, 0.4) };

    c.save();
    c.set_fill(Fill::Solid(led_color));
    if is_clipping {
      c.set_shadow(Color::hex("#ff1122"), 12.0 * res_scale);
    }
    c.fill_circle(led_x, led_y, 5.0 * res_scale);
    c.stroke_circle(led_x, led_y, 5.0 * res_scale);
    c.draw_text(
      "PEAK",
      led_x - 14.0 * res_scale,
      led_y + 3.0 * res_scale,
      font_sz * 0.75,
      "sans-serif",
      600.0,
      false,
      TextAlign::Right,
      Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.4)),
      1.0,
      &Default::default(),
    );
    c.restore();

    // --- E. Dynamic Needle with Drop Shadow & Pivot ---
    let needle_a = start_angle + level * sweep_angle;
    let needle_len = gauge_r * 0.96;

    let needle_tip_x = pivot_x + needle_a.cos() * needle_len;
    let needle_tip_y = pivot_y + needle_a.sin() * needle_len;

    // Soft Needle Shadow for 3D depth on dial plate
    c.save();
    c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, 0.5)));
    c.set_line_width(2.5 * res_scale);
    c.stroke_line(pivot_x + 3.0 * res_scale, pivot_y + 4.0 * res_scale, needle_tip_x + 3.0 * res_scale, needle_tip_y + 4.0 * res_scale);
    c.restore();

    // Glowing Needle Shaft
    c.save();
    let needle_col = theme_accent(theme);
    c.set_stroke(Fill::Solid(needle_col));
    c.set_line_width(2.5 * res_scale);
    c.set_line_cap(LineCap::Round);
    c.set_shadow(theme_glow(theme), 8.0 * res_scale);
    c.stroke_line(pivot_x, pivot_y, needle_tip_x, needle_tip_y);
    c.restore();

    // Needle Pivot Cap
    c.save();
    c.set_fill(Fill::Solid(Color::hex("#111318")));
    c.fill_circle(pivot_x, pivot_y, 9.0 * res_scale);
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.25)));
    c.set_line_width(1.5 * res_scale);
    c.stroke_circle(pivot_x, pivot_y, 9.0 * res_scale);
    c.set_fill(Fill::Solid(Color::WHITE));
    c.fill_circle(pivot_x, pivot_y, 3.5 * res_scale);
    c.restore();

    // --- F. Channel Subtitle & Unit Labels ---
    let ch_label = if ch == 0 { "LEFT  [CH 1]" } else { "RIGHT  [CH 2]" };
    let ch_label_sz = card_w * 0.042;

    c.draw_text(
      ch_label,
      card_cx,
      card_cy + card_h * 0.38,
      ch_label_sz,
      "monospace",
      600.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.70)),
      1.0,
      &Default::default(),
    );

    c.draw_text(
      "VU",
      card_cx,
      pivot_y - gauge_r * 0.35,
      ch_label_sz * 1.1,
      "sans-serif",
      700.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.25)),
      1.0,
      &Default::default(),
    );

    c.restore();
  }

  c.restore();

  // -------------------------------------------------------------------------
  // 4. BOTTOM CENTER SYSTEM TITLE
  // -------------------------------------------------------------------------
  let title_sz = ctx.width * 0.016 * user_scale;
  c.draw_text(
    "ANALOG VU METER SYSTEM",
    cx,
    ctx.height - 18.0 * res_scale,
    title_sz,
    "monospace",
    500.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.45)),
    1.0,
    &Default::default(),
  );
}
