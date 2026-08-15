//! Rust port of the visualizer draw loop (export path).
//! Mirrors `src/services/canvasRenderer.ts` drawFrame + the style renderers.
//! Phase 4/5: frequency/time-domain renderers + complex particle & 3D-style
//! renderers. Background/screen effects are Phase 6 (structured hooks below).

pub mod background;
pub mod helpers;
pub mod screen_effects;
pub mod three_d_engine;
mod styles;

// Re-exported so tests can verify slider parity without exposing `styles`.
pub use styles::minimal::effective_bar_count;
pub mod text;

use crate::config::{
  AudioReactivitySettings, BackgroundEffect, BackgroundFillType, ColorTheme, VisualizerConfig,
};
use crate::gpu2d::{Color, Fill, GpuCanvas, Scene3D};

// ---------------------------------------------------------------------------
// Deterministic RNG (mulberry32) so exports are reproducible per session.
// ---------------------------------------------------------------------------

pub struct Rng(u32);

impl Rng {
  pub fn new(seed: u32) -> Self {
    Rng(seed)
  }

  pub fn next(&mut self) -> f32 {
    self.0 = self.0.wrapping_add(0x6D2B79F5);
    let mut t = self.0;
    t = (t ^ (t >> 15)).wrapping_mul(0x1B873593);
    t = (t ^ (t >> 13)).wrapping_mul(0x5E0F8C7D);
    t ^= t >> 16;
    (t & 0x00FF_FFFF) as f32 / 16777216.0
  }
}

// ---------------------------------------------------------------------------
// Per-frame / per-session state (reset per export, mirrors TS class state).
// ---------------------------------------------------------------------------

pub struct VuChannel {
  pub level: f32,
  pub peak: f32,
  pub peak_hold: f32,
}

impl Default for VuChannel {
  fn default() -> Self {
    VuChannel { level: 0.0, peak: 0.0, peak_hold: 0.0 }
  }
}

pub struct PulseRing {
  pub radius: f32,
  pub max_radius: f32,
  pub alpha: f32,
  pub speed: f32,
  pub thickness: f32,
  pub color: Color,
}

pub struct RenderState {
  pub peak_data: Vec<f32>,
  pub vu: [VuChannel; 2],
  pub rings: Vec<PulseRing>,
  pub prev_beat: f32,
  pub aurora_t: f32,
  pub rotation_angle: f32,
  // export-mode envelope (canvasRenderer drawFrame)
  pub bass_energy: f32,
  pub bass_energy_raw: f32,
  pub bass_floor: f32,
  pub prev_target_bass: f32,
  pub prev_raw_bass: f32,
  pub beat_strength: f32,
  pub beat_strength_raw: f32,
  /// Count of detected beat onsets; styles use it to pick a new pseudo-random
  /// emphasis angle on every beat instead of advancing in a fixed order.
  pub beat_count: u64,
  pub rng: Rng,
  pub advanced: helpers::AdvancedState,
  /// Custom background image uploaded once to a persistent atlas layer.
  /// `w`/`h` are the scaled layer-space dimensions (for UV mapping).
  pub background_image: Option<BackgroundImage>,
  pub radial_center_image: Option<BackgroundImage>,
  pub stars: Vec<background::Star>,
  pub particles: Vec<background::Particle>,
  pub music_notes: Vec<background::MusicNote>,
  pub screen_fx: screen_effects::ScreenFxState,
  /// Text fade-in state, mirroring the module-level `playStartFrame` /
  /// `wasPlaying` in `src/services/renderers/textOverlay.ts`. Persisted across
  /// preview frames (like the rest of RenderState) so `Fade In on Play` restarts
  /// exactly when playback starts — and shows fully when paused.
  pub text_play_start_frame: f32,
  pub text_was_playing: bool,
}

/// A custom background image pre-uploaded into atlas layer `layer`.
pub struct BackgroundImage {
  pub layer: u32,
  pub w: u32,
  pub h: u32,
}

impl RenderState {
  pub fn new(bar_count: usize, seed: u32) -> Self {
    RenderState {
      peak_data: vec![0.0; bar_count],
      vu: [
        VuChannel { level: 0.0, peak: 0.0, peak_hold: 0.0 },
        VuChannel { level: 0.0, peak: 0.0, peak_hold: 0.0 },
      ],
      rings: Vec::new(),
      prev_beat: 0.0,
      aurora_t: 0.0,
      rotation_angle: 0.0,
      bass_energy: 0.0,
      bass_energy_raw: 0.0,
      bass_floor: 0.0,
      prev_target_bass: 0.0,
      prev_raw_bass: 0.0,
      beat_strength: 0.0,
      beat_strength_raw: 0.0,
      beat_count: 0,
      rng: Rng::new(seed),
      advanced: helpers::AdvancedState::default(),
      background_image: None,
      radial_center_image: None,
      stars: background::build_stars(),
      particles: background::init_particles(&mut Rng::new(seed.wrapping_add(0x1234))),
      // Start empty like the TS renderer (notes are spawned on beats); avoids
      // an initial batch stuck at fractional (0..1) pixel coordinates.
      music_notes: Vec::new(),
      screen_fx: screen_effects::ScreenFxState::new(),
      text_play_start_frame: 0.0,
      text_was_playing: false,
    }
  }
}

