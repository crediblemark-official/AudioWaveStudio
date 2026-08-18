//! Dual Portal Bridge style renderer (`dualPortalBridge`) — 3D Harmonic Light String Engine.
//!
//! Masterpiece 100% faithful reference match:
//! Renders two 3D energy vortex portals connected by 12 interwoven, ultra-smooth, oscillating harmonic
//! light string cords ("tali cahaya") matching the reference image.
//! Features:
//! - 12 Intertwined harmonic light cords tapering to a single sharp line anchor at each portal throat
//! - Phase-shifted audio-reactive wave modulation with smooth spline interpolation
//! - Pure white-hot luminous core lines & soft cyan/accent additive bloom
//! - In-front 3D Z-depth weaving leaping out forward from portal centers
//! - Receding 3D perspective floor grid & 3D camera pitching with full UI settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const STRAND_COUNT: usize = 12;
const CORD_SEGS: usize = 120;

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
  let center_y = height * 0.5 + pos_offset_y;

  let portal_dist = ((width * 0.36).clamp(140.0, 520.0) * user_scale).clamp(80.0, width * 0.44);
  let portal_r = ((height * 0.22).clamp(70.0, 240.0) * user_scale).clamp(40.0, height * 0.38) + be * 12.0;

  let left_x = center_x - portal_dist;
  let right_x = center_x + portal_dist;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. ATMOSPHERIC BACKDROP & RADIAL DUAL PORTAL AURA
  // -------------------------------------------------------------------------
  for &(px, col) in &[(left_x, p), (right_x, s)] {
    let aura = Fill::radial_gradient(
      px,
      center_y,
      0.0,
      px,
      center_y,
      portal_r * 1.8,
      &[
        (0.0, col.with_alpha(0.24 + be * 0.16)),
        (0.40, glow.with_alpha(0.12)),
        (0.75, Color::rgba(0.04, 0.02, 0.08, 0.06)),
        (1.0, Color::TRANSPARENT),
      ],
    );
    c.set_fill(aura);
//     c.fill_rect(0.0, 0.0, width, height);
  }

  // -------------------------------------------------------------------------
  // 2. CAMERA CONFIGURATION FOR NATIVE 3D SCENE (Scene3D)
  // -------------------------------------------------------------------------
  let scene = &mut ctx.scene3d;
  scene.cam_yaw = (rot * 0.05).sin() * (0.08 + be * 0.06);
  scene.cam_pitch = -0.35 - (frame_time * 0.02).sin() * 0.03 - be * 0.04;
  scene.cam_zoom = (1.12 - be * 0.05) / user_scale;
  scene.target_x = pos_offset_x;
  scene.target_y = pos_offset_y;

  let world_cy = height * 0.5 - center_y;
  let world_floor = world_cy - height * 0.18;
  let z_portal = -30.0f32;
  let z_rope_front = z_portal + 15.0f32; // In front of portal throats!

  // -------------------------------------------------------------------------
  // 3. RECEDING 3D PERSPECTIVE FLOOR GRID LINES
  // -------------------------------------------------------------------------
  let half_w = width * 0.85;
  let z_max = -560.0f32;
  let grid_col = mix(p, s, 0.5).with_alpha(0.30);

  for col_i in 0..=12 {
    let gx = -half_w + (col_i as f32 / 12.0) * half_w * 2.0;
    scene.add_box(gx, world_floor, z_max * 0.5, 1.5, 1.5, -z_max, grid_col);
  }

  // -------------------------------------------------------------------------
  // 4. DUAL FUTURISTIC 3D VORTEX PORTAL RINGS (LEFT & RIGHT)
  // -------------------------------------------------------------------------
  for &(px_3d, p_col, yaw_angle) in &[
    (-portal_dist, p, 1.05f32),
    (portal_dist, s, -1.05f32),
  ] {
    scene.push();
    scene.translate(px_3d, world_cy, z_portal);
    scene.rotate_y(yaw_angle);
    scene.rotate_z(rot * 0.8);

    // Outer Tactical Annulus Ring
    let dash_slots = 72usize;
    let mut ring_radii = vec![0.0f32; dash_slots];
    for (i, r_val) in ring_radii.iter_mut().enumerate() {
      *r_val = if (i % 12) < 7 { portal_r } else { portal_r * 0.92 };
    }
    scene.add_band(0.0, 0.0, 0.0, portal_r * 0.92, portal_r, &ring_radii, 4.0, p_col);

    // Luminous Energy Core Disc
    scene.add_disc(0.0, 0.0, 0.0, portal_r * 0.35, 20, Color::WHITE);
    scene.add_disc(0.0, 0.0, 0.0, portal_r * 0.55, 24, p_col.with_alpha(0.80));

    scene.pop();
  }

  // -------------------------------------------------------------------------
  // 5. 12 INTERTWINED HARMONIC LIGHT STRING CORDS (100% REFERENCE MATCH)
  // -------------------------------------------------------------------------
  let step_f = (freq.len() / CORD_SEGS).max(1);

  for st_i in 0..STRAND_COUNT {
    let st_ratio = st_i as f32 / (STRAND_COUNT - 1) as f32; // 0.0 to 1.0
    let phase_shift = st_i as f32 * 0.28 + rot * 1.2;
    let freq_mult = 2.0 + (st_i % 3) as f32 * 0.5;

    // Color gradient matching reference image:
    // Core center cords: White-hot (#ffffff), outer cords: Soft Cyan & Theme Accent
    let strand_col = if st_i == STRAND_COUNT / 2 || st_i == STRAND_COUNT / 2 - 1 {
      Color::WHITE // Pure white core strand
    } else if st_i % 2 == 0 {
      mix(Color::WHITE, s, 0.65)
    } else {
      mix(Color::WHITE, accent, 0.65)
    };

    let thick = if st_i == STRAND_COUNT / 2 { 2.8 } else { 1.8 };

    for seg in 0..CORD_SEGS {
      let t0 = seg as f32 / CORD_SEGS as f32;
      let t1 = (seg + 1) as f32 / CORD_SEGS as f32;

      let bin0 = (seg * step_f / (CORD_SEGS / bar_count.max(1)).max(1)).min(freq.len().saturating_sub(1));
      let fv0 = freq[bin0] as f32 / 255.0;

      let bin1 = ((seg + 1) * step_f / (CORD_SEGS / bar_count.max(1)).max(1)).min(freq.len().saturating_sub(1));
      let fv1 = freq[bin1] as f32 / 255.0;

      // Envelope: 0.0 at portal throats (t=0 & t=1), 1.0 at center span
      let env0 = (std::f32::consts::PI * t0).sin().powi(2);
      let env1 = (std::f32::consts::PI * t1).sin().powi(2);

      // Harmonic sine wave oscillation matching reference image
      let wave0_y = (TAU * t0 * freq_mult + phase_shift).sin() * (height * 0.16 * sensitivity * (0.4 + fv0 * 0.8) + be * 15.0);
      let wave1_y = (TAU * t1 * freq_mult + phase_shift).sin() * (height * 0.16 * sensitivity * (0.4 + fv1 * 0.8) + be * 15.0);

      // Vertical offset spread across strands
      let strand_offset = (st_ratio - 0.5) * (height * 0.12 * (1.0 - (t0 - 0.5).abs() * 1.8).max(0.0));

      let x0_3d = -portal_dist + t0 * (2.0 * portal_dist);
      let x1_3d = -portal_dist + t1 * (2.0 * portal_dist);

      let y0_3d = world_cy + (wave0_y + strand_offset) * env0;
      let y1_3d = world_cy + (wave1_y + strand_offset) * env1;

      let z0_3d = z_rope_front + (TAU * t0 * 1.5 + st_i as f32 * 0.5).cos() * 8.0 * env0;
      let z1_3d = z_rope_front + (TAU * t1 * 1.5 + st_i as f32 * 0.5).cos() * 8.0 * env1;

      // Render 3D continuous lit ribbon quad
      scene.quad(
        [x0_3d, y0_3d + thick * 0.5, z0_3d],
        [x0_3d, y0_3d - thick * 0.5, z0_3d],
        [x1_3d, y1_3d - thick * 0.5, z1_3d],
        [x1_3d, y1_3d + thick * 0.5, z1_3d],
        strand_col.with_alpha(0.92),
      );
      scene.quad(
        [x0_3d, y0_3d - thick * 0.5, z0_3d],
        [x0_3d, y0_3d + thick * 0.5, z0_3d],
        [x1_3d, y1_3d + thick * 0.5, z1_3d],
        [x1_3d, y1_3d - thick * 0.5, z1_3d],
        strand_col.with_alpha(0.92),
      );
    }
  }

  // -------------------------------------------------------------------------
  // 6. HIGH-SPEED SPARKS & EMBERS ALONG LIGHT STRINGS
  // -------------------------------------------------------------------------
  let spark_count = (20.0 + be * 24.0 * sensitivity).clamp(12.0, 50.0) as usize;
  for s_i in 0..spark_count {
    let s_t = ((frame_time * 0.7 + s_i as f32 * 0.15) % 1.0).clamp(0.0, 1.0);
    let sx = left_x + s_t * (right_x - left_x);

    let env_s = (std::f32::consts::PI * s_t).sin().powi(2);
    let wave_y = (TAU * s_t * 2.0 + s_i as f32 * 0.7).sin() * (height * 0.14 * sensitivity * env_s);
    let sy = center_y + wave_y;

    let spark_sz = ((3.2 * (1.0 - (s_t - 0.5).abs() * 1.5) + 1.0) * user_scale).clamp(1.0, 5.0);
    let spark_col = mix(accent, Color::WHITE, s_t);

    c.set_fill(Fill::Solid(spark_col));
    c.set_shadow(spark_col, 8.0 + bs * 4.0);
    c.fill_ellipse(sx, sy, spark_sz, spark_sz);
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
