//! Headless UI screenshot tool for AudioWave Studio.
//!
//! Renders the REAL `AppWindow` (ui/*.slint, same build.rs output as the app)
//! to a PNG using Slint's testing backend + software renderer — no display, no
//! Wayland/X11 needed. This is the pixel-accurate replacement for the static
//! HTML mockups used in `docs/parity/`.
//!
//! Usage:
//!   cargo run --example ui_screenshot -- [--out path.png] [--tab style|colors|background|effects|text|export] [--style <id>] [--hover x,y] [--modal export] [--toast]
//!
//! Examples:
//!   cargo run --example ui_screenshot -- --out docs/parity/native_shot.png
//!   cargo run --example ui_screenshot -- --tab colors
//!   cargo run --example ui_screenshot -- --tab style --hover 500,300   # hover state
//!   cargo run --example ui_screenshot -- --modal export
//!   cargo run --example ui_screenshot -- --toast --out /tmp/toast.png
//!   cargo run --example ui_screenshot -- --keys   # verify playback keyboard shortcuts
//!   cargo run --example ui_screenshot -- --empty --out /tmp/empty.png   # onboarding empty state
//!   cargo run --example ui_screenshot -- --drop --out /tmp/drop.png     # drag & drop highlight
//!   cargo run --example ui_screenshot -- --panel-collapsed --out /tmp/collapsed.png  # panel closed

use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::platform::WindowEvent;
use std::rc::Rc;

// Brings `row_count` / `row_data` into scope for the toast dismiss handler.
use slint::Model;

// Include the exact same generated UI module as the app (build.rs output).
slint::include_modules!();