// ---------------------------------------------------------------------------
// RenderContext (mirror of src/services/renderers/types.ts)
// ---------------------------------------------------------------------------

pub struct RenderContext<'a> {
  pub width: f32,
  pub height: f32,
  pub config: &'a VisualizerConfig,
  pub freq_data: &'a [u8],
  pub time_data: &'a [u8],
  pub bass_energy: f32,
  pub beat_strength: f32,
  pub beat_count: u64,
  pub rotation_angle: f32,
  pub frame_time: f32,
  pub state: &'a mut RenderState,
  /// Native 3D scene (see `crate::gpu2d::scene3d`). Styles that support true
  /// 3D push geometry here; `GpuRenderer` draws it in a depth-tested pass
  /// right after the 2D canvas.
  pub scene3d: &'a mut Scene3D,
}

// ---------------------------------------------------------------------------
// Theme / color helpers
// ---------------------------------------------------------------------------

pub fn theme_primary(theme: &ColorTheme) -> Color {
  Color::hex(&theme.primary_color)
}

pub fn theme_secondary(theme: &ColorTheme) -> Color {
  Color::hex(&theme.secondary_color)
}

pub fn theme_accent(theme: &ColorTheme) -> Color {
  Color::hex(&theme.accent_color)
}

pub fn theme_glow(theme: &ColorTheme) -> Color {
  Color::hex(&theme.glow_color)
}

/// HSL -> RGB (s, l in 0..1, h in degrees). Mirrors CSS `hsla()`.
pub fn hsl_to_color(h: f32, s: f32, l: f32, a: f32) -> Color {
  let h = ((h % 360.0) + 360.0) % 360.0;
  let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
  let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
  let m = l - c / 2.0;
  let (r, g, b) = match (h / 60.0).floor() as u32 {
    0 => (c, x, 0.0),
    1 => (x, c, 0.0),
    2 => (0.0, c, x),
    3 => (0.0, x, c),
    4 => (x, 0.0, c),
    _ => (c, 0.0, x),
  };
  Color::rgba(r + m, g + m, b + m, a)
}

pub fn bin_value(freq: &[u8], step: usize, idx: usize) -> f32 {
  let mut sum = 0usize;
  let mut n = 0;
  for j in 0..step {
    let k = idx * step + j;
    if k < freq.len() {
      sum += freq[k] as usize;
      n += 1;
    }
  }
  if n == 0 {
    return 0.0;
  }
  sum as f32 / (n as f32 * 255.0)
}

// ---------------------------------------------------------------------------
// Background (Phase 4 subset: solid/gradient + overlay; effects Phase 6)
// ---------------------------------------------------------------------------

pub fn get_active_effects(bg: &crate::config::BackgroundSettings) -> Vec<BackgroundEffect> {
  let mut active: Vec<BackgroundEffect> = Vec::new();
  if let Some(effects) = &bg.effects {
    if !effects.is_empty() {
      active = effects.clone();
    }
  }
  if active.is_empty() {
    if let Some(effect) = &bg.effect {
      if !matches!(effect, BackgroundEffect::None) {
        active.push(effect.clone());
      }
    }
  }
  if active.is_empty() {
    use crate::config::BackgroundMode;
    let from_mode = match bg.mode {
      BackgroundMode::Grid => Some(BackgroundEffect::Grid),
      BackgroundMode::Aurora => Some(BackgroundEffect::Aurora),
      BackgroundMode::Noise => Some(BackgroundEffect::Noise),
      BackgroundMode::Bokeh => Some(BackgroundEffect::Bokeh),
      BackgroundMode::Starfield => Some(BackgroundEffect::Starfield),
      BackgroundMode::Nebula => Some(BackgroundEffect::Nebula),
      BackgroundMode::Psychedelic => Some(BackgroundEffect::Psychedelic),
      _ => None,
    };
    if let Some(effect) = from_mode {
      active.push(effect);
    }
  }
  active
}

