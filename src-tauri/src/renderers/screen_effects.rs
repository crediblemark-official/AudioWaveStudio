//! Screen effects port (Phase 6). Mirrors `src/services/renderers/screenEffects.ts`.
//!
//! Only effects reproducible with the single-pass mesh pipeline are ported:
//! shake, vignette, pulse, spotlight, strobe, scanline, hueShift.
//! Frame-snapshot / per-pixel effects (glitch, chromatic, zoom, bars,
//! shockwave, pixelate, tilt, heatHaze, invert) need post-processing passes
//! and stay canvas-only (`is_gpu_supported` returns false for them).

use super::{hsl_to_color, RenderContext};
use crate::config::{ScreenEffect, ScreenEffectsSettings};
use crate::gpu2d::{Color, Fill, GpuCanvas};

/// Per-session screen-effect state (mirrors the TS module-level vars).
#[derive(Default)]
pub struct ScreenFxState {
  pub shake_bucket: i64,
  pub shake_x: f32,
  pub shake_y: f32,
}

impl ScreenFxState {
  pub fn new() -> Self {
    ScreenFxState {
      shake_bucket: -1,
      shake_x: 0.0,
      shake_y: 0.0,
    }
  }
}

fn mulberry32(seed: u32) -> impl FnMut() -> f32 {
  let mut a = seed;
  move || {
    a = a.wrapping_add(0x6D2B79F5);
    let t = a ^ (a >> 15);
    let t = t.wrapping_mul(a | 1);
    let t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
    let r = t ^ (t >> 14);
    r as f32 / 4294967296.0
  }
}

/// Mirrors `getShakeOffset` (screenEffects.ts). Returns the (x, y) offset the
/// caller should translate the background / style by.
pub fn compute_shake_offset(
  state: &mut ScreenFxState,
  settings: &ScreenEffectsSettings,
  above_floor: f32,
  beat_strength: f32,
  frame_time: f32,
) -> (f32, f32) {
  if !settings.enabled || !matches!(settings.main_effect, ScreenEffect::Shake) {
    return (0.0, 0.0);
  }
  let use_be = above_floor.max(0.0);
  let does_beat = beat_strength > 0.15;
  let smooth = settings.shake_intensity * use_be * 8.0;
  let beat = if does_beat { settings.shake_intensity * beat_strength * 50.0 } else { 0.0 };
  let mut intensity = smooth + beat;
  if settings.shake_on_beat && !does_beat {
    intensity = 0.0;
  }
  if intensity < 0.5 {
    return (0.0, 0.0);
  }
  let max_offset = settings.shake_max_offset.max(1.0);
  let frames_per_hold = ((1.0 - settings.shake_frequency) * 8.0).round() + 1.0;
  let bucket = ((frame_time * 1000.0) / (frames_per_hold * 16.67)).floor() as i64;
  if bucket != state.shake_bucket {
    state.shake_bucket = bucket;
    let mut rand = mulberry32(bucket as u32);
    let angle = if does_beat && beat_strength > 0.3 {
      -std::f32::consts::PI / 2.0
    } else {
      rand() * 2.0 * std::f32::consts::PI
    };
    let dist = (intensity * (0.5 + rand() * 0.5)).min(max_offset);
    state.shake_x = angle.cos() * dist;
    state.shake_y = angle.sin() * dist;
  }
  (state.shake_x, state.shake_y)
}

/// Draws the overlay-style screen effects (applied after text, mirroring
/// `applyScreenEffects`). Shake is handled as a frame translate in draw_frame.
pub fn apply_overlay(c: &mut GpuCanvas, ctx: &RenderContext, above_floor: f32) {
  let settings = &ctx.config.screen_effects;
  if !settings.enabled {
    return;
  }
  let use_be = above_floor.max(0.0);
  let w = ctx.width;
  let h = ctx.height;
  let beat = ctx.beat_strength;
  let frame_time = ctx.frame_time;
  match settings.main_effect {
    ScreenEffect::Vignette => vignette(c, w, h, settings, use_be, beat),
    ScreenEffect::Pulse => pulse(c, w, h, settings, use_be, beat),
    ScreenEffect::Spotlight => spotlight(c, w, h, settings, use_be, beat),
    ScreenEffect::Strobe => strobe(c, w, h, settings, use_be, beat, frame_time),
    ScreenEffect::Scanline => scanline(c, w, h, settings, use_be, beat),
    ScreenEffect::HueShift => hue_shift(c, w, h, settings, use_be, beat, frame_time),
    // Shake is a translate (draw_frame); remaining effects need snapshots.
    _ => {}
  }
}

