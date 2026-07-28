use crate::audio_decoder::AudioData;
use crate::fft_analyzer::FftAnalyzer;
use crate::gpu_renderer::GpuRenderer;
use crate::renderer::{RenderConfig, RustRenderer};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportProgressRust {
  pub percent: f32,
  pub current_frame: usize,
  pub total_frames: usize,
  pub is_finished: bool,
}

pub struct VideoEncoderRust;

impl VideoEncoderRust {
  pub fn export_to_mp4<P: AsRef<Path>>(
    audio: &AudioData,
    audio_file_path: &str,
    config: &RenderConfig,
    output_mp4_path: P,
    fps: u32,
    ffmpeg_exe: &str,
    include_audio: bool,
    on_progress: impl Fn(&ExportProgressRust),
  ) -> Result<String, String> {
    let total_frames = (audio.duration_seconds * fps as f64).ceil() as usize;
    let ffmpeg_path = output_mp4_path.as_ref();

    let mut cmd = Command::new(ffmpeg_exe);
    cmd.arg("-y")
      .arg("-loglevel").arg("error")
      .arg("-f").arg("rawvideo")
      .arg("-pix_fmt").arg("rgba")
      .arg("-s").arg(format!("{}x{}", config.width, config.height))
      .arg("-r").arg(fps.to_string())
      .arg("-i").arg("-");

    if include_audio {
      cmd.arg("-i").arg(audio_file_path)
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
      .arg(ffmpeg_path)
      .stdin(Stdio::piped())
      .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
      if e.kind() == std::io::ErrorKind::NotFound {
        format!("FFmpeg not found at '{}'.", ffmpeg_exe)
      } else {
        format!("Failed to start ffmpeg: {}", e)
      }
    })?;

    let stdin = child.stdin.take().ok_or_else(|| "No ffmpeg stdin".to_string())?;
    let mut stderr = child.stderr.take().ok_or_else(|| "No ffmpeg stderr".to_string())?;

    let stderr_handle = thread::spawn(move || {
      let mut buf = String::new();
      let _ = stderr.read_to_string(&mut buf);
      buf
    });

    let gpu = pollster::block_on(GpuRenderer::new(config.width, config.height));

    match gpu {
      Ok(gpu_renderer) => {
        eprintln!("[export] Using GPU renderer");
        let gpu_renderer = gpu_renderer;
        let mut stdin = stdin;
        let analyzer = FftAnalyzer::new(1024);
        let mut last_pct = -1i32;

        for frame_idx in 0..total_frames {
          let time_sec = frame_idx as f64 / fps as f64;
          let samples = audio.get_sample_window(time_sec, 1024);
          let (spectrum, bass_energy) = analyzer.compute_spectrum(&samples, config.bar_count)
            .map_err(|e| format!("FFT error: {}", e))?;
          let pixels = gpu_renderer.render_frame(config, &spectrum, &samples, bass_energy, time_sec as f32);

          if stdin.write_all(&pixels).is_err() {
            break;
          }

          let pct = ((frame_idx + 1) * 100 / total_frames) as i32;
          if pct != last_pct {
            last_pct = pct;
            on_progress(&ExportProgressRust {
              percent: pct as f32,
              current_frame: frame_idx + 1,
              total_frames,
              is_finished: frame_idx + 1 >= total_frames,
            });
          }
        }

        drop(stdin);
        drop(gpu_renderer);
      }
      Err(e) => {
        eprintln!("[export] GPU unavailable ({}), using CPU renderer", e);
        let config_clone = config.clone();
        let audio_clone = audio.clone();
        let (frame_tx, frame_rx) = mpsc::sync_channel::<Vec<u8>>(64);

        let writer_handle = thread::spawn(move || {
          let mut stdin = stdin;
          for frame in frame_rx {
            if stdin.write_all(&frame).is_err() {
              break;
            }
          }
        });

        let render_handle = thread::spawn(move || {
          let analyzer = FftAnalyzer::new(1024);
          let mut renderer = RustRenderer::new();
          for frame_idx in 0..total_frames {
            let time_sec = frame_idx as f64 / fps as f64;
            let samples = audio_clone.get_sample_window(time_sec, 1024);
            let (spectrum, bass_energy) = analyzer.compute_spectrum(&samples, config_clone.bar_count)
              .unwrap_or_else(|_| (vec![0.0; config_clone.bar_count], 0.0));
            let img = renderer.render_frame(&config_clone, &spectrum, &samples, bass_energy);
            if frame_tx.send(img.into_raw()).is_err() {
              break;
            }
          }
        });

        let mut last_pct = -1i32;
        for frame_idx in 0..total_frames {
          let pct = ((frame_idx + 1) * 100 / total_frames) as i32;
          if pct != last_pct {
            last_pct = pct;
            on_progress(&ExportProgressRust {
              percent: pct as f32,
              current_frame: frame_idx + 1,
              total_frames,
              is_finished: frame_idx + 1 >= total_frames,
            });
          }
        }

        let _ = render_handle.join();
        let _ = writer_handle.join();
      }
    }

    let stderr_buf = stderr_handle.join().unwrap_or_default();
    let status = child.wait().map_err(|e| e.to_string())?;

    if status.success() {
      Ok(format!(
        "Successfully exported MP4 video to {}",
        ffmpeg_path.display()
      ))
    } else {
      let msg = if stderr_buf.is_empty() {
        "FFmpeg exited with an error".to_string()
      } else {
        format!("FFmpeg error:\n{}", stderr_buf.trim())
      };
      Err(msg)
    }
  }
}
