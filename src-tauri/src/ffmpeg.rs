use std::process::Command;
use tauri::Manager;

pub fn resolve_ffmpeg(app_handle: &tauri::AppHandle) -> Result<String, String> {
  // 1. Check PATH (system-installed ffmpeg)
  if is_in_path("ffmpeg") {
    return Ok("ffmpeg".to_string());
  }

  // 2. Check app data dir (previously auto-installed)
  if let Ok(data_dir) = app_handle.path().app_data_dir() {
    let candidates = [
      data_dir.join("ffmpeg").join("ffmpeg"),
      data_dir.join("ffmpeg").join("ffmpeg.exe"),
      data_dir.join("bin").join("ffmpeg"),
      data_dir.join("bin").join("ffmpeg.exe"),
    ];
    for c in &candidates {
      if c.exists() {
        return Ok(c.to_string_lossy().to_string());
      }
    }
  }

  Err(format!(
    "FFmpeg not found. Install it via package manager or download from:\n  {}",
    download_url()
  ))
}

pub fn download_url() -> &'static str {
  if cfg!(target_os = "windows") {
    "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
  } else if cfg!(target_os = "macos") {
    "https://evermeet.cx/ffmpeg/ffmpeg-7.1.zip"
  } else {
    "https://johnvansickle.com/ffmpeg/"
  }
}

fn is_in_path(name: &str) -> bool {
  let cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
  Command::new(cmd).arg(name).output().is_ok()
}
