//! Retro Radio style renderer (`retroRadio`).
//!
//! Renders a vintage hardware radio/boombox chassis featuring dual chrome antennas,
//! gold-trimmed screen bezel, 3 rotary control knobs, speaker vent grilles,
//! and a warm amber/orange LED pixel matrix spectrum display.

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const MATRIX_COLS: usize = 36;
const MATRIX_ROWS: usize = 14;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  let radio_w = (width * 0.56).clamp(300.0, 740.0);
  let radio_h = radio_w * 0.48;
  let left_x = center_x - radio_w / 2.0;
  let top_y = center_y - radio_h / 2.0;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. DUAL TELESCOPING CHROME ANTENNAS (TOP OF RADIO)
  // -------------------------------------------------------------------------
  let ant_w = (radio_w * 0.018).clamp(5.0, 10.0);
  let ant_h = radio_h * 0.45;
  let ant_left_x = center_x - radio_w * 0.22;
  let ant_right_x = center_x + radio_w * 0.22;
  let ant_top_y = top_y - ant_h;

  for &ax in &[ant_left_x, ant_right_x] {
    let ant_grad = Fill::linear_gradient(
      ax - ant_w / 2.0,
      ant_top_y,
      ax + ant_w / 2.0,
      top_y,
      &[
        (0.0, Color::rgba(0.9, 0.9, 0.95, 0.95)),
        (0.5, Color::rgba(0.4, 0.4, 0.45, 0.95)),
        (1.0, Color::rgba(0.85, 0.85, 0.9, 0.95)),
      ],
    );
    c.set_fill(ant_grad);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 6.0);
    c.fill_rounded_rect(ax - ant_w / 2.0, ant_top_y, ant_w, ant_h + 4.0, 2.0);

    // Antenna metallic tip cap
    c.set_fill(Fill::Solid(Color::rgba(0.95, 0.95, 1.0, 0.98)));
    c.fill_ellipse(ax, ant_top_y, ant_w * 0.8, ant_w * 0.8);
  }

  // -------------------------------------------------------------------------
  // 2. RADIO CHASSIS OUTER SHELL
  // -------------------------------------------------------------------------
  c.set_fill(Fill::Solid(Color::rgba(0.08, 0.08, 0.1, 0.96)));
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.7), 24.0);
  c.fill_rounded_rect(left_x, top_y, radio_w, radio_h, 16.0);

  c.set_stroke(Fill::Solid(Color::rgba(0.22, 0.2, 0.26, 0.8)));
  c.set_line_width(2.0);
  c.stroke_rect(left_x, top_y, radio_w, radio_h);

  // -------------------------------------------------------------------------
  // 3. GOLD / COPPER TRIMMED SCREEN BEZEL FRAME
  // -------------------------------------------------------------------------
  let scr_w = radio_w * 0.88;
  let scr_h = radio_h * 0.54;
  let scr_x = center_x - scr_w / 2.0;
  let scr_y = top_y + radio_h * 0.07;

  // Gold Bezel Frame
  c.set_fill(Fill::Solid(Color::rgba(0.72, 0.52, 0.28, 0.92)));
  c.set_shadow(Color::rgba(0.8, 0.5, 0.2, 0.4), 10.0);
  c.fill_rounded_rect(scr_x - 4.0, scr_y - 4.0, scr_w + 8.0, scr_h + 8.0, 8.0);

  // Inner Dark CRT Glass Panel
  c.set_fill(Fill::Solid(Color::rgba(0.03, 0.02, 0.04, 0.98)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_rounded_rect(scr_x, scr_y, scr_w, scr_h, 4.0);

  // -------------------------------------------------------------------------
  // 4. LED MATRIX SPECTRUM DISPLAY (36 COLS x 14 ROWS OF AMBER/ORANGE LEDS)
  // -------------------------------------------------------------------------
  let pad_x = 8.0f32;
  let pad_y = 6.0f32;
  let cell_w = (scr_w - pad_x * 2.0) / MATRIX_COLS as f32;
  let cell_h = (scr_h - pad_y * 2.0) / MATRIX_ROWS as f32;

  let step = (freq.len() / MATRIX_COLS).max(1);

  for col in 0..MATRIX_COLS {
    let raw_v = *freq.get(col * step).unwrap_or(&0) as f32 / 255.0;
    let val = (raw_v * sensitivity).clamp(0.0, 1.2);
    let active_rows = (val * MATRIX_ROWS as f32) as usize;

    let cx = scr_x + pad_x + col as f32 * cell_w;

    for row in 0..MATRIX_ROWS {
      let is_active = row < active_rows;
      let row_ratio = row as f32 / MATRIX_ROWS as f32;

      let cy = scr_y + scr_h - pad_y - (row as f32 + 1.0) * cell_h;

      let (col_active, shadow_glow) = if row_ratio < 0.35 {
        // Lower rows: Fiery Red / Deep Magenta
        (Color::rgba(1.0, 0.15, 0.25, 0.95), Color::rgba(1.0, 0.1, 0.2, 0.7))
      } else if row_ratio < 0.75 {
        // Middle rows: Warm Amber / Deep Orange
        (Color::rgba(1.0, 0.55, 0.1, 0.95), Color::rgba(1.0, 0.5, 0.1, 0.7))
      } else {
        // Top tip rows: Bright Peach / Yellow
        (Color::rgba(1.0, 0.85, 0.3, 0.98), Color::rgba(1.0, 0.8, 0.2, 0.85))
      };

      let px_w = cell_w - 2.0;
      let px_h = cell_h - 2.0;

      if is_active {
        c.set_fill(Fill::Solid(col_active));
        c.set_shadow(shadow_glow, 6.0 + bs * 4.0);
        c.fill_rect(cx + 1.0, cy + 1.0, px_w.max(1.5), px_h.max(1.5));
      } else {
        // Dim unlit LED grid pixel background
        c.set_fill(Fill::Solid(Color::rgba(0.12, 0.08, 0.1, 0.25)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_rect(cx + 1.0, cy + 1.0, px_w.max(1.5), px_h.max(1.5));
      }
    }
  }

  // -------------------------------------------------------------------------
  // 5. LOWER CONTROL PANEL (3 ROTARY KNOBS + SPEAKER VENT GRILLES)
  // -------------------------------------------------------------------------
  let ctrl_y = scr_y + scr_h + radio_h * 0.08;
  let knob_r = (radio_h * 0.075).clamp(10.0, 20.0);

  // Labels: MODES | MENU | AMPLITUDE
  let labels = ["MODES", "MENU", "AMPLITUDE"];
  let knob_offsets = [-radio_w * 0.14, 0.0f32, radio_w * 0.14];

  for i in 0..3 {
    let kx = center_x + knob_offsets[i];
    let ky = ctrl_y + knob_r + 14.0;

    // Knob Label Text
    c.draw_text(
      labels[i],
      kx,
      ky - knob_r - 7.0,
      9.0,
      "sans-serif",
      600.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(0.7, 0.65, 0.55, 0.7)),
      1.0,
      &Default::default(),
    );

    // Rotary Knob Body
    c.set_fill(Fill::Solid(Color::rgba(0.15, 0.14, 0.18, 0.98)));
    c.set_stroke(Fill::Solid(Color::rgba(0.5, 0.45, 0.4, 0.8)));
    c.set_line_width(1.5);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 8.0);

    c.fill_ellipse(kx, ky, knob_r, knob_r);
    c.stroke_circle(kx, ky, knob_r);

    // Knob Indicator Notch (rotates slightly on bass)
    let knob_angle = -std::f32::consts::FRAC_PI_4 + (i as f32 * 0.4) + (be * 0.15);
    let notch_x = kx + knob_angle.cos() * (knob_r * 0.7);
    let notch_y = ky + knob_angle.sin() * (knob_r * 0.7);

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.0);
    c.stroke_line(kx, ky, notch_x, notch_y);
  }

  // Left & Right Speaker Vent Grilles
  let grille_w = radio_w * 0.12;
  let grille_h = radio_h * 0.12;
  let grille_left_x = left_x + radio_w * 0.05;
  let grille_right_x = left_x + radio_w * 0.83;
  let grille_y = ctrl_y + 10.0;

  for &gx in &[grille_left_x, grille_right_x] {
    c.set_fill(Fill::Solid(Color::rgba(0.65, 0.48, 0.25, 0.85)));
    c.stroke_rect(gx, grille_y, grille_w, grille_h);

    // Horizontal grill slats
    c.set_stroke(Fill::Solid(Color::rgba(0.1, 0.08, 0.12, 0.9)));
    c.set_line_width(1.5);
    for s_idx in 1..4 {
      let sy = grille_y + (s_idx as f32 / 4.0) * grille_h;
      c.stroke_line(gx + 2.0, sy, gx + grille_w - 2.0, sy);
    }
  }

  c.restore();
}