fn parse_args() -> (String, String, String, Option<(f32, f32)>, Option<String>, bool, bool, bool, bool, bool) {
    let mut out = "docs/parity/native_shot.png".to_string();
    let mut tab = "style".to_string();
    let mut style = "spectrum".to_string();
    let mut hover: Option<(f32, f32)> = None;
    let mut modal: Option<String> = None;
    let mut toast = false;
    let mut keys = false;
    let mut empty = false;
    let mut drop = false;
    let mut panel_collapsed = false;

    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--out" => out = it.next().unwrap_or(out),
            "--tab" => tab = it.next().unwrap_or(tab),
            "--style" => style = it.next().unwrap_or(style),
            "--hover" => {
                let v = it.next().unwrap_or_default();
                if let Some((x, y)) = v.split_once(',') {
                    if let (Ok(x), Ok(y)) = (x.trim().parse(), y.trim().parse()) {
                        hover = Some((x, y));
                    }
                }
            }
            "--modal" => modal = it.next(),
            "--toast" => toast = true,
            "--keys" => keys = true,
            "--empty" => empty = true,
            "--drop" => drop = true,
            "--panel-collapsed" => panel_collapsed = true,
            _ => {}
        }
    }
    (out, tab, style, hover, modal, toast, keys, empty, drop, panel_collapsed)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (out, tab, style, hover, modal, toast, keys, empty, drop, panel_collapsed) = parse_args();

    // Headless platform: software rasterizer so take_snapshot() produces real
    // pixels. The mock clock is only enabled for --toast (we need to advance
    // time to fire the ToastCard reveal timers + entrance animation). The
    // threaded event loop is only enabled for --keys (dispatch_event queues the
    // events and the loop drains them).
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: toast,
        threading: keys,
        renderer_name: Some("software".into()),
    })))
    .map_err(|e| format!("set_platform failed: {e}"))?;

    let app = AppWindow::new()?;

    // Seed state so the screenshot shows a realistic UI.
    app.set_active_tab(tab.into());
    app.set_style_val(style.into());
    app.set_track_title("Test Track - CredibleMark".into());
    app.set_track_artist("AudioWave Studio".into());
    app.set_show_export_modal(modal.as_deref() == Some("export"));
    app.set_show_hardware_modal(modal.as_deref() == Some("hardware"));
    app.set_show_about_modal(modal.as_deref() == Some("about"));

    // Onboarding empty state / drag-drop highlight states (has-track defaults
    // to false). For --drop, also wire + fire the audio-file-dropped callback
    // to prove the Rust binding exists and receives the dropped path.
    if empty {
        // has-track stays false: the EmptyStateOverlay shows over the canvas.
        println!("empty state: has-track={} drop-hover={}", app.get_has_track(), app.get_drop_hover());
    }
    if drop {
        app.set_drop_hover(true);
        app.on_audio_file_dropped(|p| println!("DROP audio-file-dropped fired with: {p}"));
        app.invoke_audio_file_dropped("song.mp3".into());
        println!("drop state: has-track={} drop-hover={}", app.get_has_track(), app.get_drop_hover());
    }
    if panel_collapsed {
        app.set_panel_open(false);
        println!("panel collapsed: panel-open={}", app.get_panel_open());
    }

    app.show()?;

    // Optional toast stack: seed the real toast-list model with one toast per
    // kind, then advance the mock clock so the ToastCard reveal timers fire and
    // the entrance fade completes before the snapshot. A dismiss handler
    // mirrors the real app's `dismiss_toast` + `sync_toasts` so the full
    // auto-dismiss round-trip (Timer -> dismissed(id) -> model removal) is
    // exercised end-to-end.
    if toast {
        let model = Rc::new(slint::VecModel::from(vec![
            ToastItem {
                id: 1,
                kind: "error".into(),
                text: "Could not open audio file: unsupported codec".into(),
                duration_ms: 6000,
            },
            ToastItem {
                id: 2,
                kind: "success".into(),
                text: "Preset saved to my-preset.awpreset".into(),
                duration_ms: 6000,
            },
            ToastItem {
                id: 3,
                kind: "info".into(),
                text: "Loaded Track.mp3 • 215.3s".into(),
                duration_ms: 6000,
            },
        ]));
        app.set_toast_list(slint::ModelRc::new(model.clone()));

        let dismiss_model = model.clone();
        app.on_toast_dismissed(move |id| {
            for i in (0..dismiss_model.row_count()).rev() {
                if dismiss_model.row_data(i).map(|r| r.id) == Some(id) {
                    dismiss_model.remove(i);
                    println!("dismissed toast id {id} (row {i})");
                }
            }
        });

        // Fire the ToastCard reveal timers (60ms) and let the opacity animation
        // complete: the first advance triggers the reveal (the fade starts at
        // that tick), the second advance finishes the 180ms entrance fade.
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(400));
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(250));

        // Snapshot #1 is taken below (with toasts visible). Then this second
        // block advances past the 6s auto-dismiss durations and snapshots
        // again to prove the toasts disappear.
        if let Ok(png) = std::env::var("TOAST_AFTER_PNG") {
            let buffer = app.window().take_snapshot()?;
            let img = image::RgbaImage::from_raw(
                buffer.width(),
                buffer.height(),
                buffer.as_bytes().to_vec(),
            )
            .ok_or("snapshot buffer size mismatch")?;
            img.save(&png)?;
            println!("saved {png} (toasts visible)");
        }

        // Advance past the 6s auto-dismiss duration (reveal at 400ms restarts
        // the timer with 6000ms -> deadline 6400ms), firing dismissed(id) for
        // every card.
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(6500));
    }

    // Optional keyboard-shortcut verification: bind the playback callbacks to
    // print, seed a fake track (200s, progress 50%), then dispatch each key
    // inside the running event loop and confirm the right callback fires. Arrow
    // keys arrive as Slint's internal PUA strings (Key.LeftArrow = "\u{F702}",
    // Right = "\u{F703}", Up = "\u{F700}", Down = "\u{F701}").
    if keys {
        app.set_duration_sec(200.0);
        app.set_playback_progress(0.5);
        app.on_toggle_play_clicked(|| println!("KEY toggle-play fired"));
        app.on_stop_clicked(|| println!("KEY stop fired"));
        app.on_mute_toggle_clicked(|| println!("KEY mute fired"));
        app.on_seek_position(|pct| println!("KEY seek fired pct={pct:.4}"));
        app.on_volume_changed(|v| println!("KEY volume fired vol={v:.2}"));

        let app_weak = app.as_weak();
        let dispatcher = slint::Timer::default();
        dispatcher.start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(50),
            move || {
                let Some(a) = app_weak.upgrade() else { return; };
                let key = |text: &str| {
                    a.window()
                        .dispatch_event(WindowEvent::KeyPressed { text: text.into() });
                    a.window()
                        .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
                };
                key(" "); // expect toggle-play
                key("s"); // expect stop
                key("S"); // expect stop (shift)
                key("m"); // expect mute
                key("M"); // expect mute (shift)
                key("\u{F702}"); // left arrow -> seek 0.475
                key("\u{F703}"); // right arrow -> seek 0.525
                key("\u{F700}"); // up arrow -> volume 0.85
                key("\u{F701}"); // down arrow -> volume 0.80
                let _ = slint::quit_event_loop();
            },
        );
        slint::run_event_loop()?;
        return Ok(());
    }

    // Optional hover: move the pointer to (x,y) so hover micro-interactions
    // (scale + glow on cards/tabs/icon buttons) show up in the render.
    if let Some((x, y)) = hover {
        app.window().dispatch_event(WindowEvent::PointerMoved {
            position: slint::LogicalPosition::new(x as _, y as _),
        });
    }

    // Force a layout + render pass, then grab the pixels.
    app.window().request_redraw();
    let buffer = app.window().take_snapshot()?;

    let w = buffer.width();
    let h = buffer.height();
    let bytes = buffer.as_bytes();
    let img = image::RgbaImage::from_raw(w, h, bytes.to_vec())
        .ok_or("snapshot buffer size mismatch")?;

    img.save(&out)?;
    println!("saved {out} ({w}x{h})");
    Ok(())
}
