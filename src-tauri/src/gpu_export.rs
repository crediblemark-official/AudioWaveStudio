//! export_gpu — full-Rust wgpu export path.
//! Decodes audio, precomputes FFT per frame, draws with the Rust renderers
//! (gpu2d) and pipes JPEG frames to ffmpeg. Falls back via the JS caller to
//! the canvas exporter when the GPU is unavailable.

use crate::audio_decoder::AudioData;
use crate::config::{ExportResolution, ExportSettings, VisualizerConfig};
use crate::ffmpeg::resolve_ffmpeg;
use crate::fft_analyzer::FftAnalyzer;
use crate::gpu2d::{GpuCanvas, GpuRenderer, IMAGE_LAYER, RADIAL_CENTER_IMAGE_LAYER};
use crate::renderers::{advance_envelope, draw_frame_pass, BackgroundImage, FramePass, RenderState};
use base64::{Engine as _, engine::general_purpose};
use serde::Serialize;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tauri::Emitter;

/// Decode a custom background image from a data URL (`data:...;base64,`),
/// `file://` URI, or plain file path into RGBA pixels.
fn decode_background_image(uri: Option<&str>) -> Option<(Vec<u8>, u32, u32)> {
  let uri = uri?;
  let bytes: Vec<u8> = if let Some((_, b64)) = uri.split_once("base64,") {
    general_purpose::STANDARD.decode(b64).ok()?
  } else if let Some(path) = uri.strip_prefix("file://") {
    std::fs::read(path).ok()?
  } else if std::path::Path::new(uri).exists() {
    std::fs::read(uri).ok()?
  } else {
    return None;
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

pub fn export_dimensions(config: &VisualizerConfig) -> (u32, u32) {
  let ExportSettings { resolution, aspect_ratio, .. } = &config.export;
  let (mut width, mut height) = match resolution {
    ExportResolution::P1080 => (1920u32, 1080u32),
    ExportResolution::P720 => (1280, 720),
    ExportResolution::K4 => (3840, 2160),
  };
  use crate::config::AspectRatio;
  match aspect_ratio {
    AspectRatio::Portrait => {
      std::mem::swap(&mut width, &mut height);
    }
    AspectRatio::Square => {
      height = width;
    }
    AspectRatio::Widescreen => {}
  }
  (width, height)
}

/// Applies per-encoder quality args plus a resolution-scaled bitrate cap.
///
/// Hardware encoders (NVENC/QSV/VAAPI/AMF) silently ignore `-maxrate` when
/// left in a constant-quality mode (NVENC `-cq`, VAAPI `-qp`, QSV
/// `-global_quality`), so they get an explicit bitrate target + VBV cap.
/// Software encoders (x264/x265/SVT-AV1) use CRF with a VBV cap instead.
pub fn apply_encoder_args(cmd: &mut Command, encoder_id: &str, cap_kbps: u32) {
  let cap = format!("{cap_kbps}k");
  let buf = format!("{}k", cap_kbps * 2);
  let software = matches!(
    encoder_id,
    "libx264" | "libx265" | "libsvtav1" | "libaom-av1" | "libvpx-vp9"
  );

  match encoder_id {
    "libx264" => {
      cmd.arg("-preset").arg("ultrafast").arg("-crf").arg("23");
    }
    "libx265" => {
      cmd.arg("-preset").arg("ultrafast").arg("-crf").arg("26");
    }
    "libsvtav1" => {
      cmd.arg("-preset").arg("8").arg("-crf").arg("30");
    }
    "h264_nvenc" | "hevc_nvenc" => {
      cmd.arg("-preset").arg("p1").arg("-rc").arg("vbr").arg("-cq").arg("23");
    }
    "av1_nvenc" => {
      cmd.arg("-preset").arg("p1").arg("-rc").arg("vbr").arg("-cq").arg("30");
    }
    "h264_qsv" | "hevc_qsv" | "av1_qsv" => {
      cmd.arg("-preset").arg("veryfast");
    }
    "h264_vaapi" | "hevc_vaapi" | "av1_vaapi" => {
      cmd.arg("-rc_mode").arg("VBR");
    }
    "h264_amf" => {
      cmd.arg("-quality")
        .arg("speed")
        .arg("-rc")
        .arg("vbr_peak")
        .arg("-qp_i")
        .arg("23")
        .arg("-qp_p")
        .arg("23");
    }
    _ => {}
  }

  if software {
    cmd.arg("-maxrate").arg(&cap).arg("-bufsize").arg(&buf);
  } else {
    cmd.arg("-b:v").arg(&cap).arg("-maxrate").arg(&cap).arg("-bufsize").arg(&buf);
  }
}

fn spawn_ffmpeg(
  app_handle: &tauri::AppHandle,
  fps: u32,
  width: u32,
  height: u32,
  output_mp4_path: &str,
  audio_file_path: &str,
  include_audio: bool,
  encoder_preference: &str,
) -> Result<(Child, Arc<Mutex<String>>, JoinHandle<()>), String> {
  let ffmpeg_exe = resolve_ffmpeg(app_handle)?;

  let encoder_name = crate::hardware::pick_encoder(&ffmpeg_exe, encoder_preference);

  let mut cmd = Command::new(&ffmpeg_exe);
  cmd.arg("-y").arg("-loglevel").arg("warning");

  if encoder_name.ends_with("_vaapi") {
    if let Some(dev) = crate::hardware::pick_vaapi_device() {
      cmd.arg("-vaapi_device").arg(dev);
    }
  }

  cmd.arg("-thread_queue_size").arg("2048")
    .arg("-f").arg("rawvideo")
    .arg("-pix_fmt").arg("rgba")
    .arg("-r").arg(fps.to_string())
    .arg("-s").arg(format!("{}x{}", width, height))
    .arg("-i").arg("pipe:0");

  let vf_filter = if encoder_name.ends_with("_vaapi") {
    "format=nv12,hwupload"
  } else {
    "scale=out_color_matrix=bt709:out_range=limited,format=yuv420p"
  };

  if include_audio {
    cmd.arg("-thread_queue_size").arg("2048")
      .arg("-i").arg(audio_file_path)
      .arg("-map").arg("0:v:0")
      .arg("-map").arg("1:a:0?")
      .arg("-c:v").arg(&encoder_name)
      .arg("-vf").arg(vf_filter)
      .arg("-c:a").arg("aac");
  } else {
    cmd.arg("-map").arg("0:v:0")
      .arg("-c:v").arg(&encoder_name)
      .arg("-vf").arg(vf_filter)
      .arg("-an");
  }

  // Cap the bitrate so exports stay small even for incompressible content.
  // The visualizer's film-grain/noise overlay used to be per-frame random
  // noise, which H.264 cannot compress (a pure CRF/CQ encode of a noisy scene
  // lands at ~100 Mbps -> multi-GB files for 5 minutes). -crf/-cq keeps simple
  // scenes small; -maxrate/-bufsize bounds noisy scenes. Cap scales with
  // resolution: ~12 Mbps @1080p, ~5 Mbps @720p.
  let px = (width * height) as f64;
  let cap_mbps = (px / 172_800.0).clamp(4.0, 15.0);
  let cap_kbps = (cap_mbps * 1000.0).round() as u32;
  apply_encoder_args(&mut cmd, &encoder_name, cap_kbps);

  // Put the moov atom at the front so players can start instantly instead of
  // scanning to the end of the file (a no-faststart MP4 won't begin playback
  // until the muxer's trailer is read — effectively unplayable at GB sizes).
  cmd.arg("-movflags").arg("+faststart").arg(output_mp4_path)
    .stdin(Stdio::piped())
    .stderr(Stdio::piped());

  let mut child = cmd.spawn().map_err(|e| {
    if e.kind() == std::io::ErrorKind::NotFound {
      format!("FFmpeg not found at '{}'.", ffmpeg_exe)
    } else {
      format!("Failed to start ffmpeg: {}", e)
    }
  })?;

  let stderr_buf = Arc::new(Mutex::new(String::new()));
  let stderr_buf_clone = Arc::clone(&stderr_buf);
  let mut stderr_handle = child.stderr.take().unwrap();

  let stderr_reader = std::thread::spawn(move || {
    let mut buf = [0u8; 2048];
    loop {
      match std::io::Read::read(&mut stderr_handle, &mut buf) {
        Ok(0) => break,
        Ok(n) => {
          if let Ok(mut lock) = stderr_buf_clone.lock() {
            lock.push_str(&String::from_utf8_lossy(&buf[..n]));
          }
        }
        Err(_) => break,
      }
    }
  });

  Ok((child, stderr_buf, stderr_reader))
}

fn get_stderr_msg(buf: &Arc<Mutex<String>>) -> String {
  buf.lock().map(|s| s.trim().to_string()).unwrap_or_default()
}

#[tauri::command]
pub async fn export_gpu(
  app_handle: tauri::AppHandle,
  state: tauri::State<'_, crate::AppState>,
  config: VisualizerConfig,
  audio_file_path: String,
  output_path: String,
  include_audio: bool,
) -> Result<String, String> {
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

  let _ = app_handle.emit(
    "gpu-export-progress",
    GpuExportProgress {
      percent: 0.0,
      current_frame: 0,
      total_frames,
      elapsed_time: 0.0,
    },
  );

  let (mut child, stderr_buf, stderr_reader) = spawn_ffmpeg(
    &app_handle,
    fps,
    width,
    height,
    &output_path,
    &audio_file_path,
    include_audio,
    config.export.encoder.as_deref().unwrap_or("auto"),
  )?;

  let cancel_flag = Arc::new(AtomicBool::new(false));
  if let Ok(mut guard) = state.gpu_cancel.lock() {
    *guard = Some(Arc::clone(&cancel_flag));
  }

  let app_handle2 = app_handle.clone();
  let start = std::time::Instant::now();

  let render_result = std::thread::spawn(move || -> Result<(), String> {
    let mut gpu = pollster::block_on(GpuRenderer::new(width, height))
      .map_err(|e| format!("GPU unavailable: {}", e))?;

    let analyzer = FftAnalyzer::new(fft_size);
    let mut rstate = RenderState::new(bar_count, 0xC0FFEE);

    // Custom background image: decode once, upload to a persistent atlas layer.
    if let Some((rgba, w, h)) = decode_background_image(config.background.custom_image_uri.as_deref()) {
      if let Some((tw, th)) = gpu.upload_image_layer(IMAGE_LAYER, &rgba, w, h) {
        rstate.background_image = Some(BackgroundImage { layer: IMAGE_LAYER, w: tw, h: th });
      }
    }

    // Radial center image: decode once, upload to a persistent atlas layer.
    if let Some((rgba, w, h)) = decode_background_image(config.background.radial_center_image_uri.as_deref()) {
      if let Some((tw, th)) = gpu.upload_image_layer(RADIAL_CENTER_IMAGE_LAYER, &rgba, w, h) {
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
      // Advance the envelope ONCE per frame; the passes below only read it.
      let env = advance_envelope(&mut rstate, &config, &freq_u8, frame_time, true);
      let fx = crate::renderers::screen_effects::post_fx(
        &mut rstate.screen_fx,
        &config.screen_effects,
        env.above_floor,
        rstate.beat_strength,
        frame_time,
      );
      let bg_only = config.screen_effects.background_only.unwrap_or(true);

      if fx.is_some() && bg_only {
        // backgroundOnly: apply the frame-sampling effect to the background
        // layer only, then draw the style/particles/text over it (mirrors
        // canvasRenderer drawFrame).
        let mut bg_canvas = GpuCanvas::new(width, height);
        draw_frame_pass(
          &mut bg_canvas, &mut rstate, &config, &freq_u8, &time_u8, frame_time, &env,
          FramePass::BackgroundOnly,
        );
        let bg_mesh = bg_canvas.finish();
        let mut fg_canvas = GpuCanvas::new(width, height);
        draw_frame_pass(
          &mut fg_canvas, &mut rstate, &config, &freq_u8, &time_u8, frame_time, &env,
          FramePass::ForegroundOnly,
        );
        let fg_mesh = fg_canvas.finish();
        gpu.render_bg_fx_then_over(&bg_mesh, &fg_mesh, fx.as_ref().unwrap(), slot);
      } else {
        let mut canvas = GpuCanvas::new(width, height);
        draw_frame_pass(
          &mut canvas, &mut rstate, &config, &freq_u8, &time_u8, frame_time, &env,
          FramePass::All,
        );
        let mesh = canvas.finish();
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

        if frame % 3 == 0 || frame + 1 == total_frames {
          let pct = ((frame + 1) as f32 * 100.0) / total_frames as f32;
          let _ = app_handle2.emit(
            "gpu-export-progress",
            GpuExportProgress {
              percent: pct,
              current_frame: frame + 1,
              total_frames,
              elapsed_time: start.elapsed().as_secs_f32(),
            },
          );
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

    // Signal EOF to FFmpeg and reap the process.
    if let Some(stdin) = child.stdin.take() {
      drop(stdin);
    }
    let status = child.wait().map_err(|e| format!("FFmpeg wait failed: {}", e))?;
    let _ = stderr_reader.join();

    if let Some(e) = final_error {
      return Err(e);
    }

    if status.success() {
      Ok(())
    } else {
      let msg = get_stderr_msg(&stderr_buf);
      Err(format!("FFmpeg error (exit {}): {}", status.code().unwrap_or(-1), msg))
    }
  })
  .join()
  .map_err(|_| "GPU export thread panicked".to_string())?;

  if let Ok(mut guard) = state.gpu_cancel.lock() {
    *guard = None;
  }

  if let Err(err) = render_result {
    let _ = std::fs::remove_file(&output_path);
    return Err(err);
  }

  Ok(output_path)
}

#[tauri::command]
pub fn cancel_gpu_export(state: tauri::State<'_, crate::AppState>) {
  if let Ok(guard) = state.gpu_cancel.lock() {
    if let Some(flag) = guard.as_ref() {
      flag.store(true, Ordering::SeqCst);
    }
  }
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
    // Bar count changed: restart per-bar state (peaks, particles, RNG) but
    // keep the fade clock — the TS module-level fade state survives config
    // changes, so the preview must not re-fade the text from 0.
    Some(rs) => {
      let mut fresh = RenderState::new(bar_count, 0xC0FFEE);
      fresh.text_play_start_frame = rs.text_play_start_frame;
      fresh.text_was_playing = rs.text_was_playing;
      fresh
    }
    _ => RenderState::new(bar_count, 0xC0FFEE),
  }
}

/// Render one live-preview frame into `engine`, reusing the cached render
/// state (see [`take_or_init_render_state`]). Kept separate from the tauri
/// command so the persistence logic can be unit-tested without a tauri app.
pub fn render_preview_frame_inner(
  engine: &mut crate::GpuPreviewEngine,
  config: &VisualizerConfig,
  freq_data: &[u8],
  time_data: &[u8],
  frame_time: f32,
  width: u32,
  height: u32,
  is_playing: bool,
) -> Result<Vec<u8>, String> {
  let bar_count = config.reactivity.bar_count.min(128);

  // Reuse the previous frame's render state so particles, music notes, peaks,
  // VU decay and RNG continuity persist across frames. Rebuild only on the
  // first frame or when the bar count changed (peak_data is sized to it).
  let mut rstate = take_or_init_render_state(&mut engine.render_state, bar_count);

  let cur_bg_uri = config.background.custom_image_uri.clone();
  if engine.bg_image_uri != cur_bg_uri {
    engine.bg_image_uri = cur_bg_uri.clone();
    if let Some((rgba, w, h)) = decode_background_image(cur_bg_uri.as_deref()) {
      if let Some((tw, th)) = engine.renderer.upload_image_layer(IMAGE_LAYER, &rgba, w, h) {
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
  }

  let cur_radial_uri = config.background.radial_center_image_uri.clone();
  if engine.radial_image_uri != cur_radial_uri {
    engine.radial_image_uri = cur_radial_uri.clone();
    if let Some((rgba, w, h)) = decode_background_image(cur_radial_uri.as_deref()) {
      if let Some((tw, th)) = engine.renderer.upload_image_layer(RADIAL_CENTER_IMAGE_LAYER, &rgba, w, h) {
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
  }

  // Advance the envelope once; the passes below only read it.
  let env = advance_envelope(&mut rstate, config, freq_data, frame_time, is_playing);
  let fx = crate::renderers::screen_effects::post_fx(
    &mut rstate.screen_fx,
    &config.screen_effects,
    env.above_floor,
    rstate.beat_strength,
    frame_time,
  );
  let bg_only = config.screen_effects.background_only.unwrap_or(true);

  if fx.is_some() && bg_only {
    // backgroundOnly: effect applies to the background layer only.
    let mut bg_canvas = GpuCanvas::new(width, height);
    draw_frame_pass(
      &mut bg_canvas, &mut rstate, config, freq_data, time_data, frame_time, &env,
      FramePass::BackgroundOnly,
    );
    let bg_mesh = bg_canvas.finish();
    let mut fg_canvas = GpuCanvas::new(width, height);
    draw_frame_pass(
      &mut fg_canvas, &mut rstate, config, freq_data, time_data, frame_time, &env,
      FramePass::ForegroundOnly,
    );
    let fg_mesh = fg_canvas.finish();
    engine.renderer.render_bg_fx_then_over(&bg_mesh, &fg_mesh, fx.as_ref().unwrap(), 0);
  } else {
    let mut canvas = GpuCanvas::new(width, height);
    draw_frame_pass(
      &mut canvas, &mut rstate, config, freq_data, time_data, frame_time, &env,
      FramePass::All,
    );
    let mesh = canvas.finish();
    match fx {
      Some(fx) => engine.renderer.render_into_fx(&mesh, &fx, 0),
      None => engine.renderer.render_into(&mesh, 0),
    }
  }
  let rgba = engine.renderer.readback(0);

  // Persist the state for the next preview frame.
  engine.render_state = Some(rstate);

  Ok(rgba)
}

#[tauri::command]
pub fn render_rust_preview_frame(
  state: tauri::State<'_, crate::AppState>,
  config: VisualizerConfig,
  freq_data: Vec<u8>,
  time_data: Vec<u8>,
  frame_time: f32,
  width: u32,
  height: u32,
  is_playing: bool,
) -> Result<Vec<u8>, String> {
  let mut lock = state.preview_gpu.lock().map_err(|e| e.to_string())?;

  if lock.is_none() || lock.as_ref().unwrap().width != width || lock.as_ref().unwrap().height != height {
    let renderer = pollster::block_on(GpuRenderer::new(width, height))
      .map_err(|e| format!("GPU unavailable: {}", e))?;
    *lock = Some(crate::GpuPreviewEngine {
      renderer,
      width,
      height,
      bg_image_uri: None,
      bg_image_info: None,
      radial_image_uri: None,
      radial_image_info: None,
      render_state: None,
    });
  }

  let engine = lock.as_mut().unwrap();
  render_preview_frame_inner(engine, &config, &freq_data, &time_data, frame_time, width, height, is_playing)
}
