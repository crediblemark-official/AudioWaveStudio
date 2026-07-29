mod audio_decoder;
mod fft_analyzer;
mod ffmpeg;
mod gpu_renderer;
mod renderer;
mod video_encoder;

use audio_decoder::AudioData;
use base64::{Engine as _, engine::general_purpose};
use fft_analyzer::FftAnalyzer;
use ffmpeg::resolve_ffmpeg;
use image::codecs::jpeg::JpegEncoder;
use renderer::{RenderConfig, RustRenderer};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use video_encoder::VideoEncoderRust;

struct ExportSession {
  child: Child,
  stderr_handle: std::process::ChildStderr,
}

struct ListenSession {
  child: Child,
}

struct AppState {
  audio_data: Mutex<Option<Arc<AudioData>>>,
  export_session: Mutex<Option<ExportSession>>,
  listen_session: Mutex<Option<ListenSession>>,
  prev_smoothed: Mutex<Option<Vec<f32>>>,
}

#[derive(Serialize, Deserialize)]
pub struct AudioMetadataRust {
  pub duration: f64,
  pub sample_rate: u32,
  pub channels: usize,
}

#[derive(Serialize, Deserialize)]
pub struct AudioDecodeResult {
  pub sample_rate: u32,
  pub channels: usize,
  pub duration: f64,
  pub full_duration: f64,
  pub samples_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SpectrumResultRust {
  pub freq_data: Vec<u8>,
  pub time_data: Vec<u8>,
  pub bass_energy: f32,
}

#[tauri::command]
async fn decode_audio(state: tauri::State<'_, AppState>, file_path: String) -> Result<AudioMetadataRust, String> {
  let file_path2 = file_path.clone();
  let audio = tauri::async_runtime::spawn_blocking(move || {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      AudioData::decode_file(&file_path2)
    }))
    .map_err(|_| "Internal panic during audio decode".to_string())?
  })
  .await
  .map_err(|e| e.to_string())?
  .map_err(|e| e.to_string())?;

  let meta = AudioMetadataRust {
    duration: audio.duration_seconds,
    sample_rate: audio.sample_rate,
    channels: audio.channels,
  };

  let mut guard = state.audio_data.lock().map_err(|e| e.to_string())?;
  *guard = Some(Arc::new(audio));

  Ok(meta)
}

#[tauri::command]
async fn decode_audio_playback(
  state: tauri::State<'_, AppState>,
  file_path: String,
) -> Result<AudioDecodeResult, String> {
  let file_path2 = file_path.clone();
  let audio = tauri::async_runtime::spawn_blocking(move || {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      AudioData::decode_file(&file_path2)
    }))
    .map_err(|_| "Internal panic during audio decode".to_string())?
  })
  .await
  .map_err(|e| e.to_string())?
  .map_err(|e| e.to_string())?;

  let result = AudioDecodeResult {
    sample_rate: audio.sample_rate,
    channels: audio.channels,
    duration: audio.duration_seconds,
    full_duration: audio.duration_seconds,
    samples_count: audio.samples.len(),
  };

  let mut guard = state.audio_data.lock().map_err(|e| e.to_string())?;
  *guard = Some(Arc::new(audio));

  Ok(result)
}

#[tauri::command]
async fn get_audio_chunk_b64(
  state: tauri::State<'_, AppState>,
  start_sec: f64,
  duration_sec: f64,
) -> Result<String, String> {
  let guard = state.audio_data.lock().map_err(|e| e.to_string())?;
  let audio = guard.as_ref().ok_or_else(|| "No audio loaded".to_string())?;

  let sample_rate = audio.sample_rate as f64;
  let start_frame = (start_sec * sample_rate).round() as usize;
  let num_frames = (duration_sec * sample_rate).round() as usize;
  let end_frame = (start_frame + num_frames).min(audio.samples.len());

  if start_frame >= audio.samples.len() {
    return Ok(String::new());
  }

  let chunk = &audio.samples[start_frame..end_frame];
  let bytes: Vec<u8> = chunk.iter().flat_map(|&s| s.to_le_bytes()).collect();
  Ok(general_purpose::STANDARD.encode(&bytes))
}