fn draw_background(c: &mut GpuCanvas, ctx: &RenderContext, margin: f32) {
  let bg = &ctx.config.background;
  let fill_type = bg.fill_type.as_ref().cloned().unwrap_or_else(|| {
    if matches!(bg.mode, crate::config::BackgroundMode::Gradient) {
      BackgroundFillType::Gradient
    } else {
      BackgroundFillType::Solid
    }
  });
  match fill_type {
    BackgroundFillType::Gradient => {
      let g_start = if bg.gradient_start.trim().is_empty() { "#0f0c20" } else { bg.gradient_start.as_str() };
      let g_end = if bg.gradient_end.trim().is_empty() { "#06101e" } else { bg.gradient_end.as_str() };
      let g = Fill::linear_gradient(0.0, 0.0, ctx.width, ctx.height, &[
        (0.0, Color::hex(g_start)),
        (1.0, Color::hex(g_end)),
      ]);
      c.set_fill(g);
      c.fill_rect(-margin, -margin, ctx.width + margin * 2.0, ctx.height + margin * 2.0);
    }
    BackgroundFillType::Solid => {
      let solid = if bg.solid_color.trim().is_empty() { "#0b0c10" } else { bg.solid_color.as_str() };
      c.set_fill(Fill::Solid(Color::hex(solid)));
      c.fill_rect(-margin, -margin, ctx.width + margin * 2.0, ctx.height + margin * 2.0);
    }
  }

  // Custom background image (cover-fit, mirrors canvas drawCoverImage).
  if let Some(img) = &ctx.state.background_image {
    let default_opacity = if matches!(bg.mode, crate::config::BackgroundMode::CustomImage) { 1.0 } else { 0.7 };
    let raw_op = bg.image_opacity.unwrap_or(default_opacity);
    let alpha = (if raw_op > 1.0 { raw_op / 100.0 } else { raw_op }).clamp(0.0, 1.0);
    let (iw, ih) = (img.w as f32, img.h as f32);
    let img_ratio = iw / ih;
    let canvas_ratio = ctx.width / ctx.height;
    let (mut rw, mut rh, mut ox, mut oy) = (ctx.width, ctx.height, 0.0, 0.0);
    if img_ratio > canvas_ratio {
      rw = ctx.height * img_ratio;
      ox = (ctx.width - rw) / 2.0;
    } else {
      rh = ctx.width / img_ratio;
      oy = (ctx.height - rh) / 2.0;
    }
    let blur = bg.blur_amount.max(0.0);
    // TS drawCoverImage adds blur*2 padding to avoid edge artifacts: pad = margin + (blur > 0 ? blur * 2 : 0)
    let pad = margin + if blur > 0.0 { blur * 2.0 } else { 0.0 };
    // The image lives in a dedicated native-resolution texture (see
    // GpuRenderer::upload_background_image), so the quad UVs span the full
    // [0,1]^2 — NOT [0, img.w/LAYER_SIZE] which would sample only part of it.
    let full_uv = [0.0, 0.0, 1.0, 1.0];
    if blur > 0.0 {
      // Approximate the browser's Gaussian `blur(px)` filter (sigma = blur/2)
      // with a 5x5 weighted offset kernel so the exported background image
      // matches the TS preview. The old flat 3x3 box read as chunky and
      // inconsistent with canvas filter: blur().
      let step = (blur * 0.1).clamp(0.5, 2.5);
      let mut total = 0.0f64;
      let mut w = [[0.0f64; 5]; 5];
      for dy in 0..5 {
        for dx in 0..5 {
          let u = dx as f64 - 2.0;
          let v = dy as f64 - 2.0;
          w[dy][dx] = (-(u * u + v * v) / 2.0).exp();
          total += w[dy][dx];
        }
      }
      for dy in 0..5 {
        for dx in 0..5 {
          let k = w[dy][dx] / total;
          let effective_alpha = (alpha as f64).clamp(0.0001, 0.9999);
          let quad_alpha = (1.0 - (1.0 - effective_alpha).powf(k)).clamp(0.0, 1.0) as f32;
          let ox = ox + (dx as f32 - 2.0) * step;
          let oy = oy + (dy as f32 - 2.0) * step;
          c.push_textured_quad(
            img.layer,
            ox - pad,
            oy - pad,
            rw + pad * 2.0,
            rh + pad * 2.0,
            full_uv,
            Color::rgba(1.0, 1.0, 1.0, quad_alpha),
          );
        }
      }
    } else {
      c.push_textured_quad(
        img.layer,
        ox - pad,
        oy - pad,
        rw + pad * 2.0,
        rh + pad * 2.0,
        full_uv,
        Color::rgba(1.0, 1.0, 1.0, alpha),
      );
    }
  }

  // Overlay visual effects (grid/aurora/noise/...).
  let active = get_active_effects(bg);
  for effect in active {
    match effect {
      BackgroundEffect::Grid => background::render_grid(c, ctx),
      BackgroundEffect::Aurora => background::render_aurora(c, ctx),
      BackgroundEffect::Noise => background::render_noise(c, ctx),
      BackgroundEffect::Bokeh => background::render_bokeh(c, ctx),
      BackgroundEffect::Starfield => background::render_starfield(c, ctx, &ctx.state.stars),
      BackgroundEffect::Nebula => background::render_nebula(c, ctx),
      BackgroundEffect::Psychedelic => background::render_psychedelic(c, ctx),
      BackgroundEffect::None | BackgroundEffect::Particles | BackgroundEffect::MusicNotes => {}
    }
  }
}

// ---------------------------------------------------------------------------
// Style dispatch
// ---------------------------------------------------------------------------

pub fn render_style(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  styles::render_style(&ctx.config.style, c, ctx);
}

