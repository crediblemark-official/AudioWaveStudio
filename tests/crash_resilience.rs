//! Stress test: run every visualizer style across extreme configuration
//! batteries through BOTH the CPU fallback path and the live GPU preview path,
//! catching panics that would force-close the app during normal use.
//!
//! This is a diagnostic test (temporary). It exercises the exact same code the
//! 60 FPS timer runs: FFT + smoothing, render_frame_to_rgb (CPU) and
//! render_preview_frame_inner (GPU).

use audiowave_studio_lib::config::{
    BackgroundMode, ScreenEffect, VisualizerConfig, VisualizerStyle,
};
use audiowave_studio_lib::fft_analyzer::FftAnalyzer;
use audiowave_studio_lib::renderers::render_frame_to_rgb;
use std::panic::{catch_unwind, AssertUnwindSafe};

const W: u32 = 640;
const H: u32 = 360;

fn lcg(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    *seed >> 8
}

fn freq_data(n: usize, seed: &mut u32) -> Vec<u8> {
    (0..n).map(|_| (lcg(seed) % 256) as u8).collect()
}

fn all_styles() -> Vec<VisualizerStyle> {
    vec![
        VisualizerStyle::Spectrum,
        VisualizerStyle::Radial,
        VisualizerStyle::Oscilloscope,
        VisualizerStyle::Equalizer,
        VisualizerStyle::Minimal,
        VisualizerStyle::WaveformFill,
        VisualizerStyle::CircularBars,
        VisualizerStyle::SmoothSpectrum,
        VisualizerStyle::PulseRings,
        VisualizerStyle::VuMeter,
        VisualizerStyle::AuroraWave,
        VisualizerStyle::FlameFire,
        VisualizerStyle::SpiralGalaxy,
        VisualizerStyle::ThreeD,
        VisualizerStyle::Api3D,
        VisualizerStyle::NeonCity3D,
        VisualizerStyle::Speaker3D,
        VisualizerStyle::SpeakerTrio,
        VisualizerStyle::SpeakerSplatter,
    ]
}

fn all_effects() -> Vec<ScreenEffect> {
    vec![
        ScreenEffect::None,
        ScreenEffect::Shake,
        ScreenEffect::Glitch,
        ScreenEffect::Vignette,
        ScreenEffect::Pulse,
        ScreenEffect::Spotlight,
        ScreenEffect::Strobe,
        ScreenEffect::Scanline,
        ScreenEffect::Chromatic,
        ScreenEffect::Zoom,
        ScreenEffect::Invert,
        ScreenEffect::Bars,
        ScreenEffect::Shockwave,
        ScreenEffect::Pixelate,
        ScreenEffect::Tilt,
        ScreenEffect::HeatHaze,
        ScreenEffect::HueShift,
    ]
}

