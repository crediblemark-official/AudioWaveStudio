use crate::audio_decoder::AudioData;
use crate::audio_player::AudioPlayer;
use crate::config::VisualizerConfig;
use slint::{Image, Rgb8Pixel, SharedPixelBuffer};
use std::collections::VecDeque;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Toast notifications
//
// A bounded queue of user-visible notifications. The Rust side pushes messages
// for failures/successes that previously only went to stderr (decode errors,
// preset I/O, mic capture), and `callbacks::sync_toasts` mirrors them into the
// Slint `toast-list` model (mutating the SAME VecModel in place so existing
// toasts keep their animation/timer state).
// ---------------------------------------------------------------------------

/// Maximum number of toasts kept on screen at once (oldest evicted first).
pub const MAX_TOASTS: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Error,
    Success,
    Info,
}

impl ToastKind {
    /// Slint-side `kind` discriminator consumed by ToastCard for accent/icon.
    pub fn as_str(&self) -> &'static str {
        match self {
            ToastKind::Error => "error",
            ToastKind::Success => "success",
            ToastKind::Info => "info",
        }
    }

    /// How long the toast stays on screen before auto-dismissing.
    pub fn duration_ms(&self) -> u64 {
        match self {
            ToastKind::Error => 6000,
            ToastKind::Success | ToastKind::Info => 3200,
        }
    }
}

pub struct ToastMsg {
    pub id: u64,
    pub kind: ToastKind,
    pub text: String,
    pub duration_ms: u64,
}

/// Live microphone capture backed by an `arecord` (ALSA) subprocess streaming raw
/// S16_LE PCM on stdout, pushed into a bounded ring buffer.
pub struct MicCapture {
    pub child: Option<Child>,
    pub thread: Option<std::thread::JoinHandle<()>>,
    pub running: Arc<AtomicBool>,
}

impl Drop for MicCapture {
    /// Ensure the arecord child is reaped even if the app exits while listening.
    ///
    /// NEVER blocks the UI thread indefinitely: arecord blocked on a device can
    /// ignore signals for a while, and `Child::wait()` would hang the app on
    /// close. Reap with a bounded deadline, re-signalling SIGKILL while the
    /// child is dying; if it still won't die, drop the handle (zombie reaped by
    /// init at exit) rather than freezing the UI.
    fn drop(&mut self) {
        if let Some(ref mut c) = self.child {
            let end = std::time::Instant::now() + std::time::Duration::from_millis(1200);
            loop {
                match c.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) if std::time::Instant::now() < end => {
                        let _ = c.kill(); // idempotent, re-signal while dying
                        std::thread::sleep(std::time::Duration::from_millis(25));
                    }
                    _ => break,
                }
            }
        }
    }
}

/// Length of the mic ring buffer in seconds (sliding window for the FFT).
pub const MIC_BUFFER_SECONDS: usize = 2;

/// A capture (input) device discovered via `arecord -l`.
/// `arg` is the `-D` device string passed to arecord; empty = system default.
pub struct MicDevice {
    pub label: String,
    pub arg: String,
}

/// Enumerate ALSA capture devices by parsing `arecord -l` (the same tool the
/// capture backend spawns, so the list always matches what can actually be used).
/// The first entry is always the system default; entries are deduplicated.
pub fn list_mic_devices() -> Vec<MicDevice> {
    let mut out = vec![MicDevice {
        label: "Default (System)".to_string(),
        arg: String::new(),
    }];
    let Ok(output) = Command::new("arecord").arg("-l").output() else {
        return out;
    };
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        // e.g. "card 0: PCH [HDA Intel PCH], device 0: ALC671 Analog [ALC671 Analog]"
        let Some((card_part, dev_part)) = line.split_once(", device ") else {
            continue;
        };
        let Some((_, card_desc_full)) = card_part.split_once(": ") else {
            continue;
        };
        let (card_name, card_desc) = match card_desc_full.split_once(" [") {
            Some((n, rest)) => (n.trim().to_string(), rest.trim_end_matches(']').to_string()),
            None => (card_desc_full.trim().to_string(), String::new()),
        };
        let Some((dev_idx, dev_desc_full)) = dev_part.split_once(": ") else {
            continue;
        };
        let dev_desc = dev_desc_full
            .split(" [")
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        // Prefer the descriptive names (e.g. "HDA Intel PCH — ALC671 Analog").
        let label = if card_desc.is_empty() {
            format!("{card_name} — {dev_desc}")
        } else {
            format!("{card_desc} — {dev_desc}")
        };
        // plughw allows the S16_LE mono 44.1kHz conversion the capture needs.
        let arg = format!("plughw:CARD={card_name},DEV={dev_idx}");
        if !out.iter().any(|d| d.arg == arg) {
            out.push(MicDevice { label, arg });
        }
    }
    out
}

