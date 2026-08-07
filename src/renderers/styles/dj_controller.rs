//! DJ Controller style renderer (`djController`).
//!
//! Renders an angled perspective DJ mixer console complete with dual spinning
//! neon jog wheels, 4-channel audio faders, glowing LED knobs, illuminated cue buttons,
//! and a surrounding terrain of dynamic pink and cyan neon horizon waves.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

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

  // -------------------------------------------------------------------------
  // 1. BACKGROUND HORIZON NEON MOUNTAIN WAVES
  // -------------------------------------------------------------------------
  let horizon_y = center_y - height * 0.08;
  let wave_steps = 60usize;

  for wave_layer in 0..4 {
    let layer_ratio = (wave_layer + 1) as f32 / 4.0;
    let wave_amp = (height * 0.12 * layer_ratio) * (0.6 + be * 0.8) * sensitivity;
    let wave_col = if wave_layer % 2 == 0 { hot_pink } else { neon_purple };
    let alpha = 0.4 + layer_ratio * 0.55;

    let mut wave_pts = Vec::with_capacity(wave_steps + 1);
    for i in 0..=wave_steps {
      let x = (i as f32 / wave_steps as f32) * width;
      let bin_idx = (i * freq.len() / wave_steps).min(freq.len().saturating_sub(1));
      let f_val = *freq.get(bin_idx).unwrap_or(&0) as f32 / 255.0;

      let y = horizon_y - (f_val * wave_amp) - ((x * 0.015 + rot * (wave_layer as f32 + 1.0)).sin() * 15.0);
      wave_pts.push((x, y));
    }

    c.set_stroke(Fill::Solid(wave_col.with_alpha(alpha)));
    c.set_line_width(2.5 + layer_ratio * 1.5);
    c.set_shadow(wave_col, 10.0 + bs * 8.0);
    c.stroke_polyline(&wave_pts);
  }

  // -------------------------------------------------------------------------
  // 2. PERSPECTIVE FLOOR NEON RIBBON LINES
  // -------------------------------------------------------------------------
  let num_floor_lines = 24usize;
  c.set_shadow(Color::TRANSPARENT, 0.0);

  for i in 0..num_floor_lines {
    let t_val = i as f32 / (num_floor_lines - 1) as f32;
    let x_top = center_x + (t_val - 0.5) * (width * 0.4);
    let x_bot = center_x + (t_val - 0.5) * (width * 1.3);

    let line_col = if i % 2 == 0 { electric_cyan } else { hot_pink };
    let line_alpha = 0.35 + (be * 0.4).min(0.5);

    c.set_stroke(Fill::Solid(line_col.with_alpha(line_alpha)));
    c.set_line_width(1.8);
    c.stroke_line(x_top, horizon_y + 10.0, x_bot, height);
  }

  // -------------------------------------------------------------------------
  // 3. 3D PERSPECTIVE DJ MIXER CONSOLE BOARD (CENTER STAGE)
  // -------------------------------------------------------------------------
  let board_w_top = (width * 0.46).clamp(280.0, 680.0);
  let board_w_bot = board_w_top * 1.25;
  let board_h = board_w_top * 0.46;

  let b_top_y = center_y - board_h * 0.4;
  let b_bot_y = center_y + board_h * 0.6;

  let tl = (center_x - board_w_top * 0.5, b_top_y);
  let tr = (center_x + board_w_top * 0.5, b_top_y);
  let br = (center_x + board_w_bot * 0.5, b_bot_y);
  let bl = (center_x - board_w_bot * 0.5, b_bot_y);

  // Dark Metallic Plinth Base
  c.set_fill(Fill::Solid(Color::rgba(0.06, 0.05, 0.08, 0.98)));
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.85), 30.0);
  c.fill_rounded_rect(center_x - board_w_bot * 0.5, b_top_y, board_w_bot, board_h, 12.0);

  c.set_stroke(Fill::Solid(Color::rgba(0.3, 0.28, 0.4, 0.8)));
  c.set_line_width(2.0);
  c.stroke_polyline(&[tl, tr, br, bl, tl]);

  // Front lip highlight
  c.set_stroke(Fill::Solid(electric_cyan.with_alpha(0.8)));
  c.set_line_width(2.0);
  c.stroke_line(bl.0, bl.1, br.0, br.1);

  // -------------------------------------------------------------------------
  // 4. LEFT & RIGHT NEON JOG WHEELS
  // -------------------------------------------------------------------------
  let jog_r = board_h * 0.28;
  let left_jog_cx = center_x - board_w_top * 0.28;
  let right_jog_cx = center_x + board_w_top * 0.28;
  let jog_cy = center_y + board_h * 0.05;

  // Left Jog Wheel (Electric Cyan Neon Ring)
  c.set_fill(Fill::Solid(Color::rgba(0.1, 0.1, 0.14, 0.98)));
  c.set_stroke(Fill::Solid(electric_cyan));
  c.set_line_width(3.0);
  c.set_shadow(electric_cyan.with_alpha(0.8), 16.0);
  c.fill_ellipse(left_jog_cx, jog_cy, jog_r, jog_r);
  c.stroke_circle(left_jog_cx, jog_cy, jog_r);

  // Right Jog Wheel (Hot Pink Neon Ring)
  c.set_stroke(Fill::Solid(hot_pink));
  c.set_shadow(hot_pink.with_alpha(0.8), 16.0);
  c.fill_ellipse(right_jog_cx, jog_cy, jog_r, jog_r);
  c.stroke_circle(right_jog_cx, jog_cy, jog_r);

  // Spinning jog wheel center marker notches
  for &(jcx, jcol) in &[(left_jog_cx, electric_cyan), (right_jog_cx, hot_pink)] {
    let nx = jcx + rot.cos() * (jog_r * 0.65);
    let ny = jog_cy + rot.sin() * (jog_r * 0.65);

    c.set_fill(Fill::Solid(jcol));
    c.set_shadow(jcol, 8.0);
    c.fill_ellipse(nx, ny, 4.0, 4.0);
  }

  // -------------------------------------------------------------------------
  // 5. CENTER MIXER PANEL (4-CHANNEL FADERS + EQ KNOBS)
  // -------------------------------------------------------------------------
  let mixer_w = board_w_top * 0.32;
  let mixer_left = center_x - mixer_w / 2.0;

  // Channel Fader Slots & Glowing Fader Caps
  let num_channels = 4usize;
  let chan_step = mixer_w / (num_channels as f32 + 1.0);
  let slot_h = board_h * 0.35;
  let slot_y = jog_cy - slot_h * 0.3;

  c.set_shadow(Color::TRANSPARENT, 0.0);

  for ch in 0..num_channels {
    let ch_x = mixer_left + (ch as f32 + 1.0) * chan_step;

    // Slot track line
    c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.25, 0.35, 0.8)));
    c.set_line_width(2.0);
    c.stroke_line(ch_x, slot_y, ch_x, slot_y + slot_h);

    // Audio-reactive fader position
    let bin = (ch * freq.len() / num_channels).min(freq.len().saturating_sub(1));
    let f_level = (*freq.get(bin).unwrap_or(&0) as f32 / 255.0 * sensitivity).clamp(0.1, 0.95);
    let cap_y = slot_y + slot_h * (1.0 - f_level);

    // Illuminated Fader Knob Cap
    let cap_w = 12.0f32;
    let cap_h = 6.0f32;
    let cap_col = if ch % 2 == 0 { electric_cyan } else { hot_pink };

    c.set_fill(Fill::Solid(cap_col));
    c.set_shadow(cap_col, 8.0);
    c.fill_rounded_rect(ch_x - cap_w / 2.0, cap_y - cap_h / 2.0, cap_w, cap_h, 2.0);

    // EQ Knob dots above fader slot
    for eq_k in 1..3 {
      let eq_y = slot_y - (eq_k as f32 * 14.0);
      c.set_fill(Fill::Solid(Color::rgba(0.8, 0.8, 0.9, 0.9)));
      c.set_shadow(Color::TRANSPARENT, 0.0);
      c.fill_ellipse(ch_x, eq_y, 3.5, 3.5);
    }
  }

  // Crossfader Horizontal Slider at bottom of mixer
  let xfader_w = mixer_w * 0.7;
  let xfader_y = b_bot_y - board_h * 0.12;
  let xfader_x = center_x + (be * 0.3).sin() * (xfader_w * 0.3);

  c.set_stroke(Fill::Solid(Color::rgba(0.3, 0.3, 0.4, 0.8)));
  c.set_line_width(2.5);
  c.stroke_line(center_x - xfader_w / 2.0, xfader_y, center_x + xfader_w / 2.0, xfader_y);

  c.set_fill(Fill::Solid(Color::WHITE));
  c.set_shadow(electric_cyan, 10.0);
  c.fill_rounded_rect(xfader_x - 6.0, xfader_y - 4.0, 12.0, 8.0, 2.0);

  c.restore();
}