/// Build the configuration battery: defaults, every style, every effect,
/// extreme slider values, every background mode, and odd text/particle setups.
fn config_battery() -> Vec<(String, VisualizerConfig)> {
    let mut out: Vec<(String, VisualizerConfig)> = Vec::new();

    // 1. Every style with default settings.
    for s in all_styles() {
        let mut c = VisualizerConfig::default();
        c.style = s.clone();
        out.push((format!("style:{:?}", s), c));
    }

    // 2. Extreme reactivity on the default style.
    let extremes: Vec<(&str, fn(&mut VisualizerConfig))> = vec![
        ("bar_count=1", |c| c.reactivity.bar_count = 1),
        ("bar_count=2", |c| c.reactivity.bar_count = 2),
        ("bar_count=256", |c| c.reactivity.bar_count = 256),
        ("bar_count=512", |c| c.reactivity.bar_count = 512),
        ("sensitivity=0.05", |c| c.reactivity.sensitivity = 0.05),
        ("sensitivity=6", |c| c.reactivity.sensitivity = 6.0),
        ("smoothing=0", |c| c.reactivity.smoothing = 0.0),
        ("smoothing=0.95", |c| c.reactivity.smoothing = 0.95),
        ("mirror", |c| c.reactivity.mirror_bars = true),
        ("scale=3.0", |c| c.scale = 3.0),
        ("scale=0.1", |c| c.scale = 0.1),
        ("pos=(-1,-1)", |c| { c.position_x = -1.0; c.position_y = -1.0; }),
        ("pos=(2,2)", |c| { c.position_x = 2.0; c.position_y = 2.0; }),
        ("fft=64", |c| c.reactivity.fft_size = 64),
        ("fft=4096", |c| c.reactivity.fft_size = 4096),
        ("bar_width=0.5", |c| c.reactivity.bar_width = 0.5),
        ("bar_width=60", |c| c.reactivity.bar_width = 60.0),
        ("bar_rounding=0", |c| c.reactivity.bar_rounding = 0.0),
        ("bar_rounding=40", |c| c.reactivity.bar_rounding = 40.0),
        ("show_peaks", |c| c.reactivity.show_peaks = true),
        ("bass_mult=4", |c| c.reactivity.bass_multiplier = 4.0),
        ("bar_count=0", |c| c.reactivity.bar_count = 0),
        ("scale=0", |c| c.scale = 0.0),
        ("scale=-1", |c| c.scale = -1.0),
        ("pos=(-5,-5)", |c| { c.position_x = -5.0; c.position_y = -5.0; }),
        ("pos=(9,9)", |c| { c.position_x = 9.0; c.position_y = 9.0; }),
        ("gap=50", |c| c.reactivity.bar_gap = 50.0),
        ("gap=-10", |c| c.reactivity.bar_gap = -10.0),
    ];
    for (name, f) in extremes {
        let mut c = VisualizerConfig::default();
        f(&mut c);
        out.push((format!("extreme:{name}"), c));
    }

    // 3. Every screen effect enabled (with background_only both ways).
    for e in all_effects() {
        let mut c = VisualizerConfig::default();
        c.screen_effects.enabled = true;
        c.screen_effects.main_effect = e.clone();
        c.screen_effects.background_only = Some(false);
        out.push((format!("fx:{e:?}"), c.clone()));
        c.screen_effects.background_only = Some(true);
        out.push((format!("fx-bgonly:{e:?}"), c));
    }

    // 4. Every background mode.
    let bgs: Vec<(&str, BackgroundMode)> = vec![
        ("solid", BackgroundMode::Solid),
        ("gradient", BackgroundMode::Gradient),
        ("customImage", BackgroundMode::CustomImage),
        ("grid", BackgroundMode::Grid),
        ("aurora", BackgroundMode::Aurora),
        ("noise", BackgroundMode::Noise),
        ("bokeh", BackgroundMode::Bokeh),
        ("starfield", BackgroundMode::Starfield),
        ("nebula", BackgroundMode::Nebula),
        ("psychedelic", BackgroundMode::Psychedelic),
    ];
    for (name, m) in bgs {
        let mut c = VisualizerConfig::default();
        c.background.mode = m;
        c.background.custom_image_uri = Some("/nonexistent/bogus.png".to_string());
        out.push((format!("bg:{name}"), c));
    }

    // 5. Particles + music notes + text edge cases.
    let mut c = VisualizerConfig::default();
    c.background.show_particles = true;
    c.background.particle_count = Some(500);
    c.background.particle_size = Some(40.0);
    out.push(("particles-extreme".to_string(), c));

    let mut c = VisualizerConfig::default();
    c.background.show_music_notes = Some(true);
    c.background.music_note_count = Some(80);
    c.background.music_note_size = Some(200.0);
    out.push(("notes-extreme".to_string(), c));

    let mut c = VisualizerConfig::default();
    c.text.show_title = true;
    c.text.show_artist = true;
    c.text.title.font_size = 2.0;
    c.text.title.text = "A".repeat(500);
    c.text.title.wave_effect = true;
    c.text.title.letter_spacing = 50.0;
    out.push(("text-extreme".to_string(), c));

    let mut c = VisualizerConfig::default();
    c.text.show_title = true;
    c.text.title.text = "\u{0627}\u{0644}\u{0633}\u{0644}\u{0627}\u{0645}".to_string(); // Arabic RTL
    c.text.title.font_size = 100.0;
    out.push(("text-rtl".to_string(), c));

    let mut c = VisualizerConfig::default();
    c.background.solid_color = "notacolor".to_string();
    c.background.gradient_start = "#zz".to_string();
    c.background.gradient_end = "".to_string();
    out.push(("bad-colors".to_string(), c));

    out
}

#[test]
fn stress_cpu_render_all_configs() {
    let battery = config_battery();
    let mut seed = 0xC0FFEEu32;
    let mut panics: Vec<String> = Vec::new();

    for (name, config) in &battery {
        for frame in 0..10u32 {
            let freq = freq_data(512, &mut seed);
            let time = freq_data(1024, &mut seed);
            let t = frame as f32 / 30.0;
            let r = catch_unwind(AssertUnwindSafe(|| {
                render_frame_to_rgb(config, &freq, &time, 0.2, t, W, H)
            }));
            match r {
                Ok(buf) => assert_eq!(buf.len() as u32, W * H * 3, "buf len {name} f{frame}"),
                Err(e) => {
                    let msg = panic_payload(&e);
                    panics.push(format!("CPU {name} frame {frame}: {msg}"));
                }
            }
        }
    }

    assert!(panics.is_empty(), "CPU path panicked:\n{}", panics.join("\n"));
}

