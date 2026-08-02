//! Rust port of the visualizer draw loop (export path).
//! Mirrors `src/services/canvasRenderer.ts` drawFrame + the style renderers.
//! Phase 4/5: frequency/time-domain renderers + complex particle & 3D-style
//! renderers. Background/screen effects are Phase 6 (structured hooks below).

pub mod advanced;
mod background;
pub mod screen_effects;
mod styles;
mod text;

use crate::config::{
  AudioReactivitySettings, BackgroundEffect, BackgroundFillType, ColorTheme, VisualizerConfig,
};
use crate::gpu2d::{Color, Fill, GpuCanvas};

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
  pub rng: Rng,
  pub advanced: advanced::AdvancedState,
  /// Custom background image uploaded once to a persistent atlas layer.
  /// `w`/`h` are the scaled layer-space dimensions (for UV mapping).
  pub background_image: Option<BackgroundImage>,
  pub radial_center_image: Option<BackgroundImage>,
  pub stars: Vec<background::Star>,
  pub particles: Vec<background::Particle>,
  pub music_notes: Vec<background::MusicNote>,
  pub screen_fx: screen_effects::ScreenFxState,
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
      rng: Rng::new(seed),
      advanced: advanced::AdvancedState::default(),
      background_image: None,
      radial_center_image: None,
      stars: background::build_stars(),
      particles: background::init_particles(&mut Rng::new(seed.wrapping_add(0x1234))),
      music_notes: Vec::new(),
      screen_fx: screen_effects::ScreenFxState::new(),
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
  pub rotation_angle: f32,
  pub frame_time: f32,
  pub state: &'a mut RenderState,
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

fn draw_background(c: &mut GpuCanvas, ctx: &RenderContext, margin: f32) {
  let bg = &ctx.config.background;
  let fill_type = match &bg.fill_type {
    Some(BackgroundFillType::Gradient) => BackgroundFillType::Gradient,
    _ => BackgroundFillType::Solid,
  };
  match fill_type {
    BackgroundFillType::Gradient => {
      let g = Fill::linear_gradient(0.0, 0.0, ctx.width, ctx.height, &[
        (0.0, Color::hex(&bg.gradient_start)),
        (1.0, Color::hex(&bg.gradient_end)),
      ]);
      c.set_fill(g);
      c.fill_rect(-margin, -margin, ctx.width + margin * 2.0, ctx.height + margin * 2.0);
    }
    BackgroundFillType::Solid => {
      c.set_fill(Fill::Solid(Color::hex(&bg.solid_color)));
      c.fill_rect(-margin, -margin, ctx.width + margin * 2.0, ctx.height + margin * 2.0);
    }
  }

  // Custom background image (cover-fit, mirrors canvas drawCoverImage).
  if let Some(img) = &ctx.state.background_image {
    let alpha = bg.image_opacity.unwrap_or(1.0).clamp(0.0, 1.0);
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
    let layer_size = crate::gpu2d::LAYER_SIZE as f32;
    c.push_textured_quad(
      img.layer,
      ox - margin,
      oy - margin,
      rw + margin * 2.0,
      rh + margin * 2.0,
      [0.0, 0.0, (img.w as f32) / layer_size, (img.h as f32) / layer_size],
      Color::rgba(1.0, 1.0, 1.0, alpha),
    );
  }

  // Overlay visual effects (grid/aurora/noise/...).
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
  for effect in active {
    match effect {
      BackgroundEffect::Grid => background::render_grid(c, ctx),
      BackgroundEffect::Aurora => background::render_aurora(c, ctx),
      BackgroundEffect::Noise => background::render_noise(c, ctx),
      BackgroundEffect::Bokeh => background::render_bokeh(c, ctx),
      BackgroundEffect::Starfield => background::render_starfield(c, ctx, &ctx.state.stars),
      BackgroundEffect::Nebula => background::render_nebula(c, ctx),
      BackgroundEffect::Psychedelic => background::render_psychedelic(c, ctx),
      BackgroundEffect::None => {}
      // Particles / music notes are gated separately (showParticles etc.).
      BackgroundEffect::Particles | BackgroundEffect::MusicNotes => {}
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

pub fn draw_frame(
  c: &mut GpuCanvas,
  state: &mut RenderState,
  config: &VisualizerConfig,
  freq: &[u8],
  time: &[u8],
  frame_time: f32,
) {
  let width = c.width;
  let height = c.height;
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

  state.rotation_angle += 0.003;

  // Screen-effect shake offsets (mirrors canvasRenderer drawFrame).
  let (shake_x, shake_y) = screen_effects::compute_shake_offset(
    &mut state.screen_fx,
    &config.screen_effects,
    above_floor,
    state.beat_strength,
    frame_time,
  );
  let (bg_shake_x, bg_shake_y) = ((shake_x * 1.8).round(), (shake_y * 1.8).round());
  let shake_margin = (bg_shake_x * bg_shake_x + bg_shake_y * bg_shake_y).sqrt().ceil();

  let mut ctx = RenderContext {
    width,
    height,
    config,
    freq_data: freq,
    time_data: time,
    bass_energy: state.bass_energy,
    beat_strength: state.beat_strength,
    rotation_angle: state.rotation_angle,
    frame_time,
    state,
  };

  c.save();
  c.translate(bg_shake_x, bg_shake_y);
  draw_background(c, &ctx, shake_margin);
  c.restore();

  let overlay = config.background.overlay_opacity;
  if overlay > 0.0 {
    c.save();
    c.translate(shake_x, shake_y);
    c.set_fill(Fill::Solid(Color::rgba(10.0 / 255.0, 10.0 / 255.0, 15.0 / 255.0, overlay)));
    c.fill_rect(-shake_margin, -shake_margin, width + shake_margin * 2.0, height + shake_margin * 2.0);
    c.restore();
  }

  c.save();
  c.translate(shake_x, shake_y);
  c.translate(width / 2.0 + config.position_x, height / 2.0 + config.position_y);
  let sx = config.scale;
  if (sx - 1.0).abs() > 1e-6 {
    c.scale(sx, sx);
  }
  c.translate(-width / 2.0, -height / 2.0);
  render_style(c, &mut ctx);
  c.restore();

  // Particles / music notes render in screen space after the style.
  if config.background.show_particles {
    background::render_particles(c, &mut ctx);
  }
  if config.background.show_music_notes.unwrap_or(false) {
    background::render_music_notes(c, &mut ctx);
  }

  // Text overlay (title/artist/blocks) — Phase 6 text port.
  text::draw_text_overlay(c, &ctx);

  // Overlay-style screen effects (vignette/pulse/spotlight/strobe/scanline/hueShift).
  screen_effects::apply_overlay(c, &ctx, above_floor);
}
