//! Spiral Galaxy style renderer (`spiralGalaxy`) — Cosmic Musical Galaxy Engine.
//!
//! Masterpiece 100% faithful port matching the reference photo:
//! Tilted 3D cosmic accretion vortex ring with intense glowing Star Gold inner core,
//! concentric Electric Cyan outer disc trails, floating 3D musical notes (♪, ♫, ♩)
//! orbiting in deep space, cosmic stardust field, silky smooth idle motion,
//! audio-reactive beat pulsing, and full UI settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{bin_sum, mix, GalaxyParticle};
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const GALAXY_PARTICLES: usize = 750;
const DUST_PARTICLES: usize = 120;
const FLOATING_NOTES_COUNT: usize = 8;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;

  let p = theme_primary(theme);
  let s = theme_secondary(theme);
  let _accent = theme_accent(theme);
  let _glow = theme_glow(theme);

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

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  let cx = width / 2.0 + pos_offset_x;
  let cy = height / 2.0 - pos_offset_y;
  let max_r = (width.min(height) * 0.48).clamp(180.0, 580.0);

  // Audio energy calculation for dynamic motion (silky smooth when idle!)
  let step_f = (freq.len() / bar_count).max(1);
  let raw_audio = bin_sum(freq, step_f, 0);
  let audio_energy = (raw_audio * sensitivity * 1.2 + be * 0.8).clamp(0.0, 2.5);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. DEEP COSMIC NEBULA SPACE BACKDROP (2D Background)
  // -------------------------------------------------------------------------
  // Dark midnight blue to cosmic violet gradient backdrop
  let space_bg = Fill::radial_gradient(
    cx,
    cy,
    0.0,
    cx,
    cy,
    max_r * 1.8 * user_scale,
    &[
      (0.0, Color::rgba(0.08, 0.12, 0.28, 0.35 + audio_energy * 0.15)),
      (0.35, Color::rgba(0.03, 0.06, 0.18, 0.20)),
      (0.70, Color::rgba(0.01, 0.02, 0.07, 0.10)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(space_bg);
  c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 2. INITIALIZE 3D COSMIC PARTICLE SWARM & FLOATING NOTES
  // -------------------------------------------------------------------------
  if st.galaxy.is_empty() {
    for i in 0..GALAXY_PARTICLES {
      let r = (i as f32 / GALAXY_PARTICLES as f32).powf(0.85);
      let h_variance = (rng.next() - 0.5) * 2.2; // 3D height dispersion
      st.galaxy.push(GalaxyParticle {
        angle: rng.next() * TAU,
        radius: r,
        speed: 0.0012 + (1.0 - r) * 0.004,
        size: 0.8 + r * 3.5,
        arm: (rng.next() * 2.0) as u32,
        offset: h_variance,
      });
    }
  }

  // -------------------------------------------------------------------------
  // 3. CAMERA CONFIGURATION FOR NATIVE 3D SCENE (Scene3D)
  // -------------------------------------------------------------------------
  let scene = &mut ctx.scene3d;
  // High-angle 3D perspective looking down at tilted accretion ring (matching reference photo!)
  let cam_sway_speed = 0.012 + audio_energy * 0.025;
  scene.cam_yaw = (frame_time * cam_sway_speed).sin() * (0.05 + audio_energy * 0.08);
  scene.cam_pitch = -0.62 - (frame_time * 0.01).sin() * 0.03 - be * 0.04;

  // Respect Visualizer Scale setting for 3D Camera Zoom
  scene.cam_zoom = (1.05 - be * 0.05) / user_scale;
  scene.target_x = pos_offset_x;
  scene.target_y = pos_offset_y;

  let core_x = 0.0f32;
  let core_y = 0.0f32;

  // -------------------------------------------------------------------------
  // 4. RENDER 3D COSMIC ACCRETION RING PARTICLES (STAR GOLD INNER, CYAN OUTER)
  // -------------------------------------------------------------------------
  let orbit_rot_speed = 0.0018 + audio_energy * 0.005 + bs * 0.006;

  for gp in st.galaxy.iter_mut() {
    let bin_idx = ((gp.radius * 0.9) * (freq.len() as f32 - 1.0)) as usize;
    let freq_val = if bin_idx < freq.len() {
      (freq[bin_idx] as f32 / 255.0) * sensitivity
    } else {
      0.0
    };

    let audio_pulse = (freq_val * 0.8 + be * 0.7 + bs * 0.5).clamp(0.0, 2.5);

    // Audio-driven orbital rotation
    gp.angle += gp.speed + orbit_rot_speed * (1.0 + audio_pulse * 1.2);

    // Particle distribution matching photo:
    // Inner Ring: Dense glowing Gold/Amber ring at r in [0.28, 0.50]
    // Outer Disc: Electric Cyan / Deep Blue concentric trails at r > 0.50
    let dist = (0.28 + gp.radius * 0.72) * max_r * (1.0 + audio_pulse * 0.12);
    let wobble = (gp.angle * 3.0).sin() * (max_r * 0.03 * gp.radius);
    let r_3d = dist + wobble;

    let x = core_x + gp.angle.cos() * r_3d;
    let z = gp.angle.sin() * r_3d;

    // 3D vertical height dispersion (thin flat ring disc)
    let y = core_y + gp.offset * (12.0 * (1.0 - gp.radius * 0.5)) + (gp.angle * 2.0).sin() * (4.0 * gp.radius);

    // Color gradient matching photo:
    // Inner Ring (gp.radius < 0.42): Star Gold & Radiant Amber (#fff0aa / #ffb400 / #ffffff)
    // Outer Ring (gp.radius >= 0.42): Electric Cyan & Deep Cosmic Blue (#00e5ff / #0044ff)
    let p_color = if gp.radius < 0.42 {
      let factor = gp.radius / 0.42;
      let gold = mix(Color::rgba(1.0, 0.96, 0.75, 0.98), Color::rgba(1.0, 0.65, 0.10, 0.92), factor);
      mix(gold, p, 0.12)
    } else {
      let factor = ((gp.radius - 0.42) / 0.58).min(1.0);
      let cyan_blue = mix(Color::rgba(0.0, 0.90, 1.0, 0.85), Color::rgba(0.0, 0.30, 0.95, 0.55), factor);
      mix(cyan_blue, s, 0.18)
    };

    let p_size = (gp.size * (1.0 + audio_pulse * 0.7)).clamp(1.5, 11.0);
    let alpha = (0.35 + (1.0 - gp.radius * 0.4) * 0.55).clamp(0.15, 0.98);

    scene.add_disc(x, y, z, p_size, 8, p_color.with_alpha(alpha));
  }

  // -------------------------------------------------------------------------
  // 5. RENDER FLOATING 3D MUSICAL NOTES (♪, ♫, ♩) MATCHING REFERENCE PHOTO
  // -------------------------------------------------------------------------
  let note_coords = [
    (-max_r * 0.75, 35.0 + be * 20.0, -max_r * 0.20, true),   // Outer left note
    (-max_r * 0.10, 75.0 + be * 35.0, -max_r * 0.35, false),  // Top left golden eighth note ♪
    (max_r * 0.25, 85.0 + be * 30.0, -max_r * 0.40, false),   // Top right golden sixteenth note ♫
    (max_r * 0.65, 60.0 + be * 25.0, -max_r * 0.10, false),   // Right side note ♪
    (max_r * 0.85, 25.0 + be * 15.0, max_r * 0.25, false),    // Far right note
    (-max_r * 0.50, -45.0 + be * 15.0, max_r * 0.50, true),   // Bottom left note ♪
    (max_r * 0.40, -55.0 + be * 15.0, max_r * 0.60, true),    // Bottom right red note ♪
    (0.0, 95.0 + be * 40.0, 0.0, false),                      // High central note
  ];

  for (idx, &(base_x, base_y, base_z, is_red)) in note_coords.iter().enumerate().take(FLOATING_NOTES_COUNT) {
    let note_t = idx as f32 / FLOATING_NOTES_COUNT as f32;
    let float_a = frame_time * (0.8 + audio_energy * 0.5) + note_t * TAU;

    let nx = core_x + base_x + float_a.cos() * 8.0;
    let ny = core_y + base_y + float_a.sin() * 12.0;
    let nz = base_z + (float_a * 1.3).cos() * 8.0;

    let note_col = if is_red {
      Color::rgba(1.0, 0.30, 0.20, 0.95) // Warm crimson / red note outline
    } else {
      Color::rgba(1.0, 0.85, 0.25, 0.98) // Bright star gold note outline
    };

    let note_sz = (12.0 + (note_t * 5.0).sin().abs() * 6.0 + be * 6.0).clamp(10.0, 22.0);

    // 3D Note Head Disc
    scene.add_disc(nx, ny, nz, note_sz * 0.6, 10, note_col);

    // 3D Note Stem Box
    scene.add_box(
      nx + note_sz * 0.4,
      ny + note_sz * 0.9,
      nz,
      2.5,
      note_sz * 1.8,
      2.5,
      note_col.with_alpha(0.90),
    );

    // 3D Note Flag / Beam Box (for double eighth / sixteenth notes)
    if idx % 2 == 1 {
      scene.add_box(
        nx + note_sz * 0.9,
        ny + note_sz * 1.7,
        nz,
        note_sz * 1.1,
        2.5,
        2.5,
        note_col.with_alpha(0.90),
      );
      // Second note head for double note ♫
      scene.add_disc(nx + note_sz * 1.4, ny, nz, note_sz * 0.6, 10, note_col);
      scene.add_box(
        nx + note_sz * 1.8,
        ny + note_sz * 0.9,
        nz,
        2.5,
        note_sz * 1.8,
        2.5,
        note_col.with_alpha(0.90),
      );
    }
  }

  // -------------------------------------------------------------------------
  // 6. BACKGROUND & FOREGROUND 3D STARDUST FIELD
  // -------------------------------------------------------------------------
  for d_i in 0..DUST_PARTICLES {
    let d_t = d_i as f32 / DUST_PARTICLES as f32;
    let d_angle = frame_time * 0.015 + d_t * TAU * 3.0;
    let d_r = max_r * (0.3 + (d_t * 13.0).cos().abs() * 1.2);

    let dx = d_angle.cos() * d_r;
    let dz = d_angle.sin() * d_r;
    let dy = (d_t * TAU * 4.0).sin() * (80.0 + d_t * 40.0);

    // Dual color stardust motes (cyan dust & amber dust matching photo!)
    let d_col = if d_i % 2 == 0 {
      Color::rgba(0.0, 0.90, 1.0, (0.30 + (d_t * 5.0).sin().abs() * 0.50).clamp(0.1, 0.85))
    } else {
      Color::rgba(1.0, 0.75, 0.20, (0.30 + (d_t * 5.0).cos().abs() * 0.50).clamp(0.1, 0.85))
    };

    let d_sz = (1.5 + (d_t * 7.0).sin().abs() * 2.5).clamp(1.0, 4.5);
    scene.add_disc(dx, dy, dz, d_sz, 6, d_col);
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
