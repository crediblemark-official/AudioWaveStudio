//! Retro Radio style renderer (`retroRadio`) — Vintage Transistor Radio Engine.
//!
//! Renders a hyper-realistic 1970s mahogany wood-grain & polished chrome transistor radio complete with:
//! - Rich walnut wood-grain chassis with gold/chrome corner brackets
//! - Backlit glass AM/FM frequency tuning dial with FM (88..108 MHz) & AM scales
//! - Analog orange tuning needle pointer swaying dynamically across the dial
//! - Audio-reactive amber/orange LED matrix spectrum display inside the glass window
//! - Woven fabric speaker grilles with metallic brand emblem
//! - Dual telescoping chrome antennas & rotary Volume / Tuning / Bass / Treble control knobs
//! - Full UI settings integration (Scale, Position X & Y, Sensitivity, Bass Boost, Bar Count).

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};


const MATRIX_ROWS: usize = 12;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;

  let p = theme_primary(theme);
  let s = theme_secondary(theme);
  let accent = theme_accent(theme);
  let _glow = theme_glow(theme);

  // Settings integration
  let sensitivity = ctx.config.reactivity.sensitivity;
  let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let frame_time = ctx.frame_time;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  let radio_w = ((width * 0.58).clamp(320.0, 780.0)).clamp(180.0, width * 0.95);
  let radio_h = radio_w * 0.52;
  let left_x = center_x - radio_w / 2.0;
  let top_y = center_y - radio_h / 2.0;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. ATMOSPHERIC DEEP BACKDROP & WARM AMBER AURA
  // -------------------------------------------------------------------------
  let bg_haze = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    radio_w * 0.85,
    &[
      (0.0, Color::rgba(1.0, 0.55, 0.10, 0.20 + be * 0.15)),
      (0.40, p.with_alpha(0.12)),
      (0.75, Color::rgba(0.04, 0.02, 0.08, 0.06)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(bg_haze);
//   c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 2. DUAL TELESCOPING CHROME ANTENNAS (TOP OF RADIO)
  // -------------------------------------------------------------------------
  let ant_w = (radio_w * 0.016).clamp(5.0, 10.0);
  let ant_h = radio_h * 0.42;
  let ant_left_x = center_x - radio_w * 0.24;
  let ant_right_x = center_x + radio_w * 0.24;
  let ant_top_y = top_y - ant_h;

  for &ax in &[ant_left_x, ant_right_x] {
    let ant_grad = Fill::linear_gradient(
      ax - ant_w / 2.0,
      ant_top_y,
      ax + ant_w / 2.0,
      top_y,
      &[
        (0.0, Color::rgba(0.95, 0.95, 0.98, 0.95)),
        (0.5, Color::rgba(0.45, 0.48, 0.55, 0.95)),
        (1.0, Color::rgba(0.85, 0.88, 0.92, 0.95)),
      ],
    );
    c.set_fill(ant_grad);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 6.0);
    c.fill_rounded_rect(ax - ant_w / 2.0, ant_top_y, ant_w, ant_h + 4.0, 2.0);

    // Antenna metallic tip cap
    c.set_fill(Fill::Solid(Color::rgba(0.98, 0.98, 1.0, 0.98)));
    c.fill_ellipse(ax, ant_top_y, ant_w * 0.85, ant_w * 0.85);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 3. RICH MAHOGANY WOOD-GRAIN CHASSIS & GOLD CORNER BRACKETS
  // -------------------------------------------------------------------------
  // Drop shadow behind radio
  c.save();
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.82), 26.0);
  c.set_fill(Fill::Solid(Color::rgba(0.18, 0.09, 0.04, 0.98)));
  c.fill_rounded_rect(left_x, top_y, radio_w, radio_h, 14.0);
  c.restore();

  // Mahogany Wood-Grain Body Gradient
  let wood_grad = Fill::linear_gradient(
    left_x,
    top_y,
    left_x + radio_w,
    top_y + radio_h,
    &[
      (0.0, Color::rgba(0.24, 0.11, 0.05, 0.98)),
      (0.35, Color::rgba(0.38, 0.18, 0.08, 0.98)),
      (0.70, Color::rgba(0.18, 0.08, 0.03, 0.98)),
      (1.0, Color::rgba(0.28, 0.13, 0.06, 0.98)),
    ],
  );
  c.set_fill(wood_grad);
  c.fill_rounded_rect(left_x, top_y, radio_w, radio_h, 14.0);

  // Outer bevel rim highlight
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.75, 0.40, 0.30)));
  c.set_line_width(1.5);
  c.stroke_rect(left_x, top_y, radio_w, radio_h);

  // Gold Corner Brackets (4 Corners)
  let bracket_sz = (radio_h * 0.08).clamp(8.0, 18.0);
  let gold_bracket = Fill::Solid(Color::rgba(0.85, 0.65, 0.30, 0.95));

  for &(bx, by) in &[
    (left_x + 6.0, top_y + 6.0),
    (left_x + radio_w - bracket_sz - 6.0, top_y + 6.0),
    (left_x + 6.0, top_y + radio_h - bracket_sz - 6.0),
    (left_x + radio_w - bracket_sz - 6.0, top_y + radio_h - bracket_sz - 6.0),
  ] {
    c.set_fill(gold_bracket.clone());
    c.fill_rounded_rect(bx, by, bracket_sz, bracket_sz, 2.0);
  }

  // -------------------------------------------------------------------------
  // 4. BACKLIT AMBER GLASS AM/FM TUNING DIAL WINDOW
  // -------------------------------------------------------------------------
  let dial_w = radio_w * 0.88;
  let dial_h = radio_h * 0.52;
  let dial_x = center_x - dial_w / 2.0;
  let dial_y = top_y + radio_h * 0.08;

  // Gold Bezel Frame around Tuning Dial
  c.set_fill(Fill::Solid(Color::rgba(0.78, 0.58, 0.26, 0.95)));
  c.set_shadow(Color::rgba(0.8, 0.5, 0.15, 0.4), 12.0);
  c.fill_rounded_rect(dial_x - 4.0, dial_y - 4.0, dial_w + 8.0, dial_h + 8.0, 8.0);

  // Inner Dark CRT Glass Panel with Warm Amber Backlight Glow
  let glass_bg = Fill::radial_gradient(
    center_x,
    dial_y + dial_h * 0.5,
    0.0,
    center_x,
    dial_y + dial_h * 0.5,
    dial_w * 0.6,
    &[
      (0.0, Color::rgba(0.20, 0.08, 0.02, 0.98)),
      (0.60, Color::rgba(0.08, 0.04, 0.02, 0.98)),
      (1.0, Color::rgba(0.03, 0.02, 0.04, 0.98)),
    ],
  );
  c.set_fill(glass_bg);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_rounded_rect(dial_x, dial_y, dial_w, dial_h, 4.0);

  // AM/FM Frequency Dial Marking Lines
  let freq_y = dial_y + 16.0;
  c.draw_text(
    "FM 88   92   96   100   104   108 MHz",
    center_x,
    freq_y,
    (dial_h * 0.10).clamp(8.0, 12.0),
    "monospace",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(1.0, 0.85, 0.40, 0.90)),
    1.0,
    &Default::default(),
  );

  c.draw_text(
    "AM 54  70  90  110  140  160 x10kHz",
    center_x,
    freq_y + dial_h * 0.14,
    (dial_h * 0.08).clamp(7.0, 10.0),
    "monospace",
    500.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(1.0, 0.70, 0.30, 0.75)),
    1.0,
    &Default::default(),
  );

  // Frequency tick marks
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.80, 0.30, 0.5)));
  c.set_line_width(1.0);
  for t_i in 0..16 {
    let tx = dial_x + 20.0 + t_i as f32 * ((dial_w - 40.0) / 15.0);
    c.stroke_line(tx, freq_y + 4.0, tx, freq_y + 12.0);
  }

  // -------------------------------------------------------------------------
  // 5. LED MATRIX SPECTRUM DISPLAY INSIDE TUNING WINDOW
  // -------------------------------------------------------------------------
  let pad_x = 10.0f32;
  let pad_y = 8.0f32;
  let matrix_y0 = dial_y + dial_h * 0.34;
  let matrix_h = dial_h * 0.60;
  let cell_w = (dial_w - pad_x * 2.0) / bar_count as f32;
  let cell_h = (matrix_h - pad_y * 2.0) / MATRIX_ROWS as f32;

  let step_f = (freq.len() / bar_count).max(1);

  for col in 0..bar_count {
    let k = (col * step_f).min(freq.len().saturating_sub(1));
    let raw_v = freq[k] as f32 / 255.0;
    let val = (raw_v * sensitivity).clamp(0.0, 1.4);
    let active_rows = (val * MATRIX_ROWS as f32) as usize;

    let cx_col = dial_x + pad_x + col as f32 * cell_w;

    for row in 0..MATRIX_ROWS {
      let is_active = row < active_rows;
      let row_ratio = row as f32 / MATRIX_ROWS as f32;

      let cy_row = matrix_y0 + matrix_h - pad_y - (row as f32 + 1.0) * cell_h;

      let (col_active, shadow_glow) = if row_ratio < 0.35 {
        (mix(Color::rgba(1.0, 0.15, 0.25, 0.95), p, 0.15), Color::rgba(1.0, 0.1, 0.2, 0.7))
      } else if row_ratio < 0.75 {
        (mix(Color::rgba(1.0, 0.55, 0.1, 0.95), s, 0.15), Color::rgba(1.0, 0.5, 0.1, 0.7))
      } else {
        (mix(Color::rgba(1.0, 0.85, 0.3, 0.98), accent, 0.15), Color::rgba(1.0, 0.8, 0.2, 0.85))
      };

      let px_w = (cell_w - 1.5).max(1.5);
      let px_h = (cell_h - 1.5).max(1.5);

      if is_active {
        c.set_fill(Fill::Solid(col_active));
        c.set_shadow(shadow_glow, 6.0 + bs * 4.0);
        c.fill_rect(cx_col + 0.8, cy_row + 0.8, px_w, px_h);
      } else {
        // Dim unlit LED grid pixel background
        c.set_fill(Fill::Solid(Color::rgba(0.14, 0.08, 0.06, 0.30)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_rect(cx_col + 0.8, cy_row + 0.8, px_w, px_h);
      }
    }
  }

  // Analog Orange Tuning Needle Pointer (Sways along tuning dial with music/time!)
  let needle_progress = ((frame_time * 0.04 + be * 0.10) % 1.0).clamp(0.05, 0.95);
  let needle_x = dial_x + 15.0 + needle_progress * (dial_w - 30.0);

  c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.30, 0.0, 0.95)));
  c.set_line_width(2.5);
  c.set_shadow(Color::rgba(1.0, 0.30, 0.0, 0.8), 8.0);
  c.stroke_line(needle_x, dial_y + 4.0, needle_x, dial_y + dial_h - 4.0);

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 6. LOWER CONTROL PANEL (ROTARY KNOBS + WOVEN SPEAKER GRILLES)
  // -------------------------------------------------------------------------
  let ctrl_y = dial_y + dial_h + radio_h * 0.06;
  let knob_r = (radio_h * 0.070).clamp(9.0, 18.0);

  let labels = ["VOLUME", "TUNING", "BASS", "TREBLE"];
  let knob_offsets = [-radio_w * 0.28, -radio_w * 0.09, radio_w * 0.09, radio_w * 0.28];

  for i in 0..4 {
    let kx = center_x + knob_offsets[i];
    let ky = ctrl_y + knob_r + 12.0;

    // Knob Label Text
    c.draw_text(
      labels[i],
      kx,
      ky - knob_r - 6.0,
      (knob_r * 0.70).clamp(7.0, 10.0),
      "sans-serif",
      700.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(0.9, 0.80, 0.65, 0.85)),
      1.0,
      &Default::default(),
    );

    // Rotary Metallic Knob Body
    let knob_grad = Fill::radial_gradient(
      kx - knob_r * 0.3,
      ky - knob_r * 0.3,
      0.0,
      kx,
      ky,
      knob_r,
      &[
        (0.0, Color::rgba(0.85, 0.88, 0.92, 0.98)),
        (0.6, Color::rgba(0.50, 0.52, 0.58, 0.98)),
        (1.0, Color::rgba(0.20, 0.22, 0.26, 0.98)),
      ],
    );
    c.set_fill(knob_grad);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 6.0);
    c.fill_circle(kx, ky, knob_r);

    c.set_stroke(Fill::Solid(Color::rgba(0.9, 0.9, 0.95, 0.8)));
    c.set_line_width(1.2);
    c.stroke_circle(kx, ky, knob_r);

    // Knob Indicator Notch (rotates slightly on bass)
    let knob_angle = -std::f32::consts::FRAC_PI_4 + (i as f32 * 0.35) + (be * 0.12);
    let notch_x = kx + knob_angle.cos() * (knob_r * 0.75);
    let notch_y = ky + knob_angle.sin() * (knob_r * 0.75);

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(1.8);
    c.stroke_line(kx, ky, notch_x, notch_y);
  }

  // Woven Fabric Speaker Vent Grilles (Left & Right Sides)
  let grille_w = radio_w * 0.10;
  let grille_h = radio_h * 0.12;
  let grille_left_x = left_x + radio_w * 0.04;
  let grille_right_x = left_x + radio_w * 0.86;
  let grille_y = ctrl_y + 6.0;

  for &gx in &[grille_left_x, grille_right_x] {
    c.set_fill(Fill::Solid(Color::rgba(0.12, 0.08, 0.05, 0.95)));
    c.fill_rounded_rect(gx, grille_y, grille_w, grille_h, 3.0);

    c.set_stroke(Fill::Solid(Color::rgba(0.70, 0.52, 0.30, 0.7)));
    c.set_line_width(1.2);
    c.stroke_rect(gx, grille_y, grille_w, grille_h);

    // Woven mesh horizontal slats
    c.set_stroke(Fill::Solid(Color::rgba(0.40, 0.28, 0.15, 0.8)));
    c.set_line_width(1.2);
    for s_idx in 1..4 {
      let sy = grille_y + (s_idx as f32 / 4.0) * grille_h;
      c.stroke_line(gx + 2.0, sy, gx + grille_w - 2.0, sy);
    }
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