#[tauri::command]
fn compute_spectrum_rust(
  state: tauri::State<'_, AppState>,
  time_sec: f64,
  bar_count: usize,
  fft_size: usize,
  smoothing: f32,
  bass_multiplier: f32,
) -> Result<SpectrumResultRust, String> {
  let guard = state.audio_data.lock().map_err(|e| e.to_string())?;
  let audio = guard.as_ref().ok_or_else(|| "No audio loaded".to_string())?;

  let window = audio.get_sample_window(time_sec, fft_size);
  let analyzer = FftAnalyzer::new(fft_size);
  let (spectrum_linear, _bass_raw) = analyzer.compute_spectrum(&window, bar_count)?;

  // Convert linear magnitude → dB → byte (matches AnalyserNode.getByteFrequencyData)
  // AnalyserNode defaults: minDecibels=-100, maxDecibels=-30
  let mut freq_data = Vec::with_capacity(bar_count);
  let mut prev = state.prev_smoothed.lock().map_err(|e| e.to_string())?;
  let mut prev_data = prev.as_deref_mut();

  for i in 0..bar_count.min(spectrum_linear.len()) {
    let mag = spectrum_linear[i].max(1e-10);
    let db = 20.0 * mag.log10();
    let mut byte_val = ((db + 100.0) / 70.0) * 255.0;
    byte_val = byte_val.clamp(0.0, 255.0);
    // Apply smoothing (matches AnalyserNode smoothingTimeConstant)
    if let Some(ref mut prev_vec) = prev_data {
      if i < prev_vec.len() {
        byte_val = prev_vec[i] * smoothing + byte_val * (1.0 - smoothing);
      }
    }
    freq_data.push(byte_val.round() as u8);
  }

  // Store current smoothed values
  let mut smoothed = Vec::with_capacity(bar_count);
  for i in 0..bar_count.min(spectrum_linear.len()) {
    smoothed.push(freq_data[i] as f32);
  }
  *prev = Some(smoothed);

  // Convert waveform: f32 (-1..1) → u8 (0..255, 128 = center)
  let time_data: Vec<u8> = window.iter().take(bar_count).map(|&s| {
    ((s + 1.0) * 127.5).clamp(0.0, 255.0).round() as u8
  }).collect();

  // Bass energy from dB-scaled freq_data (matches JS formula)
  let bass_bins = 16.min(bar_count);
  let bass_energy = if bass_bins > 0 {
    let sum: usize = freq_data.iter().take(bass_bins).map(|&v| v as usize).sum();
    (sum as f32 / (bass_bins as f32 * 255.0)) * bass_multiplier
  } else {
    0.0
  };

  Ok(SpectrumResultRust {
    freq_data,
    time_data,
    bass_energy,
  })
}

#[tauri::command]
fn render_frame_rust(
  state: tauri::State<'_, AppState>,
  config: RenderConfig,
  time_sec: f64,
) -> Result<Vec<u8>, String> {
  let guard = state.audio_data.lock().map_err(|e| e.to_string())?;
  let audio = guard.as_ref().ok_or_else(|| "No audio loaded".to_string())?;

  let window = audio.get_sample_window(time_sec, 1024);
  let analyzer = FftAnalyzer::new(1024);
  let (spectrum, bass_energy) = analyzer.compute_spectrum(&window, config.bar_count)?;

  let mut renderer = RustRenderer::new();
  let img = renderer.render_frame(&config, &spectrum, &window, bass_energy);

  let mut png_bytes: Vec<u8> = Vec::new();
  let mut cursor = std::io::Cursor::new(&mut png_bytes);
  img
    .write_to(&mut cursor, image::ImageFormat::Png)
    .map_err(|e| e.to_string())?;

  Ok(png_bytes)
}

