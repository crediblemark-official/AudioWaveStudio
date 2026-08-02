pub mod audio_decoder;
pub mod config;
pub mod ffmpeg;
pub mod fft_analyzer;
pub mod gpu2d;
pub mod gpu_export;
pub mod hardware;
pub mod renderers;

use audio_decoder::AudioData;
use base64::{engine::general_purpose, Engine as _};
use ffmpeg::resolve_ffmpeg;
use fft_analyzer::FftAnalyzer;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tauri::Emitter;

struct ExportSession {
    child: Child,
    stderr_buf: Arc<Mutex<String>>,
    stderr_reader: Option<JoinHandle<()>>,
}

fn get_stderr_msg(session: &ExportSession) -> String {
    session
        .stderr_buf
        .lock()
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

struct ListenSession {
    child: Child,
}

pub struct GpuPreviewEngine {
    pub renderer: crate::gpu2d::GpuRenderer,
    pub width: u32,
    pub height: u32,
}

pub struct AppState {
    audio_data: Mutex<Option<Arc<AudioData>>>,
    export_session: Mutex<Option<ExportSession>>,
    listen_session: Mutex<Option<ListenSession>>,
    prev_smoothed: Mutex<Option<Vec<f32>>>,
    gpu_cancel: Mutex<Option<Arc<AtomicBool>>>,
    pub preview_gpu: Mutex<Option<GpuPreviewEngine>>,
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
async fn decode_audio(
    state: tauri::State<'_, AppState>,
    file_path: String,
) -> Result<AudioMetadataRust, String> {
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

    // Reset smoothing state for fresh playback
    if let Ok(mut prev) = state.prev_smoothed.lock() {
        *prev = None;
    }

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
    let audio = guard
        .as_ref()
        .ok_or_else(|| "No audio loaded".to_string())?;

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
    _bar_count: usize,
    fft_size: usize,
    smoothing: f32,
    _bass_multiplier: f32,
) -> Result<SpectrumResultRust, String> {
    let guard = state.audio_data.lock().map_err(|e| e.to_string())?;
    let audio = guard
        .as_ref()
        .ok_or_else(|| "No audio loaded".to_string())?;

    let window = audio.get_sample_window(time_sec, fft_size);
    let analyzer = FftAnalyzer::new(fft_size);
    let (magnitudes, _bass_raw) = analyzer.compute_full_spectrum(&window)?;
    let num_bins = magnitudes.len();

    // Convert linear magnitude → dB → byte (matches AnalyserNode.getByteFrequencyData)
    // AnalyserNode defaults: minDecibels=-100, maxDecibels=-30
    let mut freq_data = Vec::with_capacity(num_bins);
    let mut prev = state.prev_smoothed.lock().map_err(|e| e.to_string())?;
    let mut prev_data = prev.as_deref_mut();

    for i in 0..num_bins {
        let mag = magnitudes[i].max(1e-10);
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
    let mut smoothed = Vec::with_capacity(num_bins);
    for i in 0..num_bins {
        smoothed.push(freq_data[i] as f32);
    }
    *prev = Some(smoothed);

    // Convert waveform: f32 (-1..1) → u8 (0..255, 128 = center)
    let time_data: Vec<u8> = window
        .iter()
        .take(num_bins)
        .map(|&s| ((s + 1.0) * 127.5).clamp(0.0, 255.0).round() as u8)
        .collect();

    // Bass energy from dB-scaled freq_data (matches JS formula)
    let bass_bins = 16.min(num_bins);
    let bass_energy = if bass_bins > 0 {
        let sum: usize = freq_data.iter().take(bass_bins).map(|&v| v as usize).sum();
        sum as f32 / (bass_bins as f32 * 255.0)
    } else {
        0.0
    };

    Ok(SpectrumResultRust {
        freq_data,
        time_data,
        bass_energy,
    })
}

#[derive(Serialize, Deserialize)]
struct PrecomputedSpectra {
    freq_data_all: Vec<u8>,
    time_data_all: Vec<u8>,
    bass_energies: Vec<f32>,
    bar_count: usize,
}

#[tauri::command]
async fn precompute_spectra(
    state: tauri::State<'_, AppState>,
    fps: u32,
    start_frame: usize,
    num_frames: usize,
    bar_count: usize,
    fft_size: usize,
    smoothing: f32,
    _bass_multiplier: f32,
) -> Result<PrecomputedSpectra, String> {
    let audio = {
        let guard = state.audio_data.lock().map_err(|e| e.to_string())?;
        guard
            .as_ref()
            .ok_or_else(|| "No audio loaded".to_string())?
            .clone()
    };

    // Chunked calls continue the smoothing state across calls. A fresh run
    // (start_frame == 0) always starts from a clean slate.
    if start_frame == 0 {
        if let Ok(mut prev) = state.prev_smoothed.lock() {
            *prev = None;
        }
    }
    let prev_seed = match state.prev_smoothed.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => None,
    };

    let (spectra, final_smoothed) = tauri::async_runtime::spawn_blocking(move || {
        let analyzer = FftAnalyzer::new(fft_size);
        let num_bins = fft_size / 2;
        let mut freq_data_all = Vec::with_capacity(num_frames * num_bins);
        let mut time_data_all = Vec::with_capacity(num_frames * num_bins);
        let mut bass_energies = Vec::with_capacity(num_frames);
        let mut prev_smoothed: Option<Vec<f32>> = prev_seed;

        for frame in 0..num_frames {
            let time_sec = (start_frame + frame) as f64 / fps as f64;
            let window = audio.get_sample_window(time_sec, fft_size);
            let (magnitudes, _bass_raw) = analyzer
                .compute_full_spectrum(&window)
                .map_err(|e| e.to_string())?;

            for i in 0..num_bins {
                let mag = magnitudes[i].max(1e-10);
                let db = 20.0 * mag.log10();
                let mut byte_val = ((db + 100.0) / 70.0) * 255.0;
                byte_val = byte_val.clamp(0.0, 255.0);
                if let Some(ref prev_vec) = prev_smoothed {
                    if i < prev_vec.len() {
                        byte_val = prev_vec[i] * smoothing + byte_val * (1.0 - smoothing);
                    }
                }
                freq_data_all.push(byte_val.round() as u8);
            }

            let frame_start = freq_data_all.len() - num_bins;
            let mut smoothed = Vec::with_capacity(num_bins);
            for i in 0..num_bins {
                smoothed.push(freq_data_all[frame_start + i] as f32);
            }
            prev_smoothed = Some(smoothed);

            for &s in window.iter().take(num_bins) {
                time_data_all.push(((s + 1.0) * 127.5).clamp(0.0, 255.0).round() as u8);
            }

            let bass_bins = 16.min(num_bins);
            let bass_energy = if bass_bins > 0 {
                let start = freq_data_all.len() - num_bins;
                let sum: usize = freq_data_all[start..start + bass_bins]
                    .iter()
                    .map(|&v| v as usize)
                    .sum();
                sum as f32 / (bass_bins as f32 * 255.0)
            } else {
                0.0
            };
            bass_energies.push(bass_energy);
        }

        Ok::<_, String>((
            PrecomputedSpectra {
                freq_data_all,
                time_data_all,
                bass_energies,
                bar_count,
            },
            prev_smoothed.unwrap_or_default(),
        ))
    })
    .await
    .map_err(|e| format!("Precompute panicked: {}", e))??;

    // Persist smoothing state so the next chunk call continues seamlessly.
    if let Ok(mut prev) = state.prev_smoothed.lock() {
        *prev = Some(final_smoothed);
    }

    Ok(spectra)
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

    let encoder_name = hardware::detect_encoders(&ffmpeg_exe)
        .into_iter()
        .find(|e| e.supported && e.id != "libx264")
        .map(|e| e.id)
        .unwrap_or_else(|| "libx264".to_string());

    let mut cmd = Command::new(&ffmpeg_exe);
    cmd.arg("-y").arg("-loglevel").arg("warning");

    if encoder_name == "h264_vaapi" {
        if let Some(dev) = hardware::pick_vaapi_device() {
            cmd.arg("-vaapi_device").arg(dev);
        }
    }

    cmd.arg("-thread_queue_size")
        .arg("2048")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("rgba")
        .arg("-r")
        .arg(fps.to_string())
        .arg("-s")
        .arg(format!("{}x{}", width, height))
        .arg("-i")
        .arg("pipe:0");

    let vf_filter = if encoder_name == "h264_vaapi" {
        "format=nv12,hwupload"
    } else {
        "scale=out_color_matrix=bt709:out_range=limited,format=yuv420p"
    };

    if include_audio {
        cmd.arg("-thread_queue_size")
            .arg("2048")
            .arg("-i")
            .arg(&audio_file_path)
            .arg("-map")
            .arg("0:v:0")
            .arg("-map")
            .arg("1:a:0?")
            .arg("-c:v")
            .arg(&encoder_name)
            .arg("-vf")
            .arg(vf_filter)
            .arg("-c:a")
            .arg("aac");
    } else {
        cmd.arg("-map")
            .arg("0:v:0")
            .arg("-c:v")
            .arg(&encoder_name)
            .arg("-vf")
            .arg(vf_filter)
            .arg("-an");
    }

    if encoder_name != "h264_vaapi" {
        let preset = if encoder_name == "libx264" {
            "ultrafast"
        } else {
            "fast"
        };
        cmd.arg("-preset").arg(preset);
    }

    cmd.arg(&output_mp4_path)
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

    // Read stderr progressively so error messages are available immediately
    // (e.g. when write_frame fails because FFmpeg already exited).
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

    // Reset smoothing state so the export starts with a clean slate
    if let Ok(mut prev) = state.prev_smoothed.lock() {
        *prev = None;
    }

    let mut guard = state.export_session.lock().map_err(|e| e.to_string())?;
    *guard = Some(ExportSession {
        child,
        stderr_buf,
        stderr_reader: Some(stderr_reader),
    });

    Ok(())
}

#[tauri::command]
fn write_frame_rgba(
    state: tauri::State<'_, AppState>,
    width: u32,
    height: u32,
    rgba_data: Vec<u8>,
) -> Result<(), String> {
    if (width * height * 4) as usize != rgba_data.len() {
        return Err(format!(
            "RGBA size mismatch: expected {} bytes, got {}",
            width * height * 4,
            rgba_data.len()
        ));
    }
    let mut guard = state.export_session.lock().map_err(|e| e.to_string())?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "No export session active".to_string())?;

    let stdin = session
        .child
        .stdin
        .as_mut()
        .ok_or_else(|| "FFmpeg stdin not available".to_string())?;

    match stdin.write_all(&rgba_data) {
        Ok(()) => Ok(()),
        Err(e) => {
            let err_msg = get_stderr_msg(session);
            Err(format!(
                "FFmpeg write failed: {}. FFmpeg stderr: {}",
                e, err_msg
            ))
        }
    }
}

#[tauri::command]
async fn finish_export_session(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut session = {
        let mut guard = state.export_session.lock().map_err(|e| e.to_string())?;
        guard
            .take()
            .ok_or_else(|| "No export session active".to_string())?
    };

    // Drop stdin to signal EOF to FFmpeg
    if let Some(stdin) = session.child.stdin.take() {
        drop(stdin);
    }

    // Wait for FFmpeg to finish
    let status = session
        .child
        .wait()
        .map_err(|e| format!("FFmpeg wait failed: {}", e))?;

    // Ensure all stderr has been drained before reading the buffer
    if let Some(reader) = session.stderr_reader.take() {
        let _ = reader.join();
    }

    if status.success() {
        Ok("Export completed successfully".to_string())
    } else {
        let err_msg = get_stderr_msg(&session);
        Err(format!(
            "FFmpeg error (exit {}): {}",
            status.code().unwrap_or(-1),
            err_msg
        ))
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
async fn read_file_b64(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
        Ok(general_purpose::STANDARD.encode(&bytes))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn save_pcm_to_file(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let audio = {
        let guard = state.audio_data.lock().map_err(|e| e.to_string())?;
        guard
            .as_ref()
            .ok_or_else(|| "No audio loaded".to_string())?
            .clone()
    };

    let dir = std::env::temp_dir().join("audiowave_pcm");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("pcm.raw");

    let bytes: Vec<u8> = audio
        .samples
        .iter()
        .flat_map(|&s| s.to_le_bytes())
        .collect();
    std::fs::write(&path, &bytes).map_err(|e| format!("Failed to write PCM file: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn read_file_range_b64(path: String, offset: usize, length: usize) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        use std::io::{Read, Seek};
        let mut file =
            std::fs::File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
        file.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(|e| format!("Failed to seek: {}", e))?;
        let mut buf = vec![0u8; length];
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("Failed to read: {}", e))?;
        buf.truncate(n);
        Ok(general_purpose::STANDARD.encode(&buf))
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
async fn delete_file(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        if std::path::Path::new(&path).exists() {
            std::fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))?;
        }
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
fn ffmpeg_auto_install_supported() -> bool {
    ffmpeg::auto_install_supported()
}

#[tauri::command]
async fn install_ffmpeg(app_handle: tauri::AppHandle) -> Result<String, String> {
    let handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        ffmpeg::install_ffmpeg(&handle, |phase| {
            let _ = handle.emit("ffmpeg-install-progress", phase);
        })
    })
    .await
    .map_err(|e| format!("FFmpeg install task panicked: {}", e))?
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

    let encoder_name = hardware::detect_encoders(&ffmpeg_exe)
        .into_iter()
        .find(|e| e.supported && e.id != "libx264")
        .map(|e| e.id)
        .unwrap_or_else(|| "libx264".to_string());

    let mut cmd = Command::new(&ffmpeg_exe);
    cmd.arg("-y").arg("-loglevel").arg("warning");

    if encoder_name == "h264_vaapi" {
        if let Some(dev) = hardware::pick_vaapi_device() {
            cmd.arg("-vaapi_device").arg(dev);
        }
    }

    cmd.arg("-thread_queue_size")
        .arg("2048")
        .arg("-i")
        .arg(&webm_path);

    let vf_filter = if encoder_name == "h264_vaapi" {
        "format=nv12,hwupload"
    } else {
        "scale=out_color_matrix=bt709:out_range=limited,format=yuv420p"
    };

    if include_audio {
        cmd.arg("-thread_queue_size")
            .arg("2048")
            .arg("-i")
            .arg(&audio_path)
            .arg("-map")
            .arg("0:v:0")
            .arg("-map")
            .arg("1:a:0?")
            .arg("-c:v")
            .arg(&encoder_name)
            .arg("-vf")
            .arg(vf_filter)
            .arg("-c:a")
            .arg("aac")
            .arg("-shortest");
    } else {
        cmd.arg("-map")
            .arg("0:v:0")
            .arg("-c:v")
            .arg(&encoder_name)
            .arg("-vf")
            .arg(vf_filter)
            .arg("-an");
    }

    if encoder_name != "h264_vaapi" {
        let preset = if encoder_name == "libx264" {
            "ultrafast"
        } else {
            "fast"
        };
        cmd.arg("-preset").arg(preset);
    }

    cmd.arg(&output_mp4_path);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

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
        let path = dir
            .join(format!("upload_{}", std::process::id()))
            .with_extension(&ext);
        std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
        Ok(path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn write_text_file(path: String, content: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::write(&path, &content).map_err(|e| format!("Failed to write file: {}", e))
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
            } else {
                None
            }
        })
        .collect();

    let src = monitors.first().ok_or("No sink found")?.clone();

    let mut child = std::process::Command::new("ffmpeg")
        .args([
            "-f", "pulse", "-i", &src, "-ac", "1", "-ar", "44100", "-f", "f32le", "-",
        ])
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
                Err(e) => {
                    eprintln!("[Listen] read err: {}", e);
                    break;
                }
            };

            let converted: Vec<f32> = raw[..n]
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            samples.extend(converted);

            while samples.len() >= fft_size {
                let (spectrum, bass_raw) =
                    match analyzer.compute_spectrum(&samples[..fft_size], 256) {
                        Ok(r) => r,
                        Err(_) => {
                            samples.drain(0..fft_size / 2);
                            continue;
                        }
                    };

                let freq_data: Vec<u8> = spectrum
                    .iter()
                    .map(|&mag| {
                        ((20.0 * (mag.max(1e-10)).log10() + 100.0) / 70.0 * 255.0)
                            .clamp(0.0, 255.0)
                            .round() as u8
                    })
                    .collect();

                let time_data: Vec<u8> = samples[..fft_size]
                    .iter()
                    .map(|&s| ((s * 127.0 + 128.0).round() as i16).clamp(0, 255) as u8)
                    .collect();

                samples.drain(0..fft_size / 2);

                let _ = handle.emit(
                    "listen-freq-data",
                    serde_json::json!({
                      "freq_data": freq_data,
                      "time_data": time_data,
                      "bass_energy": (bass_raw * 4.0).min(1.0),
                    }),
                );
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

#[tauri::command]
fn open_detached_preview_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("detached-preview") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let url = tauri::WebviewUrl::App("index.html?detached=true".into());
    let window = tauri::WebviewWindowBuilder::new(&app, "detached-preview", url)
        .title("AudioWave Studio - Live Preview")
        .inner_size(1280.0, 720.0)
        .resizable(true)
        .decorations(false)
        .visible(true)
        .build()
        .map_err(|e| e.to_string())?;

    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
fn toggle_detached_fullscreen(window: tauri::WebviewWindow) -> Result<bool, String> {
    let current = window.is_fullscreen().unwrap_or(false);
    let target = !current;
    let _ = window.set_fullscreen(target);
    if !target {
        let _ = window.set_focus();
    }
    Ok(target)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("audiowave_pcm"));
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            audio_data: Mutex::new(None),
            export_session: Mutex::new(None),
            listen_session: Mutex::new(None),
            prev_smoothed: Mutex::new(None),
            gpu_cancel: Mutex::new(None),
            preview_gpu: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            decode_audio,
            decode_audio_playback,
            get_audio_chunk_b64,
            read_file_bytes,
            read_file_b64,
            save_pcm_to_file,
            read_file_range_b64,
            write_text_file,
            copy_file_to_path,
            delete_file,
            check_ffmpeg,
            ffmpeg_auto_install_supported,
            install_ffmpeg,
            ffmpeg_download_url,
            save_upload_to_temp,
            compute_spectrum_rust,
            precompute_spectra,
            start_export_session,
            write_frame_rgba,
            finish_export_session,
            gpu_export::export_gpu,
            gpu_export::cancel_gpu_export,
            gpu_export::render_rust_preview_frame,
            convert_webm_to_mp4,
            start_system_listen,
            stop_system_listen,
            hardware::check_hardware,
            hardware::get_system_memory_cmd,
            open_detached_preview_window,
            toggle_detached_fullscreen
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