fn vignette(
  c: &mut GpuCanvas,
  w: f32,
  h: f32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
) {
  let beat_pulse = if beat > 0.15 { beat * settings.pulse_intensity * 2.5 } else { 0.0 };
  let pulse = use_be * settings.pulse_intensity * 0.5 + beat_pulse;
  let max_radius = (w * w + h * h).sqrt() / 2.0;
  let radius = max_radius * (0.5 + pulse * 0.3).max(0.2);
  let alpha = (0.4 + pulse * 0.4).clamp(0.0, 1.0);
  let fill = Fill::radial_gradient(
    w / 2.0,
    h / 2.0,
    radius * 0.3,
    w / 2.0,
    h / 2.0,
    radius,
    &[
      (0.0, Color::rgba(0.0, 0.0, 0.0, 0.0)),
      (0.6, Color::rgba(0.0, 0.0, 0.0, 0.0)),
      (1.0, Color::rgba(0.0, 0.0, 0.0, alpha)),
    ],
  );
  c.set_fill(fill);
  c.fill_rect(0.0, 0.0, w, h);
}

fn pulse(
  c: &mut GpuCanvas,
  w: f32,
  h: f32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
) {
  let b = if beat > 0.15 { beat * settings.pulse_intensity } else { 0.0 };
  let smooth = use_be * settings.pulse_intensity * 0.15;
  let alpha = smooth + b;
  if alpha < 0.01 {
    return;
  }
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, alpha.min(1.0))));
  c.fill_rect(0.0, 0.0, w, h);
}

fn spotlight(
  c: &mut GpuCanvas,
  w: f32,
  h: f32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
) {
  let pulse = 0.5 + use_be * 0.3 + if beat > 0.15 { beat * 0.3 } else { 0.0 };
  let alpha = (0.6 * pulse).min(1.0);
  let base = Color::hex(&settings.spotlight_color);
  let r0 = (base.r * 255.0).round() as i32;
  let g0 = (base.g * 255.0).round() as i32;
  let b0 = (base.b * 255.0).round() as i32;
  let max_dim = w.max(h);
  let corners: [[f32; 3]; 4] = [
    [r0 as f32, g0 as f32, b0 as f32],
    [g0 as f32, b0 as f32, r0 as f32],
    [b0 as f32, r0 as f32, g0 as f32],
    [
      (g0 + 40).min(255) as f32,
      (r0 + 20).min(255) as f32,
      (b0 + 30).min(255) as f32,
    ],
  ];
  let positions = [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];
  for (i, [x, y]) in positions.iter().enumerate() {
    let [r, g, b] = corners[i];
    let fill = Fill::radial_gradient(
      *x,
      *y,
      0.0,
      *x,
      *y,
      max_dim * 1.1,
      &[
        (0.0, Color::rgba(r / 255.0, g / 255.0, b / 255.0, alpha * 0.3)),
        (0.35, Color::rgba(r / 255.0, g / 255.0, b / 255.0, alpha * 0.08)),
        (1.0, Color::rgba(r / 255.0, g / 255.0, b / 255.0, 0.0)),
      ],
    );
    c.set_fill(fill);
    c.fill_rect(0.0, 0.0, w, h);
  }
}

fn strobe(
  c: &mut GpuCanvas,
  w: f32,
  h: f32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
  frame_time: f32,
) {
  let b = if beat > 0.15 { settings.strobe_intensity * 0.9 } else { 0.0 };
  let smooth = settings.strobe_intensity * use_be * 0.12;
  let alpha = smooth + b;
  if alpha < 0.02 {
    return;
  }
  let on = ((frame_time * 10.0).floor() as i64) % 2 == 0;
  if !on {
    return;
  }
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, alpha.min(0.9))));
  c.fill_rect(0.0, 0.0, w, h);
}

fn scanline(
  c: &mut GpuCanvas,
  w: f32,
  h: f32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
) {
  let opacity = settings.scanline_opacity;
  if opacity <= 0.01 {
    return;
  }
  let b = if beat > 0.15 { beat } else { 0.0 };
  let darken = 0.08 * (use_be * 0.5 + b);
  c.set_fill(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, opacity.min(0.6))));
  let mut y = 0.0;
  while y < h {
    c.fill_rect(0.0, y, w, 1.0);
    y += 4.0;
  }
  if darken > 0.01 {
    c.set_fill(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, darken.min(0.35))));
    c.fill_rect(0.0, 0.0, w, h);
  }
}

fn hue_shift(
  c: &mut GpuCanvas,
  w: f32,
  h: f32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
  frame_time: f32,
) {
  let b = if beat > 0.15 { settings.hue_shift_intensity * beat } else { 0.0 };
  let smooth = settings.hue_shift_intensity * use_be * 0.3;
  let amount = (smooth + b).min(0.9);
  if amount < 0.02 {
    return;
  }
  let hue = (frame_time * 25.0) % 360.0;
  let fill = Fill::linear_gradient(
    0.0,
    0.0,
    w,
    h,
    &[
      (0.0, hsl_to_color(hue, 0.85, 0.5, amount)),
      (1.0, hsl_to_color((hue + 180.0) % 360.0, 0.85, 0.5, amount)),
    ],
  );
  c.set_fill(fill);
  c.fill_rect(0.0, 0.0, w, h);
}
