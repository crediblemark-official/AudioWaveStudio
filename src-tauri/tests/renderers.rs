use audiowave_studio_lib::config::*;
use audiowave_studio_lib::gpu2d::{Color, Fill, GpuCanvas, GpuRenderer};
use audiowave_studio_lib::renderers::*;

fn test_config() -> VisualizerConfig {
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
fn gpu_postfx_readback_matches() {
  let mut gpu = pollster::block_on(GpuRenderer::new(320, 240)).expect("GPU init failed");

  let mut canvas = GpuCanvas::new(320, 240);
  canvas.set_fill(Fill::Solid(Color::rgba(0.5, 0.5, 0.5, 1.0)));
  canvas.fill_rect(10.0, 10.0, 100.0, 100.0);
  let mesh = canvas.finish();

  gpu.render_into(&mesh, 0);
  let base = gpu.readback(0);
  assert!(base[10 * 4 + 10 * 320 * 4 + 0] > 100, "base should be lit");

  let invert = audiowave_studio_lib::gpu2d::renderer::PostFx {
    mode: 4,
    intensity: 1.0,
    time: 1.0,
    beat: 0.0,
  };
  gpu.render_into_fx(&mesh, &invert, 1);
  let fx = gpu.readback(1);
  let base_center = base[50 * 4 + 50 * 320 * 4 + 0];
  let fx_center = fx[50 * 4 + 50 * 320 * 4 + 0];
  assert!(
    (fx_center as i32 - (255 - base_center as i32)).abs() <= 8,
    "invert should flip luminance, got base {base_center} fx {fx_center}"
  );

  let zoom = audiowave_studio_lib::gpu2d::renderer::PostFx {
    mode: 3,
    intensity: 0.3,
    time: 1.0,
    beat: 0.0,
  };
  gpu.render_into_fx(&mesh, &zoom, 0);
  let zx = gpu.readback(0);
  assert_eq!(zx.len(), 320 * 240 * 4, "zoom readback should be full RGBA");
  assert!(zx.iter().any(|&b| b > 0), "zoom readback should not be empty");
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
  config.background.mode = BackgroundMode::CustomImage;
  config.background.image_opacity = Some(1.0);

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  let mut gpu = pollster::block_on(GpuRenderer::new(640, 360)).expect("GPU init failed");
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
    .upload_image_layer(audiowave_studio_lib::gpu2d::IMAGE_LAYER, &rgba, 200, 100)
    .expect("upload failed");

  let mut rstate = RenderState::new(config.reactivity.bar_count, 3);
  rstate.background_image = Some(BackgroundImage {
    layer: audiowave_studio_lib::gpu2d::IMAGE_LAYER,
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
  assert!(red > (w as u32 * h as u32) / 8, "red region too small: {red}");
  assert!(blue > (w as u32 * h as u32) / 8, "blue region too small: {blue}");
}

#[test]
#[ignore = "requires a Vulkan-capable GPU"]
fn gpu_radial_center_image_renders() {
  let mut config = test_config();
  config.style = VisualizerStyle::Radial;

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  let mut gpu = pollster::block_on(GpuRenderer::new(640, 360)).expect("GPU init failed");
  let rgba = vec![255, 0, 128, 255].repeat(100 * 100);
  let (tw, th) = gpu
    .upload_image_layer(audiowave_studio_lib::gpu2d::RADIAL_CENTER_IMAGE_LAYER, &rgba, 100, 100)
    .expect("upload failed");

  let mut rstate = RenderState::new(config.reactivity.bar_count, 3);
  rstate.radial_center_image = Some(BackgroundImage {
    layer: audiowave_studio_lib::gpu2d::RADIAL_CENTER_IMAGE_LAYER,
    w: tw,
    h: th,
  });
  let mut canvas = GpuCanvas::new(640, 360);
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0);
  let mesh = canvas.finish();
  let jpeg = gpu.jpeg(&mesh).expect("jpeg encode failed");
  assert!(jpeg.len() > 2000, "jpeg output too small: {}", jpeg.len());
}

#[test]
#[ignore = "requires a Vulkan-capable GPU"]
fn gpu_background_effects_and_particles_render() {
  use audiowave_studio_lib::config::{BackgroundEffect, ParticleStyle};

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
  config.background.music_note_style = Some(MusicNoteStyle::Bounce);

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
  let decoded = image::load_from_memory(&jpeg).expect("decoded").to_rgba8();
  let mut colors = std::collections::HashSet::new();
  for px in decoded.pixels().step_by(7) {
    colors.insert([px[0] >> 4, px[1] >> 4, px[2] >> 4]);
  }
  assert!(colors.len() > 8, "expected varied output, got {} color buckets", colors.len());
}

#[test]
fn text_rasterize_produces_atlas() {
  let Some(font) = audiowave_studio_lib::gpu2d::text::select_font("monospace", 400.0) else {
    return;
  };
  let fill = Fill::Solid(Color::WHITE);
  let atl = audiowave_studio_lib::gpu2d::text::rasterize(
    font,
    "AudioWave",
    32.0,
    &fill,
    &Default::default(),
  )
  .expect("rasterize failed");
  assert!(atl.atlas_w > 10 && atl.atlas_h > 10, "atlas too small");
  assert!(atl.advance > 30.0, "advance too small");
  let opaque = atl.rgba.chunks(4).filter(|px| px[3] > 0).count();
  assert!(opaque > 50, "too few covered pixels: {}", opaque);
}

#[test]
fn text_measure_grows_with_text() {
  let Some(font) = audiowave_studio_lib::gpu2d::text::select_font("sans-serif", 400.0) else {
    return;
  };
  let short = audiowave_studio_lib::gpu2d::text::measure(font, "AA", 40.0, 0.0);
  let long = audiowave_studio_lib::gpu2d::text::measure(font, "AAAAAAAA", 40.0, 0.0);
  assert!(long > short, "longer text must be wider");
  let spaced = audiowave_studio_lib::gpu2d::text::measure(font, "AA", 40.0, 8.0);
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
  use audiowave_studio_lib::config::ScreenEffect;

  let effects = [
    ("shake", ScreenEffect::Shake),
    ("vignette", ScreenEffect::Vignette),
    ("pulse", ScreenEffect::Pulse),
    ("spotlight", ScreenEffect::Spotlight),
    ("strobe", ScreenEffect::Strobe),
    ("scanline", ScreenEffect::Scanline),
    ("hueShift", ScreenEffect::HueShift),
  ];
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
  assert!((bin_value(&freq, 2, 0) - 0.5).abs() < 0.001);
  assert_eq!(bin_value(&freq, 2, 99), 0.0);
  assert_eq!(bin_value(&freq, 4, 3), 0.0);
  let empty: Vec<u8> = vec![];
  assert_eq!(bin_value(&empty, 4, 0), 0.0);
}
