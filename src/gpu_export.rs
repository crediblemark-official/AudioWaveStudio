//! export_gpu — full-Rust wgpu export path.
//! Decodes audio, precomputes FFT per frame, draws with the Rust renderers
//! (gpu2d) and pipes JPEG frames to ffmpeg. Falls back via the JS caller to
//! the canvas exporter when the GPU is unavailable.

use crate::audio_decoder::AudioData;
use crate::config::VisualizerConfig;
use crate::fft_analyzer::FftAnalyzer;
use crate::gpu2d::{GpuCanvas, GpuRenderer, IMAGE_LAYER, RADIAL_CENTER_IMAGE_LAYER, Scene3D};
use crate::renderers::{advance_envelope, draw_frame_pass, BackgroundImage, FramePass, RenderState};
use base64::{Engine as _, engine::general_purpose};
use serde::Serialize;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

fn decode_uri_to_path(s: &str) -> String {
  let clean = if let Some(p) = s.strip_prefix("file://") {
    p
  } else if let Some(p) = s.strip_prefix("http://asset.localhost/") {
    p
  } else if let Some(p) = s.strip_prefix("https://asset.localhost/") {
    p
  } else if let Some(p) = s.strip_prefix("asset://") {
    p
  } else {
    s
  };

  let mut out = String::with_capacity(clean.len());
  let bytes = clean.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == b'%' && i + 2 < bytes.len() {
      if let Ok(val) = u8::from_str_radix(&clean[i + 1..i + 3], 16) {
        out.push(val as char);
        i += 3;
        continue;
      }
    }
    out.push(bytes[i] as char);
    i += 1;
  }
  out
}

/// Decode a custom background image URI (base64 data URI or file path) into
/// raw RGBA. Shared by the GPU export path and the CPU fallback compositor.
pub(crate) fn decode_background_image(uri: Option<&str>) -> Option<(Vec<u8>, u32, u32)> {
  let uri = uri?;
  if uri.trim().is_empty() {
    return None;
  }

  let bytes: Vec<u8> = if let Some((_, b64)) = uri.split_once("base64,") {
    let clean = b64.split(&['?', '#'][..]).next().unwrap_or(b64).trim();
    let sanitized: String = clean.chars().filter(|c| !c.is_whitespace()).collect();
    general_purpose::STANDARD
      .decode(&sanitized)
      .or_else(|_| general_purpose::URL_SAFE.decode(&sanitized))
      .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(&sanitized))
      .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(&sanitized))
      .ok()?
  } else {
    let file_path = decode_uri_to_path(uri);
    if let Ok(b) = std::fs::read(&file_path) {
      b
    } else if let Ok(b) = std::fs::read(uri) {
      b
    } else {
      crate::logline!("[Rust GPU Export] Background image not found: '{}'", uri);
      return None;
    }
  };

  let img = image::load_from_memory(&bytes).ok()?;
  let rgba = img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  if w == 0 || h == 0 {
    return None;
  }
  Some((rgba.into_raw(), w, h))
}

#[derive(Serialize, Clone)]
pub struct GpuExportProgress {
  pub percent: f32,
  pub current_frame: usize,
  pub total_frames: usize,
  pub elapsed_time: f32,
}

use crate::export_ffmpeg::{export_dimensions, spawn_ffmpeg};

fn get_stderr_msg(buf: &Arc<Mutex<String>>) -> String {
  buf.lock().map(|s| s.trim().to_string()).unwrap_or_default()
}

