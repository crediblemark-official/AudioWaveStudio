//! Dual Wave Horizon style renderer (`dualWaveHorizon`) — native 3D redesign.
//!
//! "Twin Wave Canyon": two mirrored spectrum walls rise from the near frame
//! edges, lean inward over the corridor and melt back into a glowing vanishing
//! point on the horizon. Each side carries a *dual* waveform — the tall red
//! wall is driven by one stereo channel while a thinner white "twin" ribbon,
//! offset inward and floating closer to the camera, is driven by the mirrored
//! channel, so the two layers visibly separate in real 3D depth. A bright crest
//! line runs along each wall's wave ridge. A bass-pulsed red river runs down
//! the corridor floor, and a white energy pillar with a throbbing sun disc marks
//! the convergence point. A few embers drift through the canyon in perspective.

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const WALL_SEGS: usize = 48;
const MOTE_COUNT: usize = 26;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let bright_red = Color::rgba(1.0, 0.05, 0.12, 0.98);

  // -------------------------------------------------------------------------
  // 2D HAZE: soft red atmosphere floating around the horizon line (under 3D).
  // -------------------------------------------------------------------------
  let haze = Fill::radial_gradient(
    center_x,
    center_y + height * 0.05,
    0.0,
    center_x,
    center_y + height * 0.05,
    height * 0.5,
    &[
      (0.0, Color::rgba(1.0, 0.05, 0.12, 0.14 + bs * 0.06)),
      (0.6, Color::rgba(0.8, 0.05, 0.15, 0.05)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(haze);
  c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // CAMERA: pitched down the canyon, slow yaw sway, beat zoom-in.
  // -------------------------------------------------------------------------
  let scene = &mut ctx.scene3d;
  scene.cam_yaw = (rot * 0.05).sin() * 0.05 + be * 0.04;
  scene.cam_pitch = -0.42 - be * 0.04;
  scene.cam_zoom = 1.25 - be * 0.06;

  // World layout: baseline (world y = 0) is the canyon floor; the corridor
  // recedes along −z and both walls converge onto the vanishing point.
  let max_h = height * 0.36;
  let z_far = -620.0f32;
  let step = (freq.len() / WALL_SEGS).max(1);

  let x_near = width * 0.42;
  let x_far = width * 0.012;
  let twin_gap = (width * 0.03).clamp(12.0, 40.0);
  let twin_depth = 10.0;

  let z_at = |i: usize| -(i as f32 / WALL_SEGS as f32) * z_far.abs();

  // -------------------------------------------------------------------------
  // CANYON WALLS + CREST LINES + TWIN RIBBONS (two-sided lit quad strips).
  // -------------------------------------------------------------------------
  for side in [-1.0f32, 1.0] {
    // Per-sample: bottom x, top x (leaning inward so the face catches the
    // camera), z, wall height, twin height.
    let mut b_x = [0.0f32; WALL_SEGS + 1];
    let mut t_x = [0.0f32; WALL_SEGS + 1];
    let mut z_s = [0.0f32; WALL_SEGS + 1];
    let mut wall_h = [0.0f32; WALL_SEGS + 1];
    let mut twin_h = [0.0f32; WALL_SEGS + 1];

    for i in 0..=WALL_SEGS {
      let t = i as f32 / WALL_SEGS as f32;
      let x = side * (x_near + (x_far - x_near) * t);
      let z = z_at(i);

      let bin = if side < 0.0 {
        (i * step).min(freq.len().saturating_sub(1))
      } else {
        ((WALL_SEGS - i) * step).min(freq.len().saturating_sub(1))
      };
      let mirror_bin = if side < 0.0 {
        ((WALL_SEGS - i) * step).min(freq.len().saturating_sub(1))
      } else {
        (i * step).min(freq.len().saturating_sub(1))
      };

      let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
      let mfv = *freq.get(mirror_bin).unwrap_or(&0) as f32 / 255.0;
      let envelope = 0.12 + 0.88 * (PI * t).sin().powf(0.9);

      let h = max_h * envelope * (0.25 + 0.75 * fv) * (1.0 + be * 0.4) * sensitivity;

      b_x[i] = x;
      // Top edge pulled toward the corridor centre by ~half the height, so the
      // wall face (not just its edge) is visible from the camera.
      t_x[i] = x - side * h * 0.5;
      z_s[i] = z;
      wall_h[i] = h;
      twin_h[i] = (h * 0.5 + 8.0) * (0.6 + 0.8 * mfv) * (1.0 + be * 0.5);
    }

    // Tall red wall + bright crest ridge.
    for i in 0..WALL_SEGS {
      let (bx0, bx1) = (b_x[i], b_x[i + 1]);
      let (tx0, tx1) = (t_x[i], t_x[i + 1]);
      let (h0, h1) = (wall_h[i], wall_h[i + 1]);
      let (z0, z1) = (z_s[i], z_s[i + 1]);

      scene.quad([bx0, 0.0, z0], [tx0, h0, z0], [tx1, h1, z1], [bx1, 0.0, z1], bright_red.with_alpha(0.95));
      scene.quad([tx0, h0, z0], [bx0, 0.0, z0], [bx1, 0.0, z1], [tx1, h1, z1], bright_red.with_alpha(0.95));

      // Bright crest line hugging the wave ridge.
      let cy0 = (h0 - 3.5).max(0.0);
      let cy1 = (h1 - 3.5).max(0.0);
      scene.quad([tx0, h0, z0], [tx0, cy0, z0], [tx1, cy1, z1], [tx1, h1, z1], Color::rgba(1.0, 0.55, 0.6, 0.95));
      scene.quad([tx0, cy0, z0], [tx0, h0, z0], [tx1, h1, z1], [tx1, cy1, z1], Color::rgba(1.0, 0.55, 0.6, 0.95));
    }

    // Thin white twin ribbon floating inward + toward the camera.
    for i in 0..WALL_SEGS {
      let (tx0, tx1) = (t_x[i] - side * twin_gap, t_x[i + 1] - side * twin_gap);
      let (h0, h1) = (twin_h[i], twin_h[i + 1]);
      let (z0, z1) = (z_s[i] + twin_depth, z_s[i + 1] + twin_depth);

      scene.quad([tx0, h0, z0], [tx0, 0.0, z0], [tx1, 0.0, z1], [tx1, h1, z1], Color::rgba(1.0, 0.92, 0.95, 0.8));
      scene.quad([tx0, 0.0, z0], [tx0, h0, z0], [tx1, h1, z1], [tx1, 0.0, z1], Color::rgba(1.0, 0.92, 0.95, 0.8));
    }
  }

  // -------------------------------------------------------------------------
  // BASS RIVER: pulsing red ribbon down the corridor floor.
  // -------------------------------------------------------------------------
  let rw = 5.0 + be * 42.0 * sensitivity;
  for i in 0..WALL_SEGS {
    let z0 = z_at(i);
    let z1 = z_at(i + 1);
    scene.quad([-rw, 0.0, z0], [rw, 0.0, z0], [rw, 0.0, z1], [-rw, 0.0, z1], bright_red.with_alpha(0.9));
    scene.quad([-rw, -1.5, z0], [rw, -1.5, z0], [rw, -1.5, z1], [-rw, -1.5, z1], bright_red.with_alpha(0.6));
  }

  // -------------------------------------------------------------------------
  // HORIZON: white energy pillar + throbbing sun disc at the vanishing point.
  // -------------------------------------------------------------------------
  scene.add_box(0.0, max_h * 0.5, z_far - 4.0, 8.0, max_h, 8.0, Color::WHITE);
  scene.add_disc(0.0, 0.0, z_far + 2.0, (10.0 + be * 28.0).clamp(8.0, 60.0), 26, Color::WHITE);

  // -------------------------------------------------------------------------
  // EMBERS: a few deterministic drifting specks through the canyon.
  // -------------------------------------------------------------------------
  for i in 0..MOTE_COUNT {
    let t = (i as f32 * 0.6180339887).fract();
    let z = -40.0 - t * (z_far.abs() - 120.0);
    let x = (i as f32 * 0.371).fract() - 0.5;
    let side = if x < 0.0 { -1.0 } else { 1.0 };
    let drift = (rot * 0.7 + i as f32 * 0.7).sin() * 4.0;
    let px = side * (24.0 + x.abs() * width * 0.38) + drift;
    let py = 10.0 + (i as f32 * 0.29).fract() * max_h * 0.7;
    let s = 1.4 + (i as f32 * 0.11).fract() * 2.0;
    scene.add_box(px, py, z, s, s, s, bright_red.with_alpha(0.4));
  }

  c.restore();
}
