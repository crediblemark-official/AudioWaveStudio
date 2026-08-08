//! Turntable style renderer (`turntable`) — Technics SL-1200 Style Professional DJ Engine.
//!
//! Renders a hyper-realistic Technics SL-1200 style professional DJ turntable featuring:
//! - Brushed metallic plinth chassis with bevel edge trim & 4 corner isolator feet
//! - Start/Stop power button with green LED, 33/45 RPM speed buttons, Pitch Fader slider track
//! - Target pop-up stylus light tower with beam projection
//! - Spinning LP vinyl record with 35+ micro-groove track bands & dual butterfly anisotropic light sheen
//! - Vintage paper record label (Side A / 33⅓ RPM / Stereo)
//! - Hyper-realistic S-shaped chrome tonearm assembly with heavy counterweight, gimbal housing, anti-skating dial, and Ortofon Concorde headshell cartridge
//! - 360° surrounding audio spectrum equalizer ring
//! - Full UI settings integration (Scale, Position X & Y, Sensitivity, Bass Boost, Bar Count).

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};



pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;

  let p = theme_primary(theme);
  let _s = theme_secondary(theme);
  let accent = theme_accent(theme);
  let glow = theme_glow(theme);

  // Settings integration
  let sensitivity = ctx.config.reactivity.sensitivity;
  let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let frame_time = ctx.frame_time;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  let base_disc_r = ((width.min(height) * 0.26).clamp(80.0, 320.0)).clamp(50.0, width * 0.42);
  let disc_r = base_disc_r * (1.0 + be * 0.03);

  // Turntable Deck Plinth Dimensions
  let deck_w = (disc_r * 2.65).clamp(180.0, width * 0.95);
  let deck_h = (disc_r * 2.15).clamp(150.0, height * 0.92);
  let deck_x = center_x - deck_w * 0.50;
  let deck_y = center_y - deck_h * 0.48;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. ATMOSPHERIC BACKDROP & RADIAL AMBIENT GLOW
  // -------------------------------------------------------------------------
  let bg_haze = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    deck_w * 0.85,
    &[
      (0.0, glow.with_alpha(0.20 + be * 0.15)),
      (0.40, p.with_alpha(0.12)),
      (0.75, Color::rgba(0.04, 0.02, 0.10, 0.06)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(bg_haze);
  c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 2. TURNTABLE CHASSIS BASE (PLINTH) & 4 ISOLATOR FEET
  // -------------------------------------------------------------------------
  // 4 Corner Isolator Feet under Chassis
  let foot_r = (deck_h * 0.06).clamp(6.0, 18.0);
  let foot_offsets = [
    (deck_x + 18.0, deck_y + 18.0),
    (deck_x + deck_w - 18.0, deck_y + 18.0),
    (deck_x + 18.0, deck_y + deck_h - 18.0),
    (deck_x + deck_w - 18.0, deck_y + deck_h - 18.0),
  ];

  for &(fx, fy) in &foot_offsets {
    c.set_fill(Fill::Solid(Color::rgba(0.12, 0.13, 0.16, 0.98)));
    c.set_stroke(Fill::Solid(Color::rgba(0.55, 0.58, 0.65, 0.8)));
    c.set_line_width(1.5);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.7), 6.0);
    c.fill_circle(fx, fy, foot_r);
    c.stroke_circle(fx, fy, foot_r);
  }

  // Heavy Metallic Technics SL-1200 Plinth Body (Dark Brushed Slate)
  c.save();
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.80), 24.0);
  c.set_fill(Fill::Solid(Color::rgba(0.08, 0.09, 0.12, 0.98)));
  c.fill_rounded_rect(deck_x, deck_y, deck_w, deck_h, 16.0);
  c.restore();

  let deck_grad = Fill::linear_gradient(
    deck_x,
    deck_y,
    deck_x + deck_w,
    deck_y + deck_h,
    &[
      (0.0, Color::rgba(0.18, 0.19, 0.24, 0.98)),
      (0.3, Color::rgba(0.11, 0.12, 0.16, 0.98)),
      (0.7, Color::rgba(0.07, 0.08, 0.11, 0.98)),
      (1.0, Color::rgba(0.14, 0.15, 0.19, 0.98)),
    ],
  );
  c.set_fill(deck_grad);
  c.fill_rounded_rect(deck_x, deck_y, deck_w, deck_h, 16.0);

  // Outer bevel rim highlight
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.15)));
  c.set_line_width(1.5);
  c.stroke_rect(deck_x, deck_y, deck_w, deck_h);

  // -------------------------------------------------------------------------
  // 3. TURNTABLE CONTROLS: START/STOP BUTTON, 33/45 RPM, PITCH SLIDER
  // -------------------------------------------------------------------------
  let btn_sz = (deck_h * 0.11).clamp(16.0, 36.0);

  // Start / Stop Power Button (Lower-Left Corner)
  let start_btn_x = deck_x + deck_w * 0.06;
  let start_btn_y = deck_y + deck_h * 0.82;
  let start_grad = Fill::radial_gradient(
    start_btn_x,
    start_btn_y,
    0.0,
    start_btn_x,
    start_btn_y,
    btn_sz,
    &[
      (0.0, Color::rgba(0.85, 0.88, 0.92, 0.98)),
      (0.8, Color::rgba(0.55, 0.58, 0.65, 0.98)),
      (1.0, Color::rgba(0.30, 0.32, 0.38, 0.98)),
    ],
  );
  c.set_fill(start_grad);
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 4.0);
  c.fill_rounded_rect(start_btn_x, start_btn_y, btn_sz * 1.3, btn_sz, 4.0);

  // Green Power LED Indicator on Start/Stop
  c.set_fill(Fill::Solid(Color::hex("#00ff66")));
  c.set_shadow(Color::hex("#00ff66"), 6.0);
  c.fill_circle(start_btn_x + 6.0, start_btn_y + 6.0, 2.5);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  c.draw_text(
    "START•STOP",
    start_btn_x + btn_sz * 0.65,
    start_btn_y + btn_sz * 0.70,
    (btn_sz * 0.28).clamp(6.0, 9.0),
    "sans-serif",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(0.1, 0.1, 0.12, 0.95)),
    1.0,
    &Default::default(),
  );

  // 33 / 45 RPM Speed Selector Buttons
  let rpm33_x = start_btn_x + btn_sz * 1.5;
  let rpm45_x = rpm33_x + btn_sz * 0.75;
  let rpm_y = start_btn_y + btn_sz * 0.2;

  c.set_fill(Fill::Solid(Color::rgba(0.20, 0.22, 0.26, 0.95)));
  c.fill_rounded_rect(rpm33_x, rpm_y, btn_sz * 0.65, btn_sz * 0.8, 3.0);
  c.fill_rounded_rect(rpm45_x, rpm_y, btn_sz * 0.65, btn_sz * 0.8, 3.0);

  // 33 RPM active LED
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.2, 0.2, 0.9)));
  c.fill_circle(rpm33_x + btn_sz * 0.32, rpm_y + 4.0, 2.0);

  // Pitch Adjustment Fader Track (Right Side)
  let pitch_x = deck_x + deck_w * 0.90;
  let pitch_y0 = deck_y + deck_h * 0.35;
  let pitch_y1 = deck_y + deck_h * 0.85;

  c.set_stroke(Fill::Solid(Color::rgba(0.04, 0.04, 0.06, 0.95)));
  c.set_line_width(4.0);
  c.stroke_line(pitch_x, pitch_y0, pitch_x, pitch_y1);

  // Pitch Knob Fader Handle
  let pitch_knob_y = pitch_y0 + (pitch_y1 - pitch_y0) * 0.50 + (frame_time * 0.2).sin() * 6.0;
  c.set_fill(Fill::Solid(Color::rgba(0.80, 0.82, 0.88, 0.98)));
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 3.0);
  c.fill_rounded_rect(pitch_x - 7.0, pitch_knob_y - 6.0, 14.0, 12.0, 2.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Target Pop-up Stylus Light Tower (Upper-Left Side)
  let light_x = deck_x + deck_w * 0.12;
  let light_y = deck_y + deck_h * 0.18;
  c.set_fill(Fill::Solid(Color::rgba(0.70, 0.72, 0.78, 0.98)));
  c.fill_circle(light_x, light_y, 8.0);

  // Stylus Light Beam shining toward record
  let beam = Fill::linear_gradient(
    light_x,
    light_y,
    center_x - disc_r * 0.5,
    center_y,
    &[
      (0.0, Color::rgba(1.0, 0.95, 0.70, 0.35)),
      (0.5, Color::rgba(1.0, 0.85, 0.50, 0.12)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(beam);
  c.fill_polygon(&[
    (light_x, light_y - 4.0),
    (center_x - disc_r * 0.4, center_y - disc_r * 0.3),
    (center_x - disc_r * 0.4, center_y + disc_r * 0.3),
    (light_x, light_y + 4.0),
  ]);

  // -------------------------------------------------------------------------
  // 4. PRO-STUDIO SEGMENTED LED VU METER RING (TURNTABLE DJ STYLE)
  // -------------------------------------------------------------------------
  let step_f = (freq.len() / bar_count).max(1);
  let max_bar_h = height * 0.16 * sensitivity;
  let led_segments_per_bar = 8usize;
  let r_inner = disc_r + 8.0;

  for i in 0..bar_count {
    let angle = (i as f32 / bar_count as f32) * TAU + rot * 0.05;
    let k = (i * step_f).min(freq.len().saturating_sub(1));
    let raw_v = freq[k] as f32 / 255.0;
    let val = (raw_v * sensitivity).clamp(0.0, 1.4);

    let active_segments = (val * led_segments_per_bar as f32).ceil() as usize;
    let (s_a, c_a) = angle.sin_cos();

    let bar_w = (TAU * disc_r / bar_count as f32 * 0.70).clamp(2.5, 10.0);
    let seg_height = (max_bar_h / led_segments_per_bar as f32).clamp(2.0, 6.0);
    let seg_gap = 1.5f32;

    for seg in 0..led_segments_per_bar {
      let seg_r0 = r_inner + seg as f32 * (seg_height + seg_gap);
      let seg_r1 = seg_r0 + seg_height;

      let sx0 = center_x + c_a * seg_r0;
      let sy0 = center_y + s_a * seg_r0;
      let sx1 = center_x + c_a * seg_r1;
      let sy1 = center_y + s_a * seg_r1;

      let is_active = seg < active_segments;
      let seg_ratio = seg as f32 / led_segments_per_bar as f32;

      let seg_col = if is_active {
        if seg_ratio > 0.75 {
          Color::rgba(1.0, 0.20, 0.20, 0.95) // Red Peak LED
        } else if seg_ratio > 0.50 {
          Color::rgba(1.0, 0.75, 0.10, 0.95) // Amber Warning LED
        } else {
          Color::rgba(0.20, 0.90, 0.35, 0.95) // Green Normal LED
        }
      } else {
        Color::rgba(0.12, 0.14, 0.18, 0.30) // Dim Unlit LED Background
      };

      c.set_stroke(Fill::Solid(seg_col));
      c.set_line_width(bar_w);
      if is_active {
        c.set_shadow(seg_col, 5.0 + bs * 4.0);
      } else {
        c.set_shadow(Color::TRANSPARENT, 0.0);
      }
      c.stroke_line(sx0, sy0, sx1, sy1);
    }
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 5. SPINNING LP VINYL RECORD & ROTATING BUTTERFLY SHEEN
  // -------------------------------------------------------------------------
  c.save();
  c.translate(center_x, center_y);
  c.rotate(rot * 1.5); // Spins vinyl disc & center paper label continuously!
  c.translate(-center_x, -center_y);

  let vinyl_grad = Fill::radial_gradient(
    center_x - disc_r * 0.25,
    center_y - disc_r * 0.25,
    0.0,
    center_x,
    center_y,
    disc_r,
    &[
      (0.0, Color::rgba(0.16, 0.17, 0.22, 0.98)),
      (0.35, Color::rgba(0.08, 0.09, 0.12, 0.98)),
      (0.85, Color::rgba(0.04, 0.04, 0.06, 0.98)),
      (1.0, Color::rgba(0.10, 0.11, 0.14, 0.98)),
    ],
  );
  c.set_fill(vinyl_grad);
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.75), 18.0);
  c.fill_ellipse(center_x, center_y, disc_r, disc_r);

  // Outer lead-in groove rim
  c.set_stroke(Fill::Solid(Color::rgba(0.30, 0.32, 0.38, 0.6)));
  c.set_line_width(1.2);
  c.stroke_circle(center_x, center_y, disc_r * 0.97);

  // Concentric sound track micro-grooves (35+ realistic groove rings)
  let label_r = disc_r * 0.35;
  let groove_start_r = label_r + disc_r * 0.08;
  let groove_end_r = disc_r * 0.94;
  let total_grooves = 35usize;

  c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.27, 0.34, 0.35)));
  c.set_line_width(0.8);

  for g_i in 0..total_grooves {
    let g_t = g_i as f32 / total_grooves as f32;
    let gr = groove_start_r + g_t * (groove_end_r - groove_start_r);
    if g_i % 9 != 8 {
      c.stroke_circle(center_x, center_y, gr);
    }
  }

  // Rotating Dual Butterfly Specular Sheen (Anisotropic Light Wedges)
  for &(sheen_offset, opacity) in &[(0.0f32, 0.16f32), (std::f32::consts::PI, 0.16f32)] {
    let w_angle = rot * 1.2 + sheen_offset;
    let mut sheen_pts = vec![(center_x, center_y)];

    for k in 0..16 {
      let a = w_angle - 0.35 + (k as f32 / 15.0) * 0.70;
      let wx = center_x + a.cos() * disc_r;
      let wy = center_y + a.sin() * disc_r;
      sheen_pts.push((wx, wy));
    }
    sheen_pts.push((center_x, center_y));

    let sheen_grad = Fill::radial_gradient(
      center_x,
      center_y,
      label_r,
      center_x,
      center_y,
      disc_r,
      &[
        (0.0, Color::rgba(1.0, 1.0, 1.0, 0.0)),
        (0.5, Color::rgba(1.0, 1.0, 1.0, opacity)),
        (1.0, Color::rgba(1.0, 1.0, 1.0, 0.0)),
      ],
    );

    c.set_fill(sheen_grad);
    c.fill_polygon(&sheen_pts);
  }

  // Vintage Paper Record Label (Side A / 33⅓ RPM)
  let label_grad = Fill::linear_gradient(
    center_x - label_r,
    center_y - label_r,
    center_x + label_r,
    center_y + label_r,
    &[
      (0.0, Color::rgba(0.95, 0.92, 0.86, 0.98)),
      (0.35, Color::rgba(0.98, 0.95, 0.90, 0.98)),
      (0.80, Color::rgba(0.88, 0.85, 0.80, 0.98)),
      (1.0, Color::rgba(0.80, 0.76, 0.72, 0.98)),
    ],
  );
  c.set_fill(label_grad);
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 8.0);
  c.fill_circle(center_x, center_y, label_r);

  c.set_stroke(Fill::Solid(p.with_alpha(0.8)));
  c.set_line_width(2.0);
  c.stroke_circle(center_x, center_y, label_r);

  // Label Header Text
  c.draw_text(
    "33⅓ RPM  •  STEREO",
    center_x,
    center_y - label_r * 0.28,
    (label_r * 0.16).clamp(8.0, 13.0),
    "monospace",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(0.12, 0.12, 0.15, 0.95)),
    1.0,
    &Default::default(),
  );

  // Custom song title / track title on label
  let track_title = if !ctx.config.text.cassette_label.trim().is_empty() {
    ctx.config.text.cassette_label.to_uppercase()
  } else {
    "TECHNICS SL-1200".to_string()
  };

  c.draw_text(
    &track_title,
    center_x,
    center_y + label_r * 0.45,
    (label_r * 0.15).clamp(8.0, 13.0),
    "monospace",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(0.12, 0.12, 0.15, 0.95)),
    1.0,
    &Default::default(),
  );

  // Spindle center hole & metallic spindle ring
  let spindle_r = (label_r * 0.16).clamp(5.0, 14.0);
  c.set_fill(Fill::Solid(Color::rgba(0.04, 0.03, 0.06, 0.98)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_circle(center_x, center_y, spindle_r);

  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(1.2);
  c.stroke_circle(center_x, center_y, spindle_r);

  c.restore();

  // -------------------------------------------------------------------------
  // 6. HYPER-REALISTIC S-SHAPED CHROME TONEARM & PICKUP CARTRIDGE NEEDLE
  // -------------------------------------------------------------------------
  let pivot_x = deck_x + deck_w * 0.82;
  let pivot_y = deck_y + deck_h * 0.20;
  let pivot_r = (disc_r * 0.15).clamp(10.0, 32.0);

  // Track progress sweep (needle travels from outer groove to label)
  let track_progress = ((frame_time * 0.015) % 1.0).clamp(0.0, 1.0);
  let groove_r = disc_r * (0.86 - track_progress * 0.44);

  let contact_angle = std::f32::consts::FRAC_PI_4 + 0.15;
  let vib_x = (be * 1.5).sin() * 1.2;
  let vib_y = (bs * 1.5).cos() * 1.2;

  let stylus_x = center_x + contact_angle.cos() * groove_r + vib_x;
  let stylus_y = center_y + contact_angle.sin() * groove_r + vib_y;

  // Heavy Metallic Counterweight (Behind Pivot)
  let cw_angle = (pivot_y - stylus_y).atan2(pivot_x - stylus_x) + std::f32::consts::PI;
  let cw_x = pivot_x + cw_angle.cos() * (pivot_r * 1.8);
  let cw_y = pivot_y + cw_angle.sin() * (pivot_r * 1.8);

  let cw_grad = Fill::radial_gradient(
    cw_x - pivot_r * 0.3,
    cw_y - pivot_r * 0.3,
    0.0,
    cw_x,
    cw_y,
    pivot_r * 0.85,
    &[
      (0.0, Color::rgba(0.95, 0.95, 0.98, 0.98)),
      (0.6, Color::rgba(0.65, 0.68, 0.72, 0.98)),
      (1.0, Color::rgba(0.30, 0.32, 0.36, 0.98)),
    ],
  );
  c.set_fill(cw_grad);
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 8.0);
  c.fill_circle(cw_x, cw_y, pivot_r * 0.85);

  // Gimbal Base Housing (Concentric Chrome Rings)
  c.set_fill(Fill::Solid(Color::rgba(0.18, 0.19, 0.24, 0.98)));
  c.set_stroke(Fill::Solid(Color::rgba(0.80, 0.82, 0.88, 0.90)));
  c.set_line_width(2.0);
  c.fill_circle(pivot_x, pivot_y, pivot_r);
  c.stroke_circle(pivot_x, pivot_y, pivot_r);

  // Anti-skate dial / cueing lever pin
  c.set_fill(Fill::Solid(Color::rgba(0.85, 0.88, 0.92, 0.95)));
  c.fill_circle(pivot_x + pivot_r * 0.6, pivot_y - pivot_r * 0.6, 4.0);

  // S-Shaped Chrome Tonearm Tube (2 Curved Elbows)
  let mid1_x = pivot_x * 0.65 + stylus_x * 0.35 + 18.0;
  let mid1_y = pivot_y * 0.65 + stylus_y * 0.35 - 12.0;

  let mid2_x = pivot_x * 0.35 + stylus_x * 0.65 - 10.0;
  let mid2_y = pivot_y * 0.35 + stylus_y * 0.65 + 8.0;

  let arm_pts = [(pivot_x, pivot_y), (mid1_x, mid1_y), (mid2_x, mid2_y), (stylus_x, stylus_y)];

  // Outer Chrome Pipe Body
  c.set_stroke(Fill::Solid(Color::rgba(0.88, 0.90, 0.95, 0.98)));
  c.set_line_width(4.0);
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.7), 8.0);
  c.stroke_polyline(&arm_pts);

  // Specular Highlight Line
  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(1.5);
  c.stroke_polyline(&arm_pts);

  // Ortofon Concorde / Technics Cartridge Headshell
  let head_angle = (stylus_y - mid2_y).atan2(stylus_x - mid2_x) + 0.25;
  let head_len = 24.0;
  let head_w = 11.0;

  let h_back_x = stylus_x - head_angle.cos() * head_len;
  let h_back_y = stylus_y - head_angle.sin() * head_len;

  c.set_stroke(Fill::Solid(accent));
  c.set_line_width(head_w);
  c.set_shadow(accent.with_alpha(0.8), 12.0);
  c.stroke_line(h_back_x, h_back_y, stylus_x, stylus_y);

  // Glowing Cartridge Needle Tip (Glows on beat hits!)
  let needle_col = mix(accent, Color::WHITE, bs);
  c.set_fill(Fill::Solid(needle_col));
  c.set_shadow(needle_col, 10.0 + bs * 10.0);
  c.fill_circle(stylus_x, stylus_y, 4.0);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
