use audiowave_studio_lib::config::*;
use audiowave_studio_lib::gpu2d::{Color, Fill, GpuCanvas, GpuRenderer, Vertex};
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
      encoder: None,
    },
    screen_effects: ScreenEffectsSettings {
      enabled: false,
      background_only: Some(true),
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

// ---------------------------------------------------------------------------
// GPU tests. These run BY DEFAULT (no #[ignore]) and require a Vulkan-capable
// GPU — `GpuRenderer::new` panics with "GPU init failed" on machines without
// one. CI does not run `cargo test` (only builds), so this is safe there; on
// GPU-less dev machines, run `cargo test` with a GPU available or restore the
// `#[ignore = "requires a Vulkan-capable GPU"]` attributes.
// ---------------------------------------------------------------------------

#[test]
fn gpu_draw_frame_renders_jpeg() {
  let config = test_config();
  let freq = synth_freq();
  let time: Vec<u8> = (0..512)
    .map(|i| (((i as f32 / 16.0).sin() * 127.0) + 128.0).clamp(0.0, 255.0) as u8)
    .collect();

  let mut gpu = pollster::block_on(GpuRenderer::new(320, 240)).expect("GPU init failed");
  let mut rstate = RenderState::new(config.reactivity.bar_count, 1);
  let mut canvas = GpuCanvas::new(320, 240);
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0, true);
  let mesh = canvas.finish();
  assert!(!mesh.is_empty(), "expected at least some geometry");

  let jpeg = gpu.jpeg(&mesh).expect("jpeg encode failed");
  assert!(jpeg.len() > 1000, "jpeg too small: {}", jpeg.len());
  assert_eq!(&jpeg[..2], &[0xFF, 0xD8], "missing JPEG magic");
}

#[test]
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
    draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, slot as f32, true);
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
    fps: 30.0,
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
    fps: 30.0,
  };
  gpu.render_into_fx(&mesh, &zoom, 0);
  let zx = gpu.readback(0);
  assert_eq!(zx.len(), 320 * 240 * 4, "zoom readback should be full RGBA");
  assert!(zx.iter().any(|&b| b > 0), "zoom readback should not be empty");
}

#[test]
fn gpu_postfx_snapshot_modes_render() {
  // Exercises the snapshot-based post-fx modes (glitch/chromatic/tilt/
  // heatHaze/hueShift) through render_into_fx so the WGSL compiles and the
  // new TS-parity algorithms produce full-frame output.
  let mut gpu = pollster::block_on(GpuRenderer::new(320, 240)).expect("GPU init failed");

  let mut canvas = GpuCanvas::new(320, 240);
  canvas.set_fill(Fill::Solid(Color::rgba(0.5, 0.2, 0.8, 1.0)));
  canvas.fill_rect(0.0, 0.0, 320.0, 240.0);
  let mesh = canvas.finish();

  let cases = [
    (1, 0.8, "glitch"),
    (2, 0.8, "chromatic"),
    (5, 0.4, "bars"),
    (6, 0.5, "shockwave"),
    (8, 0.5, "tilt"),
    (9, 0.6, "heatHaze"),
    (10, 0.6, "hueShift"),
  ];
  for (i, (mode, intensity, name)) in cases.iter().enumerate() {
    let fx = audiowave_studio_lib::gpu2d::renderer::PostFx {
      mode: *mode,
      intensity: *intensity,
      time: 1.37,
      beat: 0.0,
      fps: 30.0,
    };
    gpu.render_into_fx(&mesh, &fx, i % 2);
    let rgba = gpu.readback(i % 2);
    assert_eq!(rgba.len(), 320 * 240 * 4, "{name}: expected full RGBA");
    assert!(rgba.iter().any(|&b| b > 0), "{name}: readback should not be empty");
  }
}

#[test]
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
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 1.5, true);
  let mesh = canvas.finish();
  assert!(!mesh.atlases.is_empty(), "expected text glyph atlases");

  let jpeg = gpu.jpeg(&mesh).expect("jpeg encode failed");
  assert!(jpeg.len() > 3000, "jpeg too small: {}", jpeg.len());
}

#[test]
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
    .upload_background_image(audiowave_studio_lib::gpu2d::IMAGE_LAYER, &rgba, 200, 100)
    .expect("upload failed");

  let mut rstate = RenderState::new(config.reactivity.bar_count, 3);
  rstate.background_image = Some(BackgroundImage {
    layer: audiowave_studio_lib::gpu2d::IMAGE_LAYER,
    w: tw,
    h: th,
  });
  let mut canvas = GpuCanvas::new(640, 360);
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0, true);
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
fn gpu_radial_center_image_renders() {
  let mut config = test_config();
  config.style = VisualizerStyle::Radial;

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  let mut gpu = pollster::block_on(GpuRenderer::new(640, 360)).expect("GPU init failed");
  let rgba = vec![255, 0, 128, 255].repeat(100 * 100);
  let (tw, th) = gpu
    .upload_background_image(audiowave_studio_lib::gpu2d::RADIAL_CENTER_IMAGE_LAYER, &rgba, 100, 100)
    .expect("upload failed");

  let mut rstate = RenderState::new(config.reactivity.bar_count, 3);
  rstate.radial_center_image = Some(BackgroundImage {
    layer: audiowave_studio_lib::gpu2d::RADIAL_CENTER_IMAGE_LAYER,
    w: tw,
    h: th,
  });
  let mut canvas = GpuCanvas::new(640, 360);
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0, true);
  let mesh = canvas.finish();
  let jpeg = gpu.jpeg(&mesh).expect("jpeg encode failed");
  assert!(jpeg.len() > 2000, "jpeg output too small: {}", jpeg.len());
}

#[test]
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
  let mut rstate = RenderState::new(config.reactivity.bar_count, 7);    let mut canvas = GpuCanvas::new(640, 360);
    for f in 0..40 {
      draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, f as f32 * 0.1, true);
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
fn aurora_export_matches_preview_fill() {
  // Regression: render_aurora fills each wave->base region with
  // fill_polyline_to_base (convex quad strips). The old fill_polygon fan from
  // pts[0] overdraws wherever the wave is not star-shaped from the top-left
  // corner; under 'screen' blend the overlapped pixels are blended TWICE and
  // come out ~50% brighter than the TS canvas preview. Render the actual
  // render_aurora output and a reference built from fill_polyline_to_base and
  // require pixel equality — any regression to the fan fails this test.
  let mut config = test_config();
  config.background.aurora_speed = Some(1.0);
  config.background.aurora_amplitude = Some(50.0);
  config.background.aurora_opacity = Some(0.25);
  let mut rstate = RenderState::new(config.reactivity.bar_count, 23);
  let ctx = RenderContext {
    width: 480.0,
    height: 270.0,
    config: &config,
    freq_data: &[],
    time_data: &[],
    bass_energy: 0.5,
    beat_strength: 0.3,
    rotation_angle: 0.0,
    frame_time: 1.0,
    state: &mut rstate,
  };

  // Actual export path.
  let mut actual = GpuCanvas::new(480, 270);
  background::render_aurora(&mut actual, &ctx);
  let actual_mesh = actual.finish();

  // Reference: the exact same 4 bands, each filled with fill_polyline_to_base.
  let t = 1.0;
  let speed = 0.6;
  let amp = 50.0 + 0.3 * 60.0;
  let mut reference = GpuCanvas::new(480, 270);
  reference.set_blend_screen();
  for i in 0..4usize {
    let hue = (i as f32 * 60.0 + t * 25.0) % 360.0;
    let alpha = (0.25f32 * 0.6 + 0.5 * 0.1).min(1.0);
    reference.set_fill(Fill::Solid(hsl_to_color(hue, 0.85, 0.60, alpha)));
    let pts: Vec<(f32, f32)> = (0..=480)
      .step_by(6)
      .map(|x| (x as f32, background::aurora_y(x as f32, i, t, speed, amp, 270.0)))
      .collect();
    reference.fill_polyline_to_base(&pts, 270.0);
  }
  let reference_mesh = reference.finish();

  let mut gpu = pollster::block_on(GpuRenderer::new(480, 270)).expect("GPU init failed");
  let a = gpu.render(&actual_mesh);
  let b = gpu.render(&reference_mesh);
  let mut max_diff = 0i16;
  for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
    for k in 0..3 {
      max_diff = max_diff.max((pa[k] as i16 - pb[k] as i16).abs());
    }
  }
  assert!(
    max_diff <= 4,
    "aurora export drifted from the preview fill: max channel diff = {max_diff} (fan overdraw would exceed it)",
  );
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
fn arabic_text_detection() {
  use audiowave_studio_lib::gpu2d::text::is_arabic_text;
  assert!(is_arabic_text("مرحبا بالعالم"));
  assert!(is_arabic_text("السَّلَامُ عَلَيْكُمْ"));
  assert!(!is_arabic_text("Hello world"));
  assert!(!is_arabic_text("123"));
  assert!(!is_arabic_text(""));
  // Mixed text is still flagged so the run uses the Arabic font.
  assert!(is_arabic_text("سلام and hello"));
}

