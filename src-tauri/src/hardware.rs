use crate::ffmpeg::resolve_ffmpeg;
use serde::{Deserialize, Serialize};
use std::process::Command;

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

pub fn test_encoder(ffmpeg_exe: &str, encoder_id: &str) -> bool {
  if encoder_id == "libx264" {
    return true;
  }

  if encoder_id == "h264_vaapi" {
    let Some(dev) = pick_vaapi_device() else {
      return false;
    };

    let output = Command::new(ffmpeg_exe)
      .args([
        "-hide_banner",
        "-vaapi_device",
        dev,
        "-f",
        "lavfi",
        "-i",
        "color=c=black:s=64x64:d=0.1",
        "-vf",
        "format=nv12,hwupload",
        "-c:v",
        "h264_vaapi",
        "-f",
        "null",
        "-",
      ])
      .output();

    return output.map(|o| o.status.success()).unwrap_or(false);
  }

  let output = Command::new(ffmpeg_exe)
    .args([
      "-hide_banner",
      "-f",
      "lavfi",
      "-i",
      "color=c=black:s=64x64:d=0.1",
      "-c:v",
      encoder_id,
      "-f",
      "null",
      "-",
    ])
    .output();

  match output {
    Ok(out) => out.status.success(),
    Err(_) => false,
  }
}

pub fn detect_encoders(ffmpeg_exe: &str) -> Vec<EncoderCapability> {
  let candidates = vec![
    ("h264_nvenc", "NVIDIA NVENC H.264", "Akselerasi Hardware GPU NVIDIA (GeForce/RTX)"),
    ("h264_qsv", "Intel QuickSync (QSV)", "Akselerasi Hardware GPU Intel (iGPU & Arc)"),
    ("h264_vaapi", "Linux VAAPI H.264", "Akselerasi Hardware Driver Linux (Intel/AMD)"),
    ("h264_amf", "AMD AMF H.264", "Akselerasi Hardware GPU AMD Radeon"),
    ("h264_videotoolbox", "Apple VideoToolbox", "Akselerasi Hardware Mac (M1/M2/M3 & Intel)"),
    ("libx264", "Software x264 (CPU)", "Enkoder Software standar berbasis CPU"),
  ];

  let mut list = Vec::new();
  for (id, name, desc) in candidates {
    let supported = test_encoder(ffmpeg_exe, id);
    list.push(EncoderCapability {
      id: id.to_string(),
      name: name.to_string(),
      supported,
      description: desc.to_string(),
    });
  }

  list
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

#[tauri::command]
pub async fn get_system_memory_cmd() -> Option<SystemMemoryInfo> {
  get_system_memory()
}

#[tauri::command]
pub async fn check_hardware(app_handle: tauri::AppHandle) -> Result<HardwareInfo, String> {
  let gpus = get_gpu_adapters();
  let (ffmpeg_installed, ffmpeg_path, encoders) = match resolve_ffmpeg(&app_handle) {
    Ok(path) => {
      let encs = detect_encoders(&path);
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