pub struct GpuPreviewEngine {
    pub renderer: crate::gpu2d::GpuRenderer,
    pub width: u32,
    pub height: u32,
    pub bg_image_uri: Option<String>,
    pub bg_image_info: Option<(u32, u32)>,
    pub radial_image_uri: Option<String>,
    pub radial_image_info: Option<(u32, u32)>,
    pub render_state: Option<crate::renderers::RenderState>,
}

pub struct SlintAppState {
    pub audio_data: Option<Arc<AudioData>>,
    pub audio_player: AudioPlayer,
    pub audio_path: Option<String>,
    pub config: VisualizerConfig,
    pub gpu_engine: Option<GpuPreviewEngine>,
    pub _prev_smoothed: Option<Vec<f32>>,
    /// Mute state for the audio player bar.
    pub is_muted: bool,
    /// Volume restored when unmuting.
    pub prev_volume: f32,
    /// Live mic capture state (Listen feature).
    pub is_listening: bool,
    pub mic_buffer: Arc<Mutex<VecDeque<f32>>>,
    pub mic_capture: Option<MicCapture>,
    /// Seconds since capture started (used to detect a silent/dead stream).
    pub mic_elapsed: f32,
    /// Selected capture device (label for display, `arg` for arecord `-D`).
    pub mic_device_label: String,
    pub mic_device_arg: String,
    /// Toast notification queue (see module docs above).
    pub toasts: VecDeque<ToastMsg>,
    /// Monotonic id counter so dismissals are unambiguous.
    pub next_toast_id: u64,
}

impl SlintAppState {
    pub fn new() -> Self {
        let mut config = VisualizerConfig::default();
        // Mirror the legacy default (presets.ts): screen effects are enabled.
        config.screen_effects.enabled = true;
        Self {
            audio_data: None,
            audio_player: AudioPlayer::new(),
            audio_path: None,
            config,
            gpu_engine: None,
            _prev_smoothed: None,
            is_muted: false,
            prev_volume: 0.8,
            is_listening: false,
            mic_buffer: Arc::new(Mutex::new(VecDeque::new())),
            mic_capture: None,
            mic_elapsed: 0.0,
            mic_device_label: "Default (System)".to_string(),
            mic_device_arg: String::new(),
            toasts: VecDeque::new(),
            next_toast_id: 1,
        }
    }

    /// Append a toast to the queue, evicting the oldest when full.
    pub fn push_toast(&mut self, kind: ToastKind, text: impl Into<String>) {
        self.toasts.push_back(ToastMsg {
            id: self.next_toast_id,
            kind,
            text: text.into(),
            duration_ms: kind.duration_ms(),
        });
        self.next_toast_id += 1;
        while self.toasts.len() > MAX_TOASTS {
            self.toasts.pop_front();
        }
    }

    /// Remove a toast by id (manual ✕ or auto-dismiss timer). Returns whether
    /// it was actually present.
    pub fn dismiss_toast(&mut self, id: u64) -> bool {
        if let Some(pos) = self.toasts.iter().position(|t| t.id == id) {
            self.toasts.remove(pos);
            true
        } else {
            false
        }
    }

    /// Select the capture device used by the next `start_listen()`.
    pub fn set_mic_device(&mut self, label: &str, arg: &str) {
        self.mic_device_label = label.to_string();
        self.mic_device_arg = arg.to_string();
    }