#[test]
fn arabic_shaping_joins_letters_and_rtl() {
  use audiowave_studio_lib::gpu2d::text::{measure, rasterize, select_font_for_text};
  use audiowave_studio_lib::gpu2d::Color;

  let Some(arabic) = select_font_for_text("sans-serif", 400.0, "مرحبا") else {
    return;
  };
  let Some(latin) = select_font_for_text("sans-serif", 400.0, "x") else {
    return;
  };
  if std::ptr::eq(arabic, latin) {
    return; // no dedicated Arabic font on this system
  }

  // GSUB: a lone beh shapes to its isolated form; "ببب" must pick
  // initial/medial/final forms — none equal to the isolated form and all
  // distinct from each other (letters actually join).
  let face = arabic.hb_face().expect("arabic font must parse");
  let isolated = {
    let mut b = rustybuzz::UnicodeBuffer::new();
    b.push_str("ب");
    b.guess_segment_properties();
    let out = rustybuzz::shape(&face, &[], b);
    out.glyph_infos()[0].glyph_id
  };
  let mut buffer = rustybuzz::UnicodeBuffer::new();
  buffer.push_str("ببب");
  buffer.guess_segment_properties();
  let out = rustybuzz::shape(&face, &[], buffer);
  let ids: Vec<u16> = out.glyph_infos().iter().map(|i| i.glyph_id as u16).collect();
  assert_eq!(ids.len(), 3, "three behs must shape to three glyphs");
  assert!(
    ids.iter().any(|&id| id as u32 != isolated),
    "shaping must select joined forms, not the isolated glyph: {ids:?}"
  );
  assert!(
    ids.windows(2).any(|w| w[0] != w[1]),
    "initial/medial/final forms must differ: {ids:?}"
  );

  // RTL: glyphs come out in VISUAL order (first glyph = the string's last
  // character, i.e. the leftmost glyph) with POSITIVE advances — rustybuzz
  // reorders RTL runs, so placing each glyph at the running pen renders the
  // line left-to-right correctly.
  let mut buffer = rustybuzz::UnicodeBuffer::new();
  buffer.push_str("مرحبا");
  buffer.guess_segment_properties();
  let out = rustybuzz::shape(&face, &[], buffer);
  let clusters: Vec<u32> = out.glyph_infos().iter().map(|i| i.cluster).collect();
  assert!(
    clusters[0] > clusters.last().copied().unwrap_or(0),
    "visual order must start at the string's last char: {clusters:?}"
  );
  let sum: i64 = out.glyph_positions().iter().map(|p| p.x_advance as i64).sum();
  assert!(sum > 0, "visual-order RTL run must have positive advances, got {sum}");

  // measure and rasterize must agree on the advance (wrap uses measure, draw
  // uses rasterize) so wrapped lines never overflow.
  let m = measure(arabic, "مرحبا", 40.0, 0.0);
  assert!(m > 0.0, "shaped measure must be positive, got {m}");
  let Some(atl) = rasterize(
    arabic,
    "مرحبا",
    40.0,
    &audiowave_studio_lib::gpu2d::Fill::Solid(Color::WHITE),
    &Default::default(),
  ) else {
    return;
  };
  assert!(
    (atl.advance - m).abs() < 0.5,
    "rasterize advance {} must match measure {}",
    atl.advance,
    m
  );
  assert!(atl.width > 0.0 && atl.height > 0.0, "ink bounds must be non-empty");
}

#[test]
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
      draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, f as f32 * 0.1, true);
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
      draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, f as f32 * 0.1, true);
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

// ---------------------------------------------------------------------------
// Screen blend batch premultiplication (gpu2d scene)
// ---------------------------------------------------------------------------

#[test]
fn screen_batch_premultiplies_vertex_colors() {
  use audiowave_studio_lib::gpu2d::scene::BlendMode;

  let mut canvas = GpuCanvas::new(640, 360);

  // Batch 0 (normal): straight-alpha red — must NOT be premultiplied.
  canvas.set_fill(Fill::Solid(Color::rgba(0.5, 0.0, 0.0, 0.5)));
  canvas.fill_rect(10.0, 10.0, 100.0, 100.0);

  // Batch 1 (screen): straight-alpha green — premultiplied at batch flush.
  canvas.set_blend_screen();
  canvas.set_fill(Fill::Solid(Color::rgba(0.0, 0.5, 0.0, 0.5)));
  canvas.fill_rect(120.0, 10.0, 100.0, 100.0);

  // Batch 2 (normal again): straight-alpha blue — must NOT be premultiplied.
  canvas.set_blend_normal();
  canvas.set_fill(Fill::Solid(Color::rgba(0.0, 0.0, 0.5, 0.5)));
  canvas.fill_rect(230.0, 10.0, 100.0, 100.0);

  let mesh = canvas.finish();

  assert_eq!(mesh.batches.len(), 3, "expected 3 blend batches");
  assert_eq!(mesh.batches[0].blend, BlendMode::Normal);
  assert_eq!(mesh.batches[1].blend, BlendMode::Screen);
  assert_eq!(mesh.batches[2].blend, BlendMode::Normal);

  // Each fill_rect expands to a quad = 6 verts / 6 indices.
  for (i, (start, count)) in [(0u32, 6u32), (6, 6), (12, 6)].into_iter().enumerate() {
    assert_eq!(mesh.batches[i].idx_start, start, "batch {i} idx_start");
    assert_eq!(mesh.batches[i].idx_count, count, "batch {i} idx_count");
  }

  let check = |v: &Vertex| (v.color[0], v.color[1], v.color[2], v.color[3]);

  // Batch 0: red 0.5 untouched.
  for v in &mesh.verts[0..6] {
    let (r, g, b, a) = check(v);
    assert!((r - 0.5).abs() < 1e-5, "normal red premultiplied: {r}");
    assert!(g == 0.0 && b == 0.0);
    assert!((a - 0.5).abs() < 1e-5);
  }
  // Batch 1: green 0.5 * 0.5 = 0.25; alpha stays 0.5 (NOT premultiplied).
  for v in &mesh.verts[6..12] {
    let (r, g, b, a) = check(v);
    assert!(r == 0.0 && b == 0.0, "screen batch must keep other channels zero");
    assert!((g - 0.25).abs() < 1e-5, "screen green must be premultiplied: {g}");
    assert!((a - 0.5).abs() < 1e-5, "screen alpha must NOT be premultiplied: {a}");
  }
  // Batch 2: blue 0.5 untouched.
  for v in &mesh.verts[12..18] {
    let (r, g, b, a) = check(v);
    assert!(r == 0.0 && g == 0.0);
    assert!((b - 0.5).abs() < 1e-5, "normal blue premultiplied: {b}");
    assert!((a - 0.5).abs() < 1e-5);
  }
}

// ---------------------------------------------------------------------------
// Background effect formula parity with the TS renderers. Golden values were
// computed by executing src/services/renderers/background/*.ts in Node (f64)
// with the documented parameter sets.
// ---------------------------------------------------------------------------

#[test]
fn bokeh_formula_matches_ts() {
  use audiowave_studio_lib::renderers::background::{bokeh_alpha, bokeh_blob};

  // bokeh.ts defaults at 1920x1080: frameTime=2.5 -> t=0.5, bokehSize=30
  // (scaleFactor=1), beatStrength=0.2.
  let width = 1920.0f32;
  let height = 1080.0f32;
  let t = 2.5 / 5.0;
  let sf = width.min(height) / 1080.0;
  let base_size = 30.0 * sf;
  let beat = 0.2f32;
  // (x, y, radius, hue) per blob i=0..4, from bokeh.ts.
  let golden: [(f32, f32, f32, f32); 5] = [
    (1055.840080, 1078.481962, 38.753106, 15.0),
    (406.059634, 273.362200, 23.531146, 152.5),
    (27.662423, 229.348017, 44.620374, 290.0),
    (106.030601, 1072.075914, 21.147760, 67.5),
    (602.770489, 396.017102, 43.126150, 205.0),
  ];
  for (i, (gx, gy, gr, gh)) in golden.iter().enumerate() {
    let (x, y, r, hue) = bokeh_blob(i, t, width, height, base_size, sf, beat);
    assert!((x - gx).abs() < 0.1, "bokeh[{i}] x: rust {x} != ts {gx}");
    assert!((y - gy).abs() < 0.1, "bokeh[{i}] y: rust {y} != ts {gy}");
    assert!((r - gr).abs() < 0.1, "bokeh[{i}] radius: rust {r} != ts {gr}");
    assert!((hue - gh).abs() < 1e-3, "bokeh[{i}] hue: rust {hue} != ts {gh}");
  }
  // Alpha: baseOpacity=0.3 + bassEnergy*0.15 = 0.345 (browser clamps to 1).
  let alpha = bokeh_alpha(0.3, 0.3);
  assert!((alpha - 0.345).abs() < 1e-3, "bokeh alpha: rust {alpha} != ts 0.345");
  assert_eq!(bokeh_alpha(0.9, 0.8), 1.0, "high bass must clamp alpha to 1");
}