pub fn export_gpu(
  config: VisualizerConfig,
  audio_file_path: String,
  output_path: String,
  include_audio: bool,
  cancel_flag: Arc<AtomicBool>,
  progress_cb: Option<Arc<dyn Fn(f32, usize, usize) + Send + Sync>>,
) -> Result<String, String> {
  // A stale flag (e.g. a previous cancelled run) must not abort a fresh
  // export; the caller resets it before calling, but be defensive.
  cancel_flag.store(false, Ordering::SeqCst);

  let (width, height) = export_dimensions(&config);
  let fps = config.export.fps.max(1);
  let fft_size = config.reactivity.fft_size.max(64);
  let bar_count = config.reactivity.bar_count.max(1);

  let audio = AudioData::decode_file(&audio_file_path)?;
  if audio.duration_seconds <= 0.0 {
    return Err("Audio duration is 0; cannot export.".to_string());
  }
  let total_frames = (audio.duration_seconds * fps as f64).ceil() as usize;
  if total_frames == 0 {
    return Err("No frames to render.".to_string());
  }

  let (mut child, stderr_buf, stderr_reader) = spawn_ffmpeg(
    None,
    fps,
    width,
    height,
    &output_path,
    &audio_file_path,
    include_audio,
    config.export.encoder.as_deref().unwrap_or("auto"),
  )?;

  let _start = std::time::Instant::now();

  // Windows threads default to a 1 MB stack (the main thread was raised to
  // 8 MB via /STACK in build.rs because deep mesh/scene building overflowed
  // it), and this thread does the same depth of rendering per frame. Give it
  // the same 8 MB so a long export can never abort with STATUS_STACK_OVERFLOW
  // midway.
  let render_result = std::thread::Builder::new()
    .stack_size(8 * 1024 * 1024)
    .spawn(move || -> Result<(), String> {
    let mut gpu = pollster::block_on(GpuRenderer::new(width, height))
      .map_err(|e| format!("GPU unavailable: {}", e))?;

    let analyzer = FftAnalyzer::new(fft_size);
    let mut rstate = RenderState::new(bar_count, 0xC0FFEE);

    // Custom background image: decode once, upload to a persistent atlas layer.
    if let Some((rgba, w, h)) = decode_background_image(config.background.custom_image_uri.as_deref()) {
      if let Some((tw, th)) = gpu.upload_background_image(IMAGE_LAYER, &rgba, w, h) {
        rstate.background_image = Some(BackgroundImage { layer: IMAGE_LAYER, w: tw, h: th });
      }
    }

    // Radial center image: decode once, upload to a persistent atlas layer.
    if let Some((rgba, w, h)) = decode_background_image(config.background.radial_center_image_uri.as_deref()) {
      if let Some((tw, th)) = gpu.upload_background_image(RADIAL_CENTER_IMAGE_LAYER, &rgba, w, h) {
        rstate.radial_center_image = Some(BackgroundImage { layer: RADIAL_CENTER_IMAGE_LAYER, w: tw, h: th });
      }
    }

    // Pipe raw RGBA frames to FFmpeg on a writer thread so the GPU can keep
    // rendering the next frame while FFmpeg consumes the current one.
    let mut stdin = child.stdin.take().ok_or_else(|| "No ffmpeg stdin".to_string())?;
    let stderr_buf_inner = Arc::clone(&stderr_buf);
    let stderr_buf_writer = Arc::clone(&stderr_buf);
    let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(4);
    let writer = std::thread::spawn(move || -> Result<(), String> {
      for rgba in rx {
        if stdin.write_all(&rgba).is_err() {
          let msg = get_stderr_msg(&stderr_buf_writer);
          return Err(format!("FFmpeg write failed. FFmpeg stderr: {}", msg));
        }
      }
      drop(stdin);
      Ok(())
    });

    let smoothing = clamp_smoothing(config.reactivity.smoothing);
    let mut prev_smoothed: Option<Vec<f32>> = None;

    let mut render_frame = |gpu: &mut GpuRenderer, frame: usize, slot: usize| -> Result<(), String> {
      let time_sec = frame as f64 / fps as f64;
      let samples = audio.get_sample_window(time_sec, fft_size);

      let (mag, _bass) = analyzer
        .compute_full_spectrum(&samples)
        .map_err(|e| format!("FFT error: {}", e))?;

      let smoothed_mag = match &mut prev_smoothed {
        Some(prev) if prev.len() == mag.len() => {
          for (p, &m) in prev.iter_mut().zip(mag.iter()) {
            *p = *p * smoothing + m * (1.0 - smoothing);
          }
          prev.clone()
        }
        _ => {
          prev_smoothed = Some(mag.clone());
          mag.clone()
        }
      };

      let freq_u8: Vec<u8> = smoothed_mag.iter().map(|m| (m.clamp(0.0, 1.0) * 255.0).round() as u8).collect();
      let time_u8: Vec<u8> = samples
        .iter()
        .map(|s| ((s + 1.0) * 128.0).clamp(0.0, 255.0) as u8)
        .collect();

      // Export always renders as if playing (TS export sets isPlaying=true).
      let frame_time = time_sec as f32;
      // Export time is already continuous, so it doubles as the fx clock.
      rstate.screen_fx.fx_time = frame_time;
      // Advance the envelope ONCE per frame; the passes below only read it.
      let env = advance_envelope(&mut rstate, &config, &freq_u8, frame_time, true);
      let fx = crate::renderers::screen_effects::post_fx(
        &mut rstate.screen_fx,
        &config.screen_effects,
        env.above_floor,
        env.beat_strength,
        config.export.fps.max(1) as f32,
      );
      let bg_only = config.screen_effects.background_only.unwrap_or(true);

      if let Some(fx_ref) = fx.as_ref().filter(|_| bg_only) {
        // backgroundOnly: apply the frame-sampling effect to the background
        // layer only, then draw the style/particles/text over it (mirrors
        // canvasRenderer drawFrame).
        let mut bg_canvas = GpuCanvas::new(width, height);
        let mut bg_scene = Scene3D::new();
        draw_frame_pass(
          &mut bg_canvas, &mut bg_scene, &mut rstate, &config, &freq_u8, &time_u8, frame_time, &env,
          FramePass::BackgroundOnly,
        );
        let bg_mesh = bg_canvas.finish_with(bg_scene);
        let mut fg_canvas = GpuCanvas::new(width, height);
        let mut fg_scene = Scene3D::new();
        draw_frame_pass(
          &mut fg_canvas, &mut fg_scene, &mut rstate, &config, &freq_u8, &time_u8, frame_time, &env,
          FramePass::ForegroundOnly,
        );
        let fg_mesh = fg_canvas.finish_with(fg_scene);
        gpu.render_bg_fx_then_over(&bg_mesh, &fg_mesh, fx_ref, slot);
      } else {
        let mut canvas = GpuCanvas::new(width, height);
        let mut scene3d = Scene3D::new();
        draw_frame_pass(
          &mut canvas, &mut scene3d, &mut rstate, &config, &freq_u8, &time_u8, frame_time, &env,
          FramePass::All,
        );
        let mesh = canvas.finish_with(scene3d);
        match fx {
          Some(fx) => gpu.render_into_fx(&mesh, &fx, slot),
          None => gpu.render_into(&mesh, slot),
        }
      }
      Ok(())
    };

    let mut final_error: Option<String> = None;

    // Kick off frame 0 into slot 0, then overlap each subsequent frame's GPU
    // render with the previous frame's readback + FFmpeg write.
    if let Err(e) = render_frame(&mut gpu, 0, 0) {
      final_error = Some(e);
    }

    if final_error.is_none() {
      for frame in 1..total_frames {
        if cancel_flag.load(Ordering::SeqCst) {
          final_error = Some("Export cancelled".to_string());
          break;
        }

        if let Err(e) = render_frame(&mut gpu, frame, frame % 2) {
          final_error = Some(e);
          break;
        }

        let rgba = gpu.readback((frame - 1) % 2);
        if tx.send(rgba).is_err() {
          let msg = get_stderr_msg(&stderr_buf_inner);
          final_error = Some(format!("FFmpeg write failed. FFmpeg stderr: {}", msg));
          break;
        }

        if frame % 10 == 0 || frame == total_frames - 1 {
          let percent = 10.0 + (frame as f32 / total_frames as f32) * 88.0;
          if let Some(ref cb) = progress_cb {
            cb(percent, frame + 1, total_frames);
          }
        }
      }
    }

    if final_error.is_none() {
      let rgba = gpu.readback((total_frames - 1) % 2);
      if tx.send(rgba).is_err() {
        let msg = get_stderr_msg(&stderr_buf_inner);
        final_error = Some(format!("FFmpeg write failed. FFmpeg stderr: {}", msg));
      }
    }

    drop(tx);
    let write_err = match writer.join() {
      Ok(Ok(())) => None,
      Ok(Err(e)) => Some(e),
      Err(_) => Some("FFmpeg writer panicked".to_string()),
    };
    if final_error.is_none() {
      final_error = write_err;
    }

    if let Some(e) = final_error {
      let _ = child.kill();
      let _ = child.wait();
      let _ = stderr_reader.join();
      return Err(e);
    }

    // Signal EOF to FFmpeg and reap the process.
    if let Some(stdin) = child.stdin.take() {
      drop(stdin);
    }
    let status = child.wait().map_err(|e| format!("FFmpeg wait failed: {}", e))?;
    let _ = stderr_reader.join();

    if status.success() {
      Ok(())
    } else {
      let msg = get_stderr_msg(&stderr_buf);
      Err(format!("FFmpeg error (exit {}): {}", status.code().unwrap_or(-1), msg))
    }
  })
  .expect("Failed to spawn GPU export render thread")
  .join()
  .map_err(|_| "GPU export thread panicked".to_string())?;

  if let Err(err) = render_result {
    let _ = std::fs::remove_file(&output_path);
    return Err(err);
  }

  Ok(output_path)
}

