//! Screen effects port (Phase 6). Mirrors `src/services/renderers/screenEffects.ts`.
//!
//! Overlay effects drawn with the single-pass mesh pipeline: shake (a frame
//! translate), vignette, pulse, spotlight, strobe, scanline.
//! Frame-snapshot / per-pixel effects (glitch, chromatic, zoom, invert, bars,
//! shockwave, pixelate, tilt, heatHaze, hueShift) are emitted as `PostFx`
//! modes and rendered by the GPU post-processing pipeline (`gpu2d/postfx.wgsl`).

use super::RenderContext;
use crate::config::{ScreenEffect, ScreenEffectsSettings};
use crate::gpu2d::renderer::PostFx;
use crate::gpu2d::{Color, Fill, GpuCanvas};

/// Per-session screen-effect state (mirrors the TS module-level vars).
#[derive(Default)]
pub struct ScreenFxState {
  pub shake_bucket: i64,
  pub shake_x: f32,
  pub shake_y: f32,
  pub prev_beat_high: bool,
  pub shock_start: f32,
  /// Monotonic clock (seconds, real wall time) driving time-based effects so
  /// they keep animating across pause/seek instead of freezing or jumping.
  /// Set by the caller before each frame (live preview: wall clock; export:
  /// continuous export time).
  pub fx_time: f32,
}