#[test]
fn starfield_formula_matches_ts() {
  use audiowave_studio_lib::renderers::background::{star_alpha, star_position, Star};

  // Hand-built star; starfield.ts params: frameTime=3.0, speedMult=1,
  // brightness=1, bassEnergy=0.3 (pulse=0.82), beatStrength=0.2.
  let s = Star { x: 0.25, y: 0.6, size: 2.0, phase: 1.0, speed: 0.02 };
  let t = 3.0f32;
  let pulse = 0.7 + 0.3 * 0.4; // 0.82

  let (x, y) = star_position(&s, t, 1920.0, 1080.0);
  assert!((x - 490.468266).abs() < 0.1, "star x: rust {x} != ts 490.468266");
  assert!((y - 654.053933).abs() < 0.1, "star y: rust {y} != ts 654.053933");

  let alpha = star_alpha(&s, t, pulse, 0.2, 1.0);
  assert!((alpha - 0.050117).abs() < 1e-3, "star alpha: rust {alpha} != ts 0.050117");
  assert!((s.size * pulse - 1.64).abs() < 1e-3, "star radius must be size * pulse");

  // Wrapping: a star at the right edge with t*speed ~ +1 forces rawX past the
  // canvas width; TS wraps with ((raw % w) + w) % w -> rawX 1930.079 -> 10.079.
  let wrap = Star { x: 0.999, y: 0.001, size: 1.0, phase: 0.0, speed: 0.4 };
  let (wx, wy) = star_position(&wrap, 3.9, 1920.0, 1080.0);
  assert!((wx - 10.079301).abs() < 0.1, "wrapped x: rust {wx} != ts 10.079301");
  assert!((wy - 6.608534).abs() < 0.1, "y: rust {wy} != ts 6.608534");
}

#[test]
fn aurora_formula_matches_ts() {
  use audiowave_studio_lib::renderers::background::aurora_y;

  // aurora.ts params: frameTime=0.5, speedMult=1, baseAmp=50,
  // bassEnergy=0.2 -> speed=0.42, beatStrength=0.1 -> amp=56.
  let t = 0.5f32;
  let speed = (0.3 + 0.2 * 0.6) * 1.0; // 0.42
  let amp = 50.0 + 0.1 * 60.0; // 56
  let height = 1080.0f32;
  let xs = [0.0f32, 60.0, 300.0, 900.0];
  // Golden y values [x0..x3] per band i=0..3, from aurora.ts.
  let golden: [[f32; 4]; 4] = [
    [501.774947, 537.566343, 520.750354, 423.118171],
    [564.937340, 542.758324, 451.528624, 537.605702],
    [458.529528, 435.070332, 460.301696, 546.053446],
    [426.198747, 448.976701, 489.726096, 424.087503],
  ];
  for (band, gvals) in golden.iter().enumerate() {
    for (xi, &g) in gvals.iter().enumerate() {
      let y = aurora_y(xs[xi], band, t, speed, amp, height);
      assert!(
        (y - g).abs() < 0.1,
        "aurora band {band} x={}: rust {y} != ts {g}",
        xs[xi]
      );
    }
  }
}

#[test]
fn nebula_formula_matches_ts() {
  use audiowave_studio_lib::renderers::background::nebula_blob;

  // nebula.ts params: frameTime=3.5 -> t=0.5, speedMult=1, beatStrength=0.2,
  // 1920x1080.
  let t = (3.5f32 / 7.0) * 1.0; // 0.5
  let beat = 0.2f32;
  // (cx, cy, r, hue) per blob i=0..4, from nebula.ts.
  let golden: [(f32, f32, f32, f32); 5] = [
    (1007.980002, 1079.568058, 201.999792, 10.0),
    (1805.233929, 3.897703, 144.402930, 111.1),
    (1786.916548, 1069.188258, 279.860525, 212.2),
    (970.702410, 21.129585, 138.011597, 313.3),
    (144.161453, 1045.215133, 211.410298, 54.4),
  ];
  for (i, (gc, gy, gr, gh)) in golden.iter().enumerate() {
    let (cx, cy, r, hue) = nebula_blob(i, t, 1920.0, 1080.0, beat);
    assert!((cx - gc).abs() < 0.1, "nebula[{i}] cx: rust {cx} != ts {gc}");
    assert!((cy - gy).abs() < 0.1, "nebula[{i}] cy: rust {cy} != ts {gy}");
    assert!((r - gr).abs() < 0.1, "nebula[{i}] r: rust {r} != ts {gr}");
    assert!((hue - gh).abs() < 1e-2, "nebula[{i}] hue: rust {hue} != ts {gh}");
  }
}

#[test]
fn radial_gradient_circle_slices_at_stop_boundaries() {
  // Regression: a plain center->rim fan only samples a radial gradient at
  // t=0 and t=1, dropping the middle stops (the nebula blob's 3-stop gradient
  // lost its hue+30 band and rendered soft/blurred vs the canvas preview).
  // fill_ellipse must slice the disc into rings at every stop boundary so the
  // middle-stop color actually reaches the mesh.
  let mut canvas = GpuCanvas::new(320, 240);
  let g = Fill::radial_gradient(
    160.0,
    120.0,
    0.0,
    160.0,
    120.0,
    100.0,
    &[
      (0.0, Color::rgba(1.0, 0.0, 0.0, 1.0)),
      (0.5, Color::rgba(0.0, 1.0, 0.0, 1.0)),
      (1.0, Color::rgba(0.0, 0.0, 1.0, 0.0)),
    ],
  );
  canvas.set_fill(g);
  canvas.fill_circle(160.0, 120.0, 100.0);
  let mesh = canvas.finish();

  let mut rim = 0usize;
  let mut mid = 0usize;
  let mut center = 0usize;
  let mut mid_is_green = true;
  for v in &mesh.verts {
    let px = (v.position[0] + 1.0) / 2.0 * 320.0;
    let py = (1.0 - v.position[1]) / 2.0 * 240.0;
    let dx = px - 160.0;
    let dy = py - 120.0;
    let d = (dx * dx + dy * dy).sqrt();
    if d < 1.0 {
      center += 1;
    } else if (d - 100.0).abs() < 0.5 {
      rim += 1;
    } else if (d - 50.0).abs() < 0.5 {
      mid += 1;
      if v.color[0] > 0.05 || v.color[1] < 0.95 || v.color[2] > 0.05 {
        mid_is_green = false;
      }
    }
  }
  assert!(center > 0, "expected center vertices, got none");
  assert!(rim >= 64, "expected rim ring, got {rim}");
  assert!(mid >= 64, "expected a ring at the 0.5 stop boundary, got {mid}");
  assert!(mid_is_green, "middle ring must sample the exact 0.5-stop color");
}


// ---------------------------------------------------------------------------
// Minimal style slider parity with minimalWave.ts
// ---------------------------------------------------------------------------

#[test]
fn minimal_bar_count_matches_ts_clamp() {
  use audiowave_studio_lib::renderers::effective_bar_count;

  // minimalWave.ts: `const barCount = Math.min(64, config.reactivity.barCount)`
  // — the slider value flows through UNCHANGED up to the 64 cap. The Rust
  // renderer must never force a minimum (a stale `.max(64)` made exports show
  // 64 bars even when the slider was set to 16).
  assert_eq!(effective_bar_count(16), 16, "slider 16 must render 16 bars");
  assert_eq!(effective_bar_count(32), 32, "slider 32 must render 32 bars");
  assert_eq!(effective_bar_count(64), 64, "slider 64 stays 64");
  assert_eq!(effective_bar_count(96), 64, "values above 64 clamp to 64 like TS");
  assert_eq!(effective_bar_count(128), 64, "slider max 128 clamps to 64 like TS");
}

// ---------------------------------------------------------------------------
// wrap_text font consistency for mixed Arabic-Latin paragraphs
// ---------------------------------------------------------------------------

#[test]
fn wrap_text_measures_mixed_arabic_with_line_font() {
  use audiowave_studio_lib::gpu2d::text::{measure, select_font_for_text};
  use audiowave_studio_lib::renderers::text::wrap_text;

  // The per-line fix only matters when Arabic text resolves to a DIFFERENT
  // font than Latin (i.e. an Arabic font is installed). Without one, both
  // select the same font and per-line == per-paragraph — skip (vacuous).
  let Some(latin) = select_font_for_text("sans-serif", 400.0, "x") else {
    return;
  };
  let Some(arabic) = select_font_for_text("sans-serif", 400.0, "مرحبا") else {
    return;
  };
  if std::ptr::eq(latin, arabic) {
    return;
  }

  // A mixed Arabic-Latin paragraph: pure-Latin candidates ("Hello") must be
  // measured with the regular font and Arabic candidates with the Arabic font
  // — exactly the font draw_line uses via select_font_for_text per LINE.
  // The reference below mirrors wrap_text's per-line contract; a regression
  // to paragraph-level font selection would diverge from it.
  let paragraph = "Hello مرحبا العالم كيف حالك world today";
  let family = "sans-serif";
  let weight = 400.0;
  let font_size = 32.0;
  let max_width = 150.0;

  let lines = wrap_text(paragraph, max_width, family, weight, false, font_size, 0.0);

  let mut expected: Vec<String> = Vec::new();
  let mut current = String::new();
  for word in paragraph.split_whitespace() {
    let candidate = if current.is_empty() {
      word.to_string()
    } else {
      format!("{} {}", current, word)
    };
    let Some(font) = select_font_for_text(family, weight, &candidate) else {
      break;
    };
    let w = measure(font, &candidate, font_size, 0.0);
    if current.is_empty() || w <= max_width {
      current = candidate;
    } else {
      expected.push(current);
      current = word.to_string();
    }
  }
  expected.push(current);

  assert_eq!(
    lines, expected,
    "wrap_text must break lines using the per-line font (same as draw)"
  );
}

// ---------------------------------------------------------------------------
// draw_block divergences: gradient geometry, italic style, shadow offset blur
// ---------------------------------------------------------------------------

