use crate::ffmpeg::resolve_ffmpeg;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GpuAdapterInfo {
  pub name: String,
  pub device_type: String,
  pub backend: String,
  pub vendor_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncoderCapability {
  pub id: String,
  pub name: String,
  pub supported: bool,
  pub description: String,
  /// Why the encoder is (not) usable, surfaced in the Hardware modal:
  /// "works", "not compiled into this FFmpeg build", or the ffmpeg stderr
  /// tail from the probe (driver/session error). Never silently discarded.
  pub detail: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemMemoryInfo {
  pub used_mb: u64,
  pub total_mb: u64,
  pub used_percent: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HardwareInfo {
  pub gpus: Vec<GpuAdapterInfo>,
  pub ffmpeg_installed: bool,
  pub ffmpeg_path: Option<String>,
  pub encoders: Vec<EncoderCapability>,
  pub recommended_encoder: String,
  pub recommended_label: String,
  pub memory: Option<SystemMemoryInfo>,
  pub os: String,
  pub arch: String,
}

pub fn get_gpu_adapters() -> Vec<GpuAdapterInfo> {
  let instance = wgpu::Instance::default();
  let adapters = instance.enumerate_adapters(wgpu::Backends::all());
  let mut gpu_list = Vec::new();

  for adapter in adapters {
    let info = adapter.get_info();
    let dev_type = match info.device_type {
      wgpu::DeviceType::IntegratedGpu => "IntegratedGpu",
      wgpu::DeviceType::DiscreteGpu => "DiscreteGpu",
      wgpu::DeviceType::Cpu => "Cpu",
      wgpu::DeviceType::VirtualGpu => "VirtualGpu",
      wgpu::DeviceType::Other => "Other",
    };
    let backend = match info.backend {
      wgpu::Backend::Vulkan => "Vulkan",
      wgpu::Backend::Metal => "Metal",
      wgpu::Backend::Dx12 => "DirectX 12",
      wgpu::Backend::Gl => "OpenGL",
      wgpu::Backend::BrowserWebGpu => "WebGPU",
      wgpu::Backend::Empty => "None",
    };

    gpu_list.push(GpuAdapterInfo {
      name: info.name,
      device_type: dev_type.to_string(),
      backend: backend.to_string(),
      vendor_id: info.vendor,
    });
  }

  gpu_list
}

/// Pick the first VAAPI render device that actually exists. Some systems expose
/// card0/renderD129 instead of card1/renderD128, so fall back across the common
/// device nodes rather than assuming a single hardcoded path.
pub fn pick_vaapi_device() -> Option<&'static str> {
  const CANDIDATES: [&str; 6] = [
    "/dev/dri/renderD128",
    "/dev/dri/renderD129",
    "/dev/dri/card0",
    "/dev/dri/card1",
    "/dev/dri/renderD127",
    "/dev/dri/renderD130",
  ];
  CANDIDATES
    .iter()
    .copied()
    .find(|dev| std::path::Path::new(dev).exists())
}

/// Which VAAPI rate-control mode THIS machine's driver actually supports:
/// Some("VBR") or Some("CQP"), or None when the driver cannot encode at all.
/// VBR is preferred because it honors -b:v/-maxrate; a CQP-only driver (e.g.
/// the older i965 Intel driver) ignores bitrate caps, so a long export would
/// balloon to ~100 Mbps unless the export switches to -global_quality.
/// Cached once per process — the driver cannot change while the app runs.
pub fn vaapi_rc_mode(ffmpeg_exe: &str) -> Option<&'static str> {
  static MODE: OnceLock<Option<&'static str>> = OnceLock::new();
  *MODE
    .get_or_init(|| {
      let Some(dev) = pick_vaapi_device() else { return None; };
      let variants: [(&str, &[&str]); 2] = [
        ("VBR", &["-b:v", "1000k"]),
        ("CQP", &["-global_quality", "23"]),
      ];
      for (mode, extra) in variants {
        let mut cmd = Command::new(ffmpeg_exe);
        #[cfg(target_os = "windows")]
        {
          use std::os::windows::process::CommandExt;
          cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        cmd.args([
          "-hide_banner",
          "-vaapi_device",
          dev,
          "-f",
          "lavfi",
          "-i",
          "color=c=black:s=256x256:d=0.1",
          "-vf",
          "format=nv12,hwupload",
          "-c:v",
          "h264_vaapi",
          "-rc_mode",
          mode,
        ]);
        cmd.args(extra);
        cmd.args(["-f", "null", "-"]);
        let ok = run_with_timeout(&mut cmd, ENCODER_PROBE_TIMEOUT)
          .map(|o| o.status.success())
          .unwrap_or(false);
        if ok {
          return Some(mode);
        }
      }
      None
    })
}

/// Max time a single HW-encoder probe may take before it is killed. Probes
/// run on the UI thread during the startup/rescan hardware scan, so a hung
/// nvenc/qsv/amf session must never freeze the app.
const ENCODER_PROBE_TIMEOUT: Duration = Duration::from_secs(8);

/// Spawn `cmd`, capture stdout/stderr, and wait up to `timeout`, killing the
/// process if it overruns (a stuck GPU session must not hang the app).
fn run_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<std::process::Output, String> {
  let mut child = cmd
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|e| format!("spawn failed: {e}"))?;
  let start = Instant::now();
  loop {
    match child.try_wait() {
      Ok(Some(_)) => break,
      Ok(None) => {
        if start.elapsed() > timeout {
          let _ = child.kill();
          let _ = child.wait();
          return Err("probe timed out (killed)".to_string());
        }
        std::thread::sleep(Duration::from_millis(40));
      }
      Err(e) => return Err(format!("wait failed: {e}")),
    }
  }
  let mut stdout = Vec::new();
  let mut stderr = Vec::new();
  if let Some(mut so) = child.stdout.take() {
    let _ = so.read_to_end(&mut stdout);
  }
  if let Some(mut se) = child.stderr.take() {
    let _ = se.read_to_end(&mut stderr);
  }
  let status = child.wait().map_err(|e| format!("reap failed: {e}"))?;
  Ok(std::process::Output { status, stdout, stderr })
}

/// ffmpeg's end-of-run noise lines that hide the REAL cause of a failed probe
/// ("Nothing was written into output file", the frame/size stats, the
/// "Conversion failed!" summary). Filtering them out lets the actual error —
/// e.g. "Driver does not support VBR RC mode (supported modes: CQP)" — reach
/// the Hardware modal.
fn is_ffmpeg_noise(l: &str) -> bool {
  l.starts_with("Stream #")
    || l.starts_with("Press [q]")
    || l.starts_with("Metadata:")
    || l.starts_with("Input #")
    || l.starts_with("Output #")
    || l.starts_with("[out#")
    || l.contains("Nothing was written into output file")
    || l.contains("Conversion failed!")
    || l.contains("frame=")
    || l.contains("Lsize=")
    || l.contains("muxing overhead")
    || l.contains("video:")
    || l.contains("audio:")
}

/// Short readable tail of ffmpeg stderr for surfacing a probe failure.
fn stderr_tail(bytes: &[u8]) -> String {
  let text = String::from_utf8_lossy(bytes);
  let t = text.trim();
  if t.is_empty() {
    return String::new();
  }
  let useful: Vec<&str> = t
    .lines()
    .map(|l| l.trim_end_matches('\r').trim())
    .filter(|l| !l.is_empty() && !is_ffmpeg_noise(l))
    .collect();
  if useful.is_empty() {
    return "encode failed (no detail in stderr)".to_string();
  }
  let take = useful.len().min(3);
  let joined = useful[useful.len() - take..].join(" | ");
  let chars: Vec<char> = joined.chars().collect();
  if chars.len() > 240 {
    let s: String = chars[chars.len() - 240..].iter().collect();
    format!("…{s}")
  } else {
    joined
  }
}

/// `ffmpeg -hide_banner -encoders` output, cached once per process. The build
/// does not change while the app runs, so this is a fast, no-GPU check that
/// distinguishes "encoder missing from the build" from "encoder present but
/// not usable on this machine".
fn cached_encoder_list(ffmpeg_exe: &str) -> Option<String> {
  static LIST: OnceLock<Option<String>> = OnceLock::new();
  LIST
    .get_or_init(|| {
      let mut cmd = Command::new(ffmpeg_exe);
      #[cfg(target_os = "windows")]
      {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
      }
      match run_with_timeout(&mut cmd.args(["-hide_banner", "-encoders"]), ENCODER_PROBE_TIMEOUT) {
        Ok(o) if o.status.success() => Some(String::from_utf8_lossy(&o.stdout).to_string()),
        _ => None,
      }
    })
    .clone()
}

/// Does an `ffmpeg -encoders` listing contain `encoder_id`? (Each line looks
/// like " V..... h264_nvenc  NVIDIA NVENC H.264 encoder ..." — the encoder id
/// is the second token.)
fn encoders_list_contains(list: &str, encoder_id: &str) -> bool {
  list.lines().any(|l| l.split_whitespace().nth(1) == Some(encoder_id))
}

/// Does this FFmpeg BUILD contain `encoder_id`?
fn encoder_in_build(ffmpeg_exe: &str, encoder_id: &str) -> bool {
  cached_encoder_list(ffmpeg_exe)
    .map(|list| encoders_list_contains(&list, encoder_id))
    .unwrap_or(false)
}

/// Probe whether `encoder_id` is actually usable on THIS machine (GPU + driver
/// present, session can init). Two stages: the build check above, then a real
/// short encode. Returns (usable, detail) — `detail` explains a negative so
/// the Hardware modal can show WHY instead of a silent "✗".
pub fn probe_encoder(ffmpeg_exe: &str, encoder_id: &str) -> (bool, String) {
  if encoder_id == "libx264" {
    return (true, "always available (CPU)".to_string());
  }

  if !encoder_in_build(ffmpeg_exe, encoder_id) {
    return (false, "not compiled into this FFmpeg build".to_string());
  }

  // VAAPI drivers vary wildly: some only expose CQP rate control, which
  // silently ignores -maxrate/-bufsize (a 1080p export then balloons to
  // ~100 Mbps). Probe with -rc_mode VBR so only drivers that can actually
  // honor a bitrate cap are considered usable.
  let mut cmd = Command::new(ffmpeg_exe);
  #[cfg(target_os = "windows")]
  {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
  }

  // NOTE: the test frame must be >=256x256. NVENC on Pascal-generation GPUs
  // requires a minimum of 128x128 — the old 64x64 probe made h264_nvenc
  // report "unusable" on perfectly healthy Pascal cards.
  if encoder_id.ends_with("_vaapi") {
    let Some(dev) = pick_vaapi_device() else {
      return (false, "no VAAPI device found".to_string());
    };
    // Probe with the rate-control mode THIS driver supports (VBR preferred,
    // CQP fallback) so the probe verdict and the export command always agree.
    let Some(mode) = vaapi_rc_mode(ffmpeg_exe) else {
      return (false, "VAAPI driver cannot encode (VBR and CQP both failed)".to_string());
    };
    cmd.args([
      "-hide_banner",
      "-vaapi_device",
      dev,
      "-f",
      "lavfi",
      "-i",
      "color=c=black:s=256x256:d=0.1",
      "-vf",
      "format=nv12,hwupload",
      "-c:v",
      encoder_id,
    ]);
    if mode == "VBR" {
      cmd.args(["-rc_mode", "VBR", "-b:v", "1000k"]);
    } else {
      cmd.args(["-rc_mode", "CQP", "-global_quality", "23"]);
    }
    cmd.args(["-f", "null", "-"]);
    return match run_with_timeout(&mut cmd, ENCODER_PROBE_TIMEOUT) {
      Ok(o) if o.status.success() => (
        true,
        if mode == "VBR" {
          "works (VBR)".to_string()
        } else {
          "works (CQP-only driver — bitrate cap not enforced)".to_string()
        },
      ),
      Ok(o) => (false, stderr_tail(&o.stderr)),
      Err(e) => (false, e),
    };
  } else {
    cmd.args([
      "-hide_banner",
      "-f",
      "lavfi",
      "-i",
      "color=c=black:s=256x256:d=0.1",
      "-c:v",
      encoder_id,
      "-f",
      "null",
      "-",
    ]);
  }

  match run_with_timeout(&mut cmd, ENCODER_PROBE_TIMEOUT) {
    Ok(o) if o.status.success() => (true, "works".to_string()),
    Ok(o) => (false, stderr_tail(&o.stderr)),
    Err(e) => (false, e),
  }
}

pub fn test_encoder(ffmpeg_exe: &str, encoder_id: &str) -> bool {
  probe_encoder(ffmpeg_exe, encoder_id).0
}

pub fn detect_encoders(ffmpeg_exe: &str) -> Vec<EncoderCapability> {
  let candidates = vec![
    ("h264_nvenc", "NVIDIA NVENC H.264", "NVIDIA GPU hardware acceleration (GeForce/RTX)"),
    ("h264_qsv", "Intel QuickSync (QSV)", "Intel GPU hardware acceleration (iGPU & Arc)"),
    ("h264_vaapi", "Linux VAAPI H.264", "Linux hardware-accelerated driver (Intel/AMD)"),
    ("h264_amf", "AMD AMF H.264", "AMD Radeon GPU hardware acceleration"),
    ("h264_videotoolbox", "Apple VideoToolbox", "Apple hardware acceleration (M1/M2/M3 & Intel)"),
    ("libx264", "Software x264 (CPU)", "Standard CPU-based software encoder"),
  ];

  let mut list = Vec::new();
  for (id, name, desc) in candidates {
    let (supported, detail) = probe_encoder(ffmpeg_exe, id);
    list.push(EncoderCapability {
      id: id.to_string(),
      name: name.to_string(),
      supported,
      description: desc.to_string(),
      detail,
    });
  }

  list
}

#[cfg(test)]
mod encoder_detection_tests {
  use super::*;

  #[test]
  fn parses_realistic_encoders_listing() {
    let list = "Encoders:
 V..... = Video
 A..... = Audio
 S..... = Subtitle
 .F.... = Frame-level multithreading
 ..S... = Slice-level multithreading
 ...X.. = Codec is experimental
 ....B. = Supports draw_horiz_band
 .....D = Supports direct rendering method 1
 ------
 V..... libx264              libx264 H.264 / AVC / MPEG-4 AVC / MPEG-4 part 10 (codec h264)
 V....D h264_nvenc           NVIDIA NVENC H.264 encoder (codec h264)
 V..... h264_qsv             H.264 / AVC / MPEG-4 AVC / MPEG-4 part 10 (Intel Quick Sync Video acceleration) (codec h264)
 V....D h264_amf             AMD AMF H.264 encoder (codec h264)
 V..... h264_videotoolbox    VideoToolbox H.264 Encoder (codec h264)
 V....D h264_vaapi           H.264 VAAPI encoder (codec h264)";

    assert!(encoders_list_contains(list, "h264_nvenc"));
    assert!(encoders_list_contains(list, "h264_qsv"));
    assert!(encoders_list_contains(list, "h264_amf"));
    assert!(encoders_list_contains(list, "libx264"));
    // Present in the file as a substring of another id, but NOT as its own row.
    assert!(!encoders_list_contains(list, "h264"));
    assert!(!encoders_list_contains(list, "nvenc"));
    // Encoder absent from the listing entirely.
    assert!(!encoders_list_contains(list, "libsvtav1"));
  }

  #[test]
  fn empty_or_malformed_listing_finds_nothing() {
    assert!(!encoders_list_contains("", "h264_nvenc"));
    assert!(!encoders_list_contains("junk line without tokens", "h264_nvenc"));
  }

  #[test]
  fn stderr_tail_surfaces_the_real_error_not_the_summary() {
    // Real stderr from a CQP-only Intel i965 driver being probed with VBR:
    // the useful line is buried under ffmpeg's end-of-run noise.
    let stderr = b"\n  Stream #0:0 -> #0:0 (wrapped_avframe (native) -> h264 (h264_vaapi))\nPress [q] to stop, [?] for help\n[h264_vaapi @ 0x606944fc7580] Driver does not support VBR RC mode (supported modes: CQP).\n[vost#0:0/h264_vaapi @ 0x606944fc7180] Error while opening encoder - maybe incorrect parameters such as bit_rate, rate, width or height.\nError while filtering: Invalid argument\n[out#0/null @ 0x606944fc5e80] Nothing was written into output file, because at least one of its streams received no packets.\nframe=    0 fps=0.0 q=0.0 Lsize=       0kB time=N/A bitrate=N/A speed=N/A    \nConversion failed!\n";
    let tail = stderr_tail(stderr);
    // The real cause survives...
    assert!(tail.contains("Driver does not support VBR RC mode"), "got: {tail}");
    assert!(tail.contains("Error while opening encoder"), "got: {tail}");
    // ...while the misleading "no packets" summary and stats are filtered out.
    assert!(!tail.contains("no packets"), "got: {tail}");
    assert!(!tail.contains("Conversion failed"), "got: {tail}");
    assert!(!tail.contains("frame="), "got: {tail}");
    assert!(!tail.contains("Lsize="), "got: {tail}");
  }

  #[test]
  fn stderr_tail_with_only_noise_reports_no_detail() {
    assert_eq!(
      stderr_tail(b"\nConversion failed!\nframe=    0 fps=0.0 q=0.0 Lsize=0kB\n"),
      "encode failed (no detail in stderr)"
    );
  }
}

/// Ordered candidate encoders per codec family. Hardware encoders that honor a
/// bitrate cap come first, then software encoders (CRF + VBV) as the reliable
/// fallback that works on every machine.
pub const ENCODER_PREFERENCES: &[(&str, &[&str])] = &[
  (
    "h264",
    &[
      "h264_qsv",
      "h264_nvenc",
      "h264_vaapi",
      "h264_amf",
      "h264_videotoolbox",
      "libx264",
    ],
  ),
  (
    "hevc",
    &[
      "hevc_qsv",
      "hevc_nvenc",
      "hevc_vaapi",
      "hevc_videotoolbox",
      "libx265",
    ],
  ),
  ("av1", &["av1_qsv", "av1_nvenc", "av1_vaapi", "libsvtav1"]),
];

/// Pick the first usable encoder for a codec preference ("auto", "h264",
/// "hevc", "av1"). Unknown/unavailable preferences fall back to "h264".
/// `test_encoder` does the actual capability probe, so a broken HW driver
/// (e.g. VAAPI that cannot encode at all) is skipped in favor of a software
/// encoder — but a CQP-only VAAPI driver IS usable (export adapts via
/// `vaapi_rc_mode`).
pub fn pick_encoder(ffmpeg_exe: &str, preference: &str) -> String {
  let pref = match preference {
    "h264" => "h264",
    "hevc" => "hevc",
    "av1" => "av1",
    _ => "h264",
  };
  let candidates = ENCODER_PREFERENCES
    .iter()
    .find(|(key, _)| *key == pref)
    .map(|(_, list)| *list)
    .unwrap_or(&["libx264"]);
  candidates
    .iter()
    .copied()
    .find(|id| test_encoder(ffmpeg_exe, id))
    .unwrap_or("libx264")
    .to_string()
}

pub fn get_system_memory() -> Option<SystemMemoryInfo> {
  #[cfg(target_os = "linux")]
  {
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
      let mut total_kb = 0u64;
      let mut available_kb = 0u64;
      for line in content.lines() {
        if line.starts_with("MemTotal:") {
          if let Some(val) = line.split_whitespace().nth(1) {
            total_kb = val.parse().unwrap_or(0);
          }
        } else if line.starts_with("MemAvailable:") {
          if let Some(val) = line.split_whitespace().nth(1) {
            available_kb = val.parse().unwrap_or(0);
          }
        }
      }
      if total_kb > 0 {
        let used_kb = total_kb.saturating_sub(available_kb);
        let total_mb = total_kb / 1024;
        let used_mb = used_kb / 1024;
        let pct = (used_mb as f32 / total_mb as f32) * 100.0;
        return Some(SystemMemoryInfo {
          used_mb,
          total_mb,
          used_percent: pct,
        });
      }
    }
  }
  None
}