#[test]
fn stress_gpu_preview_all_configs() {
    let renderer = pollster::block_on(audiowave_studio_lib::gpu2d::GpuRenderer::new(W, H))
        .expect("create wgpu renderer");
    let mut engine = audiowave_studio_lib::app_state::GpuPreviewEngine {
        renderer,
        width: W,
        height: H,
        bg_image_uri: None,
        bg_image_info: None,
        radial_image_uri: None,
        radial_image_info: None,
        render_state: None,
    };

    let battery = config_battery();
    let mut seed = 0xDEADBEEFu32;
    let mut panics: Vec<String> = Vec::new();

    for (name, config) in &battery {
        for frame in 0..8u32 {
            let freq = freq_data(512, &mut seed);
            let time = freq_data(1024, &mut seed);
            let t = frame as f32 / 30.0;
            let r = catch_unwind(AssertUnwindSafe(|| {
                audiowave_studio_lib::gpu_export::render_preview_frame_inner(
                    &mut engine, config, &freq, &time, t, W, H, true,
                )
            }));
            match r {
                Ok(Ok(buf)) => assert_eq!(buf.len() as u32, W * H * 4, "gpu buf {name} f{frame}"),
                Ok(Err(e)) => panic!("GPU returned error {name} f{frame}: {e}"),
                Err(e) => {
                    let msg = panic_payload(&e);
                    panics.push(format!("GPU {name} frame {frame}: {msg}"));
                }
            }
        }
    }

    assert!(panics.is_empty(), "GPU path panicked:\n{}", panics.join("\n"));
}

#[test]
fn stress_fft_edge_sizes() {
    let mut seed = 12345u32;
    for size in [64usize, 128, 256, 512, 1024, 2048, 4096] {
        let analyzer = FftAnalyzer::new(size);
        // Random samples, silence, and all-peak samples.
        let mut samples: Vec<f32> = (0..size).map(|_| (lcg(&mut seed) % 1000) as f32 / 500.0 - 1.0).collect();
        let r = catch_unwind(AssertUnwindSafe(|| analyzer.compute_full_spectrum(&samples)));
        assert!(r.is_ok(), "fft random size {size}");
        samples.fill(0.0);
        assert!(analyzer.compute_full_spectrum(&samples).is_ok(), "fft silence size {size}");
        samples.fill(1.0);
        assert!(analyzer.compute_full_spectrum(&samples).is_ok(), "fft all-peak size {size}");
    }
}

/// Verify the exact recovery pattern used by the 60 FPS timer: a poisoned
/// mutex plus a transient panic must NOT abort — the frame is skipped and the
/// next frame keeps working (this is what prevents force-close in the app).
#[test]
fn stress_recovery_pattern() {
    use std::sync::{Arc, Mutex};

    let state = Arc::new(Mutex::new(42u32));
    let state2 = state.clone();
    // Simulate a panic inside another lock holder -> poisons the mutex.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = state2.lock().unwrap();
        panic!("simulated render panic while holding lock");
    }));
    assert!(state.is_poisoned());

    // Now the timer's per-frame logic: a transient panic must be caught...
    let mut frame_ok = 0;
    for _ in 0..5 {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Poison-proof lock: keep going even though the mutex is poisoned.
            let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
            *s += 1;
            // ...and a transient per-frame fault is recoverable.
            if *s == 44 {
                panic!("simulated one-off frame fault");
            }
        }));
        if r.is_ok() {
            frame_ok += 1;
        }
    }
    // The poisoned lock recovered and the one-off panic was skipped.
    assert_eq!(frame_ok, 4);
    assert_eq!(*state.lock().unwrap_or_else(|e| e.into_inner()), 47);
}

/// Regression test for "audio masih muter setelah app diclose": the player's
/// child process (pw-play/ffplay) must be killed and reaped when the shutdown
/// path runs. Uses a FIFO as the "track" — it never ends on its own, so a
/// surviving player process is directly observable.
#[test]
fn shutdown_kills_audio_player_child() {
    use audiowave_studio_lib::audio_player::AudioPlayer;
    use std::process::Stdio;

    // pw-play may not exist in every test env; skip silently if unavailable.
    let probe = std::process::Command::new("pw-play").arg("--help").stdout(Stdio::null()).stderr(Stdio::null()).status();
    let Ok(status) = probe else { return }; // pw-play missing: nothing to test
    if !status.success() {
        return;
    }

    let fifo = "/tmp/aw_player_fifo_test.raw";
    let _ = std::fs::remove_file(fifo);
    let mk = std::process::Command::new("mkfifo").arg(fifo).status();
    assert!(mk.map(|s| s.success()).unwrap_or(false), "mkfifo failed");

    let mut player = AudioPlayer::new();
    // A FIFO never produces EOF, so the player process stays alive until killed.
    assert!(player.play_pwplay_for_test(fifo).is_ok(), "failed to start pw-play on fifo");
    assert!(player.is_playing());

    // The child is alive right now.
    let pid = player.child_pid().expect("child pid");
    assert!(std::process::Command::new("kill").args(["-0", &pid.to_string()]).status().map(|s| s.success()).unwrap_or(false),
        "pw-play {pid} should be alive before shutdown");

    // Shutdown path: must reap the child so audio stops.
    player.stop_for_shutdown();
    assert!(!player.is_playing());

    // Give the OS a moment to reap, then verify the process is really gone.
    std::thread::sleep(std::time::Duration::from_millis(100));
    let alive = std::process::Command::new("kill").args(["-0", &pid.to_string()]).status().map(|s| s.success()).unwrap_or(true);
    assert!(!alive, "pw-play {pid} survived stop_for_shutdown — audio would keep playing");

    let _ = std::fs::remove_file(fifo);
}

fn panic_payload(e: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = e.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = e.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}