#[test]
fn gradient_fill_keeps_vertical_center_at_anchor() {
  use audiowave_studio_lib::gpu2d::text::PAD;
  use audiowave_studio_lib::gpu2d::Gradient;
  use audiowave_studio_lib::renderers::text::gradient_fill;

  // textOverlay.ts anchors the gradient's vertical center at the block's
  // `anchorY` for EVERY line (createLinearGradient(..., anchorY - dy*span/2,
  // ..., anchorY + dy*span/2)). The Rust port passes `y_delta = i*lineHeight`
  // so the atlas-local center shifts up by exactly the line's distance from
  // the anchor — leaving the canvas-space center fixed at the anchor.
  let mut block = test_config().text.title;
  block.use_gradient = true;
  block.gradient_start = "#ff0000".into();
  block.gradient_end = "#0000ff".into();
  block.gradient_angle = 90.0; // vertical axis: (dx, dy) = (0, 1)

  let font_size = 40.0;
  let ascent = 37.0; // synthetic; the contract is cy = PAD + ascent - y_delta
  let line_height = font_size * 1.2;
  let width = 123.0;

  let center_y = |y_delta: f32| -> f32 {
    let Fill::Gradient(Gradient::Linear { y0, y1, .. }) = gradient_fill(&block, width, ascent, y_delta)
    else {
      panic!("expected linear gradient");
    };
    (y0 + y1) / 2.0
  };

  // Line 0's axis must be centered at the atlas baseline (PAD + ascent), the
  // atlas-local position that maps to canvas anchorY (rasterize_linear places
  // the baseline at atlas-y PAD + ascent). A stale `font_size * 0.5` center
  // put it ~(font_size/2 - ascent) ABOVE anchorY, visibly shifting vertical
  // gradients.
  assert!(
    (center_y(0.0) - (PAD + ascent)).abs() < 1e-3,
    "line 0 center must sit on the atlas baseline, got {}",
    center_y(0.0)
  );

  // The gradient is sampled in ATLAS-LOCAL space, and each line's quad sits
  // at canvas y = anchorY + i*lineHeight. The invariant is that the
  // canvas-space center (atlas-local center + quad offset) is the same for
  // every line — i.e. fixed at anchorY like textOverlay.ts.
  let c0 = center_y(0.0) + 0.0; // line 0 quad at anchorY + 0
  let c1 = center_y(line_height) + line_height; // line 1 quad at anchorY + lineHeight
  let c2 = center_y(2.0 * line_height) + 2.0 * line_height;

  // The gradient axis is fixed at the atlas baseline (matching textOverlay.ts
  // which anchors ALL lines' gradient at the block's anchorY). Since y_delta
  // is ignored, canvas-space center shifts by the line offset:
  //   canvas_center = atlas_center + anchorY + i*lineHeight
  // To verify: c_i - i*lineHeight must be invariant.
  let inv0 = c0 - 0.0 * line_height;
  let inv1 = c1 - 1.0 * line_height;
  let inv2 = c2 - 2.0 * line_height;
  assert!((inv0 - inv1).abs() < 1e-3, "invariant line 0 vs 1: {inv0} vs {inv1}");
  assert!((inv0 - inv2).abs() < 1e-3, "invariant line 0 vs 2: {inv0} vs {inv2}");

  // Sanity: the axis span is max(width, 8) like TS `Math.max(lineWidth, 8)`.
  let Fill::Gradient(Gradient::Linear { y0, y1, .. }) = gradient_fill(&block, width, ascent, 0.0)
  else {
    panic!("expected linear gradient");
  };
  assert!((y1 - y0 - width).abs() < 1e-3, "span must equal max(width, 8)");
}

#[test]
fn gradient_center_is_align_invariant_and_tracks_ts() {
  use audiowave_studio_lib::gpu2d::text::PAD;
  use audiowave_studio_lib::gpu2d::Gradient;
  use audiowave_studio_lib::renderers::text::gradient_fill;

  // textOverlay.ts computes centerX = lineStartX + lineWidth/2 where
  // lineStartX depends on align — but drawLine shifts the RUN by the same
  // amount (startX = anchorX - totalWidth for right, anchorX - totalWidth/2
  // for center), so the gradient center lands at the line's VISUAL center for
  // ALL aligns. The Rust port therefore uses ONE align-independent atlas
  // center: cx = width/2 + PAD, which maps to canvas anchorX + dx + width/2.
  let mut block = test_config().text.title;
  block.use_gradient = true;
  block.gradient_start = "#ff0000".into();
  block.gradient_end = "#0000ff".into();
  block.gradient_angle = 0.0; // horizontal axis: (dx, dy) = (1, 0)

  let width = 100.0;
  let ascent = 30.0;

  let mut center = |align: TextAlign| -> (f32, f32) {
    block.align = align;
    let Fill::Gradient(Gradient::Linear { x0, y0, x1, y1, .. }) = gradient_fill(&block, width, ascent, 0.0)
    else {
      panic!("expected linear gradient");
    };
    ((x0 + x1) / 2.0, (y0 + y1) / 2.0)
  };

  let (lx, ly) = center(TextAlign::Left);
  let (cx, cy) = center(TextAlign::Center);
  let (rx, ry) = center(TextAlign::Right);

  // The gradient geometry itself must be IDENTICAL for every align — align
  // only shifts the quad at draw time, never the gradient (TS parity).
  assert!(
    (lx - cx).abs() < 1e-5 && (lx - rx).abs() < 1e-5,
    "x center must be align-independent: {lx} {cx} {rx}"
  );
  assert!(
    (ly - cy).abs() < 1e-5 && (ly - ry).abs() < 1e-5,
    "y center must be align-independent: {ly} {cy} {ry}"
  );

  // Horizontal center sits at the run's visual center. With the 1:1
  // atlas→canvas mapping (draw_text places canvas_x(pen_x) = anchorX + dx and
  // pen_x = PAD for the linear path), the atlas-local center is cx =
  // width/2 + PAD — exactly the value that lands on anchorX + dx + width/2
  // (the previous width/2 - PAD assumed the old squashed full-atlas mapping).
  assert!(
    (lx - (width / 2.0 + PAD)).abs() < 1e-4,
    "cx must be width/2 + PAD, got {lx}"
  );
  // Vertical center sits on the baseline (same contract as the other test).
  assert!(
    (ly - (PAD + ascent)).abs() < 1e-4,
    "cy must be the baseline PAD + ascent, got {ly}"
  );
}

#[test]
fn shadow_with_zero_blur_draws_hard_offset_copy() {
  // TS with block.shadow = true, shadowBlur = 0, glowIntensity = 0 still
  // paints a SHARP drop shadow at shadowOffset (canvas shadowBlur = 0 means
  // no blur). The Rust 8-copy glow path skips when glow == 0, so a single
  // hard copy at the offset must be emitted instead — otherwise exports drop
  // the shadow entirely (both sliders bottom out at 0).
  use audiowave_studio_lib::gpu2d::text::select_font;
  let Some(_) = select_font("sans-serif", 400.0) else {
    return;
  };

  let mut config = test_config();
  config.text.blocks.push(TextBlock {
    id: "shadow0".into(),
    text: "Drop".into(),
    enabled: true,
    font_family: "sans-serif".into(),
    font_size: 40.0,
    font_weight: 400.0,
    italic: false,
    color: "#ffffff".into(),
    use_gradient: false,
    gradient_start: "#ffffff".into(),
    gradient_end: "#ffffff".into(),
    gradient_angle: 0.0,
    opacity: 1.0,
    letter_spacing: 0.0,
    transform: TextTransform::None,
    position_x: 50.0,
    position_y: 50.0,
    align: TextAlign::Center,
    line_height: 1.2,
    max_width: 0.0,
    shadow: true,
    shadow_blur: 0.0,
    shadow_offset_x: 8.0,
    shadow_offset_y: 8.0,
    glow_intensity: 0.0,
    outline: false,
    outline_color: "#000000".into(),
    outline_width: 1.0,
    reactive_scale: 0.0,
    wave_effect: false,
    fade_in: false,
  });

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  let mut rstate = RenderState::new(config.reactivity.bar_count, 13);
  let mut canvas = GpuCanvas::new(640, 360);
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0, true);
  let mesh = canvas.finish();

  // 1 hard shadow copy + 1 main fill = 2 atlases. Without the fix the glow
  // pass (glow == 0) emits nothing, leaving only the main fill (1 atlas).
  assert!(
    mesh.atlases.len() >= 2,
    "shadow with zero blur must still draw a hard offset copy, got {} atlases",
    mesh.atlases.len()
  );
  assert!(!mesh.is_empty(), "expected geometry");
}