pub fn get_system_memory_cmd() -> Option<SystemMemoryInfo> {
  get_system_memory()
}

pub fn check_hardware(app_data_dir: Option<&std::path::Path>) -> Result<HardwareInfo, String> {
  let gpus = get_gpu_adapters();
  let (ffmpeg_installed, ffmpeg_path, encoders) = match resolve_ffmpeg(app_data_dir) {
    Ok(path) => {
      let encs = detect_encoders(&path);
      let usable: Vec<&str> = encs
        .iter()
        .filter(|e| e.supported)
        .map(|e| e.id.as_str())
        .collect();
      crate::logline!(
        "[Hardware] FFmpeg: {} — usable encoders: {}",
        path,
        if usable.is_empty() { "none".to_string() } else { usable.join(", ") }
      );
      for e in &encs {
        if !e.supported && e.id != "libx264" {
          crate::logline!("[Hardware]   {} unavailable: {}", e.id, e.detail);
        }
      }
      (true, Some(path), encs)
    }
    Err(_) => (false, None, Vec::new()),
  };

  let mut recommended = "libx264".to_string();
  let mut rec_label = "Software CPU (x264)".to_string();

  for e in &encoders {
    if e.supported && e.id != "libx264" {
      recommended = e.id.clone();
      rec_label = format!("⚡ GPU Accelerated ({})", e.name);
      break;
    }
  }

  let memory = get_system_memory();
  let os = std::env::consts::OS.to_string();
  let arch = std::env::consts::ARCH.to_string();

  Ok(HardwareInfo {
    gpus,
    ffmpeg_installed,
    ffmpeg_path,
    encoders,
    recommended_encoder: recommended,
    recommended_label: rec_label,
    memory,
    os,
    arch,
  })
}