#[tauri::command]
async fn export_mp4_native(
  app_handle: tauri::AppHandle,
  state: tauri::State<'_, AppState>,
  audio_file_path: String,
  output_mp4_path: String,
  config: RenderConfig,
  fps: u32,
  include_audio: bool,
) -> Result<String, String> {
  let audio = {
    let guard = state.audio_data.lock().map_err(|e| e.to_string())?;
    guard.as_ref().ok_or_else(|| "No audio loaded".to_string())?.clone()
  };

  let ffmpeg_path = ffmpeg::resolve_ffmpeg(&app_handle)?;

  let handle = app_handle.clone();
  tauri::async_runtime::spawn_blocking(move || {
    VideoEncoderRust::export_to_mp4(&audio, &audio_file_path, &config, output_mp4_path, fps, &ffmpeg_path, include_audio, |progress| {
      let _ = handle.emit("export-progress", progress);
    })
  })
  .await
  .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn start_export_session(
  app_handle: tauri::AppHandle,
  state: tauri::State<'_, AppState>,
  fps: u32,
  width: u32,
  height: u32,
  output_mp4_path: String,
  audio_file_path: String,
  include_audio: bool,
) -> Result<(), String> {
  let ffmpeg_exe = resolve_ffmpeg(&app_handle)?;

  let mut cmd = Command::new(&ffmpeg_exe);
  cmd.arg("-y")
    .arg("-loglevel").arg("error")
    .arg("-f").arg("image2pipe")
    .arg("-c:v").arg("mjpeg")
    .arg("-r").arg(fps.to_string())
    .arg("-s").arg(format!("{}x{}", width, height))
    .arg("-i").arg("pipe:0");

  if include_audio {
    cmd.arg("-i").arg(&audio_file_path)
      .arg("-c:v").arg("libx264")
      .arg("-pix_fmt").arg("yuv420p")
      .arg("-c:a").arg("aac")
      .arg("-shortest");
  } else {
    cmd.arg("-c:v").arg("libx264")
      .arg("-pix_fmt").arg("yuv420p")
      .arg("-an");
  }

  cmd.arg("-preset").arg("ultrafast")
    .arg(&output_mp4_path)
    .stdin(Stdio::piped())
    .stderr(Stdio::piped());

  let mut child = cmd.spawn().map_err(|e| {
    if e.kind() == std::io::ErrorKind::NotFound {
      format!("FFmpeg not found at '{}'.", ffmpeg_exe)
    } else {
      format!("Failed to start ffmpeg: {}", e)
    }
  })?;

  let stderr_handle = child.stderr.take().unwrap();

  let mut guard = state.export_session.lock().map_err(|e| e.to_string())?;
  *guard = Some(ExportSession { child, stderr_handle });

  Ok(())
}

#[tauri::command]
fn write_frame(state: tauri::State<'_, AppState>, jpeg_bytes: Vec<u8>) -> Result<(), String> {
  let mut guard = state.export_session.lock().map_err(|e| e.to_string())?;
  let session = guard.as_mut().ok_or_else(|| "No export session active".to_string())?;

  let stdin = session.child.stdin.as_mut().ok_or_else(|| "FFmpeg stdin not available".to_string())?;
  match stdin.write_all(&jpeg_bytes) {
    Ok(()) => Ok(()),
    Err(e) => {
      let mut stderr_out = std::io::Read::take(&mut session.stderr_handle, 4096);
      let mut err_msg = String::new();
      let _ = std::io::Read::read_to_string(&mut stderr_out, &mut err_msg);
      drop(guard);
      Err(format!("FFmpeg write failed: {}. FFmpeg stderr: {}", e, err_msg.trim()))
    }
  }
}

#[tauri::command]
fn write_frame_rgba(state: tauri::State<'_, AppState>, width: u32, height: u32, rgba_data: Vec<u8>) -> Result<(), String> {
  let mut guard = state.export_session.lock().map_err(|e| e.to_string())?;
  let session = guard.as_mut().ok_or_else(|| "No export session active".to_string())?;

  let stdin = session.child.stdin.as_mut().ok_or_else(|| "FFmpeg stdin not available".to_string())?;

  // Encode RGBA pixels to JPEG in Rust
  let mut jpeg_bytes: Vec<u8> = Vec::new();
  {
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, 95);
    encoder.encode(&rgba_data, width, height, image::ExtendedColorType::Rgba8)
      .map_err(|e| format!("JPEG encode failed: {}", e))?;
  }

  match stdin.write_all(&jpeg_bytes) {
    Ok(()) => Ok(()),
    Err(e) => {
      let mut stderr_out = std::io::Read::take(&mut session.stderr_handle, 4096);
      let mut err_msg = String::new();
      let _ = std::io::Read::read_to_string(&mut stderr_out, &mut err_msg);
      drop(guard);
      Err(format!("FFmpeg write failed: {}. FFmpeg stderr: {}", e, err_msg.trim()))
    }
  }
}

