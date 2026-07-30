use crate::audio_decoder::AudioData;
use crate::fft_analyzer::FftAnalyzer;
use crate::gpu_renderer::GpuRenderer;
use crate::renderer::{RenderConfig, RustRenderer};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
      .arg("-loglevel").arg("warning")
      .arg("-thread_queue_size").arg("2048")
      .arg("-f").arg("rawvideo")
      .arg("-pix_fmt").arg("rgba")
      .arg("-s").arg(format!("{}x{}", config.width, config.height))
      .arg("-r").arg(fps.to_string())
      .arg("-i").arg("-");

    if include_audio {
      cmd.arg("-thread_queue_size").arg("2048")
        .arg("-i").arg(audio_file_path)
        .arg("-map").arg("0:v:0")
        .arg("-map").arg("1:a:0?")
        .arg("-c:v").arg("libx264")
        .arg("-vf").arg("scale=out_color_matrix=bt709:out_range=limited,format=yuv420p")
        .arg("-c:a").arg("aac")
        .arg("-shortest");
    } else {
      cmd.arg("-map").arg("0:v:0")
        .arg("-c:v").arg("libx264")
        .arg("-vf").arg("scale=out_color_matrix=bt709:out_range=limited,format=yuv420p")
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
        let rendered = Arc::new(AtomicUsize::new(0));
        let rendered_clone = rendered.clone();

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
            rendered_clone.store(frame_idx + 1, Ordering::SeqCst);
          }
        });

        let mut last_pct = -1i32;
        loop {
          let done = rendered.load(Ordering::SeqCst);
          if done >= total_frames {
            break;
          }
          let pct = (done * 100 / total_frames) as i32;
          if pct != last_pct {
            last_pct = pct;
            on_progress(&ExportProgressRust {
              percent: pct as f32,
              current_frame: done,
              total_frames,
              is_finished: false,
            });
          }
          thread::sleep(Duration::from_millis(100));
        }

        on_progress(&ExportProgressRust {
          percent: 100.0,
          current_frame: total_frames,
          total_frames,
          is_finished: true,
        });

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn export_progress_serialization() {
    let p = ExportProgressRust {
      percent: 50.0,
      current_frame: 100,
      total_frames: 200,
      is_finished: false,
    };
    let json = serde_json::to_string(&p).unwrap();
    assert!(json.contains("\"percent\":50.0"));
    assert!(json.contains("\"current_frame\":100"));
    assert!(json.contains("\"total_frames\":200"));
    assert!(json.contains("\"is_finished\":false"));
  }

  #[test]
  fn export_progress_finished() {
    let p = ExportProgressRust {
      percent: 100.0,
      current_frame: 200,
      total_frames: 200,
      is_finished: true,
    };
    assert!(p.is_finished);
    assert_eq!(p.percent, 100.0);
    assert_eq!(p.current_frame, p.total_frames);
  }

  #[test]
  fn export_progress_deserialization() {
    let json = r#"{"percent":75.0,"current_frame":150,"total_frames":200,"is_finished":false}"#;
    let p: ExportProgressRust = serde_json::from_str(json).unwrap();
    assert_eq!(p.percent, 75.0);
    assert_eq!(p.current_frame, 150);
    assert_eq!(p.total_frames, 200);
    assert!(!p.is_finished);
  }
}
