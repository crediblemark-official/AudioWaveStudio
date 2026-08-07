pub mod app_state;
pub mod audio_decoder;
pub mod audio_player;
pub mod callbacks;
pub mod config;
pub mod export_ffmpeg;
pub mod ffmpeg;
pub mod fft_analyzer;
pub mod gpu2d;
pub mod gpu_export;
pub mod hardware;
pub mod renderers;

use app_state::{create_slint_image_from_rgb, format_time, SlintAppState};
use callbacks::bind_app_callbacks;
use fft_analyzer::FftAnalyzer;
use slint::ComponentHandle;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

slint::include_modules!();

/// Lock the shared app state without letting a poisoned mutex (a panic that
/// happened inside another lock holder) abort the app on the next frame.
fn poison_proof<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

/// Timestamp (unix seconds) of the last panic the hook actually logged, so a
/// recurring fault (e.g. a per-frame panic the timer deliberately recovers
/// from) cannot flood the console / log file with 60 full backtraces per
/// second.
static LAST_PANIC_LOG: AtomicU64 = AtomicU64::new(0);

/// Intercepts OS-level file drag & drop from winit.
///
/// Slint 1.17's winit backend IGNORES winit's DroppedFile/HoveredFile events
/// (verified in i-slint-backend-winit-1.17.1 event_loop.rs — no match arms
/// for them, so they fall into the catch-all and are discarded). The official
/// `CustomApplicationHandler` hook is the only way to receive real file drops:
/// - `HoveredFile` -> light the gold "Release to load track" highlight.
/// - `HoveredFileCancelled` -> clear the highlight.
/// - `DroppedFile(path)` -> clear the highlight and route into the
///   `audio-file-dropped` Slint callback (which reuses the Open File load path).
/// Every other event propagates to Slint unchanged, so the app behaves
/// exactly as before.
struct DropFileHandler {
    /// Weak handle to the main window, filled in right after it is created
    /// (the platform must be installed before window creation, so the handler
    /// can't hold a live handle from the start).
    app: std::rc::Rc<std::cell::RefCell<Option<slint::Weak<crate::AppWindow>>>>,
    /// True while a multi-file drop burst is in flight: winit fires one
    /// `DroppedFile` per dropped file, and we only want to load the FIRST one
    /// (otherwise dropping 3 songs decodes+toasts 3 times with the last one
    /// silently winning). Reset when a new drag enters the window.
    in_burst: bool,
}

impl i_slint_backend_winit::CustomApplicationHandler for DropFileHandler {
    fn window_event(
        &mut self,
        _event_loop: &i_slint_backend_winit::winit::event_loop::ActiveEventLoop,
        _window_id: i_slint_backend_winit::winit::window::WindowId,
        _winit_window: Option<&i_slint_backend_winit::winit::window::Window>,
        _slint_window: Option<&slint::Window>,
        event: &i_slint_backend_winit::winit::event::WindowEvent,
    ) -> i_slint_backend_winit::EventResult {
        use i_slint_backend_winit::winit::event::WindowEvent as WEvent;
        let app = self.app.borrow().as_ref().and_then(|w| w.upgrade());
        match event {
            WEvent::HoveredFile(_) => {
                // A new drag enters the window: arm the burst counter so the
                // first DroppedFile of THIS drag loads the track.
                self.in_burst = false;
                if let Some(w) = app {
                    w.set_drop_hover(true);
                }
                i_slint_backend_winit::EventResult::PreventDefault
            }
            WEvent::HoveredFileCancelled => {
                if let Some(w) = app {
                    w.set_drop_hover(false);
                }
                i_slint_backend_winit::EventResult::PreventDefault
            }
            WEvent::DroppedFile(path) => {
                if let Some(w) = app {
                    w.set_drop_hover(false);
                    if !self.in_burst {
                        self.in_burst = true;
                        w.invoke_audio_file_dropped(slint::SharedString::from(
                            path.to_string_lossy().to_string(),
                        ));
                    }
                }
                i_slint_backend_winit::EventResult::PreventDefault
            }
            _ => i_slint_backend_winit::EventResult::Propagate,
        }
    }
}