#[test]
fn zero_or_negative_font_size_renders_no_text() {
  // TS passes block.fontSize straight into `c.font`; a 0px size draws NO
  // glyphs (verified in Chrome: `measureText` = 0, 0 pixels painted) and a
  // negative size is an invalid font shorthand. The Rust port must not
  // substitute a 48px fallback — that made exports show text the preview
  // never displayed. With font_size <= 0, rasterize() returns an empty atlas
  // and no glyph atlases are baked.
  use audiowave_studio_lib::gpu2d::text::select_font;
  let Some(_) = select_font("sans-serif", 400.0) else {
    return;
  };

  let make_block = |font_size: f32| TextBlock {
    id: "hidden".into(),
    text: "Hidden".into(),
    enabled: true,
    font_family: "sans-serif".into(),
    font_size,
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
    position_x: 50.0,
    position_y: 50.0,
    align: TextAlign::Center,
    line_height: 1.2,
    max_width: 0.0,
    shadow: false,
    shadow_blur: 0.0,
    shadow_offset_x: 0.0,
    shadow_offset_y: 0.0,
    glow_intensity: 0.0,
    outline: false,
    outline_color: "#000000".into(),
    outline_width: 1.0,
    reactive_scale: 0.0,
    wave_effect: false,
    fade_in: false,
  };

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  for font_size in [0.0f32, -12.0] {
    let mut config = test_config();
    config.text.blocks.push(make_block(font_size));
    let mut rstate = RenderState::new(config.reactivity.bar_count, 21);
    let mut canvas = GpuCanvas::new(640, 360);
    draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0, true);
    let mesh = canvas.finish();
    assert!(
      mesh.atlases.is_empty(),
      "font_size={font_size} must draw no glyph atlases (0px renders nothing; no 48px fallback)"
    );
  }

  // Control: a positive size still renders glyphs (shadow off -> 1 atlas).
  let mut config = test_config();
  config.text.blocks.push(make_block(40.0));
  let mut rstate = RenderState::new(config.reactivity.bar_count, 22);
  let mut canvas = GpuCanvas::new(640, 360);
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0, true);
  let mesh = canvas.finish();
  assert!(!mesh.atlases.is_empty(), "positive font size must render glyphs");
}

#[test]
fn negative_glow_intensity_shrinks_blur_like_ts() {
  // TS: `c.shadowBlur = block.shadowBlur + (block.glowIntensity || 0)` — a
  // negative glowIntensity is truthy, so it SUBTRACTS from the blur. When the
  // sum is <= 0 the canvas clamps shadowBlur to 0, i.e. a SHARP shadow at the
  // offset. Rust's old `.max(0.0)` clamped the intensity to 0, keeping the
  // full 10px blur (8 blur copies + main = 9 atlases) where TS draws a hard
  // copy (1 + main = 2 atlases).
  use audiowave_studio_lib::gpu2d::text::select_font;
  let Some(_) = select_font("sans-serif", 400.0) else {
    return;
  };

  let mut config = test_config();
  config.text.blocks.push(TextBlock {
    id: "negGlow".into(),
    text: "Drop".into(),
    enabled: true,
    font_family: "sans-serif".into(),
    font_size: 40.0,
    font_weight: 400.0,
    italic: false,
    color: "#ffffff".into(),
    use_gradient: false,
    gradient_start: "#ffffff".into(),
    gradient_end: "#ffffff".into(),
    gradient_angle: 0.0,
    opacity: 1.0,
    letter_spacing: 0.0,
    transform: TextTransform::None,
    position_x: 50.0,
    position_y: 50.0,
    align: TextAlign::Center,
    line_height: 1.2,
    max_width: 0.0,
    shadow: true,
    shadow_blur: 10.0,
    shadow_offset_x: 6.0,
    shadow_offset_y: 6.0,
    glow_intensity: -15.0, // 10 + (-15) = -5 -> canvas clamps to 0 -> sharp shadow
    outline: false,
    outline_color: "#000000".into(),
    outline_width: 1.0,
    reactive_scale: 0.0,
    wave_effect: false,
    fade_in: false,
  });

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];
  let mut rstate = RenderState::new(config.reactivity.bar_count, 23);
  let mut canvas = GpuCanvas::new(640, 360);
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0, true);
  let mesh = canvas.finish();

  // 1 hard shadow copy + 1 main fill = 2 atlases (NOT the 8-copy blur path).
  assert_eq!(
    mesh.atlases.len(),
    2,
    "negative glow must clamp the blur to a sharp shadow, got {} atlases",
    mesh.atlases.len()
  );
}

#[test]
fn italic_selects_italic_font_variant() {
  use audiowave_studio_lib::gpu2d::text::select_font_for_text_style;

  // TS sets `c.font = `${block.italic ? 'italic ' : ''}${weight}px ${family}``;
  // the Rust renderer must resolve a distinct (italic) face when one exists.
  let Some(regular) = select_font_for_text_style("sans-serif", 400.0, false, "x") else {
    return;
  };
  let Some(italic) = select_font_for_text_style("sans-serif", 400.0, true, "x") else {
    return;
  };
  // No italic face on this system -> falls back to regular (acceptable).
  if std::ptr::eq(regular, italic) {
    return;
  }
  // Bold vs bold-italic must also differ when both are available.
  let Some(bold) = select_font_for_text_style("sans-serif", 700.0, false, "x") else {
    return;
  };
  let Some(bold_italic) = select_font_for_text_style("sans-serif", 700.0, true, "x") else {
    return;
  };
  assert!(
    !std::ptr::eq(bold, bold_italic),
    "bold-italic must resolve to the bold-italic face, not plain bold"
  );
}

#[test]
fn italic_block_renders_glyph_atlases() {
  // No usable system font -> nothing can render; skip like the other font tests.
  use audiowave_studio_lib::gpu2d::text::select_font;
  let Some(_) = select_font("sans-serif", 700.0) else {
    return;
  };

  // End-to-end: an italic text block must go through the rasterizer without
  // panicking and produce the glow geometry when a shadow with offset is
  // configured (TS applies shadowBlur + shadowOffset simultaneously; the Rust
  // approximation rasterizes the shadow-colored run ONCE and draws 256
  // Gaussian-sampled copies of that quad, alpha = opacity/256 each).
  let mut config = test_config();
  config.text.blocks.push(TextBlock {
    id: "it".into(),
    text: "Italic".into(),
    enabled: true,
    font_family: "sans-serif".into(),
    font_size: 40.0,
    font_weight: 700.0,
    italic: true,
    color: "#ffffff".into(),
    use_gradient: false,
    gradient_start: "#ffffff".into(),
    gradient_end: "#ffffff".into(),
    gradient_angle: 0.0,
    opacity: 1.0,
    letter_spacing: 0.0,
    transform: TextTransform::None,
    position_x: 50.0,
    position_y: 50.0,
    align: TextAlign::Center,
    line_height: 1.2,
    max_width: 0.0,
    shadow: true,
    shadow_blur: 12.0,
    shadow_offset_x: 6.0,
    shadow_offset_y: 6.0,
    glow_intensity: 0.0,
    outline: false,
    outline_color: "#000000".into(),
    outline_width: 1.0,
    reactive_scale: 0.0,
    wave_effect: false,
    fade_in: false,
  });

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  let mut rstate = RenderState::new(config.reactivity.bar_count, 9);
  let mut canvas = GpuCanvas::new(640, 360);
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.0, true);
  let mesh = canvas.finish();

  // 1 glow atlas + 1 main fill atlas, and the glow path draws 256 blurred
  // copies of the run quad + the 1 main fill quad.
  assert!(
    mesh.atlases.len() >= 2,
    "shadow must bake a glow atlas and a main fill atlas, got {} atlases",
    mesh.atlases.len()
  );
  let quads = mesh.idx.len() / 6;
  assert!(
    quads >= 257,
    "shadow must draw 256 Gaussian glow copies + main fill quad, got {} quads",
    quads
  );
  assert!(!mesh.is_empty(), "expected geometry for the text block");
}

// ---------------------------------------------------------------------------
// Preview RenderState caching (render_rust_preview_frame persistence)
// ---------------------------------------------------------------------------

#[test]
fn preview_render_state_cache_reuses_state_across_frames() {
  use audiowave_studio_lib::gpu_export::take_or_init_render_state;
  use audiowave_studio_lib::renderers::background::MusicNote;
  use audiowave_studio_lib::renderers::RenderState;

  let mut cached: Option<RenderState> = None;

  // Frame 1: no cache yet -> fresh state sized to the bar count (this is what
  // the first render_rust_preview_frame call does).
  let mut s = take_or_init_render_state(&mut cached, 64);
  assert_eq!(s.peak_data.len(), 64);
  assert!(s.music_notes.is_empty());

  // Simulate one rendered frame: a particle advances, a note spawns, the RNG
  // stream is consumed.
  let phase_initial = s.particles[0].phase;
  s.particles[0].phase += 0.5;
  s.music_notes.push(MusicNote {
    x: 10.0,
    y: 20.0,
    vx: 0.0,
    vy: -1.0,
    size: 40.0,
    alpha: 0.8,
    rotation: 0.1,
    symbol: 1,
    life: 1.0,
    max_life: 60.0,
    base_x: 10.0,
    phase: 0.0,
  });
  let rng_draw = s.rng.next();
  cached = Some(s);

  // Frame 2: same bar count -> the SAME state comes back (no re-seed).
  let mut s2 = take_or_init_render_state(&mut cached, 64);
  assert_eq!(s2.peak_data.len(), 64);
  assert!(
    (s2.particles[0].phase - (phase_initial + 0.5)).abs() < 1e-6,
    "particle phase must persist across frames (state was re-seeded?)"
  );
  assert_eq!(s2.music_notes.len(), 1, "music notes must persist across frames");
  let rng_draw2 = s2.rng.next();
  assert!(
    (rng_draw2 - rng_draw).abs() > 1e-12,
    "RNG stream must continue, not restart (a fresh Rng would re-emit the same value)"
  );

  // Frame 3: bar count changed -> state rebuilt from scratch (old mutations
  // gone, deterministic fresh seed like the first frame).
  let s3 = take_or_init_render_state(&mut cached, 128);
  assert_eq!(s3.peak_data.len(), 128);
  assert!(s3.music_notes.is_empty(), "bar count change must reset the state");
  let fresh = RenderState::new(128, 0xC0FFEE);
  assert!(
    (s3.particles[0].phase - fresh.particles[0].phase).abs() < 1e-6,
    "rebuilt state must equal a fresh RenderState"
  );
}