/// Clamp the smoothing value to the same range the TS analyser uses
/// (`audioEngine.setSmoothing` → `Math.max(0, Math.min(0.99, smoothing))`).
/// Web Audio's `smoothingTimeConstant` accepts up to 1.0; the TS guard is
/// 0.99, so the export path must use the same ceiling for preview/export
/// parity when a config carries a value above the slider max (0.95).
pub fn clamp_smoothing(value: f32) -> f32 {
  value.clamp(0.0, 0.99)
}

/// Take the cached render state for a preview frame, or build a fresh one.
///
/// The cached state is reused when its `peak_data` length still matches the
/// current bar count, so particles, music notes, peaks, VU decay and RNG
/// continuity persist across frames (mirrors the canvas renderer, which keeps
/// this state alive). A bar count change — or the first frame — rebuilds from
/// scratch with the export seed, with one exception: the text fade-in clock
/// (`text_play_start_frame` / `text_was_playing`) is carried over. textOverlay.ts
/// keeps those at module scope and only resets them via `resetTextFadeState()`
/// (export start / playback restart at offset 0) — never when the bar-count
/// slider moves — so a rebuild here must not make fade-in text blink.
pub fn take_or_init_render_state(
  cached: &mut Option<RenderState>,
  bar_count: usize,
) -> RenderState {
  match cached.take() {
    Some(rs) if rs.peak_data.len() == bar_count => rs,
    // Bar count changed: only resize peak_data — keep all other live state
    // (particles, music notes, stars, RNG, aurora, screen fx, text clock)
    // so that decorations don't teleport/flash when the user drags the slider.
    Some(mut rs) => {
      rs.peak_data.resize(bar_count, 0.0);
      rs
    }
    _ => RenderState::new(bar_count, 0xC0FFEE),
  }
}

