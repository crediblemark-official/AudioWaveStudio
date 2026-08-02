//! Rust port of the visualizer draw loop (export path).
//! Mirrors `src/services/canvasRenderer.ts` drawFrame + the style renderers.
//! Phase 4/5: frequency/time-domain renderers + complex particle & 3D-style
//! renderers. Background/screen effects are Phase 6 (structured hooks below).

mod advanced;
mod background;
mod effects;
mod screen_effects;
mod spectral;
mod text;

use crate::config::{
  AudioReactivitySettings, BackgroundEffect, BackgroundFillType, ColorTheme, VisualizerConfig,
  VisualizerStyle,
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

fn bin_value(freq: &[u8], step: usize, idx: usize) -> f32 {
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
    let default_opacity = if matches!(bg.mode, crate::config::BackgroundMode::CustomImage) { 1.0 } else { 0.7 };
    let alpha = bg.image_opacity.unwrap_or(default_opacity).clamp(0.0, 1.0);
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
  match &ctx.config.style {
    VisualizerStyle::Spectrum => spectral::spectrum_bars(c, ctx),
    VisualizerStyle::Radial => spectral::radial(c, ctx),
    VisualizerStyle::Oscilloscope => spectral::oscilloscope(c, ctx),
    VisualizerStyle::Equalizer => spectral::equalizer_matrix(c, ctx),
    VisualizerStyle::Minimal => spectral::minimal_wave(c, ctx),
    VisualizerStyle::WaveformFill => spectral::waveform_fill(c, ctx),
    VisualizerStyle::CircularBars => spectral::circular_bars(c, ctx),
    VisualizerStyle::SmoothSpectrum => spectral::smooth_spectrum(c, ctx),
    VisualizerStyle::PulseRings => effects::pulse_rings(c, ctx),
    VisualizerStyle::VuMeter => effects::vu_meter(c, ctx),
    VisualizerStyle::AuroraWave => effects::aurora_wave(c, ctx),
    VisualizerStyle::FlameFire => advanced::flame_fire(c, ctx),
    VisualizerStyle::SpiralGalaxy => advanced::spiral_galaxy(c, ctx),
    VisualizerStyle::ThreeD => advanced::three_d(c, ctx),
    VisualizerStyle::Api3D => advanced::api_3d(c, ctx),
    VisualizerStyle::NeonCity3D => advanced::neon_city_3d(c, ctx),
    VisualizerStyle::Speaker3D => advanced::speaker_3d(c, ctx),
    VisualizerStyle::SpeakerTrio => advanced::speaker_trio(c, ctx),
    VisualizerStyle::SpeakerSplatter => advanced::speaker_splatter(c, ctx),
  }
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::*;
  use crate::gpu2d::GpuRenderer;

  pub fn test_config() -> VisualizerConfig {
    VisualizerConfig {
      style: VisualizerStyle::Spectrum,
      theme: ColorTheme {
        name: ColorThemeName::Cyberpunk,
        label: "test".into(),
        primary_color: "#ff2d78".into(),
        secondary_color: "#00d9ff".into(),
        accent_color: "#ffee00".into(),
        glow_color: "#ff2d78".into(),
      },
      background: BackgroundSettings {
        mode: BackgroundMode::Gradient,
        fill_type: Some(BackgroundFillType::Gradient),
        effect: None,
        effects: None,
        solid_color: "#0b0c10".into(),
        gradient_start: "#0f0c20".into(),
        gradient_end: "#06101e".into(),
        blur_amount: 0.0,
        overlay_opacity: 0.0,
        custom_image_uri: None,
        image_opacity: None,
        grid_color: None,
        grid_size: None,
        grid_line_width: None,
        show_particles: false,
        particle_style: None,
        particle_color: "#ffffff".into(),
        particle_size: None,
        particle_speed: None,
        particle_count: None,
        show_music_notes: None,
        music_note_style: None,
        music_note_color: None,
        radial_center_image_uri: None,
        music_note_density: None,
        music_note_size: None,
        music_note_count: None,
        music_note_sensitivity: None,
        star_count: None,
        star_speed: None,
        star_brightness: None,
        nebula_intensity: None,
        nebula_speed: None,
        aurora_speed: None,
        aurora_amplitude: None,
        aurora_opacity: None,
        grain_opacity: None,
        bokeh_count: None,
        bokeh_size: None,
        bokeh_opacity: None,
        psychedelic_speed: None,
        psychedelic_bands: None,
        psychedelic_line_width: None,
      },
      text: TextSettings {
        song_title: "Test Song".into(),
        artist_name: "Test Artist".into(),
        show_title: false,
        show_artist: false,
        font_family: "monospace".into(),
        title: TextBlock {
          id: "title".into(),
          text: "".into(),
          enabled: false,
          font_family: "monospace".into(),
          font_size: 32.0,
          font_weight: 700.0,
          italic: false,
          color: "#ffffff".into(),
          use_gradient: false,
          gradient_start: "#ffffff".into(),
          gradient_end: "#ffffff".into(),
          gradient_angle: 0.0,
          opacity: 1.0,
          letter_spacing: 0.0,
          transform: TextTransform::None,
          position_x: 0.0,
          position_y: 0.0,
          align: TextAlign::Center,
          line_height: 1.2,
          max_width: 0.0,
          shadow: true,
          shadow_blur: 10.0,
          shadow_offset_x: 0.0,
          shadow_offset_y: 0.0,
          glow_intensity: 1.0,
          outline: false,
          outline_color: "#000000".into(),
          outline_width: 1.0,
          reactive_scale: 0.0,
          wave_effect: false,
          fade_in: false,
        },
        artist: TextBlock {
          id: "artist".into(),
          text: "".into(),
          enabled: false,
          font_family: "monospace".into(),
          font_size: 18.0,
          font_weight: 400.0,
          italic: false,
          color: "#aaaaaa".into(),
          use_gradient: false,
          gradient_start: "#ffffff".into(),
          gradient_end: "#ffffff".into(),
          gradient_angle: 0.0,
          opacity: 1.0,
          letter_spacing: 0.0,
          transform: TextTransform::None,
          position_x: 0.0,
          position_y: 0.0,
          align: TextAlign::Center,
          line_height: 1.2,
          max_width: 0.0,
          shadow: true,
          shadow_blur: 8.0,
          shadow_offset_x: 0.0,
          shadow_offset_y: 0.0,
          glow_intensity: 1.0,
          outline: false,
          outline_color: "#000000".into(),
          outline_width: 1.0,
          reactive_scale: 0.0,
          wave_effect: false,
          fade_in: false,
        },
        blocks: vec![],
      },
      reactivity: AudioReactivitySettings {
        fft_size: 1024,
        sensitivity: 1.0,
        bass_multiplier: 1.0,
        bar_count: 64,
        bar_width: 0.0,
        bar_gap: 4.0,
        bar_rounding: 4.0,
        smoothing: 0.8,
        mirror_bars: false,
        show_peaks: true,
        peak_color: "#ffffff".into(),
        fire_width_ratio: None,
        fire_height_scale: None,
      },
      export: ExportSettings {
        aspect_ratio: AspectRatio::Widescreen,
        resolution: ExportResolution::P720,
        fps: 60,
        format: ExportFormat::Mp4,
      },
      screen_effects: ScreenEffectsSettings {
        enabled: false,
        main_effect: ScreenEffect::None,
        shake_intensity: 1.0,
        shake_frequency: 8.0,
        shake_max_offset: 8.0,
        shake_on_beat: true,
        glitch_intensity: 0.5,
        pulse_intensity: 0.3,
        spotlight_color: "#ffffff".into(),
        strobe_intensity: 0.5,
        scanline_opacity: 0.15,
        chromatic_intensity: 0.5,
        zoom_intensity: 0.1,
        invert_intensity: 0.5,
        bars_amount: 0.3,
        shockwave_intensity: 0.5,
        pixelate_intensity: 0.5,
        tilt_intensity: 0.5,
        heat_haze_intensity: 0.5,
        hue_shift_intensity: 0.5,
      },
      position_x: 0.0,
      position_y: 0.0,
      scale: 1.0,
    }
  }

  fn synth_freq() -> Vec<u8> {
    (0..512)
      .map(|i| {
        let base = ((i as f32 / 8.0).sin() * 0.5 + 0.5) * 200.0;
        (base.clamp(0.0, 255.0)) as u8
      })
      .collect()
  }

  #[test]
  #[ignore = "requires a Vulkan-capable GPU"]
  fn gpu_draw_frame_renders_jpeg() {
    let config = test_config();
    let freq = synth_freq();
    let time: Vec<u8> = (0..512)
      .map(|i| (((i as f32 / 16.0).sin() * 127.0) + 128.0).clamp(0.0, 255.0) as u8)
      .collect();

    let mut gpu = pollster::block_on(GpuRenderer::new(320, 240)).expect("GPU init failed");
    let mut rstate = RenderState::new(config.reactivity.bar_count, 1);
    let mut canvas = GpuCanvas::new(320, 240);
    draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0);
    let mesh = canvas.finish();
    assert!(!mesh.is_empty(), "expected at least some geometry");

    let jpeg = gpu.jpeg(&mesh).expect("jpeg encode failed");
    assert!(jpeg.len() > 1000, "jpeg too small: {}", jpeg.len());
    assert_eq!(&jpeg[..2], &[0xFF, 0xD8], "missing JPEG magic");
  }

  #[test]
  #[ignore = "requires a Vulkan-capable GPU"]
  fn gpu_ping_pong_readback_matches() {
    let config = test_config();
    let freq = synth_freq();
    let time: Vec<u8> = (0..512)
      .map(|i| (((i as f32 / 16.0).sin() * 127.0) + 128.0).clamp(0.0, 255.0) as u8)
      .collect();

    let mut gpu = pollster::block_on(GpuRenderer::new(320, 240)).expect("GPU init failed");
    let mut rstate = RenderState::new(config.reactivity.bar_count, 2);
    for slot in 0..2usize {
      let mut canvas = GpuCanvas::new(320, 240);
      draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, slot as f32);
      let mesh = canvas.finish();
      gpu.render_into(&mesh, slot);
      let rgba = gpu.readback(slot);
      assert_eq!(
        rgba.len(),
        320 * 240 * 4,
        "readback {slot} should return full RGBA frame"
      );
      assert!(rgba.iter().any(|&b| b > 0), "readback {slot} should not be empty");
    }
  }

  #[test]
  #[ignore = "requires a Vulkan-capable GPU"]
  fn gpu_text_overlay_renders() {
    let mut config = test_config();
    config.text.show_title = true;
    config.text.show_artist = true;
    config.text.title.text = "AudioWave".into();
    config.text.artist.text = "Test Artist".into();
    config.text.title.use_gradient = true;
    config.text.title.gradient_start = "#ff2d78".into();
    config.text.title.gradient_end = "#00d9ff".into();
    config.text.title.outline = true;
    config.text.title.wave_effect = true;
    config.text.title.letter_spacing = 2.0;

    let freq = synth_freq();
    let time: Vec<u8> = vec![128; 512];

    let mut gpu = pollster::block_on(GpuRenderer::new(640, 360)).expect("GPU init failed");
    let mut rstate = RenderState::new(config.reactivity.bar_count, 2);
    let mut canvas = GpuCanvas::new(640, 360);
    // frame_time 1.5s -> fade-in complete
    draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 1.5);
    let mesh = canvas.finish();
    assert!(!mesh.atlases.is_empty(), "expected text glyph atlases");

    let jpeg = gpu.jpeg(&mesh).expect("jpeg encode failed");
    assert!(jpeg.len() > 3000, "jpeg too small: {}", jpeg.len());
  }

  #[test]
  #[ignore = "requires a Vulkan-capable GPU"]
  fn gpu_background_image_renders() {
    let mut config = test_config();
    config.style = VisualizerStyle::Spectrum;
    config.background.mode = crate::config::BackgroundMode::CustomImage;
    config.background.image_opacity = Some(1.0);

    let freq = synth_freq();
    let time: Vec<u8> = vec![128; 512];

    let mut gpu = pollster::block_on(GpuRenderer::new(640, 360)).expect("GPU init failed");
    // 200x100 test image: left half red, right half blue.
    let mut rgba = Vec::with_capacity(200 * 100 * 4);
    for _y in 0..100 {
      for x in 0..200 {
        if x < 100 {
          rgba.extend_from_slice(&[255, 0, 0, 255]);
        } else {
          rgba.extend_from_slice(&[0, 0, 255, 255]);
        }
      }
    }
    let (tw, th) = gpu
      .upload_image_layer(crate::gpu2d::IMAGE_LAYER, &rgba, 200, 100)
      .expect("upload failed");

    let mut rstate = RenderState::new(config.reactivity.bar_count, 3);
    rstate.background_image = Some(BackgroundImage {
      layer: crate::gpu2d::IMAGE_LAYER,
      w: tw,
      h: th,
    });
    let mut canvas = GpuCanvas::new(640, 360);
    draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0);
    let mesh = canvas.finish();
    let jpeg = gpu.jpeg(&mesh).expect("jpeg encode failed");
    let decoded = image::load_from_memory(&jpeg).expect("decoded").to_rgba8();
    let (w, h) = (decoded.width(), decoded.height());
    let mut red = 0u32;
    let mut blue = 0u32;
    for y in 0..h {
      for x in 0..w {
        let p = decoded.get_pixel(x, y);
        if p[0] > 200 && p[1] < 60 && p[2] < 60 {
          red += 1;
        } else if p[2] > 200 && p[0] < 60 && p[1] < 60 {
          blue += 1;
        }
      }
    }
    // cover-fit: image is wider than canvas ratio -> fills full height, x cropped.
    assert!(red > (w as u32 * h as u32) / 8, "red region too small: {red}");
    assert!(blue > (w as u32 * h as u32) / 8, "blue region too small: {blue}");
  }

  #[test]
  #[ignore = "requires a Vulkan-capable GPU"]
  fn gpu_background_effects_and_particles_render() {
    use crate::config::{BackgroundEffect, ParticleStyle};

    let mut config = test_config();
    config.style = VisualizerStyle::Spectrum;
    config.background.effects = Some(vec![
      BackgroundEffect::Grid,
      BackgroundEffect::Aurora,
      BackgroundEffect::Noise,
      BackgroundEffect::Bokeh,
      BackgroundEffect::Starfield,
      BackgroundEffect::Nebula,
      BackgroundEffect::Psychedelic,
    ]);
    config.background.show_particles = true;
    config.background.particle_style = Some(ParticleStyle::Float);
    config.background.show_music_notes = Some(true);
    config.background.music_note_style = Some(crate::config::MusicNoteStyle::Bounce);

    let freq = synth_freq();
    let time: Vec<u8> = vec![128; 512];

    let mut gpu = pollster::block_on(GpuRenderer::new(640, 360)).expect("GPU init failed");
    let mut rstate = RenderState::new(config.reactivity.bar_count, 7);
    let mut canvas = GpuCanvas::new(640, 360);
    for f in 0..40 {
      draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, f as f32 * 0.1);
      if f < 39 {
        canvas = GpuCanvas::new(640, 360);
      }
    }
    let mesh = canvas.finish();
    assert!(!mesh.is_empty(), "expected geometry");
    let jpeg = gpu.jpeg(&mesh).expect("jpeg encode failed");
    assert!(jpeg.len() > 3000, "jpeg too small: {}", jpeg.len());
    // Visual richness: should not be a flat background.
    let decoded = image::load_from_memory(&jpeg).expect("decoded").to_rgba8();
    let mut colors = std::collections::HashSet::new();
    for px in decoded.pixels().step_by(7) {
      colors.insert([px[0] >> 4, px[1] >> 4, px[2] >> 4]);
    }
    assert!(colors.len() > 8, "expected varied output, got {} color buckets", colors.len());
  }

  #[test]
  fn text_rasterize_produces_atlas() {
    let Some(font) = crate::gpu2d::text::select_font("monospace", 400.0) else {
      return; // no system font on this machine; skip
    };
    let fill = Fill::Solid(Color::WHITE);
    let atl = crate::gpu2d::text::rasterize(
      font,
      "AudioWave",
      32.0,
      &fill,
      &Default::default(),
    )
    .expect("rasterize failed");
    assert!(atl.atlas_w > 10 && atl.atlas_h > 10, "atlas too small");
    assert!(atl.advance > 30.0, "advance too small");
    // Some non-zero alpha pixels must exist.
    let opaque = atl.rgba.chunks(4).filter(|px| px[3] > 0).count();
    assert!(opaque > 50, "too few covered pixels: {}", opaque);
  }

  #[test]
  fn text_measure_grows_with_text() {
    let Some(font) = crate::gpu2d::text::select_font("sans-serif", 400.0) else {
      return;
    };
    let short = crate::gpu2d::text::measure(font, "AA", 40.0, 0.0);
    let long = crate::gpu2d::text::measure(font, "AAAAAAAA", 40.0, 0.0);
    assert!(long > short, "longer text must be wider");
    let spaced = crate::gpu2d::text::measure(font, "AA", 40.0, 8.0);
    assert!(spaced > short, "letter spacing must widen the run");
  }

  #[test]
  #[ignore = "requires a Vulkan-capable GPU"]
  fn gpu_advanced_styles_render() {
    let styles = [
      ("flameFire", VisualizerStyle::FlameFire),
      ("spiralGalaxy", VisualizerStyle::SpiralGalaxy),
      ("threeD", VisualizerStyle::ThreeD),
      ("api3D", VisualizerStyle::Api3D),
      ("neonCity3D", VisualizerStyle::NeonCity3D),
      ("speaker3D", VisualizerStyle::Speaker3D),
      ("speakerTrio", VisualizerStyle::SpeakerTrio),
      ("speakerSplatter", VisualizerStyle::SpeakerSplatter),
    ];
    let freq = synth_freq();
    let time: Vec<u8> = vec![128; 512];

    let mut gpu = pollster::block_on(GpuRenderer::new(480, 270)).expect("GPU init failed");
    for (name, style) in styles {
      let mut config = test_config();
      config.style = style;
      let mut rstate = RenderState::new(config.reactivity.bar_count, 42);
      let mut canvas = GpuCanvas::new(480, 270);
      // a few frames so particle spawn/update paths run
      for f in 0..5 {
        draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, f as f32 * 0.1);
        if f < 4 {
          canvas = GpuCanvas::new(480, 270);
        }
      }
      let mesh = canvas.finish();
      assert!(!mesh.is_empty(), "{name}: expected geometry");
      let jpeg = gpu.jpeg(&mesh).expect(&format!("{name}: jpeg encode failed"));
      assert!(jpeg.len() > 1000, "{name}: jpeg too small: {}", jpeg.len());
    }
  }

  #[test]
  #[ignore = "requires a Vulkan-capable GPU"]
  fn gpu_screen_effects_render() {
    use crate::config::ScreenEffect;

    let effects = [
      ("shake", ScreenEffect::Shake),
      ("vignette", ScreenEffect::Vignette),
      ("pulse", ScreenEffect::Pulse),
      ("spotlight", ScreenEffect::Spotlight),
      ("strobe", ScreenEffect::Strobe),
      ("scanline", ScreenEffect::Scanline),
      ("hueShift", ScreenEffect::HueShift),
    ];
    // Loud bass so beat/bass-driven effects actually draw.
    let freq: Vec<u8> = (0..512).map(|_| 255u8).collect();
    let time: Vec<u8> = vec![128; 512];

    let mut gpu = pollster::block_on(GpuRenderer::new(480, 270)).expect("GPU init failed");
    for (name, effect) in effects {
      let mut config = test_config();
      config.screen_effects.enabled = true;
      config.screen_effects.main_effect = effect;
      config.background.solid_color = "#222222".into();
      config.background.fill_type = Some(BackgroundFillType::Solid);

      let mut rstate = RenderState::new(config.reactivity.bar_count, 11);
      let mut canvas = GpuCanvas::new(480, 270);
      // Several frames so bass energy ramps up and strobe/shake buckets advance.
      for f in 0..12 {
        draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, f as f32 * 0.1);
        if f < 11 {
          canvas = GpuCanvas::new(480, 270);
        }
      }
      let mesh = canvas.finish();
      assert!(!mesh.is_empty(), "{name}: expected geometry");
      let jpeg = gpu.jpeg(&mesh).expect(&format!("{name}: jpeg encode failed"));
      assert!(jpeg.len() > 1500, "{name}: jpeg too small: {}", jpeg.len());
    }
  }

  #[test]
  fn render_state_reset_is_clean() {
    let mut state = RenderState::new(64, 7);
    state.bass_energy = 0.9;
    state.rotation_angle = 3.0;
    state.vu[0].level = 1.0;
    state.rings.push(PulseRing {
      radius: 1.0,
      max_radius: 2.0,
      alpha: 1.0,
      speed: 1.0,
      thickness: 1.0,
      color: Color::WHITE,
    });
    let fresh = RenderState::new(64, 7);
    assert!(fresh.bass_energy == 0.0 && fresh.rotation_angle == 0.0);
    assert!(fresh.peak_data.len() == 64);
    assert!(state.rings.len() == 1);
  }

  #[test]
  fn bin_value_aggregates_and_clamps() {
    let freq = vec![0u8, 255, 128, 0, 0, 0, 0, 0];
    // step 2, idx 0 -> bins 0,1 -> (0 + 255) / (2 * 255) = 0.5
    assert!((bin_value(&freq, 2, 0) - 0.5).abs() < 0.001);
    // out-of-range index -> 0
    assert_eq!(bin_value(&freq, 2, 99), 0.0);
    assert_eq!(bin_value(&freq, 4, 3), 0.0);
    let empty: Vec<u8> = vec![];
    assert_eq!(bin_value(&empty, 4, 0), 0.0);
  }
}