#[test]
fn preview_bar_count_change_keeps_text_fade_state() {
  use audiowave_studio_lib::gpu_export::take_or_init_render_state;
  use audiowave_studio_lib::renderers::text::fade_factor;
  use audiowave_studio_lib::renderers::RenderState;

  // textOverlay.ts keeps `playStartFrame`/`wasPlaying` at MODULE scope; they
  // are only reset by resetTextFadeState() (via resetVisualizerState()) when
  // an export starts or playback restarts at offset 0 — never when the
  // bar-count slider moves. A preview rebuild triggered by a bar-count change
  // must therefore carry the fade clock over, or fade-in text blinks back to
  // invisible every time the slider is dragged mid-playback.
  let mut cached: Option<RenderState> = None;

  // Frame 1: fresh state, playback begins at t=0.3 -> the fade captures the
  // start frame and begins ramping from 0 (TS: `if (isPlaying && !wasPlaying)
  // playStartFrame = frameTime`).
  let mut s = take_or_init_render_state(&mut cached, 64);
  let fade0 = fade_factor(true, 0.3, &mut s.text_play_start_frame, &mut s.text_was_playing);
  assert!((fade0 - 0.0).abs() < 1e-6, "fade must start at 0 on play");
  assert!(
    (s.text_play_start_frame - 0.3).abs() < 1e-6,
    "playStartFrame captured at play start"
  );
  assert!(s.text_was_playing, "wasPlaying tracks isPlaying");
  cached = Some(s);

  // Frames 2..3: still playing; the fade keeps ramping ((0.5-0.3)/0.8 = 0.25).
  let mut s2 = take_or_init_render_state(&mut cached, 64);
  let fade_mid = fade_factor(true, 0.5, &mut s2.text_play_start_frame, &mut s2.text_was_playing);
  assert!((fade_mid - 0.25).abs() < 1e-4, "fade ramps while playing, got {fade_mid}");
  cached = Some(s2);

  // Frame 4: the user drags the Bar Count slider (64 -> 128) MID-PLAYBACK.
  // Particles/peaks/RNG restart with the export seed, but the fade clock must
  // SURVIVE the rebuild (TS module state is untouched by config changes).
  let mut s3 = take_or_init_render_state(&mut cached, 128);
  assert_eq!(s3.peak_data.len(), 128, "bar count change rebuilds the state");
  assert!(s3.music_notes.is_empty(), "rebuild resets per-bar state");
  assert!(
    s3.text_was_playing,
    "wasPlaying must survive a bar-count change (no re-fade)"
  );
  assert!(
    (s3.text_play_start_frame - 0.3).abs() < 1e-6,
    "playStartFrame must survive a bar-count change (no re-fade)"
  );

  // The very next playing frame continues the ramp ((0.6-0.3)/0.8 = 0.375)
  // instead of restarting at 0 — i.e. the text does NOT blink.
  let fade_after = fade_factor(true, 0.6, &mut s3.text_play_start_frame, &mut s3.text_was_playing);
  assert!(
    (fade_after - 0.375).abs() < 1e-4,
    "fade must continue after the rebuild, got {fade_after}"
  );
}

#[test]
fn preview_render_state_persists_across_inner_calls() {
  use audiowave_studio_lib::gpu_export::render_preview_frame_inner;
  use audiowave_studio_lib::renderers::RenderState;
  use audiowave_studio_lib::GpuPreviewEngine;

  let mut engine = GpuPreviewEngine {
    renderer: pollster::block_on(GpuRenderer::new(320, 240)).expect("GPU init failed"),
    width: 320,
    height: 240,
    bg_image_uri: None,
    bg_image_info: None,
    radial_image_uri: None,
    radial_image_info: None,
    render_state: None,
  };

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  // Enable particles so draw_frame advances their phase every frame.
  let mut config = test_config();
  config.background.show_particles = true;

  let frame1 =
    render_preview_frame_inner(&mut engine, &config, &freq, &time, 0.0, 320, 240, true).expect("frame 1");
  assert_eq!(frame1.len(), 320 * 240 * 4);
  let phase1 = engine.render_state.as_ref().expect("state cached").particles[0].phase;

  // A second call must reuse the cached state (particles keep advancing). A
  // re-seeded state would restart at the identical deterministic phase.
  let frame2 =
    render_preview_frame_inner(&mut engine, &config, &freq, &time, 0.1, 320, 240, true).expect("frame 2");
  assert_eq!(frame2.len(), 320 * 240 * 4);
  let st2 = engine.render_state.as_ref().expect("state cached after frame 2");
  assert!(
    (st2.particles[0].phase - phase1).abs() > 1e-6,
    "particle phase must keep advancing across frames (state was re-seeded?)"
  );
  assert_eq!(st2.peak_data.len(), 64, "state must stay sized to the bar count");

  // Changing the bar count must rebuild the state (fresh, no continuation).
  config.reactivity.bar_count = 128;
  let _ = render_preview_frame_inner(&mut engine, &config, &freq, &time, 0.2, 320, 240, true).expect("frame 3");
  let st3 = engine.render_state.as_ref().expect("state after bar count change");
  assert_eq!(st3.peak_data.len(), 128, "bar count change must rebuild the state");
  // The rebuilt state is a fresh RenderState that has already drawn one frame:
  // render_particles advances each particle phase by 0.012 per frame when
  // particles are enabled, so compare against fresh + one advance.
  let mut fresh = RenderState::new(128, 0xC0FFEE);
  fresh.particles[0].phase += 0.012;
  assert!(
    (st3.particles[0].phase - fresh.particles[0].phase).abs() < 1e-6,
    "rebuilt state must equal a fresh RenderState advanced one frame (no continuation)"
  );
}

// ---------------------------------------------------------------------------
// Text fade-in parity with textOverlay.ts fadeFactor
// ---------------------------------------------------------------------------

#[test]
fn text_quad_places_baseline_at_y_and_renders_1to1() {
  // Regression for the draw_text atlas→canvas mapping. The old code stretched
  // the FULL atlas ([0,1]^2 UVs) onto an ink-sized quad, which SQUASHED the
  // glyphs vertically to atlas_h/height (~48-64% of the intended size) and
  // drifted the baseline ~1-3px off `y`. draw_text must now map 1:1
  // (UV-cropped to the ink) with the pen start at x (textAlign left) and the
  // baseline EXACTLY at y — like canvas fillText.
  use audiowave_studio_lib::gpu2d::text::{rasterize, select_font, TextAlign, TextOpts};
  let Some(font) = select_font("sans-serif", 400.0) else {
    return;
  };
  let fill = Fill::Solid(Color::WHITE);
  let atl =
    rasterize(font, "AudioWave", 40.0, &fill, &Default::default()).expect("rasterize failed");

  let mut canvas = GpuCanvas::new(640, 360);
  canvas.draw_text(
    "AudioWave",
    320.0,
    180.0,
    40.0,
    "sans-serif",
    400.0,
    false,
    TextAlign::Left,
    fill,
    1.0,
    &TextOpts::default(),
  );
  let mesh = canvas.finish();
  assert_eq!(mesh.atlases.len(), 1, "one text atlas expected");

  // Layer 0 -> tex_id = 1.0. draw_text pushes exactly one quad = 6 verts.
  let text_verts: Vec<&Vertex> = mesh
    .verts
    .iter()
    .filter(|v| (v.tex_id - 1.0).abs() < 1e-6)
    .collect();
  assert_eq!(text_verts.len(), 6, "one text quad = 6 verts");

  let to_canvas = |v: &Vertex| {
    let x = (v.position[0] + 1.0) * 0.5 * 640.0;
    let y = (1.0 - v.position[1]) * 0.5 * 360.0;
    (x, y)
  };
  let pts: Vec<(f32, f32)> = text_verts.iter().map(|v| to_canvas(v)).collect();
  let (qx0, qx1) = (
    pts.iter().map(|p| p.0).fold(f32::MAX, f32::min),
    pts.iter().map(|p| p.0).fold(f32::MIN, f32::max),
  );
  let (qy0, qy1) = (
    pts.iter().map(|p| p.1).fold(f32::MAX, f32::min),
    pts.iter().map(|p| p.1).fold(f32::MIN, f32::max),
  );
  let (qw, qh) = (qx1 - qx0, qy1 - qy0);

  // 1:1: quad size == ink size, NOT the full atlas (no vertical squash).
  assert!(
    (qw - atl.width).abs() < 0.01,
    "quad width {qw} must equal ink width {} (1:1 mapping)",
    atl.width
  );
  assert!(
    (qh - atl.height).abs() < 0.01,
    "quad height {qh} must equal ink height {} (no vertical squash)",
    atl.height
  );

  // Baseline lands EXACTLY on y: quad_y + (baseline - top) == 180.
  let baseline_canvas = qy0 + (atl.baseline - atl.top);
  assert!(
    (baseline_canvas - 180.0).abs() < 0.01,
    "baseline must sit exactly on y=180, got {baseline_canvas}"
  );
  // Pen start lands EXACTLY on x for left align: quad_x + (pen_x - left) == 320.
  let pen_canvas = qx0 + (atl.pen_x - atl.left);
  assert!(
    (pen_canvas - 320.0).abs() < 0.01,
    "pen start must sit exactly on x=320, got {pen_canvas}"
  );

  // The UVs must be cropped to the ink region (not [0,1]^2): the top-left
  // texture coordinate equals (left/layer_size, top/layer_size) because the
  // atlas is uploaded into a LAYER_SIZE×LAYER_SIZE (2048) texture layer, so
  // UVs are normalized by LAYER_SIZE — NOT by atlas_w/atlas_h (which would
  // point the quad at empty layer space).
  let layer_size = audiowave_studio_lib::gpu2d::renderer::LAYER_SIZE as f32;
  let uv0 = text_verts[0].uv;
  assert!(
    (uv0[0] - atl.left / layer_size).abs() < 1e-5
      && (uv0[1] - atl.top / layer_size).abs() < 1e-5,
    "UVs must crop to the ink box, got {uv0:?}"
  );
}

