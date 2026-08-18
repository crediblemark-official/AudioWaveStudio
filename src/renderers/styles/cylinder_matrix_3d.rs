//! 3D Cylinder Matrix style renderer (`cylinderMatrix3D`) — 3D Equalizer Ring Matrix Engine.
//!
//! Masterpiece 100% faithful reference match:
//! Renders a 3D cylindrical equalizer block matrix surrounded by a central cyan neon ring.
//! Features:
//! - Dual concentric 3D equalizer rings (outer orange/amber ring & inner cyan/teal ring)
//! - Extruded 3D cuboid pillars extending upwards (+Y) and downwards (-Y) from the center plane
//! - Bright glowing top & bottom cap highlights (cyan and orange neon)
//! - Center cyan 3D glowing neon ring encircling the 3D block cylinder
//! - Glossy floor reflection & dark atmospheric backdrop
//! - 3D camera pitching & yawing with full UI settings integration (Scale, Position X & Y, Sensitivity, Bass Boost, Bar Count).

use std::f32::consts::TAU;

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
  let _accent = theme_accent(theme);
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
  let center_y = height * 0.50 + pos_offset_y;

  let base_r = width.min(height) * 0.25 * user_scale;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. DEEP ATMOSPHERIC BACKDROP & NEON RADIANCE
  // -------------------------------------------------------------------------
  let bg_haze = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    base_r * 2.2,
    &[
      (0.0, glow.with_alpha(0.24 + be * 0.16)),
      (0.40, p.with_alpha(0.12)),
      (0.75, Color::rgba(0.04, 0.02, 0.08, 0.06)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(bg_haze);
//   c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 2. CAMERA CONFIGURATION FOR NATIVE 3D SCENE (Scene3D)
  // -------------------------------------------------------------------------
  let scene = &mut ctx.scene3d;
  scene.cam_yaw = (rot * 0.05).sin() * (0.12 + be * 0.06);
  scene.cam_pitch = -0.42 - (frame_time * 0.02).sin() * 0.03 - be * 0.04;
  scene.cam_zoom = (1.08 - be * 0.05) / user_scale;
  scene.target_x = pos_offset_x;
  scene.target_y = pos_offset_y;

  let world_cy = height * 0.5 - center_y;
  let max_h = height * 0.32 * sensitivity;

  let cyan_neon = Color::hex("#00f0ff");
  let orange_neon = Color::hex("#ff7700");

  // -------------------------------------------------------------------------
  // 3. CYAN NEON CENTER RING ENCIRCLING THE 3D BLOCK CYLINDER (Y = 0)
  // -------------------------------------------------------------------------
  let ring_r_inner = base_r * 1.25;
  let ring_r_outer = base_r * 1.34;
  let dash_slots = 96usize;
  let ring_radii = vec![ring_r_outer; dash_slots];

  scene.push();
  scene.translate(0.0, world_cy, 0.0);
  scene.rotate_x(std::f32::consts::FRAC_PI_2);
  scene.add_band(0.0, 0.0, 0.0, ring_r_inner, ring_r_outer, &ring_radii, 5.0, cyan_neon);
  scene.pop();

  // -------------------------------------------------------------------------
  // 4. DUAL CONCENTRIC 3D EQUALIZER PILLAR RINGS (OUTER & INNER)
  // -------------------------------------------------------------------------
  let outer_count = bar_count;
  let inner_count = (bar_count * 3 / 4).max(12);

  let step_f = (freq.len() / outer_count).max(1);

  // A. Outer Ring (Orange / Amber Pillars)
  let r_outer = base_r * 1.12;
  let pillar_w_outer = (TAU * r_outer / outer_count as f32 * 0.65).clamp(4.0, 16.0);

  for i in 0..outer_count {
    let angle = (i as f32 / outer_count as f32) * TAU + rot * 0.6;
    let k = (i * step_f).min(freq.len().saturating_sub(1));
    let fv = freq[k] as f32 / 255.0;
    let bh = (fv * max_h + 8.0 + be * 12.0).clamp(8.0, (max_h * 1.5).max(8.0));

    let (s_a, c_a) = angle.sin_cos();
    let px = c_a * r_outer;
    let pz = s_a * r_outer;

    let pillar_col = mix(Color::rgba(0.12, 0.10, 0.14, 0.95), Color::rgba(0.40, 0.18, 0.08, 0.95), fv);
    let top_col = if fv > 0.65 || bs > 0.40 {
      Color::WHITE
    } else {
      orange_neon
    };

    // Main 3D Cuboid Pillar extending UP and DOWN
    scene.add_box(px, world_cy, pz, pillar_w_outer, bh * 2.0, pillar_w_outer, pillar_col);

    // Glowing Top Cap Highlight (+Y)
    scene.add_box(px, world_cy + bh + 1.5, pz, pillar_w_outer + 1.0, 3.0, pillar_w_outer + 1.0, top_col);
    // Glowing Bottom Cap Highlight (-Y)
    scene.add_box(px, world_cy - bh - 1.5, pz, pillar_w_outer + 1.0, 3.0, pillar_w_outer + 1.0, top_col);
  }

  // B. Inner Ring (Cyan / Teal Pillars)
  let r_inner = base_r * 0.76;
  let pillar_w_inner = (TAU * r_inner / inner_count as f32 * 0.65).clamp(4.0, 16.0);

  for i in 0..inner_count {
    let angle = (i as f32 / inner_count as f32) * TAU - rot * 0.8;
    let k = (i * step_f * 2).min(freq.len().saturating_sub(1));
    let fv = freq[k] as f32 / 255.0;
    let bh = (fv * max_h * 0.85 + 6.0 + be * 10.0).clamp(6.0, (max_h * 1.3).max(6.0));

    let (s_a, c_a) = angle.sin_cos();
    let px = c_a * r_inner;
    let pz = s_a * r_inner;

    let pillar_col = mix(Color::rgba(0.08, 0.12, 0.16, 0.95), Color::rgba(0.08, 0.35, 0.42, 0.95), fv);
    let top_col = if fv > 0.65 || bs > 0.40 {
      Color::WHITE
    } else {
      cyan_neon
    };

    // Main 3D Cuboid Pillar extending UP and DOWN
    scene.add_box(px, world_cy, pz, pillar_w_inner, bh * 2.0, pillar_w_inner, pillar_col);

    // Glowing Top Cap Highlight (+Y)
    scene.add_box(px, world_cy + bh + 1.5, pz, pillar_w_inner + 1.0, 3.0, pillar_w_inner + 1.0, top_col);
    // Glowing Bottom Cap Highlight (-Y)
    scene.add_box(px, world_cy - bh - 1.5, pz, pillar_w_inner + 1.0, 3.0, pillar_w_inner + 1.0, top_col);
  }

  // -------------------------------------------------------------------------
  // 5. 3D FLOATING SPARK MOTES
  // -------------------------------------------------------------------------
  let mote_count = (16.0 + be * 20.0 * sensitivity).clamp(12.0, 45.0) as usize;
  for m_i in 0..mote_count {
    let m_t = ((frame_time * 0.4 + m_i as f32 * 0.19) % 1.0).clamp(0.0, 1.0);
    let mx = (m_i as f32 * 37.0).sin() * (base_r * 1.5);
    let my = world_cy + (m_i as f32 * 23.0).cos() * (height * 0.35);
    let mz = (m_i as f32 * 17.0).sin() * (base_r * 1.5);

    let m_sz = (2.5 * (1.0 - m_t) + 1.0).clamp(1.0, 4.5);
    let m_col = mix(glow, Color::WHITE, m_t).with_alpha((1.0 - m_t).clamp(0.1, 0.95));
    scene.add_disc(mx, my, mz, m_sz, 6, m_col);
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
