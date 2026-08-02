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

pub fn auto_install_supported() -> bool {
  cfg!(target_os = "windows") || cfg!(target_os = "macos")
}

pub fn find_ffmpeg_binary(dir: &std::path::Path) -> Option<std::path::PathBuf> {
  let exe_name = if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" };
  let mut stack = vec![dir.to_path_buf()];
  while let Some(cur) = stack.pop() {
    if let Ok(entries) = std::fs::read_dir(&cur) {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
          stack.push(path);
        } else if path
          .file_name()
          .map(|n| n.to_string_lossy().eq_ignore_ascii_case(exe_name))
          .unwrap_or(false)
        {
          return Some(path);
        }
      }
    }
  }
  None
}

/// Downloads and installs FFmpeg into the app data dir using built-in OS tools
/// (curl on Windows/macOS, Expand-Archive/ditto for extraction). Returns the path
/// to the installed ffmpeg executable.
pub fn install_ffmpeg(
  app_handle: &tauri::AppHandle,
  on_progress: impl Fn(&str),
) -> Result<String, String> {
  if !auto_install_supported() {
    return Err("Auto-install is only supported on Windows and macOS. Install FFmpeg via your package manager.".to_string());
  }

  let data_dir = app_handle
    .path()
    .app_data_dir()
    .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
  let ffmpeg_dir = data_dir.join("ffmpeg");
  std::fs::create_dir_all(&ffmpeg_dir).map_err(|e| format!("Failed to create dir: {}", e))?;

  let tmp_dir = std::env::temp_dir().join("audiowave_ffmpeg_dl");
  let _ = std::fs::remove_dir_all(&tmp_dir);
  std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

  let zip_path = tmp_dir.join("ffmpeg.zip");
  let url = download_url();

  on_progress("downloading");
  let curl = if cfg!(target_os = "windows") { "curl.exe" } else { "curl" };
  let dl_output = Command::new(curl)
    .args(["-L", "-o"])
    .arg(&zip_path)
    .arg(url)
    .output()
    .map_err(|e| format!("Failed to run curl: {}", e))?;
  if !dl_output.status.success() {
    return Err(format!(
      "FFmpeg download failed (exit {}). You can still download it manually from:\n  {}",
      dl_output.status.code().unwrap_or(-1),
      url
    ));
  }

  on_progress("extracting");
  let extract_dir = tmp_dir.join("extract");
  std::fs::create_dir_all(&extract_dir).map_err(|e| format!("Failed to create extract dir: {}", e))?;

  let extract_status = if cfg!(target_os = "windows") {
    Command::new("powershell")
      .args([
        "-NoProfile",
        "-Command",
        &format!(
          "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
          zip_path.to_string_lossy(),
          extract_dir.to_string_lossy()
        ),
      ])
      .output()
  } else {
    Command::new("ditto")
      .args(["-x", "-k"])
      .arg(&zip_path)
      .arg(&extract_dir)
      .output()
  }
  .map_err(|e| format!("Failed to extract FFmpeg archive: {}", e))?;

  if !extract_status.status.success() {
    return Err(format!(
      "FFmpeg extraction failed (exit {}).",
      extract_status.status.code().unwrap_or(-1)
    ));
  }

  on_progress("installing");
  let exe = find_ffmpeg_binary(&extract_dir)
    .ok_or_else(|| "FFmpeg archive did not contain ffmpeg executable".to_string())?;

  let target_exe = ffmpeg_dir.join(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" });

  if cfg!(target_os = "windows") {
    // Windows builds are DLL-based: copy the whole bin folder so ffmpeg.exe finds its DLLs.
    if let Some(bin_dir) = exe.parent() {
      let entries = std::fs::read_dir(bin_dir).map_err(|e| format!("Failed to read bin dir: {}", e))?;
      for entry in entries.flatten() {
        let src = entry.path();
        if src.is_file() {
          let dest = ffmpeg_dir.join(entry.file_name());
          std::fs::copy(&src, &dest).map_err(|e| format!("Failed to copy {}: {}", src.display(), e))?;
        }
      }
    }
  } else {
    std::fs::copy(&exe, &target_exe).map_err(|e| format!("Failed to copy ffmpeg: {}", e))?;
    #[cfg(target_os = "macos")]
    {
      use std::os::unix::fs::PermissionsExt;
      let _ = std::fs::set_permissions(&target_exe, std::fs::Permissions::from_mode(0o755));
    }
  }

  let _ = std::fs::remove_dir_all(&tmp_dir);

  if !target_exe.exists() {
    return Err("FFmpeg install failed: executable not found after install".to_string());
  }

  on_progress("done");
  Ok(target_exe.to_string_lossy().to_string())
}

pub fn is_in_path(name: &str) -> bool {
  let cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
  Command::new(cmd)
    .arg(name)
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}