#[tauri::command]
async fn finish_export_session(
  state: tauri::State<'_, AppState>,
) -> Result<String, String> {
  let mut session = {
    let mut guard = state.export_session.lock().map_err(|e| e.to_string())?;
    guard.take().ok_or_else(|| "No export session active".to_string())?
  };

  // Drop stdin to signal EOF to FFmpeg
  if let Some(stdin) = session.child.stdin.take() {
    drop(stdin);
  }

  // Wait for FFmpeg to finish
  let status = session.child.wait().map_err(|e| format!("FFmpeg wait failed: {}", e))?;

  if status.success() {
    Ok("Export completed successfully".to_string())
  } else {
    let mut err_msg = String::new();
    let _ = std::io::Read::read_to_string(&mut session.stderr_handle, &mut err_msg);
    Err(format!("FFmpeg error (exit {}): {}", status.code().unwrap_or(-1), err_msg.trim()))
  }
}

#[tauri::command]
async fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
  tauri::async_runtime::spawn_blocking(move || {
    std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))
  })
  .await
  .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn copy_file_to_path(source: String, destination: String) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || {
    std::fs::copy(&source, &destination).map_err(|e| format!("Failed to copy file: {}", e))?;
    Ok(())
  })
  .await
  .map_err(|e| e.to_string())?
}

#[tauri::command]
fn check_ffmpeg(app_handle: tauri::AppHandle) -> Result<bool, String> {
  Ok(ffmpeg::resolve_ffmpeg(&app_handle).is_ok())
}

#[tauri::command]
fn ffmpeg_download_url() -> String {
  ffmpeg::download_url().to_string()
}

#[tauri::command]
async fn convert_webm_to_mp4(
  app_handle: tauri::AppHandle,
  webm_path: String,
  audio_path: String,
  output_mp4_path: String,
  include_audio: bool,
) -> Result<String, String> {
  let ffmpeg_exe = resolve_ffmpeg(&app_handle)?;

  let mut cmd = Command::new(&ffmpeg_exe);
  cmd.arg("-y")
    .arg("-loglevel").arg("error")
    .arg("-i").arg(&webm_path);

  if include_audio {
    cmd.arg("-i").arg(&audio_path)
      .arg("-c:v").arg("libx264")
      .arg("-pix_fmt").arg("yuv420p")
      .arg("-c:a").arg("aac")
      .arg("-shortest");
  } else {
    cmd.arg("-c:v").arg("libx264")
      .arg("-pix_fmt").arg("yuv420p")
      .arg("-an");
  }

  cmd.arg("-preset").arg("ultrafast")
    .arg(&output_mp4_path);

  let output = cmd.output().map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

  if output.status.success() {
    Ok(format!("Converted to {}", output_mp4_path))
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("FFmpeg error: {}", stderr.trim()))
  }
}