pub fn run() {
    // Log every panic (with a backtrace) to a file so a force-close is never
    // silent: the user can share /tmp/audiowave-panic.log for a diagnosis.
    // Throttled to ~1 logged panic per second (see LAST_PANIC_LOG).
    std::panic::set_hook(Box::new(|info| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let last = LAST_PANIC_LOG.load(Ordering::Relaxed);
        if last != 0 && now > 0 && now - last < 1 {
            return;
        }
        LAST_PANIC_LOG.store(now, Ordering::Relaxed);

        let msg = panic_message(info.payload());
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let backtrace = std::backtrace::Backtrace::force_capture();
        eprintln!("[Panic] {msg} at {location}\n{backtrace}");
        let log_path = std::env::var("AUDIOWAVE_PANIC_LOG")
            .unwrap_or_else(|_| "/tmp/audiowave-panic.log".to_string());
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let _ = writeln!(f, "[{now}] {msg} at {location}\n{backtrace}");
        }
    }));

    // Install the winit backend with the file-drop handler BEFORE any window
    // is created (slint::platform::set_platform must precede window creation).
    // The handler starts with an empty window slot; it is filled in right after
    // AppWindow::new() below.
    let app_slot: std::rc::Rc<std::cell::RefCell<Option<slint::Weak<crate::AppWindow>>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));
    let backend = i_slint_backend_winit::Backend::builder().with_custom_application_handler(Box::new(
        DropFileHandler {
            app: app_slot.clone(),
            in_burst: false,
        },
    ))
        .build()
        .expect("Failed to build Slint winit backend");
    slint::platform::set_platform(Box::new(backend))
        .expect("Failed to set Slint platform");

    let window = AppWindow::new().expect("Failed to create Slint AppWindow");
    *app_slot.borrow_mut() = Some(window.as_weak());

    // Second native window for the pop-out preview (hidden until the user
    // clicks the navbar "Preview" button; shown on demand). Creating a second
    // surface/renderer can fail on constrained systems, so degrade gracefully
    // instead of panicking the whole app over an optional window.
    let preview = match PreviewWindow::new() {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("[Preview] Pop-out preview unavailable: {}", e);
            None
        }
    };

    // Apply the legacy gold (#ffd700) theme to built-in widgets: the Fluent style
    // re-hues its accent color (primary/checked buttons, sliders, selections,
    // scrollbars) from the runtime accent color, and forcing the dark color-scheme
    // keeps control surfaces dark against the pure-black UI background.
    //
    // NOTE: this uses the internal `private_unstable_api::re_exports` surface and is
    // tied to Slint 1.17 internals (pinned in Cargo.lock). Re-verify on any upgrade.
    {
        use slint::private_unstable_api::re_exports::{ColorScheme, WindowInner};
        let inner = WindowInner::from_pub(window.window());
        inner.context().set_accent_color(slint::Color::from_rgb_u8(0xff, 0xd7, 0x00));
        inner.context().set_color_scheme(ColorScheme::Dark);
    }

    let state = Arc::new(Mutex::new(SlintAppState::new()));

    // Detect system hardware info
    let window_hw = window.as_weak();
    let refresh_hardware = move || {
        if let Some(w) = window_hw.upgrade() {
            if let Some(mem) = crate::hardware::get_system_memory() {
                w.set_ram_info_text(slint::SharedString::from(format!("{:.1} GB RAM", mem.total_mb as f64 / 1024.0)));
            }
            let gpus = crate::hardware::get_gpu_adapters();
            if !gpus.is_empty() {
                let names: Vec<String> = gpus.iter().map(|g| g.name.clone()).collect();
                w.set_gpu_info_text(slint::SharedString::from(names.join(", ")));
            }
            match crate::hardware::check_hardware(None) {
                Ok(info) => {
                    if info.ffmpeg_installed {
                        w.set_ffmpeg_status_text(slint::SharedString::from(format!("Ready ({})", info.recommended_encoder)));
                    } else {
                        w.set_ffmpeg_status_text(slint::SharedString::from("Not Found (System PATH)"));
                    }
                    let mut lines = Vec::new();
                    lines.push(format!("Acceleration Status: {}", info.recommended_label));
                    lines.push(format!("Operating System: {} ({})", info.os.to_uppercase(), info.arch));
                    lines.push(String::new());
                    lines.push(format!("Graphics / GPU ({})", info.gpus.len()));
                    for g in &info.gpus {
                        let dev = match g.device_type.as_str() {
                            "IntegratedGpu" => "Integrated GPU",
                            "DiscreteGpu" => "Discrete GPU (Dedicated)",
                            "Cpu" => "CPU Software Rendering",
                            "VirtualGpu" => "Virtual / Passthrough GPU",
                            other => other,
                        };
                        lines.push(format!("  • {} — {} [Backend: {}]", g.name, dev, g.backend));
                    }
                    if info.gpus.is_empty() {
                        lines.push("  (no wgpu GPU detected directly)".to_string());
                    }
                    lines.push(String::new());
                    lines.push(format!("FFmpeg Video Encoder Technology {}", if info.ffmpeg_installed { "(FFmpeg Ready)" } else { "(FFmpeg Not Found)" }));
                    let supported: Vec<_> = info.encoders.iter().filter(|e| e.supported).collect();
                    if supported.is_empty() {
                        lines.push("  (no encoders detected)".to_string());
                    }
                    for e in supported {
                        let mark = if e.id == info.recommended_encoder { "◀ Active" } else { "" };
                        lines.push(format!("  ✓ {} ({}) {}", e.name, e.id, mark));
                        lines.push(format!("      {}", e.description));
                    }
                    if let Some(p) = &info.ffmpeg_path {
                        lines.push(String::new());
                        lines.push(format!("FFmpeg Path: {}", p));
                    }
                    w.set_hardware_detail_text(slint::SharedString::from(lines.join("\n")));
                }
                Err(e) => {
                    eprintln!("[Hardware] check_hardware error: {}", e);
                    w.set_hardware_detail_text(slint::SharedString::from(format!("Error: {}", e)));
                }
            }
        }
    };

    // Initial hardware scan.
    refresh_hardware();

    // Re-scan hardware when the user clicks the button in the modal.
    window.on_rescan_hardware_clicked(move || {
        refresh_hardware();
    });

    // Bind all Slint UI user callbacks
    bind_app_callbacks(&window, preview.as_ref(), state.clone());

    // PREVIEW WINDOW CLOSE (its native ✕ / Alt+F4): without a handler, closing
    // the pop-out preview would leave the (hidden) second window — and its
    // renderer/surface — alive while the 60 FPS timer keeps polling it. Hide it
    // explicitly: on Wayland hiding destroys the winit window (freeing the
    // surface + renderer), and the navbar Preview button re-shows it on demand.
    if let Some(p) = preview.as_ref() {
        let preview_weak = p.as_weak();
        p.window().on_close_requested(move || {
            if let Some(pw) = preview_weak.upgrade() {
                let _ = pw.hide();
            }
            slint::CloseRequestResponse::HideWindow
        });
    }

    // WINDOW-MANAGER CLOSE (Alt+F4 / WM close button / session shutdown): the
    // custom ✕ button stops audio+mic via close_clicked, but a WM-initiated
    // close bypasses that callback. Register on_close_requested so WM close
    // triggers the exact same explicit shutdown (audio + mic + preview window)
    // before the window is hidden — the player child must never outlive the
    // window, or audio keeps playing after the app looks closed.
    {
        let state_for_close = state.clone();
        let preview_weak = preview.as_ref().map(|p| p.as_weak());
        window.window().on_close_requested(move || {
            let mut s = poison_proof(&state_for_close);
            s.audio_player.stop_for_shutdown();
            s.stop_listen();
            drop(s);
            // Close the pop-out preview too (mirror of close_clicked), so the
            // event loop ends when the main window hides.
            if let Some(p) = preview_weak.as_ref().and_then(|w| w.upgrade()) {
                let _ = p.hide();
            }
            slint::CloseRequestResponse::HideWindow
        });
    }

    // 60 FPS TIMER FOR LIVE ANIMATION & GPU PREVIEW RENDERING
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    let preview_handle = preview.as_ref().map(|p| p.as_weak());
    let timer = slint::Timer::default();
    let mut idle_clock: f32 = 0.0;
    // Timestamp (in idle seconds) of the last recovered-frame log, used to
    // throttle recurring faults to ~1 line/s.
    let mut last_panic_log: f32 = -10.0;

    timer.start(slint::TimerMode::Repeated, Duration::from_millis(16), move || {
        // A panic on the UI thread (e.g. a transient wgpu/GPU hiccup inside a
        // single frame, or a render bug hit by unusual audio) would abort the
        // whole process. Catch it here so one bad frame can never force-close
        // the app: the frame is skipped and the animation continues.
        let frame_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        idle_clock += 0.016;
        let (audio_opt, time_sec, duration_sec, _is_playing, config) = {
            let mut s = poison_proof(&state_clone);
            let time_sec = s.audio_player.get_current_time_sec();
            let duration_sec = s.audio_player.get_duration_sec();
            let is_playing = s.audio_player.is_playing();
            (s.audio_data.clone(), time_sec, duration_sec, is_playing, s.config.clone())
        };
        let is_listening = poison_proof(&state_clone).is_listening;

        if let Some(w) = window_handle.upgrade() {
            if duration_sec > 0.0 {
                let pct = (time_sec / duration_sec) as f32;
                w.set_playback_progress(pct);
                w.set_current_time_str(slint::SharedString::from(format_time(time_sec)));
            }

            // Mic liveness: if capture produced no samples within ~1.5s, the stream
            // died silently (device busy / permission) — auto-stop and report it.
            if is_listening {
                let mut s = poison_proof(&state_clone);
                s.mic_elapsed += 0.016;
                let silent = s.mic_elapsed > 1.5 && s.mic_buffer.lock().unwrap_or_else(|e| e.into_inner()).is_empty();
                if silent {
                    s.stop_listen();
                    drop(s);
                    w.set_is_listening(false);
                    w.set_listen_status(slint::SharedString::from(
                        "Microphone produced no audio — check the input device or recording permissions",
                    ));
                    // The listen-status property is not rendered anywhere in the
                    // navbar, so surface the dead-mic failure as a toast too.
                    crate::callbacks::push_toast(
                        &w,
                        &mut poison_proof(&state_clone),
                        crate::app_state::ToastKind::Error,
                        "Microphone produced no audio — check the input device or recording permissions",
                    );
                }
            }

            // Live mic input takes priority over the loaded song; otherwise fall
            // back to the song window, then to the idle demo wave.
            let fft_size = config.reactivity.fft_size.max(64);
            let window_samples: Vec<f32> = if is_listening {
                poison_proof(&state_clone).mic_window(fft_size)
            } else if let Some(audio) = audio_opt {
                audio.get_sample_window(time_sec, fft_size)
            } else {
                Vec::new()
            };
            let frame_time = if is_listening { idle_clock } else { time_sec as f32 };

            let (freq_data, time_data, _bass_energy, frame_time) = if !window_samples.is_empty() {
                let analyzer = FftAnalyzer::new(fft_size);
                if let Ok((magnitudes, bass_e)) = analyzer.compute_full_spectrum(&window_samples) {
                    let mut s = poison_proof(&state_clone);
                    let smooth_factor = config.reactivity.smoothing.clamp(0.0, 0.95);
                    let alpha = (1.0 - smooth_factor).clamp(0.08, 1.0);

                    let prev_vec = s._prev_smoothed.get_or_insert_with(|| vec![0.0; magnitudes.len()]);
                    if prev_vec.len() != magnitudes.len() {
                        *prev_vec = vec![0.0; magnitudes.len()];
                    }

                    let mut freq = Vec::with_capacity(magnitudes.len());
                    for (i, &m) in magnitudes.iter().enumerate() {
                        let scaled_m = (m * config.reactivity.sensitivity).clamp(0.0, 1.0);
                        let prev = prev_vec[i];
                        let next_val = if scaled_m > prev {
                            prev + (scaled_m - prev) * (alpha * 1.4).min(1.0)
                        } else {
                            (prev * 0.90 + scaled_m * 0.10).max(0.0)
                        };
                        prev_vec[i] = next_val;
                        freq.push((next_val * 255.0).min(255.0) as u8);
                    }

                    let time: Vec<u8> = window_samples.iter().map(|&s| ((s + 1.0) * 127.5).clamp(0.0, 255.0) as u8).collect();
                    (freq, time, bass_e, frame_time)
                } else {
                    (vec![20u8; 64], vec![128u8; 128], 0.0, frame_time)
                }
            } else {
                let bar_count = config.reactivity.bar_count.max(8);
                let mut demo_freq = Vec::with_capacity(bar_count);
                for i in 0..bar_count {
                    let wave = ((idle_clock * 3.0 + i as f32 * 0.15).sin() * 0.5 + 0.5) * (config.reactivity.sensitivity * 0.8);
                    demo_freq.push((wave * 180.0 + 30.0).min(255.0) as u8);
                }
                (demo_freq, vec![128u8; 128], 0.2, frame_time)
            };

            // Render at the ACTUAL canvas viewport size (read from the UI every
            // frame). The renderer produces a buffer that matches the canvas
            // aspect ratio exactly, so circles stay round — a fixed 16:9 buffer
            // stretched into a wider viewport would squish them into ovals.
            // Clamped for sanity: tiny viewports stay renderable, and huge ones
            // (4K fullscreen) keep the 60 FPS render affordable. The size is
            // quantized to a 8px grid so fractional-logical-px jitter during
            // window drags (Wayland) cannot tear down & rebuild the wgpu engine
            // on every frame — only real size changes rebuild it.
            let vw = (w.get_canvas_viewport_w().clamp(320.0, 1920.0) as u32 / 8) * 8;
            let vh = (w.get_canvas_viewport_h().clamp(180.0, 1080.0) as u32 / 8) * 8;
            let width = vw.max(320);
            let height = vh.max(180);

            let mut rendered = false;
            if let Ok(mut s) = state_clone.lock() {
                let is_playing = s.audio_player.is_playing();
                if s.gpu_engine.is_none() || s.gpu_engine.as_ref().unwrap().width != width || s.gpu_engine.as_ref().unwrap().height != height {
                    if let Ok(renderer) = pollster::block_on(crate::gpu2d::GpuRenderer::new(width, height)) {
                        s.gpu_engine = Some(crate::app_state::GpuPreviewEngine {
                            renderer,
                            width,
                            height,
                            bg_image_uri: None,
                            bg_image_info: None,
                            radial_image_uri: None,
                            radial_image_info: None,
                            render_state: None,
                        });
                    }
                }

                if let Some(ref mut engine) = s.gpu_engine {
                    if let Ok(raw_rgba) = crate::gpu_export::render_preview_frame_inner(
                        engine,
                        &config,
                        &freq_data,
                        &time_data,
                        frame_time,
                        width,
                        height,
                        is_playing,
                    ) {
                        let slint_img = crate::app_state::create_slint_image_from_rgba(width, height, &raw_rgba);
                        w.set_preview_frame(slint_img.clone());
                        // Mirror the same live frame into the pop-out preview
                        // window when it is open (legacy BroadcastChannel sync).
                        if let Some(p) = preview_handle.as_ref().and_then(|w| w.upgrade()) {
                            if p.window().is_visible() {
                                p.set_preview_frame(slint_img);
                            }
                        }
                        rendered = true;
                    }
                }
            }

            if !rendered {
                // CPU fallback runs on the UI thread: cap it below the canvas
                // resolution so software rendering stays at 60 FPS. The aspect
                // ratio is preserved (rounds stay round), and image-fit: contain
                // upscales the smaller buffer to fill the viewport.
                let cpu_w = width.min(1280);
                let cpu_h = (cpu_w as f32 * (height as f32 / width as f32)).round() as u32;
                let raw_rgb = crate::renderers::render_frame_to_rgb(
                    &config,
                    &freq_data,
                    &time_data,
                    0.0,
                    frame_time,
                    cpu_w,
                    cpu_h,
                );
                let slint_img = create_slint_image_from_rgb(cpu_w, cpu_h, &raw_rgb);
                w.set_preview_frame(slint_img.clone());
                if let Some(p) = preview_handle.as_ref().and_then(|w| w.upgrade()) {
                    if p.window().is_visible() {
                        p.set_preview_frame(slint_img);
                    }
                }
            }
        }
        }));

        if let Err(payload) = frame_result {
            // A panic can unwind while the GPU engine was borrowed, leaving its
            // cached render state half-mutated — reusing it would re-trigger the
            // same fault every frame. Drop the engine so the next frame builds a
            // fresh one (the CPU fallback covers the gap).
            if let Ok(mut s) = state_clone.lock() {
                if s.gpu_engine.is_some() {
                    s.gpu_engine = None;
                }
            }
            // Throttle the log: the same recurring fault must not spam ~60 lines/s.
            if idle_clock - last_panic_log > 1.0 {
                last_panic_log = idle_clock;
                eprintln!(
                    "[Frame] Recovered from panic (skipping frame, continuing): {}",
                    panic_message(&payload)
                );
            }
        }
    });

    window.run().expect("Failed to run Slint event loop");

    // Event loop ended (window closed). Stop audio + mic EXPLICITLY as a safety
    // net in case the close handler was bypassed (e.g. window-manager close or
    // a fatal panic): the player child process must never outlive the app, or
    // audio keeps playing after the window is gone.
    let mut s = poison_proof(&state);
    s.audio_player.stop_for_shutdown();
    s.stop_listen();
    drop(s);

    // Remove any stale mic FIFO left behind by a crashed capture (probe/test
    // paths used /tmp/fifo_cap.raw; an orphaned writer holding it open would
    // otherwise linger forever).
    let _ = std::fs::remove_file("/tmp/fifo_cap.raw");
}
