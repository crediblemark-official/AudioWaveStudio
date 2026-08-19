//! Fire 3D style renderer (`api3D`) — 3D Cyber Fire & Laser Plasma Engine.
//!
//! Renders a 3D audio-reactive cyber fire waveform with 3D thermal plasma crests,
//! glowing laser baselines, 3D perspective floor reflections, rising ember sparks,
//! radial thermal atmospheric glow, camera orbiting, target panning, and full UI settings integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const WAVE_POINTS: usize = 120;

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
  let cols = ctx.config.reactivity.bar_count.clamp(16, 128);
  // Fire Wave sliders — previously dead settings, now wired into the geometry.
  let fire_w = ctx.config.reactivity.fire_width_ratio.unwrap_or(0.94).clamp(0.3, 1.0);
  let fire_h = ctx.config.reactivity.fire_height_scale.unwrap_or(1.0).clamp(0.3, 2.5);

  let be = ctx.bass_energy;
  let freq = ctx.freq_data;
  let frame_time = ctx.frame_time;

  let st = &mut ctx.state.advanced;

  // 2D draws use internal position offset (no longer relies on outer canvas transform).
  let cx = width * 0.5 + pos_offset_x;
  let cy = height * 0.5 + pos_offset_y;

  st.api_time += 0.02 + be * 0.015;
  let time = st.api_time;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. SMOOTH ATMOSPHERIC THERMAL GLOW (2D Background - NO sharp boxes!)
  // -------------------------------------------------------------------------
  let aura = Fill::radial_gradient(
    cx,
    cy,
    0.0,
    cx,
    cy,
    width * 0.55 * fire_w,
    &[
      (0.0, Color::rgba(1.0, 0.35, 0.05, 0.22 + be * 0.15)),
      (0.35, mix(p, accent, 0.35).with_alpha(0.12)),
      (0.70, Color::rgba(0.04, 0.02, 0.08, 0.05)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(aura);
//   c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 2. CAMERA CONFIGURATION FOR NATIVE 3D SCENE (Scene3D)
  // -------------------------------------------------------------------------
  let scene = &mut ctx.scene3d;
  scene.cam_yaw = (frame_time * 0.04).sin() * (0.10 + be * 0.12);
  scene.cam_pitch = -0.32 - (frame_time * 0.02).sin() * 0.04 - be * 0.03;
  scene.cam_zoom = (1.05 - be * 0.05) / user_scale;
  scene.target_x = pos_offset_x;
  scene.target_y = pos_offset_y;

  let total_w = width * 0.88 * fire_w;
  let start_x = -total_w / 2.0;
  let point_step = total_w / (WAVE_POINTS as f32 - 1.0);
  let max_h = height * 0.38 * sensitivity * fire_h;

  let step_f = (freq.len() / cols).max(1);

  // -------------------------------------------------------------------------
  // 3. 3D CYBER FIRE WAVEFORM CREST DISPLACEMENT
  // -------------------------------------------------------------------------
  let mut heights = vec![0.0f32; WAVE_POINTS];
  for i in 0..WAVE_POINTS {
    let col_i = (i * cols / WAVE_POINTS).min(cols - 1);
    let k = (col_i * step_f).min(freq.len().saturating_sub(1));
    let raw_v = freq[k] as f32 / 255.0;
    let val = (raw_v * sensitivity).clamp(0.0, 1.8);

    let noise = (i as f32 * 0.15 + time * 3.0).sin() * 12.0;
    heights[i] = val * max_h + noise * (0.3 + val);
  }

  // -------------------------------------------------------------------------
  // 4. RENDER 3D FIRE PLASMA WALL QUADS (Scene3D)
  // -------------------------------------------------------------------------
  let _wall_depth = 40.0f32;

  for i in 0..WAVE_POINTS - 1 {
    let x0 = start_x + i as f32 * point_step;
    let x1 = x0 + point_step;
    let h0 = heights[i];
    let h1 = heights[i + 1];

    let t_ratio = i as f32 / (WAVE_POINTS as f32 - 1.0);
    let col = mix(p, s, t_ratio).with_alpha(0.85);

    // Front 3D plasma curtain quad
    scene.quad(
      [x0, 0.0, 0.0],
      [x1, 0.0, 0.0],
      [x1, h1, 0.0],
      [x0, h0, 0.0],
      col,
    );

    // Mirror bottom 3D plasma curtain quad (toggle with the Mirror slider)
    if ctx.config.reactivity.mirror_bars {
      scene.quad(
        [x0, 0.0, 0.0],
        [x1, 0.0, 0.0],
        [x1, -h1 * 0.4, 0.0],
        [x0, -h0 * 0.4, 0.0],
        col.with_alpha(0.40),
      );
    }

    // Top Glowing Crest Line
    let cap_col = if (h0 + h1) * 0.5 > max_h * 0.5 {
      Color::WHITE
    } else {
      mix(mix(col, glow, 0.4), accent, 0.2).with_alpha(0.95)
    };

    scene.quad(
      [x0, h0, 1.0],
      [x1, h1, 1.0],
      [x1, h1 + 3.0, 1.0],
      [x0, h0 + 3.0, 1.0],
      cap_col,
    );
  }

  // -------------------------------------------------------------------------
  // 5. 3D CENTER LASER BASELINE
  // -------------------------------------------------------------------------
  scene.add_box(0.0, 0.0, 2.0, total_w, 2.5, 2.5, mix(Color::WHITE, accent, 0.25));

  // -------------------------------------------------------------------------
  // 6. FLOATING 3D MICRO EMBERS
  // -------------------------------------------------------------------------
  let ember_count = (16.0 + be * 20.0).clamp(12.0, 45.0) as usize;
  for e_i in 0..ember_count {
    let e_t = ((frame_time * 0.4 + e_i as f32 * 0.19) % 1.0).clamp(0.0, 1.0);
    let ex = (e_i as f32 * 37.0).sin() * (total_w * 0.45);
    let ey = e_t * (height * 0.45);
    let ez = (e_i as f32 * 23.0).cos() * 30.0;

    let e_sz = (2.5 * (1.0 - e_t) + 1.0).clamp(1.0, 4.5);
    let e_col = if e_i % 2 == 0 {
      Color::rgba(1.0, 0.90, 0.40, (1.0 - e_t).clamp(0.1, 0.95))
    } else {
      Color::rgba(1.0, 0.35, 0.05, (1.0 - e_t).clamp(0.1, 0.95))
    };

    scene.add_disc(ex, ey, ez, e_sz, 6, e_col);
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