impl ScreenFxState {
  pub const fn new() -> Self {
    ScreenFxState {
      shake_bucket: -1,
      shake_x: 0.0,
      shake_y: 0.0,
      prev_beat_high: false,
      shock_start: -1e9,
      fx_time: 0.0,
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
  let bucket = ((state.fx_time * 1000.0) / (frames_per_hold * 16.67)).floor() as i64;
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

/// Computes the post-processing pass parameters for frame-sampling screen
/// effects (glitch, chromatic, zoom, invert, bars, shockwave, pixelate, tilt,
/// heatHaze). Returns `None` when the effect is inactive or below threshold,
/// mirroring the per-effect early-outs in `screenEffects.ts`.
pub fn post_fx(
  state: &mut ScreenFxState,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
  fps: f32,
) -> Option<PostFx> {
  if !settings.enabled {
    return None;
  }
  // Time-based effects (shockwave, glitch bars, drift) use the monotonic
  // clock so they never freeze on pause or jump on seek.
  let ft = state.fx_time;
  let mode = match settings.main_effect {
    ScreenEffect::Glitch => 1,
    ScreenEffect::Chromatic => 2,
    ScreenEffect::Zoom => 3,
    ScreenEffect::Invert => 4,
    ScreenEffect::Bars => 5,
    ScreenEffect::Shockwave => 6,
    ScreenEffect::Pixelate => 7,
    ScreenEffect::Tilt => 8,
    ScreenEffect::HeatHaze => 9,
    ScreenEffect::HueShift => 10,
    ScreenEffect::GlassCrack => 11,
    _ => return None,
  };
  let eff_beat = |v: f32| if beat > 0.15 { v * beat } else { 0.0 };
  let intensity = match settings.main_effect {
    ScreenEffect::Glitch => {
      settings.glitch_intensity * 0.25 + settings.glitch_intensity * use_be * 0.8 + eff_beat(settings.glitch_intensity * 8.0)
    }
    ScreenEffect::Chromatic => {
      settings.chromatic_intensity * 0.3 + settings.chromatic_intensity * use_be * 0.5 + eff_beat(settings.chromatic_intensity)
    }
    ScreenEffect::Zoom => {
      (settings.zoom_intensity * 0.4 + settings.zoom_intensity * use_be * 0.5 + eff_beat(settings.zoom_intensity)).min(0.5)
    }
    ScreenEffect::Invert => {
      let amount = settings.invert_intensity * 0.35 + (settings.invert_intensity * use_be * 0.4 + eff_beat(settings.invert_intensity)) * 2.0;
      return if amount < 0.01 { None } else { Some(PostFx { mode, intensity: amount.min(1.0), time: ft, beat, fps }) };
    }
    ScreenEffect::Bars => {
      (settings.bars_amount * 0.4 + settings.bars_amount * use_be * 0.3 + eff_beat(settings.bars_amount)).min(0.5)
    }
    ScreenEffect::Shockwave => {
      let beat_high = beat > 0.15;
      if beat_high && !state.prev_beat_high {
        state.shock_start = ft * 1000.0;
      }
      state.prev_beat_high = beat_high;
      let elapsed = (ft * 1000.0 - state.shock_start) / 650.0;
      if elapsed >= 0.0 && elapsed < 1.0 {
        settings.shockwave_intensity * (1.0 - elapsed) * 1.1
      } else if settings.shockwave_intensity > 0.001 {
        // Preview ripple when audio is paused / between beats
        let preview_ripple = ((ft * 4.0).sin().abs() * 0.4 + 0.3) * settings.shockwave_intensity * 0.5;
        preview_ripple
      } else {
        0.0
      }
    }
    ScreenEffect::Pixelate => {
      settings.pixelate_intensity * 0.35 + settings.pixelate_intensity * use_be * 0.4 + eff_beat(settings.pixelate_intensity)
    }
    ScreenEffect::Tilt => {
      settings.tilt_intensity * 0.4 + settings.tilt_intensity * use_be * 0.4 + eff_beat(settings.tilt_intensity)
    }
    ScreenEffect::HeatHaze => {
      settings.heat_haze_intensity * 0.4 + settings.heat_haze_intensity * use_be * 0.3 + eff_beat(settings.heat_haze_intensity)
    }
    ScreenEffect::HueShift => {
      (settings.hue_shift_intensity * 0.4 + settings.hue_shift_intensity * use_be * 0.3 + eff_beat(settings.hue_shift_intensity)).min(0.9)
    }
    ScreenEffect::GlassCrack => {
      (settings.glass_crack_intensity * 0.4 + settings.glass_crack_intensity * use_be * 0.4 + eff_beat(settings.glass_crack_intensity * 0.8)).min(1.0)
    }
    _ => return None,
  };
  let threshold = 0.005;
  if intensity < threshold {
    return None;
  }
  Some(PostFx { mode, intensity, time: ft, beat, fps })
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
  // Time-based overlays (strobe) use the monotonic fx clock, not song time.
  let fx_time = ctx.state.screen_fx.fx_time;
  match settings.main_effect {
    ScreenEffect::Vignette => vignette(c, w, h, settings, use_be, beat),
    ScreenEffect::Pulse => pulse(c, w, h, settings, use_be, beat),
    ScreenEffect::Spotlight => spotlight(c, w, h, settings, use_be, beat),
    ScreenEffect::Strobe => strobe(c, w, h, settings, use_be, beat, fx_time),
    ScreenEffect::Scanline => scanline(c, w, h, settings, use_be, beat),
    // Shake is a translate (draw_frame); HueShift needs a snapshot (post_fx);
    // remaining effects need snapshots too.
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
  let beat_pulse = if beat > 0.15 { beat * settings.vignette_intensity * 2.5 } else { 0.0 };
  let pulse = use_be * settings.vignette_intensity * 0.5 + beat_pulse;
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
  let (r, g, b) = (base.r, base.g, base.b);

  c.set_blend_screen();

  // Soft light pool at the top behind the beams.
  c.set_fill(Fill::radial_gradient(
    w / 2.0,
    0.0,
    0.0,
    w / 2.0,
    0.0,
    w * 0.6,
    &[
      (0.0, Color::rgba(r, g, b, alpha * 0.22)),
      (1.0, Color::rgba(r, g, b, 0.0)),
    ],
  ));
  c.fill_ellipse(w / 2.0, 0.0, w * 0.6, w * 0.28);

  // Concert light cones descending from the top: narrow at the source, wide
  // at the floor, with outer beams leaning outward.
  for &fx in &[0.16f32, 0.5, 0.84] {
    let tilt = (fx - 0.5) * 0.14;
    let cx = w * fx + tilt * h;
    let top_half = w * 0.035;
    let bot_half = w * 0.17;

    c.set_fill(Fill::linear_gradient(
      cx,
      0.0,
      cx,
      h,
      &[
        (0.0, Color::rgba(r, g, b, alpha * 0.9)),
        (0.55, Color::rgba(r, g, b, alpha * 0.22)),
        (1.0, Color::rgba(r, g, b, 0.0)),
      ],
    ));
    c.fill_polygon(&[
      (cx - top_half, 0.0),
      (cx + top_half, 0.0),
      (cx + bot_half, h),
      (cx - bot_half, h),
    ]);

    // Bright source glow at the top of each beam.
    c.set_fill(Fill::radial_gradient(
      cx,
      0.0,
      0.0,
      cx,
      0.0,
      top_half * 2.6,
      &[
        (0.0, Color::rgba(r, g, b, alpha * 0.55)),
        (1.0, Color::rgba(r, g, b, 0.0)),
      ],
    ));
    c.fill_ellipse(cx, 0.0, top_half * 2.6, top_half * 1.4);
  }
  c.set_blend_normal();
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

// HueShift is rendered as post-fx mode 10 (gpu2d/postfx.wgsl): a 'hue'
// composite of a diagonal hue->hue+180 gradient, matching canvas applyHueShift.

// ---------------------------------------------------------------------------
// CPU fallback (render_frame_to_rgb) — mirrors the GPU pipeline on a raw RGB
// buffer so the software path shows the same screen effects. Shake and the
// frame-sampling effects reuse `post_fx` / `compute_shake_offset` with a
// process-wide fx state so timing (shockwave, shake buckets) stays consistent
// across frames without a RenderState.
// ---------------------------------------------------------------------------

pub struct CpuEnvelope {
  pub above_floor: f32,
  pub beat: f32,
}

/// Minimal export-style envelope for the CPU fallback: bass energy vs a slow
/// floor plus onset-based beat, mirroring `advance_envelope` without a
/// `RenderState`.
pub fn cpu_envelope(freq: &[u8]) -> CpuEnvelope {
  let bins = 16.min(freq.len());
  let mut sum = 0usize;
  for i in 0..bins {
    sum += freq[i] as usize;
  }
  let raw = if bins > 0 { sum as f32 / (bins as f32 * 255.0) } else { 0.0 };

  // (bass_floor, prev_target_bass, beat_strength)
  static ENV: std::sync::Mutex<(f32, f32, f32)> = std::sync::Mutex::new((0.0, 0.0, 0.0));
  let mut g = ENV.lock().unwrap_or_else(|e| e.into_inner());
  let (mut floor, prev_target, prev_beat) = *g;
  if raw < floor {
    floor = raw;
  } else {
    floor += (raw - floor) * 0.0008;
  }
  let above_floor = (raw - floor).max(0.0);
  let onset = (raw - prev_target).max(0.0);
  let beat = if onset > 0.03 { (onset * 6.0).max(prev_beat * 0.6) } else { prev_beat * 0.7 };
  *g = (floor, raw, beat);
  CpuEnvelope { above_floor: above_floor.max(0.0), beat }
}

/// Process-wide screen-fx state for the CPU fallback (no RenderState there).
fn cpu_fx_state_mut() -> std::sync::MutexGuard<'static, ScreenFxState> {
  static STATE: std::sync::Mutex<ScreenFxState> = std::sync::Mutex::new(ScreenFxState::new());
  STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Apply the selected screen effect to a software-rendered RGB frame.
pub fn apply_cpu_screen_effects(
  rgb: &mut Vec<u8>,
  width: u32,
  height: u32,
  settings: &ScreenEffectsSettings,
  above_floor: f32,
  beat: f32,
  fx_time: f32,
) {
  if !settings.enabled {
    return;
  }
  match settings.main_effect {
    ScreenEffect::Shake => cpu_shake(rgb, width, height, settings, above_floor, beat, fx_time),
    ScreenEffect::Vignette => cpu_vignette(rgb, width, height, settings, above_floor, beat),
    ScreenEffect::Pulse => cpu_pulse(rgb, width, height, settings, above_floor, beat),
    ScreenEffect::Spotlight => cpu_spotlight(rgb, width, height, settings, above_floor, beat),
    ScreenEffect::Strobe => cpu_strobe(rgb, width, height, settings, above_floor, beat, fx_time),
    ScreenEffect::Scanline => cpu_scanline(rgb, width, height, settings, above_floor, beat),
    _ => cpu_post_fx(rgb, width, height, settings, above_floor, beat, fx_time),
  }
}

#[inline]
fn blend_alpha(src: u8, overlay: u8, a: f32) -> u8 {
  (src as f32 * (1.0 - a) + overlay as f32 * a) as u8
}

#[inline]
fn cpu_sample(src: &[u8], w: u32, x: u32, y: u32) -> [u8; 3] {
  let idx = ((y * w + x) * 3) as usize;
  [src[idx], src[idx + 1], src[idx + 2]]
}

#[inline]
fn cpu_sample_f(src: &[u8], w: u32, h: u32, x: f32, y: f32) -> [u8; 3] {
  let xi = x.floor().clamp(0.0, (w - 1) as f32) as u32;
  let yi = y.floor().clamp(0.0, (h - 1) as f32) as u32;
  cpu_sample(src, w, xi, yi)
}

fn cpu_shake(
  rgb: &mut Vec<u8>,
  w: u32,
  h: u32,
  settings: &ScreenEffectsSettings,
  above_floor: f32,
  beat: f32,
  fx_time: f32,
) {
  let mut st = cpu_fx_state_mut();
  st.fx_time = fx_time;
  let (dx, dy) = compute_shake_offset(&mut st, settings, above_floor, beat);
  if dx.abs() < 0.5 && dy.abs() < 0.5 {
    return;
  }
  let src = rgb.clone();
  for y in 0..h {
    let sy = (y as f32 - dy).round() as i32;
    let sy = sy.clamp(0, h as i32 - 1) as u32;
    for x in 0..w {
      let sx = (x as f32 - dx).round() as i32;
      let sx = sx.clamp(0, w as i32 - 1) as u32;
      let idx = ((y * w + x) * 3) as usize;
      let s = cpu_sample(&src, w, sx, sy);
      rgb[idx] = s[0];
      rgb[idx + 1] = s[1];
      rgb[idx + 2] = s[2];
    }
  }
}

fn cpu_vignette(
  rgb: &mut Vec<u8>,
  w: u32,
  h: u32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
) {
  let beat_pulse = if beat > 0.15 { beat * settings.vignette_intensity * 2.5 } else { 0.0 };
  let pulse = use_be * settings.vignette_intensity * 0.5 + beat_pulse;
  let wf = w as f32;
  let hf = h as f32;
  let cx = wf / 2.0;
  let cy = hf / 2.0;
  let max_radius = (wf * wf + hf * hf).sqrt() / 2.0;
  let radius = max_radius * (0.5 + pulse * 0.3).max(0.2);
  let alpha = (0.4 + pulse * 0.4).clamp(0.0, 1.0);
  let inner = radius * 0.6;
  for y in 0..h {
    for x in 0..w {
      let dx = x as f32 - cx;
      let dy = y as f32 - cy;
      let d = (dx * dx + dy * dy).sqrt();
      if d > inner {
        let a = alpha * ((d - inner) / (radius - inner).max(1e-6)).clamp(0.0, 1.0);
        if a > 0.01 {
          let idx = ((y * w + x) * 3) as usize;
          rgb[idx] = blend_alpha(rgb[idx], 0, a);
          rgb[idx + 1] = blend_alpha(rgb[idx + 1], 0, a);
          rgb[idx + 2] = blend_alpha(rgb[idx + 2], 0, a);
        }
      }
    }
  }
}

fn cpu_pulse(
  rgb: &mut Vec<u8>,
  _w: u32,
  _h: u32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
) {
  let b = if beat > 0.15 { beat * settings.pulse_intensity } else { 0.0 };
  let smooth = use_be * settings.pulse_intensity * 0.15;
  let alpha = (smooth + b).min(1.0);
  if alpha < 0.01 {
    return;
  }
  for px in rgb.chunks_mut(3) {
    px[0] = blend_alpha(px[0], 255, alpha);
    px[1] = blend_alpha(px[1], 255, alpha);
    px[2] = blend_alpha(px[2], 255, alpha);
  }
}

fn cpu_spotlight(
  rgb: &mut Vec<u8>,
  w: u32,
  h: u32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
) {
  let pulse = 0.5 + use_be * 0.3 + if beat > 0.15 { beat * 0.3 } else { 0.0 };
  let alpha = (0.6 * pulse).min(1.0);
  let base = Color::hex(&settings.spotlight_color);
  let (cr, cg, cb) = (base.r * 255.0, base.g * 255.0, base.b * 255.0);
  let wf = w as f32;
  let hf = h as f32;

  // Concert light cones from the top (mirror of the GPU spotlight), screen
  // blended over the frame.
  let beams: [(f32, f32); 3] = [(0.16, -0.034), (0.5, 0.0), (0.84, 0.034)];
  for y in 0..h {
    let yf = y as f32;
    let ty = yf / hf;
    let beam_alpha = if ty <= 0.55 {
      alpha * (0.9 + (0.22 - 0.9) * (ty / 0.55))
    } else {
      alpha * 0.22 * (1.0 - (ty - 0.55) / 0.45)
    };
    for x in 0..w {
      let xf = x as f32;
      let mut sr = 0.0f32;
      let mut sg = 0.0f32;
      let mut sb = 0.0f32;

      // Soft light pool behind the beams (ellipse centered (wf/2, 0)).
      let ex = (xf - wf / 2.0) / (wf * 0.6);
      let ey = yf / (wf * 0.28);
      if ex * ex + ey * ey <= 1.0 {
        let d = ((xf - wf / 2.0).powi(2) + yf * yf).sqrt();
        let a = alpha * 0.22 * (1.0 - (d / (wf * 0.6)).min(1.0));
        sr += cr * a;
        sg += cg * a;
        sb += cb * a;
      }

      // Light cones: narrow at the top source, wide at the floor.
      for (fx, tilt) in beams {
        let cx = wf * fx + tilt * hf;
        let top_half = wf * 0.035;
        let bot_half = wf * 0.17;
        let half_w = top_half + (bot_half - top_half) * ty;
        if (xf - cx).abs() <= half_w {
          sr += cr * beam_alpha;
          sg += cg * beam_alpha;
          sb += cb * beam_alpha;
        }
      }

      let total = sr.max(sg).max(sb);
      if total > 0.01 {
        let a = total.min(1.0);
        let idx = ((y * w + x) * 3) as usize;
        // Screen blend: out = src + dst * (1 - src).
        let src_r = (sr / total.max(1e-4)).min(1.0);
        let src_g = (sg / total.max(1e-4)).min(1.0);
        let src_b = (sb / total.max(1e-4)).min(1.0);
        let dst = [rgb[idx] as f32 / 255.0, rgb[idx + 1] as f32 / 255.0, rgb[idx + 2] as f32 / 255.0];
        let s = [src_r, src_g, src_b];
        for k in 0..3 {
          let out = s[k] * a + dst[k] * (1.0 - s[k] * a);
          rgb[idx + k] = (out.clamp(0.0, 1.0) * 255.0) as u8;
        }
      }
    }
  }
}

fn cpu_strobe(
  rgb: &mut Vec<u8>,
  _w: u32,
  _h: u32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
  fx_time: f32,
) {
  let b = if beat > 0.15 { settings.strobe_intensity * 0.9 } else { 0.0 };
  let smooth = settings.strobe_intensity * use_be * 0.12;
  let alpha = (smooth + b).min(0.9);
  if alpha < 0.02 {
    return;
  }
  let on = ((fx_time * 10.0).floor() as i64) % 2 == 0;
  if !on {
    return;
  }
  for px in rgb.chunks_mut(3) {
    px[0] = blend_alpha(px[0], 255, alpha);
    px[1] = blend_alpha(px[1], 255, alpha);
    px[2] = blend_alpha(px[2], 255, alpha);
  }
}

fn cpu_scanline(
  rgb: &mut Vec<u8>,
  w: u32,
  h: u32,
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
  let line_a = opacity.min(0.6);
  let mut y = 0u32;
  while y < h {
    let idx0 = (y * w * 3) as usize;
    for x in 0..w {
      let idx = idx0 + x as usize * 3;
      rgb[idx] = blend_alpha(rgb[idx], 0, line_a);
      rgb[idx + 1] = blend_alpha(rgb[idx + 1], 0, line_a);
      rgb[idx + 2] = blend_alpha(rgb[idx + 2], 0, line_a);
    }
    y += 4;
  }
  if darken > 0.01 {
    let a = darken.min(0.35);
    for px in rgb.chunks_mut(3) {
      px[0] = blend_alpha(px[0], 0, a);
      px[1] = blend_alpha(px[1], 0, a);
      px[2] = blend_alpha(px[2], 0, a);
    }
  }
}

fn cpu_post_fx(
  rgb: &mut Vec<u8>,
  w: u32,
  h: u32,
  settings: &ScreenEffectsSettings,
  use_be: f32,
  beat: f32,
  fx_time: f32,
) {
  let mut st = cpu_fx_state_mut();
  st.fx_time = fx_time;
  let fx = match post_fx(&mut st, settings, use_be, beat, 60.0) {
    Some(f) => f,
    None => return,
  };
  let shock_start = st.shock_start;
  drop(st);

  let wf = w as f32;
  let hf = h as f32;
  let cx = wf / 2.0;
  let cy = hf / 2.0;
  let intensity = fx.intensity;
  let t = fx.time;

  let src = rgb.clone();
  let src = src.as_slice();

  match fx.mode {
    1 => {
      // glitch: horizontal slice displacement + occasional hue channel offset.
      let amp = intensity * wf * 0.16;
      for y in 0..h {
        let yf = y as f32;
        let is_shift = ((yf * 0.7 + t * 11.0).sin() * 3.0).abs() > 2.1;
        let shift = if is_shift { ((yf * 91.7).sin() * amp) as i32 } else { 0 };
        for x in 0..w {
          let sx = (x as i32 - shift).clamp(0, w as i32 - 1) as u32;
          let idx = ((y * w + x) * 3) as usize;
          let s = cpu_sample(src, w, sx, y);
          rgb[idx] = s[0];
          rgb[idx + 1] = s[1];
          rgb[idx + 2] = s[2];
        }
      }
    }
    2 => {
      // chromatic: red/blue split ghosts.
      let amp = wf * (0.004 + intensity * 0.012 * (t * 18.0).sin().abs());
      for y in 0..h {
        for x in 0..w {
          let idx = ((y * w + x) * 3) as usize;
          let rp = cpu_sample_f(src, w, h, x as f32 + amp, y as f32);
          let bp = cpu_sample_f(src, w, h, x as f32 - amp, y as f32);
          let gp = cpu_sample(src, w, x, y);
          rgb[idx] = rp[0];
          rgb[idx + 1] = gp[1];
          rgb[idx + 2] = bp[2];
        }
      }
    }
    3 => {
      // zoom: pull the frame inward from the center.
      let inv = 1.0 / (1.0 + intensity * 0.25);
      for y in 0..h {
        for x in 0..w {
          let ux = (x as f32 - cx) * inv + cx;
          let uy = (y as f32 - cy) * inv + cy;
          let idx = ((y * w + x) * 3) as usize;
          let s = cpu_sample_f(src, w, h, ux, uy);
          rgb[idx] = s[0];
          rgb[idx + 1] = s[1];
          rgb[idx + 2] = s[2];
        }
      }
    }
    4 => {
      // invert: blend toward the inverted frame.
      let m = intensity.min(1.0);
      for px in rgb.chunks_mut(3) {
        px[0] = blend_alpha(px[0], 255 - px[0], m);
        px[1] = blend_alpha(px[1], 255 - px[1], m);
        px[2] = blend_alpha(px[2], 255 - px[2], m);
      }
    }
    5 => {
      // bars: black letterbox bands from top & bottom.
      let amount = intensity * hf * 0.5;
      for y in 0..h {
        let yf = y as f32;
        let d = if yf < amount {
          amount - yf
        } else if yf > hf - amount {
          yf - (hf - amount)
        } else {
          0.0
        };
        if d > 0.0 {
          let a = (d / amount.max(1e-6)).clamp(0.0, 1.0);
          let idx0 = (y * w * 3) as usize;
          for x in 0..w {
            let idx = idx0 + x as usize * 3;
            rgb[idx] = blend_alpha(rgb[idx], 0, a);
            rgb[idx + 1] = blend_alpha(rgb[idx + 1], 0, a);
            rgb[idx + 2] = blend_alpha(rgb[idx + 2], 0, a);
          }
        }
      }
    }
    6 => {
      // shockwave: expanding ripple ring from the center.
      let elapsed = (t * 1000.0 - shock_start) / 650.0;
      if elapsed >= 0.0 && elapsed < 1.0 {
        let maxd = (wf * wf + hf * hf).sqrt() / 2.0;
        let ring_r = elapsed * maxd * 0.9;
        let amp = intensity * 40.0 * (1.0 - elapsed);
        for y in 0..h {
          for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let ripple = ((d - ring_r) * 0.12).sin() * amp;
            let nd = (d + ripple).max(0.0);
            let idx = ((y * w + x) * 3) as usize;
            if nd == d {
              continue;
            }
            let ang = dy.atan2(dx);
            let ux = cx + ang.cos() * nd;
            let uy = cy + ang.sin() * nd;
            let s = cpu_sample_f(src, w, h, ux, uy);
            rgb[idx] = s[0];
            rgb[idx + 1] = s[1];
            rgb[idx + 2] = s[2];
          }
        }
      }
    }
    7 => {
      // pixelate: chunky blocks.
      let block = (1.0 + intensity * 24.0).max(1.0) as u32;
      for y in 0..h {
        let by = (y / block) * block;
        for x in 0..w {
          let bx = (x / block) * block;
          let idx = ((y * w + x) * 3) as usize;
          let s = cpu_sample(src, w, bx, by);
          rgb[idx] = s[0];
          rgb[idx + 1] = s[1];
          rgb[idx + 2] = s[2];
        }
      }
    }
    8 => {
      // tilt: rotate the frame back & forth around the center.
      let ang = (t * 0.5).sin() * intensity * 0.15;
      let (s, c) = ang.sin_cos();
      for y in 0..h {
        for x in 0..w {
          let dx = x as f32 - cx;
          let dy = y as f32 - cy;
          let ux = c * dx + s * dy + cx;
          let uy = -s * dx + c * dy + cy;
          let idx = ((y * w + x) * 3) as usize;
          let s = cpu_sample_f(src, w, h, ux, uy);
          rgb[idx] = s[0];
          rgb[idx + 1] = s[1];
          rgb[idx + 2] = s[2];
        }
      }
    }
    9 => {
      // heat haze: vertical sine-wave strip displacement.
      for y in 0..h {
        for x in 0..w {
          let uy = y as f32 + (x as f32 * 0.012 + t * 5.0).sin() * intensity * hf * 0.02;
          let idx = ((y * w + x) * 3) as usize;
          let s = cpu_sample_f(src, w, h, x as f32, uy);
          rgb[idx] = s[0];
          rgb[idx + 1] = s[1];
          rgb[idx + 2] = s[2];
        }
      }
    }
    10 => {
      // hue shift: drift the frame's hue.
      let drift = t * 0.15 * intensity;
      for px in rgb.chunks_mut(3) {
        let (mut h, s, v) = rgb_to_hsv(px[0], px[1], px[2]);
        h = (h + drift) % 360.0;
        let (r, g, b) = hsv_to_rgb(h, s, v);
        px[0] = r;
        px[1] = g;
        px[2] = b;
      }
    }
    11 => {
      // glass crack: sharp Voronoi polygonal glass shard fractures & razor-thin specular line highlights.
      let amp = intensity * wf * 0.035;
      let grid_scale = 4.5;
      let aspect = wf / hf;

      for y in 0..h {
        let yf = y as f32;
        let uv_y = yf / hf;
        for x in 0..w {
          let xf = x as f32;
          let uv_x = xf / wf;

          let px = uv_x * aspect * grid_scale;
          let py = uv_y * grid_scale;

          let cell_x = px.floor() as i32;
          let cell_y = py.floor() as i32;
          let fx = px.fract();
          let fy = py.fract();

          let mut min_d1 = 8.0f32;
          let mut min_d2 = 8.0f32;
          let mut best_cell_id = 0.0f32;

          for gy in -1..=1 {
            for gx in -1..=1 {
              let cx = cell_x + gx;
              let cy = cell_y + gy;
              // Deterministic hash for seed point inside grid cell
              let seed = ((cx as f32 * 12.9898 + cy as f32 * 78.233).sin() * 43758.5453).fract().abs();
              let seed2 = ((cx as f32 * 39.346 + cy as f32 * 11.135).sin() * 23421.631).fract().abs();
              let rx = gx as f32 + seed - fx;
              let ry = gy as f32 + seed2 - fy;
              let d = rx * rx + ry * ry;

              if d < min_d1 {
                min_d2 = min_d1;
                min_d1 = d;
                best_cell_id = seed;
              } else if d < min_d2 {
                min_d2 = d;
              }
            }
          }

          let edge_dist = (min_d2.sqrt() - min_d1.sqrt()).abs();
          let crack_threshold = 0.04;
          let idx = ((y * w + x) * 3) as usize;

          if edge_dist < crack_threshold {
            let hl = ((1.0 - edge_dist / crack_threshold).clamp(0.0, 1.0) * intensity * 240.0) as u8;
            rgb[idx] = rgb[idx].saturating_add(hl);
            rgb[idx + 1] = rgb[idx + 1].saturating_add(hl);
            rgb[idx + 2] = rgb[idx + 2].saturating_add(hl);
          } else {
            let shift_x = (best_cell_id * 100.0).sin() * amp;
            let shift_y = (best_cell_id * 43.0).cos() * amp;
            let ux = (xf + shift_x).clamp(0.0, wf - 1.0);
            let uy = (yf + shift_y).clamp(0.0, hf - 1.0);
            let s = cpu_sample_f(src, w, h, ux, uy);
            rgb[idx] = s[0];
            rgb[idx + 1] = s[1];
            rgb[idx + 2] = s[2];
          }
        }
      }
    }
    _ => {}
  }
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
  let r = r as f32 / 255.0;
  let g = g as f32 / 255.0;
  let b = b as f32 / 255.0;
  let max = r.max(g).max(b);
  let min = r.min(g).min(b);
  let d = max - min;
  let h = if d == 0.0 {
    0.0
  } else if max == r {
    60.0 * (((g - b) / d) % 6.0)
  } else if max == g {
    60.0 * ((b - r) / d + 2.0)
  } else {
    60.0 * ((r - g) / d + 4.0)
  };
  let h = if h < 0.0 { h + 360.0 } else { h };
  (h, if max == 0.0 { 0.0 } else { d / max }, max)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
  let c = v * s;
  let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
  let m = v - c;
  let (r, g, b) = match ((h / 60.0).floor() as i32).rem_euclid(6) {
    0 => (c, x, 0.0),
    1 => (x, c, 0.0),
    2 => (0.0, c, x),
    3 => (0.0, x, c),
    4 => (x, 0.0, c),
    _ => (c, 0.0, x),
  };
  (((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8)
}