    /// Start live mic capture. Uses `arecord` (ALSA, raw S16_LE mono 44.1kHz),
    /// optionally targeting a user-selected device via `-D` (from the picker).
    /// Returns a status string on success or a diagnostic error when capture cannot start.
    pub fn start_listen(&mut self) -> Result<String, String> {
        if self.is_listening {
            return Ok("Microphone is already active".to_string());
        }

        let buffer = self.mic_buffer.clone();
        let running = Arc::new(AtomicBool::new(true));

        let mut cmd = Command::new("arecord");
        cmd.args(["-f", "S16_LE", "-r", "44100", "-c", "1", "-t", "raw"]);
        if !self.mic_device_arg.is_empty() {
            cmd.arg("-D").arg(&self.mic_device_arg);
        }
        cmd.arg("-");
        let mut child = match cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                return Err(format!(
                    "Mic capture unavailable: arecord (ALSA CLI) could not be started ({e}). \
                     Make sure the alsa-utils package is installed."
                ));
            }
        };

        let mut stdout = child.stdout.take().ok_or_else(|| "Failed to open arecord output stream".to_string())?;
        let running_clone = running.clone();
        let thread = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while running_clone.load(Ordering::SeqCst) {
                match stdout.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        // Poison-proof: a panic elsewhere must not silently kill
                        // the capture thread (which would look like a dead mic).
                        let mut b = buffer.lock().unwrap_or_else(|e| e.into_inner());
                        for chunk in buf[..n].chunks_exact(2) {
                            let s = i16::from_le_bytes([chunk[0], chunk[1]]);
                            b.push_back(s as f32 / 32768.0);
                        }
                        while b.len() > 44100 * MIC_BUFFER_SECONDS {
                            b.pop_front();
                        }
                    }
                }
            }
            running_clone.store(false, Ordering::SeqCst);
        });

        self.mic_capture = Some(MicCapture {
            child: Some(child),
            thread: Some(thread),
            running,
        });
        self.is_listening = true;
        self.mic_elapsed = 0.0;
        let dev = if self.mic_device_arg.is_empty() {
            "system default".to_string()
        } else {
            self.mic_device_label.clone()
        };
        Ok(format!("Mendengarkan via {dev} (ALSA, 44.1kHz mono)…"))
    }

    /// Stop live mic capture and clear the ring buffer.
    ///
    /// NEVER blocks the UI thread indefinitely: arecord can ignore SIGKILL for
    /// a while when blocked on a device, and `Child::wait()` would otherwise
    /// hang the whole app for seconds (a perceived force-close/freeze). Reap
    /// with a bounded deadline, re-signalling SIGKILL while the child is dying
    /// so a busy arecord cannot survive the stop (an orphaned arecord would
    /// keep grabbing the mic and, on exit, look like audio still playing).
    pub fn stop_listen(&mut self) {
        if let Some(mut cap) = self.mic_capture.take() {
            cap.running.store(false, Ordering::SeqCst);
            if let Some(ref mut c) = cap.child {
                let deadline =
                    std::time::Instant::now() + std::time::Duration::from_millis(1200);
                while std::time::Instant::now() < deadline {
                    match c.try_wait() {
                        Ok(Some(_)) => break,
                        Ok(None) => {
                            let _ = c.kill(); // idempotent, re-signal while dying
                            std::thread::sleep(std::time::Duration::from_millis(25));
                        }
                        Err(_) => break,
                    }
                }
            }
            if let Some(t) = cap.thread.take() {
                // The reader thread unblocks as soon as the pipe closes after
                // the child dies; joining with a timeout keeps us safe too.
                let handle = t;
                let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
                while std::time::Instant::now() < deadline && !handle.is_finished() {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                if handle.is_finished() {
                    let _ = handle.join();
                }
            }
        }
        self.is_listening = false;
        self.mic_buffer.lock().unwrap_or_else(|e| e.into_inner()).clear();
        self.mic_elapsed = 0.0;
    }

    /// Return the most recent `n` mono samples from the live mic buffer
    /// (zero-padded when not enough data has arrived yet).
    pub fn mic_window(&self, n: usize) -> Vec<f32> {
        let b = self.mic_buffer.lock().unwrap_or_else(|e| e.into_inner());
        let start = b.len().saturating_sub(n);
        let mut win: Vec<f32> = b.iter().skip(start).take(n).copied().collect();
        while win.len() < n {
            win.push(0.0);
        }
        win
    }
}

pub fn format_time(seconds: f64) -> String {
    let total_sec = seconds.max(0.0).round() as u64;
    let mins = total_sec / 60;
    let secs = total_sec % 60;
    format!("{:02}:{:02}", mins, secs)
}

pub fn create_slint_image_from_rgb(width: u32, height: u32, rgb_bytes: &[u8]) -> Image {
    let buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width, height);
    let pixels = buffer.as_slice();
    
    let expected = (width * height * 3) as usize;
    if rgb_bytes.len() >= expected {
        for (i, pixel) in pixels.iter().enumerate() {
            let idx = i * 3;
            let p = pixel as *const Rgb8Pixel as *mut Rgb8Pixel;
            unsafe {
                (*p).r = rgb_bytes[idx];
                (*p).g = rgb_bytes[idx + 1];
                (*p).b = rgb_bytes[idx + 2];
            }
        }
    }

    Image::from_rgb8(buffer)
}

pub fn create_slint_image_from_rgba(width: u32, height: u32, rgba_bytes: &[u8]) -> Image {
    let buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width, height);
    let pixels = buffer.as_slice();
    
    let expected = (width * height * 4) as usize;
    if rgba_bytes.len() >= expected {
        for (i, pixel) in pixels.iter().enumerate() {
            let idx = i * 4;
            let p = pixel as *const Rgb8Pixel as *mut Rgb8Pixel;
            unsafe {
                (*p).r = rgba_bytes[idx];
                (*p).g = rgba_bytes[idx + 1];
                (*p).b = rgba_bytes[idx + 2];
            }
        }
    }

    Image::from_rgb8(buffer)
}
