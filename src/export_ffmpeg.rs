use crate::config::{AspectRatio, ExportResolution, ExportSettings, VisualizerConfig};
use crate::ffmpeg::resolve_ffmpeg;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub fn export_dimensions(config: &VisualizerConfig) -> (u32, u32) {
  let ExportSettings { resolution, aspect_ratio, .. } = &config.export;
  let (mut width, mut height) = match resolution {
    ExportResolution::P1080 => (1920u32, 1080u32),
    ExportResolution::P720 => (1280, 720),
    ExportResolution::K4 => (3840, 2160),
  };
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

pub fn spawn_ffmpeg(
  app_data_dir: Option<&Path>,
  fps: u32,
  width: u32,
  height: u32,
  output_mp4_path: &str,
  audio_file_path: &str,
  include_audio: bool,
  encoder_preference: &str,
) -> Result<(Child, Arc<Mutex<String>>, JoinHandle<()>), String> {
  let ffmpeg_exe = resolve_ffmpeg(app_data_dir)?;

  let mut encoder_name = crate::hardware::pick_encoder(&ffmpeg_exe, encoder_preference);
  let is_webm = output_mp4_path.to_lowercase().ends_with(".webm");
  if is_webm && (encoder_name.contains("264") || encoder_name.contains("hevc") || encoder_name.contains("h265")) {
    encoder_name = "libvpx-vp9".to_string();
  }

  let mut cmd = Command::new(&ffmpeg_exe);
  #[cfg(target_os = "windows")]
  {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
  }
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

  let mut has_audio = false;
  if include_audio && !audio_file_path.trim().is_empty() {
    let p = Path::new(audio_file_path);
    if p.is_file() {
      cmd.arg("-thread_queue_size").arg("2048")
        .arg("-i").arg(audio_file_path);
      has_audio = true;
    }
  }

  if encoder_name.ends_with("_vaapi") {
    cmd.arg("-vf").arg("format=nv12,hwupload");
  } else {
    cmd.arg("-pix_fmt").arg("yuv420p");
  }

  cmd.arg("-c:v").arg(&encoder_name);
  let cap_kbps = match (width, height) {
    (w, h) if w >= 3800 || h >= 2100 => 35_000,
    (w, h) if w >= 1900 || h >= 1000 => 12_000,
    _ => 6_000,
  };
  apply_encoder_args(&mut cmd, &encoder_name, cap_kbps);

  if has_audio {
    if output_mp4_path.ends_with(".webm") {
      // AAC is not valid in a WebM container; use Opus instead.
      cmd.arg("-c:a").arg("libopus").arg("-b:a").arg("160k").arg("-shortest");
    } else {
      cmd.arg("-c:a").arg("aac").arg("-b:a").arg("192k").arg("-shortest");
    }
  }

  cmd.arg("-movflags").arg("+faststart");
  cmd.arg(output_mp4_path);

  cmd.stdin(Stdio::piped()).stderr(Stdio::piped()).stdout(Stdio::null());

  let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn FFmpeg: {}", e))?;
  let stderr = child.stderr.take().ok_or("Failed to capture FFmpeg stderr")?;
  let err_log = Arc::new(Mutex::new(String::new()));
  let log_clone = err_log.clone();

  let stderr_thread = std::thread::spawn(move || {
    use std::io::BufRead;
    let reader = std::io::BufReader::new(stderr);
    for line in reader.lines().map_while(Result::ok) {
      if let Ok(mut l) = log_clone.lock() {
        if l.len() < 4000 {
          l.push_str(&line);
          l.push('\n');
        }
      }
    }
  });

  Ok((child, err_log, stderr_thread))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{AspectRatio, ExportResolution};

  fn config_with(resolution: ExportResolution, aspect_ratio: AspectRatio) -> VisualizerConfig {
    let mut c = VisualizerConfig::default();
    c.export.resolution = resolution;
    c.export.aspect_ratio = aspect_ratio;
    c
  }

  #[test]
  fn export_dimensions_widescreen() {
    assert_eq!(
      export_dimensions(&config_with(ExportResolution::P1080, AspectRatio::Widescreen)),
      (1920, 1080)
    );
    assert_eq!(
      export_dimensions(&config_with(ExportResolution::P720, AspectRatio::Widescreen)),
      (1280, 720)
    );
    assert_eq!(
      export_dimensions(&config_with(ExportResolution::K4, AspectRatio::Widescreen)),
      (3840, 2160)
    );
  }

  #[test]
  fn export_dimensions_portrait_swaps() {
    assert_eq!(
      export_dimensions(&config_with(ExportResolution::P1080, AspectRatio::Portrait)),
      (1080, 1920)
    );
    assert_eq!(
      export_dimensions(&config_with(ExportResolution::K4, AspectRatio::Portrait)),
      (2160, 3840)
    );
  }

  #[test]
  fn export_dimensions_square() {
    // Square reuses the width as the side length (1280 from 720p, 1920 from 1080p, 3840 from 4K).
    assert_eq!(
      export_dimensions(&config_with(ExportResolution::P720, AspectRatio::Square)),
      (1280, 1280)
    );
    assert_eq!(
      export_dimensions(&config_with(ExportResolution::P1080, AspectRatio::Square)),
      (1920, 1920)
    );
    assert_eq!(
      export_dimensions(&config_with(ExportResolution::K4, AspectRatio::Square)),
      (3840, 3840)
    );
  }

  #[test]
  fn export_dimensions_default_is_720p_widescreen() {
    assert_eq!(export_dimensions(&VisualizerConfig::default()), (1280, 720));
  }
}