// ---------------------------------------------------------------------------
// Full frame (export-mode drawFrame)
// ---------------------------------------------------------------------------

/// Per-frame audio envelope + shake + fade values shared by the background
/// and foreground passes (mirrors the local variables in canvasRenderer
/// drawFrame). `advance_envelope` advances the state EXACTLY ONCE per frame;
/// the passes below only read the returned values.
pub struct FrameEnvelope {
  pub bass_energy: f32,
  pub beat_strength: f32,
  pub beat_count: u64,
  pub above_floor: f32,
  pub global_fade: f32,
  pub shake_x: f32,
  pub shake_y: f32,
  pub bg_shake_x: f32,
  pub bg_shake_y: f32,
  pub shake_margin: f32,
  /// Monotonic clock for time-based screen effects (see ScreenFxState.fx_time).
  pub fx_time: f32,
}

pub fn advance_envelope(
  state: &mut RenderState,
  config: &VisualizerConfig,
  freq: &[u8],
  frame_time: f32,
  is_playing: bool,
  fps: f32,
) -> FrameEnvelope {
  let react: &AudioReactivitySettings = &config.reactivity;

  // --- export-mode envelope (canvasRenderer drawFrame) ---
  let bass_bins = 16.min(freq.len());
  let mut bass_sum = 0usize;
  for i in 0..bass_bins {
    bass_sum += freq[i] as usize;
  }
  let raw_bass = if bass_bins > 0 { bass_sum as f32 / (bass_bins as f32 * 255.0) } else { 0.0 };
  let target_bass = raw_bass * react.bass_multiplier * react.sensitivity;

  state.bass_energy += (target_bass - state.bass_energy) * 0.2;
  state.bass_energy_raw += (raw_bass - state.bass_energy_raw) * 0.2;

  if target_bass < state.bass_floor {
    state.bass_floor = target_bass;
  } else {
    state.bass_floor += (target_bass - state.bass_floor) * 0.0008;
  }
  let above_floor = (state.bass_energy - state.bass_floor).max(0.0);

  let onset = (target_bass - state.prev_target_bass).max(0.0);
  state.prev_target_bass = target_bass;
  if onset > 0.03 {
    state.beat_strength = (onset * 6.0).max(state.beat_strength * 0.6);
    state.beat_count = state.beat_count.wrapping_add(1);
  } else {
    state.beat_strength *= 0.7;
  }

  let raw_onset = (raw_bass - state.prev_raw_bass).max(0.0);
  state.prev_raw_bass = raw_bass;
  state.beat_strength_raw = if raw_onset > 0.06 {
    (raw_onset * 5.0).max(state.beat_strength_raw * 0.5)
  } else {
    state.beat_strength_raw * 0.5
  };

  // Rotation is TIME-based (0.003 rad per frame at 60 FPS), so a 30 FPS export
  // spins rotating styles (vinyl, turntable, DJ controller, ...) at the same
  // angular speed as the ~60 FPS live preview instead of half as fast.
  state.rotation_angle += 0.18 / fps.max(1.0);

  // Text fade factor — mirrors textOverlay.ts fadeFactor: paused → fully
  // visible, playing → ramp from the moment playback started.
  let global_fade = crate::renderers::text::fade_factor(
    is_playing,
    frame_time,
    &mut state.text_play_start_frame,
    &mut state.text_was_playing,
  );

  // Screen-effect shake offsets (mirrors canvasRenderer drawFrame).
  let (shake_x, shake_y) = screen_effects::compute_shake_offset(
    &mut state.screen_fx,
    &config.screen_effects,
    above_floor,
    state.beat_strength,
  );
  let (bg_shake_x, bg_shake_y) = ((shake_x * 1.8).round(), (shake_y * 1.8).round());
  let shake_margin = (bg_shake_x * bg_shake_x + bg_shake_y * bg_shake_y).sqrt().ceil();

  FrameEnvelope {
    bass_energy: state.bass_energy,
    beat_strength: state.beat_strength,
    beat_count: state.beat_count,
    above_floor,
    global_fade,
    shake_x,
    shake_y,
    bg_shake_x,
    bg_shake_y,
    shake_margin,
    fx_time: state.screen_fx.fx_time,
  }
}

/// Which layers of the frame to draw. The two-pass split exists so the GPU
/// path can apply frame-sampling screen effects (glitch, chromatic, ...) to
/// the BACKGROUND ONLY when `screenEffects.backgroundOnly` is on — mirroring
/// canvasRenderer, which calls `applyScreenEffects` between the background
/// and the visualizer style.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FramePass {
  /// Everything (default).
  All,
  /// Background fill/image/effects + overlay screen effects + overlay-opacity
  /// rect, with shake applied. Does NOT advance the envelope (call
  /// [`advance_envelope`] first).
  BackgroundOnly,
  /// Style + particles + notes + text. Does NOT advance the envelope.
  ForegroundOnly,
}