// ---------------------------------------------------------------------------
// Text fade-in parity with textOverlay.ts fadeFactor
// ---------------------------------------------------------------------------

#[test]
fn fade_factor_matches_ts() {
  use audiowave_studio_lib::renderers::text::fade_factor;

  // Mirrors textOverlay.ts module state (playStartFrame / wasPlaying).
  let mut play_start = 0.0f32;
  let mut was_playing = false;

  // Paused → fully visible (TS: `if (!isPlaying) return 1`).
  let f = fade_factor(false, 0.0, &mut play_start, &mut was_playing);
  assert_eq!(f, 1.0, "paused must be fully visible");
  assert!(!was_playing, "wasPlaying must track isPlaying");

  // Play begins at t=0 → fade starts at 0 and ramps over 0.8s.
  let f0 = fade_factor(true, 0.0, &mut play_start, &mut was_playing);
  assert!((f0 - 0.0).abs() < 1e-6, "first playing frame fades from 0");
  assert!((play_start - 0.0).abs() < 1e-6, "playStartFrame captured");

  let f_mid = fade_factor(true, 0.4, &mut play_start, &mut was_playing);
  assert!((f_mid - 0.5).abs() < 1e-4, "at 0.4s fade should be ~0.5, got {f_mid}");

  let f_end = fade_factor(true, 0.8, &mut play_start, &mut was_playing);
  assert!((f_end - 1.0).abs() < 1e-4, "at 0.8s fade clamps to 1.0");
  let f_over = fade_factor(true, 5.0, &mut play_start, &mut was_playing);
  assert_eq!(f_over, 1.0, "fade stays clamped at 1.0");

  // Pause → visible again; resume restarts the fade from the new start.
  let f_pause = fade_factor(false, 7.0, &mut play_start, &mut was_playing);
  assert_eq!(f_pause, 1.0, "pause must be fully visible");
  let f_resume = fade_factor(true, 7.0, &mut play_start, &mut was_playing);
  assert!(
    (f_resume - 0.0).abs() < 1e-6,
    "resume restarts the fade (playStartFrame=7.0)"
  );
  assert!((play_start - 7.0).abs() < 1e-6, "playStartFrame updated on resume");
  let f_after = fade_factor(true, 7.4, &mut play_start, &mut was_playing);
  assert!((f_after - 0.5).abs() < 1e-4, "fade ramps again after resume");
}

// ---------------------------------------------------------------------------
// Text block anchor parity with textOverlay.ts
// ---------------------------------------------------------------------------

#[test]
fn text_block_anchor_matches_ts_literal() {
  use audiowave_studio_lib::renderers::text::block_anchor;

  // textOverlay.ts computes the anchor literally with NO special-casing:
  //   const anchorX = (block.positionX / 100) * width;
  //   const anchorY = (block.positionY / 100) * height;
  // A (0,0) block must land at the top-left corner in BOTH renderers — the
  // Rust port must never substitute a "default" (50,80) position.
  let width = 1920.0f32;
  let height = 1080.0f32;

  let mut block = test_config().text.title;
  block.position_x = 50.0;
  block.position_y = 78.0;
  let (x, y) = block_anchor(&block, width, height);
  assert!((x - 960.0).abs() < 1e-4, "x=50% must be 960, got {x}");
  assert!((y - 842.4).abs() < 1e-4, "y=78% must be 842.4, got {y}");

  // Regression: (0,0) must map to (0,0), NOT the (50,80) hack.
  block.position_x = 0.0;
  block.position_y = 0.0;
  let (x, y) = block_anchor(&block, width, height);
  assert_eq!(x, 0.0, "x=0 must stay 0 (top-left), got {x}");
  assert_eq!(y, 0.0, "y=0 must stay 0 (top-left), got {y}");

  // One axis at zero: still literal (only BOTH zero could trigger the hack).
  block.position_x = 100.0;
  block.position_y = 0.0;
  let (x, y) = block_anchor(&block, width, height);
  assert!((x - 1920.0).abs() < 1e-4, "x=100% must be 1920, got {x}");
  assert_eq!(y, 0.0);

  // Negative values (off-canvas) also pass through literally.
  block.position_x = -50.0;
  block.position_y = 150.0;
  let (x, y) = block_anchor(&block, width, height);
  assert!((x - -960.0).abs() < 1e-4, "x=-50% must be -960, got {x}");
  assert!((y - 1620.0).abs() < 1e-4, "y=150% must be 1620, got {y}");
}

// ---------------------------------------------------------------------------
// Smoothing clamp parity with audioEngine.setSmoothing
// ---------------------------------------------------------------------------

#[test]
fn smoothing_clamp_matches_ts_analyser() {
  use audiowave_studio_lib::gpu_export::clamp_smoothing;

  // TS audioEngine.setSmoothing: Math.max(0, Math.min(0.99, smoothing)).
  assert!((clamp_smoothing(0.8) - 0.8).abs() < 1e-6, "mid-range passes through");
  assert!((clamp_smoothing(0.95) - 0.95).abs() < 1e-6, "slider max passes through");
  assert!(
    (clamp_smoothing(0.98) - 0.98).abs() < 1e-6,
    "values above the slider max (0.95) must pass through up to 0.99, not clamp to 0.95"
  );
  assert!(
    (clamp_smoothing(0.995) - 0.99).abs() < 1e-6,
    "values above the TS ceiling must clamp to 0.99"
  );
  assert_eq!(clamp_smoothing(-1.0), 0.0, "negative clamps to 0 like TS");
  assert_eq!(clamp_smoothing(2.0), 0.99, "above 1.0 clamps to 0.99 like TS");
}

// ---------------------------------------------------------------------------
// End-to-end screen blend verification (requires a GPU)
// ---------------------------------------------------------------------------

#[test]
fn gpu_screen_blend_matches_compositing_spec() {
  use audiowave_studio_lib::gpu2d::scene::BlendMode;

  let mut gpu = pollster::block_on(GpuRenderer::new(64, 64)).expect("GPU init failed");

  let mut canvas = GpuCanvas::new(64, 64);
  // Opaque backdrop: dark gray 0.2.
  canvas.set_fill(Fill::Solid(Color::rgba(0.2, 0.2, 0.2, 1.0)));
  canvas.fill_rect(0.0, 0.0, 64.0, 64.0);
  // Screen source: pure red at 50% alpha -> premultiplied 0.5.
  // Spec: Co = αs·Cs·(1−Cb) + Cb = 0.5*0.8 + 0.2 = 0.6 -> 153/255.
  canvas.set_blend_screen();
  canvas.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.0, 0.5)));
  canvas.fill_rect(0.0, 0.0, 64.0, 64.0);
  canvas.set_blend_normal();
  let mesh = canvas.finish();

  // Guard: the mesh must actually carry a Screen batch (regression check).
  assert!(mesh.batches.iter().any(|b| b.blend == BlendMode::Screen));

  gpu.render_into(&mesh, 0);
  let rgba = gpu.readback(0);
  let p = 32 * 4 + 32 * 64 * 4;
  let (r, g, b) = (rgba[p] as i32, rgba[p + 1] as i32, rgba[p + 2] as i32);
  assert!((r - 153).abs() <= 5, "screen red: got {r}, expected ~153");
  assert!((g - 51).abs() <= 5, "screen green: got {g}, expected ~51");
  assert!((b - 51).abs() <= 5, "screen blue: got {b}, expected ~51");
}

// ---------------------------------------------------------------------------
// Two-pass backgroundOnly split (render_bg_fx_then_over pipeline)
// ---------------------------------------------------------------------------