/// Render one live-preview frame into `engine`, reusing the cached render
/// state (see [`take_or_init_render_state`]). Kept separate from the tauri
/// command so the persistence logic can be unit-tested without a tauri app.
pub fn render_preview_frame_inner(
  engine: &mut crate::app_state::GpuPreviewEngine,
  config: &VisualizerConfig,
  freq_data: &[u8],
  time_data: &[u8],
  frame_time: f32,
  fx_time: f32,
  fps: f32,
  width: u32,
  height: u32,
  is_playing: bool,
) -> Result<Vec<u8>, String> {
  let bar_count = config.reactivity.bar_count.clamp(8, 128);

  // Reuse the previous frame's render state so particles, music notes, peaks,
  // VU decay and RNG continuity persist across frames. Rebuild only on the
  // first frame or when the bar count changed (peak_data is sized to it).
  let mut rstate = take_or_init_render_state(&mut engine.render_state, bar_count);

  // A style switch must NOT inherit the previous style's live state: e.g.
  // waterfall/radial keep a rolling `frame_history`, pulse styles fill `rings`,
  // and VU/peak/beat counters decay at style-specific rates. Carrying that
  // state into the newly selected style renders stale artifacts of the old
  // style ("preview still shows the previous style"). Detect the change and
  // rebuild from scratch so the new style starts clean.
  if engine.last_style.as_ref() != Some(&config.style) {
    rstate = RenderState::new(bar_count, 0xC0FFEE);
    engine.last_style = Some(config.style.clone());
  }

  // Monotonic clock for time-based effects: keeps them animating across
  // pause/seek instead of freezing/jumping (fx_time, not song time).
  rstate.screen_fx.fx_time = fx_time;

  let cur_bg_uri = config.background.custom_image_uri.clone();
  if engine.bg_image_uri != cur_bg_uri {
    engine.bg_image_uri = cur_bg_uri.clone();
    if let Some((rgba, w, h)) = decode_background_image(cur_bg_uri.as_deref()) {
      if let Some((tw, th)) = engine.renderer.upload_background_image(IMAGE_LAYER, &rgba, w, h) {
        engine.bg_image_info = Some((tw, th));
      } else {
        engine.bg_image_info = None;
      }
    } else {
      engine.bg_image_info = None;
    }
  }
  if let Some((tw, th)) = engine.bg_image_info {
    rstate.background_image = Some(BackgroundImage { layer: IMAGE_LAYER, w: tw, h: th });
  } else {
    rstate.background_image = None;
  }

  let cur_radial_uri = config.background.radial_center_image_uri.clone();
  if engine.radial_image_uri != cur_radial_uri {
    engine.radial_image_uri = cur_radial_uri.clone();
    if let Some((rgba, w, h)) = decode_background_image(cur_radial_uri.as_deref()) {
      if let Some((tw, th)) = engine.renderer.upload_background_image(RADIAL_CENTER_IMAGE_LAYER, &rgba, w, h) {
        engine.radial_image_info = Some((tw, th));
      } else {
        engine.radial_image_info = None;
      }
    } else {
      engine.radial_image_info = None;
    }
  }
  if let Some((tw, th)) = engine.radial_image_info {
    rstate.radial_center_image = Some(BackgroundImage { layer: RADIAL_CENTER_IMAGE_LAYER, w: tw, h: th });
  } else {
    rstate.radial_center_image = None;
  }

  // Advance the envelope once; the passes below only read it.
  let env = advance_envelope(&mut rstate, config, freq_data, frame_time, is_playing);
  let fx = crate::renderers::screen_effects::post_fx(
    &mut rstate.screen_fx,
    &config.screen_effects,
    env.above_floor,
    env.beat_strength,
    fps,
  );
  let bg_only = config.screen_effects.background_only.unwrap_or(true);

  // Ping-pong readback: render this tick into `render_slot` while reading the
  // previous tick's frame from the OTHER slot (already complete — the UI
  // thread never spins waiting on freshly-submitted GPU work). Only the very
  // first frame waits synchronously, because there is no prior frame yet.
  let render_slot = engine.next_slot;
  let read_slot = 1 - render_slot;

  if let Some(fx_ref) = fx.as_ref().filter(|_| bg_only) {
    // backgroundOnly: effect applies to the background layer only.
    let mut bg_canvas = GpuCanvas::new(width, height);
    let mut bg_scene = Scene3D::new();
    draw_frame_pass(
      &mut bg_canvas, &mut bg_scene, &mut rstate, config, freq_data, time_data, frame_time, &env,
      FramePass::BackgroundOnly,
    );
    let bg_mesh = bg_canvas.finish_with(bg_scene);
    let mut fg_canvas = GpuCanvas::new(width, height);
    let mut fg_scene = Scene3D::new();
    draw_frame_pass(
      &mut fg_canvas, &mut fg_scene, &mut rstate, config, freq_data, time_data, frame_time, &env,
      FramePass::ForegroundOnly,
    );
    let fg_mesh = fg_canvas.finish_with(fg_scene);
    engine.renderer.render_bg_fx_then_over(&bg_mesh, &fg_mesh, fx_ref, render_slot);
  } else {
    let mut canvas = GpuCanvas::new(width, height);
    let mut scene3d = Scene3D::new();
    draw_frame_pass(
      &mut canvas, &mut scene3d, &mut rstate, config, freq_data, time_data, frame_time, &env,
      FramePass::All,
    );
    let mesh = canvas.finish_with(scene3d);
    match fx {
      Some(fx) => engine.renderer.render_into_fx(&mesh, &fx, render_slot),
      None => engine.renderer.render_into(&mesh, render_slot),
    }
  }

  let rgba = if engine.has_prev {
    engine.renderer.readback(read_slot)
  } else {
    // First frame: no prior slot to read, so wait on this one (one-time cost).
    let rgba = engine.renderer.readback(render_slot);
    engine.has_prev = true;
    rgba
  };
  engine.next_slot = read_slot;

  // Persist the state for the next preview frame.
  engine.render_state = Some(rstate);

  Ok(rgba)
}