pub fn draw_frame_pass(
  c: &mut GpuCanvas,
  scene3d: &mut Scene3D,
  state: &mut RenderState,
  config: &VisualizerConfig,
  freq: &[u8],
  time: &[u8],
  frame_time: f32,
  env: &FrameEnvelope,
  pass: FramePass,
) {
  let width = c.width;
  let height = c.height;

  let mut ctx = RenderContext {
    width,
    height,
    config,
    freq_data: freq,
    time_data: time,
    bass_energy: env.bass_energy,
    beat_strength: env.beat_strength,
    beat_count: env.beat_count,
    rotation_angle: state.rotation_angle,
    frame_time,
    state,
    scene3d,
  };

  let bg_only = config.screen_effects.background_only.unwrap_or(true);
  let draw_bg = pass != FramePass::ForegroundOnly;
  let draw_style = pass != FramePass::BackgroundOnly;

  if draw_bg {
    c.save();
    c.translate(env.bg_shake_x, env.bg_shake_y);
    draw_background(c, &ctx, env.shake_margin);
    c.restore();

    let overlay = config.background.overlay_opacity;
    if overlay > 0.0 {
      c.save();
      c.translate(env.shake_x, env.shake_y);
      c.set_fill(Fill::Solid(Color::rgba(10.0 / 255.0, 10.0 / 255.0, 15.0 / 255.0, overlay)));
      c.fill_rect(-env.shake_margin, -env.shake_margin, width + env.shake_margin * 2.0, height + env.shake_margin * 2.0);
      c.restore();
    }

    // Overlay screen effects belong on the background layer when
    // backgroundOnly is on (mirrors canvasRenderer: applyScreenEffects is
    // called between the background and the style).
    if bg_only {
      screen_effects::apply_overlay(c, &ctx, env.above_floor);
    }
  }

  if draw_style {
    c.save();
    c.translate(env.shake_x, env.shake_y);
    render_style(c, &mut ctx);
    c.restore();

    // Particles / music notes render in screen space after the style.
    // Matches TS canvasRenderer: only checks showParticles/showMusicNotes flags,
    // NOT the background effects array.
    let show_p = config.background.show_particles;
    let show_m = config.background.show_music_notes.unwrap_or(false);
    let show_f = config.background.show_fireworks.unwrap_or(false);
    let show_mr = config.background.show_matrix_rain.unwrap_or(false);
    let show_ff = config.background.show_fireflies.unwrap_or(false);
    let show_sk = config.background.show_sakura.unwrap_or(false);
    let show_cl = config.background.show_cyber_lightning.unwrap_or(false);

    if show_p {
      background::render_particles(c, &mut ctx);
    }
    if show_m {
      background::render_music_notes(c, &mut ctx);
    }
    if show_f {
      background::render_fireworks(c, &mut ctx);
    }
    if show_mr {
      background::render_matrix_rain(c, &mut ctx);
    }
    if show_ff {
      background::render_fireflies(c, &mut ctx);
    }
    if show_sk {
      background::render_sakura(c, &mut ctx);
    }
    if show_cl {
      background::render_cyber_lightning(c, &mut ctx);
    }

    // Text overlay (title/artist/blocks) — Phase 6 text port.
    text::draw_text_overlay(c, &ctx, env.global_fade);

    // When backgroundOnly is off, overlay screen effects apply to the whole
    // frame (after the text, like canvasRenderer's second applyScreenEffects).
    if !bg_only {
      screen_effects::apply_overlay(c, &ctx, env.above_floor);
    }
  }
}

pub fn draw_frame(
  c: &mut GpuCanvas,
  state: &mut RenderState,
  config: &VisualizerConfig,
  freq: &[u8],
  time: &[u8],
  frame_time: f32,
  is_playing: bool,
) {
  let env = advance_envelope(state, config, freq, frame_time, is_playing, 60.0);
  let mut scene3d = Scene3D::new();
  draw_frame_pass(c, &mut scene3d, state, config, freq, time, frame_time, &env, FramePass::All);
}

/// Composite a custom background image (cover-fit, center-cropped — mirrors
/// TS drawCoverImage) onto the CPU fallback frame.
fn cpu_background_image(
  rgb: &mut [u8],
  w: u32,
  h: u32,
  rgba: &[u8],
  iw: u32,
  ih: u32,
  opacity: f32,
) {
  if iw == 0 || ih == 0 || rgba.len() < (iw as usize) * (ih as usize) * 4 {
    return;
  }
  let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u32;
  if alpha == 0 {
    return;
  }
  let (wf, hf) = (w as f32, h as f32);
  let scale = (wf / iw as f32).max(hf / ih as f32);
  let dw = (iw as f32 * scale).ceil() as i32;
  let dh = (ih as f32 * scale).ceil() as i32;
  let ox = (w as i32 - dw) / 2;
  let oy = (h as i32 - dh) / 2;
  for y in 0..h {
    let src_y = ((y as i32 - oy) as f32 / scale) as i32;
    if src_y < 0 || src_y >= ih as i32 {
      continue;
    }
    for x in 0..w {
      let src_x = ((x as i32 - ox) as f32 / scale) as i32;
      if src_x < 0 || src_x >= iw as i32 {
        continue;
      }
      let si = ((src_y as u32 * iw + src_x as u32) * 4) as usize;
      let a = rgba[si + 3] as u32 * alpha / 255;
      if a == 0 {
        continue;
      }
      let o = ((y as usize) * (w as usize) + x as usize) * 3;
      let inv = 255 - a;
      rgb[o] = ((rgba[si] as u32 * a + rgb[o] as u32 * inv) / 255) as u8;
      rgb[o + 1] = ((rgba[si + 1] as u32 * a + rgb[o + 1] as u32 * inv) / 255) as u8;
      rgb[o + 2] = ((rgba[si + 2] as u32 * a + rgb[o + 2] as u32 * inv) / 255) as u8;
    }
  }
}