#[tauri::command]
async fn save_upload_to_temp(bytes: Vec<u8>, ext: String) -> Result<String, String> {
  tauri::async_runtime::spawn_blocking(move || {
    let dir = std::env::temp_dir().join("audiowave_uploads");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("upload_{}", std::process::id())).with_extension(&ext);
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
  })
  .await
  .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn start_system_listen(
  app_handle: tauri::AppHandle,
  state: tauri::State<'_, AppState>,
) -> Result<String, String> {
  use std::io::Read;

  let pw_output = std::process::Command::new("pw-dump")
    .arg("-N")
    .output()
    .map_err(|e| format!("pw-dump: {}", e))?;

  let monitors: Vec<String> = serde_json::from_slice::<serde_json::Value>(&pw_output.stdout)
    .map_err(|e| format!("JSON: {}", e))?
    .as_array()
    .ok_or("Not array")?
    .iter()
    .filter_map(|obj| {
      let props = obj.get("info")?.get("props")?;
      if props.get("media.class")?.as_str()? == "Audio/Sink" {
        Some(format!("{}.monitor", props.get("node.name")?.as_str()?))
      } else { None }
    })
    .collect();

  let src = monitors.first().ok_or("No sink found")?.clone();

  let mut child = std::process::Command::new("ffmpeg")
    .args(["-f", "pulse", "-i", &src, "-ac", "1", "-ar", "44100", "-f", "f32le", "-"])
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::null())
    .spawn()
    .map_err(|e| format!("ffmpeg: {}", e))?;

  let mut stdout = child.stdout.take().ok_or("No stdout")?;

  {
    let mut guard = state.listen_session.lock().map_err(|e| e.to_string())?;
    *guard = Some(ListenSession { child });
  }

  let handle = app_handle.clone();
  let fft_size = 1024usize;

  std::thread::spawn(move || {
    let analyzer = fft_analyzer::FftAnalyzer::new(fft_size);
    let mut samples = Vec::with_capacity(fft_size * 2);
    let mut raw = vec![0u8; 8192];

    loop {
      let n = match stdout.read(&mut raw) {
        Ok(0) => break,
        Ok(n) => n,
        Err(e) => { eprintln!("[Listen] read err: {}", e); break; }
      };

      let converted: Vec<f32> = raw[..n].chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
      samples.extend(converted);

      while samples.len() >= fft_size {
        let (spectrum, bass_raw) = match analyzer.compute_spectrum(&samples[..fft_size], 256) {
          Ok(r) => r,
          Err(_) => { samples.drain(0..fft_size / 2); continue; }
        };

        let freq_data: Vec<u8> = spectrum.iter().map(|&mag| {
          ((20.0 * (mag.max(1e-10)).log10() + 100.0) / 70.0 * 255.0).clamp(0.0, 255.0).round() as u8
        }).collect();

        let time_data: Vec<u8> = samples[..fft_size].iter().map(|&s| {
          ((s * 127.0 + 128.0).round() as i16).clamp(0, 255) as u8
        }).collect();

        samples.drain(0..fft_size / 2);

        let _ = handle.emit("listen-freq-data", serde_json::json!({
          "freq_data": freq_data,
          "time_data": time_data,
          "bass_energy": (bass_raw * 4.0).min(1.0),
        }));
      }
    }
  });

  Ok(src)
}

#[tauri::command]
async fn stop_system_listen(state: tauri::State<'_, AppState>) -> Result<(), String> {
  let mut guard = state.listen_session.lock().map_err(|_| "lock")?;
  if let Some(mut s) = guard.take() {
    let _ = std::process::Command::new("kill")
      .args(["-TERM", &s.child.id().to_string()])
      .output();
    let _ = s.child.wait();
  }
  Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_dialog::init())
    .manage(AppState {
      audio_data: Mutex::new(None),
      export_session: Mutex::new(None),
      listen_session: Mutex::new(None),
      prev_smoothed: Mutex::new(None),
    })
    .invoke_handler(tauri::generate_handler![
      decode_audio,
      decode_audio_playback,
      get_audio_chunk_b64,
      read_file_bytes,
      copy_file_to_path,
      check_ffmpeg,
      ffmpeg_download_url,
      save_upload_to_temp,
      compute_spectrum_rust,
      render_frame_rust,
      export_mp4_native,
      start_export_session,
      write_frame,
      write_frame_rgba,
      finish_export_session,
      convert_webm_to_mp4,
      start_system_listen,
      stop_system_listen
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
