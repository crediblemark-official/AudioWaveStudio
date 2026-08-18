//! Vinyl Record style renderer (`vinylRecord`) — Hyper-Realistic LP Vinyl Engine.
//!
//! Renders a hyper-realistic 33⅓ RPM LP vinyl record complete with micro-groove track bands,
//! rotating dual butterfly anisotropic light sheen, authentic paper record label (Side A / 33⅓ RPM),
//! metallic tonearm & pickup cartridge needle, surrounding 360° audio spectrum ring,
//! atmospheric nebula glow, and full UI settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
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
  let sensitivity = ctx.config.reactivity.sensitivity;
  let user_scale = ctx.config.scale.clamp(0.1, 5.0);
  let pos_offset_x = ctx.config.position_x * width * 0.5;
  let pos_offset_y = -ctx.config.position_y * height * 0.5;
  let bar_count = ctx.config.reactivity.bar_count.clamp(8, 128);

  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5 + pos_offset_x;
  let center_y = height * 0.5 + pos_offset_y;

  let base_disc_r = width.min(height) * 0.30 * user_scale;
  let disc_r = base_disc_r * (1.0 + be * 0.03);

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
    disc_r * 1.6,
    &[
      (0.0, glow.with_alpha(0.20 + be * 0.15)),
      (0.40, p.with_alpha(0.12)),
      (0.75, Color::rgba(0.04, 0.02, 0.10, 0.06)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(bg_haze);
//   c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 2. HYPNOTIC COSMIC PLASMA WAVE RIBBON & ORBITING AUDIO DUST (VINYL STYLE)
  // -------------------------------------------------------------------------
  let step_f = (freq.len() / bar_count).max(1);
  let wave_segs = 120usize;
  let r_base = disc_r + 12.0;

  // A. Organic Smooth Audio Plasma Wave Ribbon surrounding Vinyl
  let mut outer_pts: Vec<(f32, f32)> = Vec::with_capacity(wave_segs);
  let mut inner_pts: Vec<(f32, f32)> = Vec::with_capacity(wave_segs);

  for i in 0..wave_segs {
    let a = (i as f32 / wave_segs as f32) * TAU + rot * 0.2;
    let k = (i * step_f / (wave_segs / bar_count.max(1)).max(1)).min(freq.len().saturating_sub(1));
    let fv = freq[k] as f32 / 255.0;

    let h = (fv * sensitivity * (height * 0.18) + be * 12.0).clamp(2.0, height * 0.30);
    let (s_a, c_a) = a.sin_cos();

    let r_in = r_base;
    let r_out = r_base + h;

    inner_pts.push((center_x + c_a * r_in, center_y + s_a * r_in));
    outer_pts.push((center_x + c_a * r_out, center_y + s_a * r_out));

    // Glowing Energy Motes at peak audio points (replaces straight bristle lines)
    if fv > 0.45 {
      let bx0 = center_x + c_a * r_out;
      let by0 = center_y + s_a * r_out;
      let mote_r = 2.2 + fv * 2.5;
      c.set_fill(Fill::Solid(mix(glow, Color::WHITE, fv * 0.8)));
      c.set_shadow(glow, 10.0 + fv * 8.0);
      c.fill_circle(bx0, by0, mote_r);
    }
  }

  // Draw Continuous Plasma Wave Ribbon Fill
  let mut ribbon_polygon = inner_pts.clone();
  let mut outer_rev = outer_pts.clone();
  outer_rev.reverse();
  ribbon_polygon.extend(outer_rev);

  let ribbon_grad = Fill::radial_gradient(
    center_x,
    center_y,
    r_base,
    center_x,
    center_y,
    r_base + height * 0.20,
    &[
      (0.0, accent.with_alpha(0.70 + be * 0.20)),
      (0.5, p.with_alpha(0.50)),
      (0.85, s.with_alpha(0.25)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(ribbon_grad);
  c.fill_polygon(&ribbon_polygon);

  // Outer Wave Crest Outline Ring
  c.set_stroke(Fill::Solid(mix(glow, Color::WHITE, bs).with_alpha(0.90)));
  c.set_line_width(2.5);
  c.set_shadow(glow, 10.0 + bs * 12.0);
  c.stroke_polyline(&outer_pts);

  // B. Orbiting Floating Audio Dust Particles
  let particle_count = 42usize;
  for p_i in 0..particle_count {
    let p_t = p_i as f32 / particle_count as f32;
    let p_speed = 0.3 + (p_i % 5) as f32 * 0.15;
    let p_angle = p_t * TAU + ctx.frame_time * p_speed;
    let p_dist = r_base + (p_i as f32 * 17.0).sin().abs() * (height * 0.22) + be * 15.0;

    let px = center_x + p_angle.cos() * p_dist;
    let py = center_y + p_angle.sin() * p_dist;

    let p_sz = (2.5 + (p_i % 3) as f32 * 1.5 + bs * 2.0).clamp(1.5, 6.5);
    let p_col = mix(p, glow, (p_i as f32 * 0.2).sin().abs()).with_alpha(0.60 + (p_i as f32 * 0.5).cos().abs() * 0.35);

    c.set_fill(Fill::Solid(p_col));
    c.set_shadow(p_col, 6.0);
    c.fill_ellipse(px, py, p_sz, p_sz);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 3. TURNTABLE PLATTER & STROBOSCOPIC RIM UNDER VINYL
  // -------------------------------------------------------------------------
  let platter_r = disc_r + 5.0;
  let platter_grad = Fill::radial_gradient(
    center_x - platter_r * 0.2,
    center_y - platter_r * 0.2,
    0.0,
    center_x,
    center_y,
    platter_r,
    &[
      (0.0, Color::rgba(0.25, 0.26, 0.30, 0.98)),
      (0.85, Color::rgba(0.12, 0.13, 0.16, 0.98)),
      (1.0, Color::rgba(0.06, 0.07, 0.09, 0.98)),
    ],
  );
  c.set_fill(platter_grad);
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.7), 22.0);
  c.fill_ellipse(center_x, center_y, platter_r, platter_r);

  // Metallic Platter Bevel Rim Highlight
  c.set_stroke(Fill::Solid(Color::rgba(0.8, 0.85, 0.9, 0.4)));
  c.set_line_width(1.5);
  c.stroke_circle(center_x, center_y, platter_r);

  // Rotating Silver Stroboscopic Dots on Platter Rim
  let strobe_dots = 48usize;
  for d_i in 0..strobe_dots {
    let da = (d_i as f32 / strobe_dots as f32) * TAU + rot * 1.5;
    let dx = center_x + da.cos() * (platter_r - 2.5);
    let dy = center_y + da.sin() * (platter_r - 2.5);
    let dot_col = if d_i % 2 == 0 { Color::rgba(0.92, 0.95, 1.0, 0.90) } else { Color::rgba(0.40, 0.45, 0.55, 0.60) };
    c.set_fill(Fill::Solid(dot_col));
    c.fill_circle(dx, dy, 1.8);
  }

  // -------------------------------------------------------------------------
  // 4, 5, 6. SPINNING POLISHED BLACK VINYL DISC, GROOVES & PAPER LABEL
  // -------------------------------------------------------------------------
  c.save();
  c.translate(center_x, center_y);
  c.rotate(rot * 1.5); // Spins vinyl disc & center paper label continuously!
  c.translate(-center_x, -center_y);

  // Vinyl Base Disc
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
  c.fill_ellipse(center_x, center_y, disc_r, disc_r);

  // Outer lead-in groove rim
  c.set_stroke(Fill::Solid(Color::rgba(0.30, 0.32, 0.38, 0.6)));
  c.set_line_width(1.2);
  c.stroke_circle(center_x, center_y, disc_r * 0.97);

  // Concentric sound track micro-grooves
  let label_r = disc_r * 0.35;
  let groove_start_r = label_r + disc_r * 0.08;
  let groove_end_r = disc_r * 0.94;
  let total_grooves = 38usize;

  c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.27, 0.34, 0.35)));
  c.set_line_width(0.8);

  for g_i in 0..total_grooves {
    let g_t = g_i as f32 / total_grooves as f32;
    let gr = groove_start_r + g_t * (groove_end_r - groove_start_r);
    if g_i % 9 != 8 {
      c.stroke_circle(center_x, center_y, gr);
    }
  }

  // Dead-wax ungrooved inner run-out ring
  c.set_stroke(Fill::Solid(Color::rgba(0.18, 0.20, 0.25, 0.5)));
  c.set_line_width(1.0);
  c.stroke_circle(center_x, center_y, label_r + disc_r * 0.03);

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

  // Vintage Paper Record Label
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

  // Outer paper label border ring
  c.set_stroke(Fill::Solid(p.with_alpha(0.8)));
  c.set_line_width(2.0);
  c.stroke_circle(center_x, center_y, label_r);

  // Color racing band on label
  let stripe_h = label_r * 0.35;
  c.set_fill(Fill::Solid(s.with_alpha(0.85)));
  c.fill_rect(center_x - label_r * 0.90, center_y - label_r * 0.55, label_r * 1.80, stripe_h);

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
    Fill::Solid(Color::WHITE),
    1.0,
    &Default::default(),
  );

  c.draw_text(
    "SIDE A",
    center_x - label_r * 0.55,
    center_y + label_r * 0.15,
    (label_r * 0.16).clamp(8.0, 12.0),
    "sans-serif",
    800.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(0.15, 0.15, 0.18, 0.90)),
    1.0,
    &Default::default(),
  );

  if !ctx.config.text.cassette_label.trim().is_empty() {
    let track_title = ctx.config.text.cassette_label.to_uppercase();
    c.draw_text(
      &track_title,
      center_x,
      center_y + label_r * 0.55,
      (label_r * 0.15).clamp(8.0, 13.0),
      "monospace",
      700.0,
      false,
      TextAlign::Center,
      Fill::Solid(Color::rgba(0.12, 0.12, 0.15, 0.95)),
      1.0,
      &Default::default(),
    );
  }

  // Spindle center hole & metallic spindle ring
  let spindle_r = (label_r * 0.16).clamp(5.0, 14.0);
  let spindle_ring_r = spindle_r * 1.8;

  c.set_stroke(Fill::Solid(Color::rgba(0.70, 0.72, 0.78, 0.90)));
  c.set_line_width(1.5);
  c.stroke_circle(center_x, center_y, spindle_ring_r);

  c.set_fill(Fill::Solid(Color::rgba(0.04, 0.03, 0.06, 0.98)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_circle(center_x, center_y, spindle_r);

  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(1.2);
  c.stroke_circle(center_x, center_y, spindle_r);

  // Center image on vinyl label (spins with the record)
  draw_radial_center_image(c, ctx, center_x, center_y, label_r * 0.80);

  c.restore();

  // -------------------------------------------------------------------------
  // 7. SLEEK METALLIC TONEARM & PICKUP CARTRIDGE NEEDLE (DYNAMIC WOBBLING)
  // -------------------------------------------------------------------------
  let arm_base_x = center_x + disc_r * 1.15;
  let arm_base_y = center_y - disc_r * 1.10;

  // Arm pivot base disc
  c.set_fill(Fill::Solid(Color::rgba(0.20, 0.22, 0.26, 0.98)));
  c.set_stroke(Fill::Solid(Color::rgba(0.65, 0.68, 0.75, 0.9)));
  c.set_line_width(1.8);
  c.fill_circle(arm_base_x, arm_base_y, 16.0);
  c.stroke_circle(arm_base_x, arm_base_y, 16.0);

  // Audio bass reactivity & mechanical playback wobble
  let wobble_speed = rot * 3.5;
  let wobble_dx = (wobble_speed.sin() * 2.5 + (be * 6.0 + bs * 4.0) * sensitivity).clamp(-8.0, 8.0);
  let wobble_dy = ((wobble_speed * 1.4).cos() * 2.0 + (be * 4.0 - bs * 3.0) * sensitivity).clamp(-6.0, 6.0);

  // Smooth tracking drift across vinyl micro-grooves
  let track_drift = (rot * 0.015) % 0.35; // Slowly drifts from outer groove inward
  let needle_r = disc_r * (0.75 - track_drift);
  let needle_angle = -std::f32::consts::FRAC_PI_4 - 0.22; // ~ -65° top-right sector

  let needle_target_x = center_x + needle_angle.cos() * needle_r + wobble_dx;
  let needle_target_y = center_y + needle_angle.sin() * needle_r + wobble_dy;

  // S-curved metallic tonearm tube
  let mid_arm_x = arm_base_x - (arm_base_x - needle_target_x) * 0.48 + wobble_dx * 0.4;
  let mid_arm_y = arm_base_y + (needle_target_y - arm_base_y) * 0.52 + wobble_dy * 0.4;

  // Arm shadow
  c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, 0.5)));
  c.set_line_width(4.5);
  c.stroke_polyline(&[
    (arm_base_x + 3.0, arm_base_y + 4.0),
    (mid_arm_x + 3.0, mid_arm_y + 4.0),
    (needle_target_x + 3.0, needle_target_y + 4.0),
  ]);

  // Metallic silver tube
  c.set_stroke(Fill::Solid(Color::rgba(0.85, 0.88, 0.94, 0.95)));
  c.set_line_width(3.5);
  c.stroke_polyline(&[
    (arm_base_x, arm_base_y),
    (mid_arm_x, mid_arm_y),
    (needle_target_x, needle_target_y),
  ]);

  // Pickup Cartridge Headshell
  c.set_fill(Fill::Solid(Color::rgba(0.12, 0.13, 0.16, 0.98)));
  c.fill_rounded_rect(
    needle_target_x - 8.0,
    needle_target_y - 6.0,
    16.0,
    12.0,
    3.0,
  );

  // Glowing Cartridge Needle Tip (Glows & pulses on beat hits!)
  let needle_col = mix(accent, Color::WHITE, bs);
  c.set_fill(Fill::Solid(needle_col));
  c.set_shadow(needle_col, 8.0 + bs * 10.0);
  c.fill_circle(needle_target_x, needle_target_y, 3.0 + bs * 1.5);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
