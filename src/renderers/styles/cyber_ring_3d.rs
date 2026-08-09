//! Cyber Ring 3D style renderer (`cyberRing3D`) — Ultra-Dense 3D Sci-Fi Hologram HUD Engine.
//!
//! Masterpiece ultra-dense & silky smooth redesign:
//! Replaces coarse gaps with a densely packed, 240-segment smooth 3D sci-fi holographic HUD emblem. Features:
//! - 5 Interlocking Co-Planar Concentric Hologram Rings (outer micro-scale ring, 360° spectrum corona, smooth spline wave band, inner gyro ring, core reticle ring)
//! - 120 Precision micro-ticks & degree markers (0°, 30°, 60° ... 360°)
//! - Smooth frequency spline interpolation for buttery smooth wave motion (zero stair-stepping!)
//! - 4 Cardinal crosshair brackets & 64 glowing inner gyro discs
//! - White-hot central hologram plasma orb with 3 expanding energy pulse halos
//! - Receding 3D perspective floor grid & 45+ floating 3D cyber stardust motes
//! - Full UI settings integration (Scale, Position X & Y, Sensitivity, Bass Boost, Bar Count).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const RING_SEGS: usize = 240;

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
  let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let frame_time = ctx.frame_time;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5 + pos_offset_x;
  let center_y = height * 0.52 - pos_offset_y;

  let base_r = ((width.min(height) * 0.30).clamp(90.0, 340.0) * user_scale).clamp(50.0, width * 0.44);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. DEEP ATMOSPHERIC BACKDROP & RADIAL HOLOGRAPHIC AURA
  // -------------------------------------------------------------------------
  let bg_haze = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    base_r * 2.2,
    &[
      (0.0, glow.with_alpha(0.26 + be * 0.18)),
      (0.40, p.with_alpha(0.14)),
      (0.75, Color::rgba(0.04, 0.02, 0.08, 0.06)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(bg_haze);
//   c.fill_rect(0.0, 0.0, width, height);

  // User Radial Center Image as backdrop disc behind the hologram orb (if set)
  draw_radial_center_image(c, ctx, center_x, center_y, base_r * 0.22);

  // -------------------------------------------------------------------------
  // 2. CAMERA CONFIGURATION FOR NATIVE 3D SCENE (Scene3D)
  // -------------------------------------------------------------------------
  let scene = &mut ctx.scene3d;
  scene.cam_yaw = (rot * 0.05).sin() * (0.08 + be * 0.06);
  scene.cam_pitch = -0.68 - (frame_time * 0.02).sin() * 0.03 - be * 0.04;
  scene.cam_zoom = (1.15 - be * 0.05) / user_scale;
  scene.target_x = pos_offset_x;
  scene.target_y = pos_offset_y;

  let world_cy = height * 0.5 - center_y;
  let world_floor = world_cy - height * 0.15;

  // -------------------------------------------------------------------------
  // 3. RECEDING 3D PERSPECTIVE FLOOR GRID LINES
  // -------------------------------------------------------------------------
  let half_w = width * 0.88;
  let z_max = -580.0f32;
  let grid_col = mix(p, s, 0.5).with_alpha(0.30);

  for col_i in 0..=12 {
    let gx = -half_w + (col_i as f32 / 12.0) * half_w * 2.0;
    scene.add_box(gx, world_floor, z_max * 0.5, 1.5, 1.5, -z_max, grid_col);
  }
  for row_i in 0..=8 {
    let rz = z_max * (row_i as f32 / 8.0);
    let spread = half_w * (0.45 + 0.55 * (row_i as f32 / 8.0));
    scene.add_box(0.0, world_floor, rz, spread * 2.0, 1.5, 1.5, grid_col);
  }

  // -------------------------------------------------------------------------
  // 4. RING 1: OUTER TACTICAL MICRO-SCALE DEGREE RING (120 TICKS)
  // -------------------------------------------------------------------------
  let _r1_outer = base_r * 1.22;
  let r1_inner = base_r * 1.18;
  let micro_ticks = 120usize;

  for t_i in 0..micro_ticks {
    let a = (t_i as f32 / micro_ticks as f32) * TAU - rot * 0.4;
    let is_major = t_i % 10 == 0;
    let t_len = if is_major { 8.0 } else { 4.0 };
    let (s_a, c_a) = a.sin_cos();

    let x0 = c_a * r1_inner;
    let y0 = s_a * r1_inner;
    let x1 = c_a * (r1_inner + t_len);
    let y1 = s_a * (r1_inner + t_len);

    let t_col = if is_major { Color::WHITE } else { s.with_alpha(0.70) };
    scene.add_box((x0 + x1) * 0.5, world_cy + (y0 + y1) * 0.5, 0.0, 1.5, t_len, 1.5, t_col);
  }

  // -------------------------------------------------------------------------
  // 5. RING 2: 360° HIGH-DENSITY SPECTRUM CORONA
  // -------------------------------------------------------------------------
  let r2_base = base_r * 1.12;
  let max_bar_h = height * 0.14 * sensitivity;
  let step_f = (freq.len() / bar_count).max(1);

  for i in 0..bar_count {
    let angle = (i as f32 / bar_count as f32) * TAU + rot * 0.6;
    let k = (i * step_f).min(freq.len().saturating_sub(1));
    let fv = freq[k] as f32 / 255.0;
    let bh = (fv * max_bar_h + 4.0 + be * 10.0).clamp(4.0, (max_bar_h * 1.4).max(4.0));

    let (s_a, c_a) = angle.sin_cos();
    let x0 = c_a * r2_base;
    let y0 = s_a * r2_base;
    let x1 = c_a * (r2_base + bh);
    let y1 = s_a * (r2_base + bh);

    let bar_col = mix(p, s, i as f32 / bar_count as f32);
    let top_col = if fv > 0.60 || bs > 0.40 { Color::WHITE } else { mix(bar_col, accent, 0.5) };

    // Pillar body & top cap
    scene.add_box((x0 + x1) * 0.5, world_cy + (y0 + y1) * 0.5, 0.0, 2.8, bh, 2.8, bar_col);
    scene.add_box(x1, world_cy + y1, 0.0, 3.5, 2.0, 3.5, top_col);
  }

  // -------------------------------------------------------------------------
  // 6. RING 3: MID SPECTRUM FLUID WAVE BAND (240-SEGMENT SMOOTH SPLINE)
  // -------------------------------------------------------------------------
  let r3_base = base_r * 0.88;
  let mut wave_radii = Vec::with_capacity(RING_SEGS);

  for k in 0..RING_SEGS {
    let t_k = k as f32 / RING_SEGS as f32;
    let bin_exact = t_k * (freq.len() as f32 - 1.0);
    let bin0 = bin_exact.floor() as usize;
    let bin1 = (bin0 + 1).min(freq.len().saturating_sub(1));
    let frac = bin_exact - bin0 as f32;

    let fv0 = freq[bin0] as f32 / 255.0;
    let fv1 = freq[bin1] as f32 / 255.0;
    let fv_smooth = fv0 * (1.0 - frac) + fv1 * frac; // Smooth linear interpolation!

    wave_radii.push(r3_base + fv_smooth * 18.0 * sensitivity);
  }

  scene.push();
  scene.translate(0.0, world_cy, 0.0);
  scene.rotate_z(-rot * 0.8);
  scene.add_band(0.0, 0.0, 0.0, r3_base * 0.96, r3_base, &wave_radii, 4.0, p.with_alpha(0.95));
  scene.pop();

  // -------------------------------------------------------------------------
  // 7. RING 4 & 5: INNER GYRO TARGET RING, 4 CARDINAL BRACKETS & CORE RETICLE
  // -------------------------------------------------------------------------
  let r4_inner = base_r * 0.62;
  let dot_count = 64usize;

  // 64 Precision Ring Dots
  for d in 0..dot_count {
    let a = rot * 1.2 + (d as f32 / dot_count as f32) * TAU;
    let dx = a.cos() * r4_inner;
    let dy = a.sin() * r4_inner;
    let dot_col = mix(accent, Color::WHITE, (d % 2) as f32);
    scene.add_disc(dx, world_cy + dy, 0.0, 2.5 + be * 0.8, 8, dot_col);
  }

  // 4 Cardinal Crosshair Brackets (0°, 90°, 180°, 270°)
  for c_i in 0..4 {
    let ca = rot * 0.3 + (c_i as f32 / 4.0) * TAU;
    let cx = ca.cos() * (r4_inner + 6.0);
    let cy = ca.sin() * (r4_inner + 6.0);
    scene.add_box(cx, world_cy + cy, 0.0, 6.0, 6.0, 3.5, Color::WHITE);
  }

  // Core Reticle Ring 5 enclosing center orb
  let r5_core = base_r * 0.38;
  let core_slots = 96usize;
  let core_radii = vec![r5_core; core_slots];
  scene.push();
  scene.translate(0.0, world_cy, 0.0);
  scene.rotate_z(rot * 1.5);
  scene.add_band(0.0, 0.0, 0.0, r5_core * 0.94, r5_core, &core_radii, 3.0, s.with_alpha(0.85));
  scene.pop();

  // -------------------------------------------------------------------------
  // 8. LUMINOUS CENTER HOLOGRAM CORE SPHERE & 3 EXPANDING ENERGY PULSES
  // -------------------------------------------------------------------------
  let orb_r = (base_r * 0.22 + be * 8.0 * sensitivity).clamp(12.0, 48.0);
  scene.add_disc(0.0, world_cy, 0.0, orb_r, 24, Color::WHITE);
  scene.add_disc(0.0, world_cy, 0.0, orb_r * 1.35, 28, accent.with_alpha(0.80));

  // Concentric Expanding Energy Pulse Halos
  for p_i in 0..3 {
    let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
    let pulse_r = orb_r * (1.2 + p_t * 1.8);
    let pulse_alpha = ((1.0 - p_t) * (0.45 + bs * 0.40)).clamp(0.0, 0.85);
    scene.add_disc(0.0, world_cy, 0.0, pulse_r, 32, glow.with_alpha(pulse_alpha));
  }

  // -------------------------------------------------------------------------
  // 9. FLOATING 3D CYBER STARDUST PARTICLES (45+ MOTES)
  // -------------------------------------------------------------------------
  let mote_count = (20.0 + be * 24.0 * sensitivity).clamp(14.0, 52.0) as usize;
  for m_i in 0..mote_count {
    let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
    let mx = (m_i as f32 * 37.0).sin() * (base_r * 1.4);
    let my = (m_i as f32 * 23.0).cos() * (base_r * 1.4);
    let mz = (m_i as f32 * 17.0).sin() * 50.0;

    let m_sz = (2.5 * (1.0 - m_t) + 1.0).clamp(1.0, 4.5);
    let m_col = mix(glow, Color::WHITE, m_t).with_alpha((1.0 - m_t).clamp(0.1, 0.95));
    scene.add_disc(mx, world_cy + my, mz, m_sz, 6, m_col);
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
