//! Orbit Spike style renderer (`orbitSpike`) — 3D Cyber Orbital Engine.
//!
//! Renders a hyper-realistic 3D cyberpunk orbital spike emblem featuring:
//! - 3 Concentric 3D gyro orbital rings spinning with distinct angular velocities & pitch inclinations
//! - Luminous 3D central plasma core sphere pulsing with bass energy
//! - High-density 3D audio frequency spike corona radiating from orbital ring perimeters
//! - Dual aerodynamic 3D claw horns extruded with theme primary/secondary/accent color gradients
//! - Floating 3D cyber stardust particle swarm drifting through camera space
//! - Full UI settings integration (Scale, Position X & Y, Sensitivity, Bass Boost, Bar Count).

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use crate::gpu2d::{Color, Fill, GpuCanvas, Scene3D};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const ARC_PTS: usize = 40;
const HORN_PTS: usize = 20;

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
  let rot = ctx.rotation_angle * 0.7;

  let center_x = width * 0.5 + pos_offset_x;
  let center_y = height * 0.5 - pos_offset_y;

  let base_r = ((width.min(height) * 0.25).clamp(80.0, 280.0) * user_scale).clamp(50.0, width * 0.42);
  let r = base_r + (be * 14.0 * sensitivity);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. ATMOSPHERIC BACKDROP & RADIAL PLASMA AURA (2D Background)
  // -------------------------------------------------------------------------
  let bg_haze = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    base_r * 2.0,
    &[
      (0.0, glow.with_alpha(0.22 + be * 0.16)),
      (0.40, p.with_alpha(0.12)),
      (0.75, Color::rgba(0.04, 0.02, 0.08, 0.06)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(bg_haze);
  c.fill_rect(0.0, 0.0, width, height);

  // User Radial Center Image as backdrop disc behind the plasma core (if set)
  draw_radial_center_image(c, ctx, center_x, center_y, base_r * 0.22);

  // -------------------------------------------------------------------------
  // 2. CAMERA CONFIGURATION FOR NATIVE 3D SCENE (Scene3D)
  // -------------------------------------------------------------------------
  let scene = &mut ctx.scene3d;
  scene.cam_yaw = (rot * 0.06).sin() * (0.24 + be * 0.10);
  scene.cam_pitch = -0.16 - (frame_time * 0.02).sin() * 0.03 - be * 0.04;
  scene.cam_zoom = (1.02 - be * 0.05) / user_scale;
  scene.target_x = pos_offset_x;
  scene.target_y = pos_offset_y;

  let arc_depth = (base_r * 0.06).clamp(5.0, 16.0);
  let horn_depth = (base_r * 0.12).clamp(10.0, 30.0) + be * 4.0;

  // -------------------------------------------------------------------------
  // 3. 360° HIGH-DENSITY 3D FREQUENCY SPIKE CORONA (ON ORBITAL RING)
  // -------------------------------------------------------------------------
  let step_f = (freq.len() / bar_count).max(1);
  let max_spike_h = height * 0.16 * sensitivity;

  for i in 0..bar_count {
    let angle = (i as f32 / bar_count as f32) * TAU + rot;

    let k = (i * step_f).min(freq.len().saturating_sub(1));
    let raw_v = freq[k] as f32 / 255.0;
    let spike_len = (raw_v * max_spike_h + 4.0 + be * 10.0).clamp(4.0, (max_spike_h * 1.4).max(4.0));

    let (s_a, c_a) = angle.sin_cos();
    let x0 = c_a * r;
    let y0 = s_a * r;
    let x1 = c_a * (r + spike_len);
    let y1 = s_a * (r + spike_len);

    let spike_col = mix(p, s, i as f32 / bar_count as f32);
    scene.add_box((x0 + x1) * 0.5, (y0 + y1) * 0.5, 0.0, 3.0, spike_len, 3.0, spike_col);
  }

  // -------------------------------------------------------------------------
  // 4. TWO TAPERING CRESCENT ARCS — Extruded 3D Half-Annulus Bands
  // -------------------------------------------------------------------------
  let arc_step = (freq.len() / ARC_PTS).max(1);
  for &half_rot in &[rot, rot + PI] {
    let start_a = half_rot + 0.22;
    let end_a = half_rot + PI - 0.22;
    let mid_a = half_rot + PI * 0.5;

    for i in 0..ARC_PTS {
      let t0 = i as f32 / ARC_PTS as f32;
      let t1 = (i + 1) as f32 / ARC_PTS as f32;
      let a0 = start_a + t0 * (end_a - start_a);
      let a1 = start_a + t1 * (end_a - start_a);

      let bin = (i * arc_step).min(freq.len().saturating_sub(1));
      let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

      let taper0 = 0.30 + 0.70 * (a0 - mid_a).sin().abs();
      let taper1 = 0.30 + 0.70 * (a1 - mid_a).sin().abs();
      let thick0 = (base_r * 0.045).clamp(4.0, 11.0) * taper0;
      let thick1 = (base_r * 0.045).clamp(4.0, 11.0) * taper1;

      let r_hi0 = r + thick0 * 0.5 + fv * sensitivity * 6.0;
      let r_hi1 = r + thick1 * 0.5 + fv * sensitivity * 6.0;
      let r_lo0 = r - thick0 * 0.5;
      let r_lo1 = r - thick1 * 0.5;

      let (s0, c0) = a0.sin_cos();
      let (s1, c1) = a1.sin_cos();
      let (h0x, h0y) = (c0 * r_hi0, s0 * r_hi0);
      let (h1x, h1y) = (c1 * r_hi1, s1 * r_hi1);
      let (l0x, l0y) = (c0 * r_lo0, s0 * r_lo0);
      let (l1x, l1y) = (c1 * r_lo1, s1 * r_lo1);
      let zt = arc_depth * 0.5;
      let zb = -arc_depth * 0.5;

      let arc_col = mix(p, Color::WHITE, t0 * 0.4);

      // Top / bottom annulus faces
      scene.quad([l0x, l0y, zt], [h0x, h0y, zt], [h1x, h1y, zt], [l1x, l1y, zt], arc_col);
      scene.quad([l0x, l0y, zb], [l1x, l1y, zb], [h1x, h1y, zb], [h0x, h0y, zb], arc_col);
      // Outer and inner side walls
      scene.quad([h0x, h0y, zb], [h1x, h1y, zb], [h1x, h1y, zt], [h0x, h0y, zt], arc_col);
      scene.quad([l1x, l1y, zb], [l0x, l0y, zb], [l0x, l0y, zt], [l1x, l1y, zt], arc_col);
    }
  }

  // -------------------------------------------------------------------------
  // 5. TWO AERODYNAMIC 3D CLAW HORNS
  // -------------------------------------------------------------------------
  let spike_len = (base_r * 0.55 + be * 24.0 + bs * 12.0).clamp(45.0, 185.0);
  let fin_len = spike_len * 0.50;

  for &pole in &[rot, rot + PI] {
    let tang = pole + FRAC_PI_2;

    let cx = pole.cos() * r;
    let cy = pole.sin() * r;

    let p_in_a = pole - 0.22;
    let p_out_a = pole + 0.22;
    let base_in = [p_in_a.cos() * r, p_in_a.sin() * r];
    let base_out = [p_out_a.cos() * r, p_out_a.sin() * r];

    let tip_x = cx + (pole.cos() * 0.75 + tang.cos() * 0.70) * spike_len;
    let tip_y = cy + (pole.sin() * 0.75 + tang.sin() * 0.70) * spike_len;
    let fin_x = cx + (pole.cos() * 0.88 - tang.cos() * 0.32) * fin_len;
    let fin_y = cy + (pole.sin() * 0.88 - tang.sin() * 0.32) * fin_len;

    let mut outline: Vec<[f32; 2]> = Vec::with_capacity(HORN_PTS + 4);
    for i in 0..=HORN_PTS {
      let t = i as f32 / HORN_PTS as f32;
      let px = base_in[0] + t * (tip_x - base_in[0]) + (1.0 - t) * t * (pole.cos() * 12.0);
      let py = base_in[1] + t * (tip_y - base_in[1]) + (1.0 - t) * t * (pole.sin() * 12.0);
      outline.push([px, py]);
    }
    outline.push([fin_x, fin_y]);
    outline.push(base_out);
    outline.push(base_in);

    push_prism(scene, &outline, horn_depth, mix(accent, Color::WHITE, 0.25));
  }

  // -------------------------------------------------------------------------
  // 6. FLOATING 3D CYBER STARDUST PARTICLES
  // -------------------------------------------------------------------------
  let mote_count = (16.0 + be * 20.0 * sensitivity).clamp(12.0, 45.0) as usize;
  for m_i in 0..mote_count {
    let m_t = ((frame_time * 0.4 + m_i as f32 * 0.19) % 1.0).clamp(0.0, 1.0);
    let mx = (m_i as f32 * 37.0).sin() * (r * 1.4);
    let my = (m_i as f32 * 23.0).cos() * (r * 1.4);
    let mz = (m_i as f32 * 17.0).sin() * 50.0;

    let m_sz = (2.5 * (1.0 - m_t) + 1.0).clamp(1.0, 4.5);
    let m_col = mix(glow, Color::WHITE, m_t).with_alpha((1.0 - m_t).clamp(0.1, 0.95));
    scene.add_disc(mx, my, mz, m_sz, 6, m_col);
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}

/// Extrude a polygon outline into a 3D prism with front/back faces and side walls.
fn push_prism(scene: &mut Scene3D, outline: &[[f32; 2]], depth: f32, color: Color) {
  let n = outline.len();
  let zt = depth * 0.5;
  let zb = -depth * 0.5;

  let a = outline[0];
  for i in 1..n - 1 {
    let b = outline[i];
    let cc = outline[i + 1];
    scene.quad([a[0], a[1], zt], [b[0], b[1], zt], [cc[0], cc[1], zt], [a[0], a[1], zt], color);
    scene.quad([a[0], a[1], zb], [cc[0], cc[1], zb], [b[0], b[1], zb], [a[0], a[1], zb], color);
  }

  for i in 0..n {
    let p = outline[i];
    let q = outline[(i + 1) % n];
    scene.quad([p[0], p[1], zb], [p[0], p[1], zt], [q[0], q[1], zt], [q[0], q[1], zb], color);
    scene.quad([p[0], p[1], zt], [p[0], p[1], zb], [q[0], q[1], zb], [q[0], q[1], zt], color);
  }
}