/// Rasterize one text run via the shared text atlas and source-over it onto
/// the CPU fallback frame. `y` is the baseline; `x` is the pen anchor for the
/// given align (mirrors GpuCanvas::draw_text_quad geometry exactly).
fn cpu_draw_text(
  rgb: &mut [u8],
  w: u32,
  h: u32,
  text: &str,
  x: f32,
  baseline_y: f32,
  align: crate::config::TextAlign,
  family: &str,
  weight: f32,
  italic: bool,
  font_size: f32,
  color: [u8; 3],
  opacity: f32,
) {
  if text.trim().is_empty() || font_size <= 0.0 || opacity <= 0.0 {
    return;
  }
  let Some(font) = crate::gpu2d::text::select_font_for_text_style(family, weight, italic, text) else {
    return;
  };
  let fill = Fill::Solid(Color::rgb(
    color[0] as f32 / 255.0,
    color[1] as f32 / 255.0,
    color[2] as f32 / 255.0,
  ));
  let opts = crate::gpu2d::text::TextOpts::default();
  let Some(atl) = crate::gpu2d::text::rasterize(font, text, font_size, &fill, &opts) else {
    return;
  };
  let dx = match align {
    crate::config::TextAlign::Left => 0.0,
    crate::config::TextAlign::Center => -atl.advance / 2.0,
    crate::config::TextAlign::Right => -atl.advance,
  };
  let pen_x = x + dx;
  let alpha_mult = (opacity.clamp(0.0, 1.0) * 255.0) as u32;
  for py in 0..atl.atlas_h {
    for px in 0..atl.atlas_w {
      let i = ((py * atl.atlas_w + px) * 4) as usize;
      let a = atl.rgba[i + 3] as u32 * alpha_mult / 255;
      if a == 0 {
        continue;
      }
      let cx = (pen_x + (px as f32 - atl.pen_x)).round() as i32;
      let cy = (baseline_y + (py as f32 - atl.baseline)).round() as i32;
      if cx < 0 || cy < 0 || cx >= w as i32 || cy >= h as i32 {
        continue;
      }
      let o = ((cy as usize) * (w as usize) + cx as usize) * 3;
      let inv = 255 - a;
      rgb[o] = ((atl.rgba[i] as u32 * a + rgb[o] as u32 * inv) / 255) as u8;
      rgb[o + 1] = ((atl.rgba[i + 1] as u32 * a + rgb[o + 1] as u32 * inv) / 255) as u8;
      rgb[o + 2] = ((atl.rgba[i + 2] as u32 * a + rgb[o + 2] as u32 * inv) / 255) as u8;
    }
  }
}

/// Mirror the GPU text overlay (drawTextOverlay) on the CPU fallback: title,
/// artist and any enabled custom blocks, positioned as a percentage of the
/// frame (block_anchor) with the block's baseline at the anchor y.
fn cpu_text_overlay(rgb: &mut [u8], w: u32, h: u32, config: &VisualizerConfig) {
  let txt = &config.text;
  let default_family = if txt.font_family.trim().is_empty() {
    "Outfit"
  } else {
    txt.font_family.as_str()
  };

  struct Item<'a> {
    block: &'a crate::config::TextBlock,
    text: String,
  }
  let mut items: Vec<Item> = Vec::new();
  if txt.show_title {
    let t = if !txt.song_title.trim().is_empty() {
      txt.song_title.as_str()
    } else if !txt.title.text.trim().is_empty() {
      txt.title.text.as_str()
    } else {
      "Song Title"
    };
    items.push(Item { block: &txt.title, text: t.to_string() });
  }
  if txt.show_artist {
    let a = if !txt.artist_name.trim().is_empty() {
      txt.artist_name.as_str()
    } else if !txt.artist.text.trim().is_empty() {
      txt.artist.text.as_str()
    } else {
      "Artist Name"
    };
    items.push(Item { block: &txt.artist, text: a.to_string() });
  }
  for b in &txt.blocks {
    if b.enabled && !b.text.trim().is_empty() {
      items.push(Item { block: b, text: b.text.clone() });
    }
  }

  for item in items {
    let block = item.block;
    let (w_f, h_f) = (w as f32, h as f32);
    let anchor_x = (block.position_x / 100.0) * w_f;
    let anchor_y = (block.position_y / 100.0) * h_f;
    let family = if block.font_family.trim().is_empty() {
      default_family
    } else {
      block.font_family.as_str()
    };
    let color: [u8; 3] = if block.color.trim().is_empty() {
      [255, 255, 255]
    } else {
      let c = Color::hex(&block.color);
      [
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
      ]
    };
    cpu_draw_text(
      rgb,
      w,
      h,
      &item.text,
      anchor_x,
      anchor_y,
      block.align,
      family,
      block.font_weight,
      block.italic,
      block.font_size,
      color,
      block.opacity,
    );
  }
}