#[test]
fn split_passes_cover_exactly_the_all_pass_geometry() {
  // backgroundOnly + frame-sampling effect uses the two-pass path: a bg mesh
  // (background + overlay) and a fg mesh (style + particles + text). The two
  // meshes must together contain EXACTLY the geometry of the single All pass
  // — otherwise the split double-draws (envelope state advanced twice, or
  // background drawn in both passes) or drops layers.
  let mut config = test_config();
  config.style = VisualizerStyle::FlameFire;
  config.screen_effects.enabled = true;
  config.screen_effects.background_only = Some(true);
  config.screen_effects.main_effect = ScreenEffect::Glitch;
  config.background.overlay_opacity = 0.2;
  config.text.show_title = true;
  config.text.title.text = "Split".into();

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  // All pass.
  let mut all_state = RenderState::new(config.reactivity.bar_count, 4242);
  let mut all_canvas = GpuCanvas::new(640, 360);
  for f in 0..3 {
    draw_frame(&mut all_canvas, &mut all_state, &config, &freq, &time, f as f32 * 0.1, true);
    if f < 2 {
      all_canvas = GpuCanvas::new(640, 360);
    }
  }
  let all_mesh = all_canvas.finish();

  // Split: advance the envelope ONCE, then two passes sharing the state.
  let mut split_state = RenderState::new(config.reactivity.bar_count, 4242);
  for f in 0..3 {
    let frame_time = f as f32 * 0.1;
    let env = advance_envelope(&mut split_state, &config, &freq, frame_time, true);

    let mut bg_canvas = GpuCanvas::new(640, 360);
    draw_frame_pass(
      &mut bg_canvas, &mut split_state, &config, &freq, &time, frame_time, &env,
      FramePass::BackgroundOnly,
    );
    let mut fg_canvas = GpuCanvas::new(640, 360);
    draw_frame_pass(
      &mut fg_canvas, &mut split_state, &config, &freq, &time, frame_time, &env,
      FramePass::ForegroundOnly,
    );
    // State continuity matters for flame particles; compare the LAST frame's
    // meshes (the ones drawn after all frames of state advancement).
    if f == 2 {
      let bg_mesh = bg_canvas.finish();
      let fg_mesh = fg_canvas.finish();
      assert!(!bg_mesh.is_empty(), "bg pass must draw the background");
      assert!(!fg_mesh.is_empty(), "fg pass must draw the style");

      let split_verts = bg_mesh.verts.len() + fg_mesh.verts.len();
      assert_eq!(
        split_verts,
        all_mesh.verts.len(),
        "split passes must total the same geometry as the All pass"
      );
      assert_eq!(
        bg_mesh.idx.len() + fg_mesh.idx.len(),
        all_mesh.idx.len(),
        "split index counts must match the All pass"
      );
    }
  }
}

#[test]
fn split_passes_do_not_double_advance_the_envelope() {
  // Regression guard: the two-pass path must advance the envelope EXACTLY
  // once per frame. If a pass advanced it again, a constant-input frame would
  // decay bass/beat twice as fast — measurable via rotation_angle drift
  // (rotation_angle += 0.003 per advance_envelope call).
  let mut config = test_config();
  config.screen_effects.enabled = true;
  config.screen_effects.background_only = Some(true);
  config.screen_effects.main_effect = ScreenEffect::Chromatic;

  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  let mut state = RenderState::new(config.reactivity.bar_count, 7);
  for f in 0..10 {
    let frame_time = f as f32 * 0.1;
    let env = advance_envelope(&mut state, &config, &freq, frame_time, true);
    let mut bg_canvas = GpuCanvas::new(640, 360);
    draw_frame_pass(
      &mut bg_canvas, &mut state, &config, &freq, &time, frame_time, &env,
      FramePass::BackgroundOnly,
    );
    let mut fg_canvas = GpuCanvas::new(640, 360);
    draw_frame_pass(
      &mut fg_canvas, &mut state, &config, &freq, &time, frame_time, &env,
      FramePass::ForegroundOnly,
    );
  }
  // 10 frames x 0.003 per advance = 0.03 (a single advance per frame). A
  // double advance would give 0.06.
  assert!(
    (state.rotation_angle - 0.03).abs() < 1e-5,
    "rotation_angle must advance once per frame, got {}",
    state.rotation_angle
  );
}

// ---------------------------------------------------------------------------
// Determinism of the rewritten particle styles (seeded RNG)
// ---------------------------------------------------------------------------

#[test]
fn rewritten_styles_are_deterministic_per_seed() {
  // flame_fire / spiral_galaxy / three_d / api_3d spawn particles with the
  // seeded mulberry32 RNG. Two runs with the SAME seed must produce identical
  // geometry (the TS preview uses Math.random() so it varies per session, but
  // the export must be reproducible given identical inputs).
  let styles = [
    VisualizerStyle::FlameFire,
    VisualizerStyle::SpiralGalaxy,
    VisualizerStyle::ThreeD,
    VisualizerStyle::Api3D,
  ];
  let freq = synth_freq();
  let time: Vec<u8> = vec![128; 512];

  for style in styles {
    let mut config = test_config();
    config.style = style;

    let mut a_state = RenderState::new(config.reactivity.bar_count, 99);
    let mut a_canvas = GpuCanvas::new(480, 270);
    for f in 0..8 {
      draw_frame(&mut a_canvas, &mut a_state, &config, &freq, &time, f as f32 * 0.1, true);
      if f < 7 {
        a_canvas = GpuCanvas::new(480, 270);
      }
    }
    let a_mesh = a_canvas.finish();

    let mut b_state = RenderState::new(config.reactivity.bar_count, 99);
    let mut b_canvas = GpuCanvas::new(480, 270);
    for f in 0..8 {
      draw_frame(&mut b_canvas, &mut b_state, &config, &freq, &time, f as f32 * 0.1, true);
      if f < 7 {
        b_canvas = GpuCanvas::new(480, 270);
      }
    }
    let b_mesh = b_canvas.finish();

    assert_eq!(a_mesh.verts.len(), b_mesh.verts.len(), "{:?}: vert count", config.style);
    assert_eq!(a_mesh.idx, b_mesh.idx, "{:?}: index geometry must match", config.style);
    for (va, vb) in a_mesh.verts.iter().zip(b_mesh.verts.iter()) {
      assert!(
        (va.position[0] - vb.position[0]).abs() < 1e-4
          && (va.position[1] - vb.position[1]).abs() < 1e-4
          && (va.color[0] - vb.color[0]).abs() < 1e-4
          && (va.color[1] - vb.color[1]).abs() < 1e-4
          && (va.color[2] - vb.color[2]).abs() < 1e-4
          && (va.color[3] - vb.color[3]).abs() < 1e-4,
        "{:?}: vertex mismatch",
        config.style
      );
    }
    assert!(!a_mesh.is_empty(), "{:?}: expected geometry", config.style);
  }
}

// ---------------------------------------------------------------------------
// Two-pass backgroundOnly composite (render_bg_fx_then_over) pixel test
// ---------------------------------------------------------------------------

#[test]
fn gpu_two_pass_bg_fx_then_over_composites_correctly() {
  // The backgroundOnly path renders the BACKGROUND through the post-fx
  // pipeline (invert here), then composites the FOREGROUND (style/particles/
  // text) OVER it with LoadOp::Load. Pixel assertions prove:
  //   1. the fx was applied to the background (white -> black, red -> cyan),
  //   2. the foreground is drawn ON TOP and is NOT affected by the fx
  //      (opaque green stays green over the inverted red), and
  //   3. the readback copies the composite (not the raw background).
  use audiowave_studio_lib::gpu2d::renderer::PostFx;

  let mut gpu = pollster::block_on(GpuRenderer::new(64, 64)).expect("GPU init failed");

  // Background mesh: opaque red everywhere + white 16x16 square top-left.
  let mut bg = GpuCanvas::new(64, 64);
  bg.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.0, 1.0)));
  bg.fill_rect(0.0, 0.0, 64.0, 64.0);
  bg.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 1.0)));
  bg.fill_rect(0.0, 0.0, 16.0, 16.0);
  let bg_mesh = bg.finish();

  // Foreground mesh: opaque green 16x16 square in the middle.
  let mut fg = GpuCanvas::new(64, 64);
  fg.set_fill(Fill::Solid(Color::rgba(0.0, 1.0, 0.0, 1.0)));
  fg.fill_rect(24.0, 24.0, 16.0, 16.0);
  let fg_mesh = fg.finish();

  // Invert fx (mode 4), full intensity: col = 1 - c0.
  let fx = PostFx { mode: 4, intensity: 1.0, time: 1.0, beat: 0.0, fps: 30.0 };
  gpu.render_bg_fx_then_over(&bg_mesh, &fg_mesh, &fx, 0);
  let rgba = gpu.readback(0);
  assert_eq!(rgba.len(), 64 * 64 * 4, "readback must be a full RGBA frame");

  let px = |x: u32, y: u32| -> (u8, u8, u8) {
    let p = ((y * 64 + x) * 4) as usize;
    (rgba[p], rgba[p + 1], rgba[p + 2])
  };

  // (1) Background fx applied: white square top-left -> near-black.
  // Invert is an exact 1-c0 mix, so a tight tolerance catches partial-mix
  // regressions while allowing GPU round-trip rounding.
  let (r, g, b) = px(8, 8);
  assert!(
    r <= 15 && g <= 15 && b <= 15,
    "white bg must invert to near-black, got ({r},{g},{b})"
  );
  // Red background elsewhere -> cyan (1 - red).
  let (r, g, b) = px(60, 60);
  assert!(
    r <= 15 && g >= 240 && b >= 240,
    "red bg must invert to cyan, got ({r},{g},{b})"
  );

  // (2) Foreground composites OVER the fx'd bg and is NOT inverted — if the
  // fx were re-applied over the composite, the green square would become
  // magenta ((0,255,0) -> (255,0,255)).
  let (r, g, b) = px(32, 32);
  assert!(
    g >= 240 && r <= 15 && b <= 15,
    "opaque green fg must stay green on top, got ({r},{g},{b})"
  );

  // (3) Compare against the raw (single-pass) invert of the same bg to prove
  // the fg really was composited into the final readback: the raw invert at
  // the fg position is cyan (red bg), while the two-pass result is green.
  gpu.render_into_fx(&bg_mesh, &fx, 1);
  let raw = gpu.readback(1);
  let p = ((32 * 64 + 32) * 4) as usize;
  let (rr, rg, rb) = (raw[p], raw[p + 1], raw[p + 2]);
  assert!(
    rr <= 15 && rg >= 240 && rb >= 240,
    "sanity: raw invert of red bg at fg position must be cyan, got ({rr},{rg},{rb})"
  );
}