pub fn render_frame_to_rgb(
  config: &VisualizerConfig,
  freq: &[u8],
  time: &[u8],
  _bass_energy: f32,
  frame_time: f32,
  fx_time: f32,
  width: u32,
  height: u32,
) -> Vec<u8> {
  let mut rgb = vec![15u8; (width * height * 3) as usize];
  let (w_f, h_f) = (width as f32, height as f32);

  // Background Fill (Solid or Gradient)
  let bg_color = crate::gpu2d::Color::hex(&config.background.solid_color);
  let (bg_r, bg_g, bg_b) = ((bg_color.r * 255.0) as u8, (bg_color.g * 255.0) as u8, (bg_color.b * 255.0) as u8);

  let g_start = crate::gpu2d::Color::hex(if config.background.gradient_start.is_empty() { "#0f0c20" } else { &config.background.gradient_start });
  let g_end = crate::gpu2d::Color::hex(if config.background.gradient_end.is_empty() { "#06101e" } else { &config.background.gradient_end });

  let is_gradient = matches!(config.background.mode, crate::config::BackgroundMode::Gradient);

  for y in 0..height {
    let t = y as f32 / h_f;
    let (r, g, b) = if is_gradient {
      (
        ((g_start.r * (1.0 - t) + g_end.r * t) * 255.0) as u8,
        ((g_start.g * (1.0 - t) + g_end.g * t) * 255.0) as u8,
        ((g_start.b * (1.0 - t) + g_end.b * t) * 255.0) as u8,
      )
    } else {
      (bg_r, bg_g, bg_b)
    };

    for x in 0..width {
      let idx = ((y * width + x) * 3) as usize;
      rgb[idx] = r;
      rgb[idx + 1] = g;
      rgb[idx + 2] = b;
    }
  }

  // Custom background image (cover-fit, mirrors the GPU path + TS drawCoverImage).
  if matches!(
    config.background.mode,
    crate::config::BackgroundMode::CustomImage
  ) {
    if let Some((rgba, iw, ih)) =
      crate::gpu_export::decode_background_image(config.background.custom_image_uri.as_deref())
    {
      let default_opacity = 1.0;
      let raw_op = config.background.image_opacity.unwrap_or(default_opacity);
      let op = if raw_op > 1.0 { raw_op / 100.0 } else { raw_op };
      cpu_background_image(&mut rgb, width, height, &rgba, iw, ih, op);
    }
  }

  // Theme Colors
  let primary = crate::gpu2d::Color::hex(&config.theme.primary_color);
  let secondary = crate::gpu2d::Color::hex(&config.theme.secondary_color);
  let accent = crate::gpu2d::Color::hex(&config.theme.accent_color);

  let p_r = (primary.r * 255.0) as u8;
  let p_g = (primary.g * 255.0) as u8;
  let p_b = (primary.b * 255.0) as u8;

  let s_r = (secondary.r * 255.0) as u8;
  let s_g = (secondary.g * 255.0) as u8;
  let s_b = (secondary.b * 255.0) as u8;

  let a_r = (accent.r * 255.0) as u8;
  let a_g = (accent.g * 255.0) as u8;
  let a_b = (accent.b * 255.0) as u8;

  let pos_x_offset = config.position_x * w_f * 0.45;
  let pos_y_offset = -config.position_y * h_f * 0.45;
  let (cx, cy) = (w_f / 2.0 + pos_x_offset, h_f / 2.0 + pos_y_offset);
  let scale = config.scale.clamp(0.1, 4.0);

  let style_name = format!("{:?}", config.style).to_lowercase();
  let bar_count = config.reactivity.bar_count.clamp(8, 128);

  // Render Visualizer Style
  if style_name.contains("radial") || style_name.contains("circular") || style_name.contains("pulse") {
    // RADIAL / CIRCULAR BARS (Scalable & Positionable)
    let base_radius = (h_f * 0.25) * scale;
    for i in 0..bar_count {
      let angle = (i as f32 / bar_count as f32) * std::f32::consts::TAU + frame_time * 0.2;
      let sample = if i < freq.len() { freq[i] as f32 / 255.0 } else { 0.1 };
      let bar_len = sample * (h_f * 0.3) * config.reactivity.sensitivity * scale;

      let cos_a = angle.cos();
      let sin_a = angle.sin();
      let r_inner = base_radius;
      let r_outer = base_radius + bar_len.max(4.0);

      let steps = (bar_len.max(4.0) as usize).clamp(4, 80);
      for step in 0..steps {
        let cur_r = r_inner + (r_outer - r_inner) * (step as f32 / steps as f32);
        let px = (cx + cos_a * cur_r) as i32;
        let py = (cy + sin_a * cur_r) as i32;

        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
          let idx = ((py as u32 * width + px as u32) * 3) as usize;
          if idx + 2 < rgb.len() {
            let t = step as f32 / steps as f32;
            rgb[idx] = (p_r as f32 * (1.0 - t) + s_r as f32 * t) as u8;
            rgb[idx + 1] = (p_g as f32 * (1.0 - t) + s_g as f32 * t) as u8;
            rgb[idx + 2] = (p_b as f32 * (1.0 - t) + s_b as f32 * t) as u8;
          }
        }
      }
    }
  } else if style_name.contains("oscilloscope") || style_name.contains("waveform") {
    // OSCILLOSCOPE / WAVEFORM (Scalable & Positionable)
    let samples = if !time.is_empty() { time } else { freq };
    let n = samples.len().max(1);
    let span_w = w_f * scale;
    let x_left = cx - span_w / 2.0;

    for x_i in 0..(span_w as i32) {
      let px = x_left as i32 + x_i;
      if px >= 0 && px < width as i32 {
        let sample_idx = ((x_i as f32 / span_w) * n as f32) as usize % n;
        let val = (samples[sample_idx] as f32 / 255.0 - 0.5) * 2.0;
        let py = (cy + val * (h_f * 0.35) * config.reactivity.sensitivity * scale) as i32;

        for dy in -2..=2 {
          let y_pos = py + dy;
          if y_pos >= 0 && y_pos < height as i32 {
            let idx = ((y_pos as u32 * width + px as u32) * 3) as usize;
            if idx + 2 < rgb.len() {
              rgb[idx] = a_r;
              rgb[idx + 1] = a_g;
              rgb[idx + 2] = a_b;
            }
          }
        }
      }
    }
  } else {
    // SPECTRUM / EQUALIZER / BARS / DEFAULT (Scalable & Positionable)
    let total_span_w = w_f * scale;
    let bar_width = (total_span_w / bar_count as f32).max(1.0);
    let x_start_base = cx - total_span_w / 2.0;
  let y_base = (h_f / 2.0 + pos_y_offset + h_f * 0.5).round() as i32;
    let total_bins = freq.len().max(1) as f32;

    for i in 0..bar_count {
      let norm = i as f32 / bar_count as f32;
      let log_norm = norm.powf(1.6);
      let bin_idx = ((log_norm * (total_bins - 1.0)) as usize).min(freq.len().saturating_sub(1));

      let sample = if !freq.is_empty() {
        let prev = if bin_idx > 0 { freq[bin_idx - 1] as f32 } else { freq[bin_idx] as f32 };
        let curr = freq[bin_idx] as f32;
        let next = if bin_idx + 1 < freq.len() { freq[bin_idx + 1] as f32 } else { curr };
        ((prev * 0.25 + curr * 0.50 + next * 0.25) / 255.0).clamp(0.02, 1.0)
      } else {
        0.1
      };

      let bar_h = (sample * h_f * 0.75 * config.reactivity.sensitivity * scale).max(6.0) as i32;
      let x_start = (x_start_base + i as f32 * bar_width) as i32;
      let x_end = (x_start_base + (i as f32 + 1.0) * bar_width) as i32;
      let y_start = (y_base - bar_h).clamp(0, height as i32);
      let y_end = y_base.clamp(0, height as i32);

      for y in y_start..y_end {
        let progress = (y - y_start) as f32 / bar_h.max(1) as f32;
        let cur_r = (p_r as f32 * progress + s_r as f32 * (1.0 - progress)) as u8;
        let cur_g = (p_g as f32 * progress + s_g as f32 * (1.0 - progress)) as u8;
        let cur_b = (p_b as f32 * progress + s_b as f32 * (1.0 - progress)) as u8;

        for x in x_start..x_end {
          if x >= 0 && x < width as i32 {
            let idx = ((y as u32 * width + x as u32) * 3) as usize;
            if idx + 2 < rgb.len() {
              rgb[idx] = cur_r;
              rgb[idx + 1] = cur_g;
              rgb[idx + 2] = cur_b;
            }
          }
        }
      }
    }
  }

  // Screen effects on the software path (mirror of the GPU pipeline).
  let env = screen_effects::cpu_envelope(freq);
  screen_effects::apply_cpu_screen_effects(
    &mut rgb,
    width,
    height,
    &config.screen_effects,
    env.above_floor,
    env.beat,
    fx_time,
  );

  // Text overlay (title / artist / custom blocks) — the CPU fallback used to
  // drop text entirely even though the GPU path always renders it.
  cpu_text_overlay(&mut rgb, width, height, config);

  rgb
}
