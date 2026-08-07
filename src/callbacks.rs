use crate::app_state::{format_time, SlintAppState, ToastKind};
use crate::audio_decoder::AudioData;
use crate::config::{AspectRatio, ExportFormat, ExportResolution, ScreenEffect, VisualizerStyle};
use rfd::FileDialog;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// Brings `row_count` / `row_data` / `set_row_data` into scope for the toast
// VecModel sync (methods on the `Model` trait).
use slint::Model;

/// Run a callback body inside a panic guard so a single bad interaction (e.g.
/// a file dialog failure or subprocess hiccup) can never abort the process.
fn guarded<F: FnOnce()>(what: &'static str, f: F) {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    if let Err(e) = r {
        let msg = if let Some(s) = e.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown".to_string()
        };
        eprintln!("[Callback:{what}] recovered from panic: {msg}");
    }
}

/// Audio extensions accepted by both the Open File dialog filter and the
/// drag & drop pre-check (kept in ONE place so they cannot drift).
const SUPPORTED_AUDIO_EXTS: &[&str] = &["mp3", "wav", "flac", "ogg", "m4a", "aac"];

// ---------------------------------------------------------------------------
// ComboBox label <-> serde-id mapping
//
// The UI ComboBoxes now show READABLE labels ("Chromatic Aberration", "Custom
// Image"...) instead of raw serde ids ("chromatic", "customImage"...). The
// UI property holds the LABEL; Rust maps label <-> raw id at the config
// boundary so presets/files keep their original serde ids untouched.
// ---------------------------------------------------------------------------

/// (display label, raw serde id) pairs for the Background Mode combo.
const BG_MODE_PAIRS: &[(&str, &str)] = &[
    ("Solid", "solid"),
    ("Gradient", "gradient"),
    ("Custom Image", "customImage"),
    ("Grid", "grid"),
    ("Aurora", "aurora"),
    ("Noise", "noise"),
    ("Bokeh", "bokeh"),
    ("Starfield", "starfield"),
    ("Nebula", "nebula"),
    ("Psychedelic", "psychedelic"),
];

/// (display label, raw serde id) pairs for the Screen Effect combo.
const MAIN_EFFECT_PAIRS: &[(&str, &str)] = &[
    ("None", "none"),
    ("Shake", "shake"),
    ("Glitch", "glitch"),
    ("Vignette", "vignette"),
    ("Pulse", "pulse"),
    ("Spotlight", "spotlight"),
    ("Strobe", "strobe"),
    ("Scanline", "scanline"),
    ("Chromatic Aberration", "chromatic"),
    ("Zoom", "zoom"),
    ("Invert", "invert"),
    ("Bars", "bars"),
    ("Shockwave", "shockwave"),
    ("Pixelate", "pixelate"),
    ("Tilt", "tilt"),
    ("Heat Haze", "heatHaze"),
    ("Hue Shift", "hueShift"),
];

/// (display label, raw serde id) pairs for the theme preset combo.
const THEME_PAIRS: &[(&str, &str)] = &[
    ("Cyberpunk", "cyberpunk"),
    ("Synthwave", "synthwave"),
    ("Emerald", "emerald"),
    ("Violet", "violet"),
    ("Gold", "gold"),
    ("Custom", "custom"),
];

/// (display label, raw value) pairs for the export encoder combo.
const ENCODER_PAIRS: &[(&str, &str)] = &[
    ("Auto", "auto"),
    ("H.264", "h264"),
    ("HEVC (H.265)", "hevc"),
    ("AV1", "av1"),
];

/// (display label, raw serde id) pairs for particle movement style combo.
const PARTICLE_STYLE_PAIRS: &[(&str, &str)] = &[
    ("Float", "float"),
    ("Bounce", "bounce"),
    ("Wave", "wave"),
    ("Static", "static"),
    ("Confined", "confined"),
];

/// (display label, raw serde id) pairs for music note movement style combo.
const MUSIC_NOTE_STYLE_PAIRS: &[(&str, &str)] = &[
    ("Float", "float"),
    ("Bounce", "bounce"),
    ("Spiral", "spiral"),
    ("Wave", "wave"),
    ("Burst", "burst"),
    ("Confined", "confined"),
];

/// Label (ComboBox entry) -> raw serde id. Unknown labels pass through
/// unchanged so stale values never break the round-trip.
fn label_to_id(label: &str, pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .find(|(l, _)| *l == label)
        .map(|(_, id)| id.to_string())
        .unwrap_or_else(|| label.to_string())
}

/// Raw serde id -> display label (ComboBox entry). Unknown ids pass through
/// unchanged so legacy presets still display something sensible.
fn id_to_label(id: &str, pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .find(|(_, i)| *i == id)
        .map(|(l, _)| l.to_string())
        .unwrap_or_else(|| id.to_string())
}

#[cfg(test)]
mod label_mapping_tests {
    use super::*;

    #[test]
    fn label_id_round_trip_for_all_combo_tables() {
        for pairs in [BG_MODE_PAIRS, MAIN_EFFECT_PAIRS, THEME_PAIRS, ENCODER_PAIRS] {
            for (label, id) in pairs {
                // label -> raw id -> label must be lossless (each pair is 1:1)
                assert_eq!(label_to_id(label, pairs), *id, "label {label} maps to wrong id");
                assert_eq!(id_to_label(id, pairs), *label, "id {id} maps to wrong label");
            }
        }
    }

    #[test]
    fn unknown_values_pass_through_safely() {
        // Stale/legacy values must survive the round-trip untouched so old
        // presets never crash or silently reset.
        assert_eq!(label_to_id("unknownLabel", BG_MODE_PAIRS), "unknownLabel");
        assert_eq!(id_to_label("legacyId", BG_MODE_PAIRS), "legacyId");
        assert_eq!(label_to_id("", MAIN_EFFECT_PAIRS), "");
        assert_eq!(id_to_label("", THEME_PAIRS), "");
    }
}

// ---------------------------------------------------------------------------
// Toast pipeline
//
// `push_toast` appends to the in-state queue and immediately mirrors it into
// the Slint `toast-list` model. `sync_toasts` mutates the SAME VecModel in
// place (rows updated / added / removed) so Slint reuses the existing ToastCard
// instances — existing toasts keep their entrance animation + dismiss timers
// instead of being recreated (and blinking) on every push/dismiss.
//
// The model lives in a thread_local (NOT in SlintAppState): `Rc` is !Send, and
// SlintAppState crosses threads (export thread, invoke_from_event_loop), so
// storing it there would break `Arc<Mutex<SlintAppState>>: Send`. Every toast
// mutation happens on the UI thread (callbacks, the 60 FPS timer, and
// invoke_from_event_loop), so a thread_local is safe.
// ---------------------------------------------------------------------------

thread_local! {
    /// Stable Slint model mirroring `SlintAppState.toasts` — kept alive and
    /// mutated in place so existing toast cards never replay their entrance
    /// animation.
    static TOAST_MODEL: RefCell<Option<Rc<slint::VecModel<crate::ToastItem>>>> =
        RefCell::new(None);
}

/// Mirror `state.toasts` into the Slint toast model, creating the model on
/// first use. Evictions drop rows from the FRONT so a new toast still gets a
/// fresh card (with entrance animation) when the queue is full.
pub(crate) fn sync_toasts(w: &crate::AppWindow, s: &mut SlintAppState) {
    let model = match TOAST_MODEL.with(|slot| slot.borrow().as_ref().cloned()) {
        Some(m) => m,
        None => {
            let m = Rc::new(slint::VecModel::<crate::ToastItem>::from(Vec::new()));
            TOAST_MODEL.with(|slot| *slot.borrow_mut() = Some(m.clone()));
            w.set_toast_list(m.clone().into());
            m
        }
    };

    let old_len = model.row_count();
    if old_len > s.toasts.len() {
        // Dismissals / overflow shrink the queue: drop rows from the FRONT.
        for _ in 0..(old_len - s.toasts.len()) {
            model.remove(0);
        }
    } else if old_len > 0 && old_len == s.toasts.len() {
        // Queue overflow while FULL: push_toast evicted the OLDEST toast and
        // appended a new one, so the row count stayed the same but the front id
        // changed. Drop the front row so the incoming toast gets a fresh card
        // (entrance animation) instead of being written into the evicted card.
        let front_changed = match model.row_data(0) {
            Some(cur) => cur.id != s.toasts.front().map(|t| t.id as i32).unwrap_or(-1),
            None => true,
        };
        if front_changed {
            model.remove(0);
        }
    }

    for (i, t) in s.toasts.iter().enumerate() {
        let item = crate::ToastItem {
            id: t.id as i32,
            kind: t.kind.as_str().into(),
            text: t.text.clone().into(),
            // Slint's `duration` struct fields are i64 milliseconds.
            duration_ms: t.duration_ms as i64,
        };
        if i < model.row_count() {
            if model.row_data(i) != Some(item.clone()) {
                model.set_row_data(i, item);
            }
        } else {
            model.push(item);
        }
    }
}

/// Validate a CSS-style hex color: optional `#`, then 3 or 6 hex digits
/// (optionally allowing an 8-digit `#RRGGBBAA` form). Returns false for any
/// malformed input so invalid theme/background colors warn the user early.
fn is_valid_hex(raw: &str) -> bool {
    let h = raw.trim().trim_start_matches('#');
    let len = h.len();
    if len != 3 && len != 6 && len != 8 {
        return false;
    }
    h.chars()
        .all(|c| c.is_ascii_hexdigit())
}

/// Push a toast and sync it to the UI in one step.
pub(crate) fn push_toast(
    w: &crate::AppWindow,
    s: &mut SlintAppState,
    kind: ToastKind,
    text: impl Into<String>,
) {
    s.push_toast(kind, text);
    sync_toasts(w, s);
}

/// Shared audio-loading path used by BOTH the Open File button and the
/// canvas drag & drop (audio-file-dropped callback): decode the file, load it
/// into the player, sync the UI track fields, and toast the outcome.
pub(crate) fn load_audio_from_path(
    window_weak: &slint::Weak<crate::AppWindow>,
    state: &Arc<Mutex<SlintAppState>>,
    path: std::path::PathBuf,
) {
    let path_str = path.to_string_lossy().to_string();
    match AudioData::decode_file(&path) {
        Ok(data) => {
            let duration = data.duration_seconds;
            let file_name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Track".to_string());

            let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
            let _ = s.audio_player.load_file(&path_str, duration);
            s.audio_data = Some(Arc::new(data));
            s.audio_path = Some(path_str);
            s.config.text.song_title = file_name.clone();

            if let Some(w) = window_weak.upgrade() {
                w.set_track_title(slint::SharedString::from(file_name.clone()));
                w.set_track_artist(slint::SharedString::from(format!(
                    "{:.1}s • Ready",
                    duration
                )));
                w.set_duration_str(slint::SharedString::from(format_time(duration)));
                w.set_duration_sec(duration as f32); // arrow-seek shortcuts
                w.set_current_time_str(slint::SharedString::from("00:00"));
                w.set_playback_progress(0.0);
                w.set_is_playing(false);
                w.set_has_track(true); // dismiss the onboarding empty state
                push_toast(
                    &w,
                    &mut s,
                    ToastKind::Info,
                    format!("Loaded {file_name} • {:.1}s", duration),
                );
            }
        }
        Err(e) => {
            eprintln!("[Slint] Failed to decode audio file: {}", e);
            if let Some(w) = window_weak.upgrade() {
                push_toast(
                    &w,
                    &mut state.lock().unwrap_or_else(|e| e.into_inner()),
                    ToastKind::Error,
                    format!("Could not open audio file: {e}"),
                );
            }
        }
    }
}

pub fn bind_app_callbacks(
    window: &crate::AppWindow,
    preview: Option<&crate::PreviewWindow>,
    state: Arc<Mutex<SlintAppState>>,
) {
    // BIND CALLBACK: OPEN FILE
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_open_file_clicked(move || {
        guarded("open_file", || {
            let file = FileDialog::new()
                .add_filter("Audio Files", SUPPORTED_AUDIO_EXTS)
                .pick_file();

            if let Some(path) = file {
                load_audio_from_path(&window_handle, &state_clone, path);
            }
        });
    });

    // BIND CALLBACK: AUDIO FILE DROPPED (OS drag & drop onto the canvas — the
    // winit CustomApplicationHandler in lib.rs routes HoveredFile/DroppedFile
    // into this callback). Extension pre-check gives a friendlier message than
    // a raw decoder error for folders / unsupported files; AudioData::decode
    // still validates the content of anything with an audio extension.
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_audio_file_dropped(move |path_str| {
        guarded("audio_file_dropped", || {
            let path = std::path::PathBuf::from(path_str.as_str());
            let ext_ok = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| SUPPORTED_AUDIO_EXTS.iter().any(|x| x.eq_ignore_ascii_case(e)))
                .unwrap_or(false);
            if !ext_ok {
                if let Some(w) = window_handle.upgrade() {
                    push_toast(
                        &w,
                        &mut state_clone.lock().unwrap_or_else(|e| e.into_inner()),
                        ToastKind::Error,
                        "Unsupported file — drop an audio file (MP3, WAV, FLAC, OGG, M4A, AAC)",
                    );
                }
                return;
            }
            load_audio_from_path(&window_handle, &state_clone, path);
        });
    });

    // BIND CALLBACK: TOGGLE PLAY
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_toggle_play_clicked(move || {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        if s.audio_player.is_playing() {
            s.audio_player.pause();
            if let Some(w) = window_handle.upgrade() {
                w.set_is_playing(false);
            }
        } else {
            if s.audio_player.play().is_ok() {
                if let Some(w) = window_handle.upgrade() {
                    w.set_is_playing(true);
                }
            } else if let Some(w) = window_handle.upgrade() {
                push_toast(
                    &w,
                    &mut s,
                    ToastKind::Error,
                    "Playback failed — load a track first",
                );
            }
        }
    });

    // BIND CALLBACK: STOP
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_stop_clicked(move || {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        s.audio_player.stop();
        if let Some(w) = window_handle.upgrade() {
            w.set_is_playing(false);
            w.set_playback_progress(0.0);
            w.set_current_time_str(slint::SharedString::from("00:00"));
        }
    });

    // BIND CALLBACK: SEEK
    let state_clone = state.clone();
    window.on_seek_position(move |pct| {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        let dur = s.audio_player.get_duration_sec();
        if dur > 0.0 {
            s.audio_player.seek(pct as f64 * dur);
        }
    });

    // BIND CALLBACK: VOLUME
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_volume_changed(move |vol| {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        s.audio_player.set_volume(vol);
        s.is_muted = vol <= 0.001;
        s.prev_volume = if vol > 0.001 { vol } else { s.prev_volume };
        if let Some(w) = window_handle.upgrade() {
            w.set_is_muted(vol <= 0.001);
            w.set_volume(vol);
        }
    });

    // BIND CALLBACK: MUTE TOGGLE
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_mute_toggle_clicked(move || {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        if s.is_muted {
            let restore = s.prev_volume;
            s.is_muted = false;
            s.audio_player.set_volume(restore);
            if let Some(w) = window_handle.upgrade() {
                w.set_is_muted(false);
                w.set_volume(restore);
            }
        } else {
            s.is_muted = true;
            s.audio_player.set_volume(0.0);
            if let Some(w) = window_handle.upgrade() {
                w.set_is_muted(true);
                w.set_volume(0.0);
            }
        }
    });

    // BIND CALLBACK: LISTEN (live mic capture)
    // (The CredibleMark ticker click is wired directly in app_window.slint to open
    // the About modal, mirroring legacy behavior.)
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_listen_clicked(move || {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        match s.start_listen() {
            Ok(status) => {
                if let Some(w) = window_handle.upgrade() {
                    w.set_is_listening(true);
                    w.set_listen_status(slint::SharedString::from(status));
                }
            }
            Err(e) => {
                eprintln!("[Listen] Failed to start mic capture: {}", e);
                if let Some(w) = window_handle.upgrade() {
                    w.set_is_listening(false);
                    w.set_listen_status(slint::SharedString::from(e.clone()));
                    push_toast(
                        &w,
                        &mut s,
                        ToastKind::Error,
                        format!("Mic capture failed: {e}"),
                    );
                }
            }
        }
    });

    // BIND CALLBACK: STOP LISTEN
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_stop_listen_clicked(move || {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        s.stop_listen();
        if let Some(w) = window_handle.upgrade() {
            w.set_is_listening(false);
            w.set_listen_status(slint::SharedString::from(""));
        }
    });

    // MIC DEVICE PICKER: populate the model from `arecord -l` so the navbar can
    // offer a real device choice (mirrors the legacy enumerateDevices picker).
    let devices = crate::app_state::list_mic_devices();
    let labels: Vec<slint::SharedString> = devices
        .iter()
        .map(|d| slint::SharedString::from(d.label.clone()))
        .collect();
    window.set_mic_devices(slint::ModelRc::new(slint::VecModel::from(labels)));
    window.set_mic_device_label(slint::SharedString::from("Default (System)"));
    eprintln!("[Listen] {} capture device(s) detected", devices.len().saturating_sub(1));

    // BIND CALLBACK: MIC DEVICE SELECTED (from the navbar picker)
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_mic_device_selected(move |label| {
        // Enumerate before touching the state lock (arecord -l spawns a subprocess).
        let found = crate::app_state::list_mic_devices()
            .into_iter()
            .find(|d| d.label == label.as_str());
        let Some(dev) = found else {
            // The device disappeared since the picker was built (unplugged):
            // refuse to silently fall back to the system default.
            eprintln!("[Listen] selected device no longer available: {}", label);
            if let Some(w) = window_handle.upgrade() {
                w.set_listen_status(slint::SharedString::from(format!(
                    "Selected device is no longer available: {label}"
                )));
                push_toast(
                    &w,
                    &mut state_clone.lock().unwrap_or_else(|e| e.into_inner()),
                    ToastKind::Error,
                    format!("Selected device is no longer available: {label}"),
                );
            }
            return;
        };
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        s.set_mic_device(&dev.label, &dev.arg);
        let status = s.start_listen();
        if let Some(w) = window_handle.upgrade() {
            w.set_is_listening(s.is_listening);
            match status {
                Ok(st) => w.set_listen_status(slint::SharedString::from(st)),
                Err(e) => w.set_listen_status(slint::SharedString::from(e)),
            }
        }
    });

    // BIND CALLBACK: OPEN CUSTOM BACKGROUND IMAGE
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_open_custom_image_clicked(move || {
        let file = FileDialog::new()
            .add_filter("Image Files", &["png", "jpg", "jpeg", "webp"])
            .pick_file();

        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
            s.config.background.mode = crate::config::BackgroundMode::CustomImage;
            s.config.background.custom_image_uri = Some(path_str);
            if let Some(w) = window_handle.upgrade() {
                w.set_bg_mode(slint::SharedString::from("Custom Image"));
                push_toast(
                    &w,
                    &mut s,
                    ToastKind::Success,
                    format!("Custom background image loaded: {filename}"),
                );
            }
        }
    });

    // BIND CALLBACK: REMOVE CUSTOM BACKGROUND IMAGE
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_remove_custom_image_clicked(move || {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        s.config.background.custom_image_uri = None;
        s.config.background.mode = crate::config::BackgroundMode::Solid;
        if let Some(w) = window_handle.upgrade() {
            w.set_bg_mode(slint::SharedString::from("Solid"));
            push_toast(
                &w,
                &mut s,
                ToastKind::Info,
                "Custom background image removed",
            );
        }
    });

    // BIND CALLBACK: OPEN RADIAL CENTER IMAGE
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_open_radial_image_clicked(move || {
        let file = FileDialog::new()
            .add_filter("Image Files", &["png", "jpg", "jpeg", "webp"])
            .pick_file();

        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
            s.config.background.radial_center_image_uri = Some(path_str.clone());
            if let Some(w) = window_handle.upgrade() {
                w.set_radial_center_image_uri(slint::SharedString::from(path_str));
            }
        }
    });

    // BIND CALLBACK: REMOVE RADIAL CENTER IMAGE
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_remove_radial_image_clicked(move || {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        s.config.background.radial_center_image_uri = None;
        if let Some(w) = window_handle.upgrade() {
            w.set_radial_center_image_uri(slint::SharedString::from(""));
        }
    });

    // BIND CALLBACK: THEME PRESET SELECTED
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_theme_preset_selected(move |preset| {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        // ComboBox now sends the readable LABEL ("Synthwave"...); Cyberpunk is
        // the default for anything unknown (incl. "Custom", which keeps its
        // currently applied colors instead of re-applying a preset).
        let (p, sec, acc, glow) = match preset.as_str() {
            "Synthwave" => ("#ff71ce", "#01cdfe", "#05ffa1", "#ff71ce"),
            "Emerald" => ("#10b981", "#059669", "#34d399", "#10b981"),
            "Violet" => ("#8b5cf6", "#ec4899", "#a855f7", "#8b5cf6"),
            "Gold" => ("#f59e0b", "#ef4444", "#fbbf24", "#f59e0b"),
            _ => ("#00f0ff", "#ff007f", "#7928ca", "#00f0ff"),
        };
        s.config.theme.primary_color = p.to_string();
        s.config.theme.secondary_color = sec.to_string();
        s.config.theme.accent_color = acc.to_string();
        s.config.theme.glow_color = glow.to_string();

        if let Some(w) = window_handle.upgrade() {
            w.set_primary_color(slint::SharedString::from(p));
            w.set_secondary_color(slint::SharedString::from(sec));
            w.set_accent_color(slint::SharedString::from(acc));
            w.set_glow_color(slint::SharedString::from(glow));
        }
    });

    // BIND CALLBACK: SAVE PRESET
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_save_preset_clicked(move || {
        let config = state_clone.lock().unwrap_or_else(|e| e.into_inner()).config.clone();
        let path = FileDialog::new()
            .set_file_name("my-preset.awpreset")
            .add_filter("AudioWave Preset", &["awpreset"])
            .save_file();
        if let Some(p) = path {
            let result = serde_json::to_string_pretty(&config)
                .map(|json| std::fs::write(&p, json));
            if let Some(w) = window_handle.upgrade() {
                match result {
                    Ok(Ok(())) => {
                        push_toast(
                            &w,
                            &mut state_clone.lock().unwrap_or_else(|e| e.into_inner()),
                            ToastKind::Success,
                            format!("Preset saved to {}", p.display()),
                        );
                    }
                    Ok(Err(e)) => {
                        eprintln!("[Preset] Failed to save preset: {}", e);
                        push_toast(
                            &w,
                            &mut state_clone.lock().unwrap_or_else(|e| e.into_inner()),
                            ToastKind::Error,
                            format!("Failed to save preset: {e}"),
                        );
                    }
                    Err(e) => {
                        eprintln!("[Preset] Failed to serialize config: {}", e);
                        push_toast(
                            &w,
                            &mut state_clone.lock().unwrap_or_else(|e| e.into_inner()),
                            ToastKind::Error,
                            "Failed to save preset (config serialization error)",
                        );
                    }
                }
            }
        }
    });

    // BIND CALLBACK: LOAD PRESET
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_load_preset_clicked(move || {
        let file = FileDialog::new()
            .add_filter("AudioWave Preset", &["awpreset"])
            .pick_file();
        if let Some(path) = file {
            match std::fs::read_to_string(&path) {
                Ok(json) => match serde_json::from_str::<crate::config::VisualizerConfig>(&json) {
                    Ok(loaded) => {
                        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
                        s.config = loaded.clone();
                        if let Some(w) = window_handle.upgrade() {
                            sync_ui_from_config(&w, &s.config);
                            push_toast(
                                &w,
                                &mut s,
                                ToastKind::Success,
                                "Preset loaded",
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[Preset] Failed to parse preset: {}", e);
                        if let Some(w) = window_handle.upgrade() {
                            push_toast(
                                &w,
                                &mut state_clone.lock().unwrap_or_else(|e| e.into_inner()),
                                ToastKind::Error,
                                "Failed to load preset (invalid file)",
                            );
                        }
                    }
                },
                Err(e) => {
                    eprintln!("[Preset] Failed to read preset file: {}", e);
                    if let Some(w) = window_handle.upgrade() {
                        push_toast(
                            &w,
                            &mut state_clone.lock().unwrap_or_else(|e| e.into_inner()),
                            ToastKind::Error,
                            format!("Failed to read preset file: {e}"),
                        );
                    }
                }
            }
        }
    });

    // BIND CALLBACK: CONFIG CHANGED
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_config_changed(move || {
        if let Some(w) = window_handle.upgrade() {
            let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
            if let Ok(st) = serde_json::from_str::<VisualizerStyle>(&format!(
                "\"{}\"",
                w.get_style_val()
            )) {
                s.config.style = st;
            }
            s.config.reactivity.bar_count = w.get_bar_count() as usize;
            s.config.reactivity.bar_gap = w.get_bar_gap();
            s.config.reactivity.bar_width = w.get_bar_width();
            s.config.reactivity.bar_rounding = w.get_bar_rounding();
            s.config.reactivity.mirror_bars = w.get_mirror_bars();
            s.config.reactivity.sensitivity = w.get_sensitivity();
            s.config.reactivity.smoothing = w.get_smoothing();
            s.config.reactivity.bass_multiplier = w.get_bass_multiplier();
            s.config.reactivity.show_peaks = w.get_show_peaks();
            s.config.reactivity.peak_color = w.get_peak_color().to_string();
            s.config.reactivity.fire_width_ratio = Some(w.get_fire_width_ratio());
            s.config.reactivity.fire_height_scale = Some(w.get_fire_height_scale());
            s.config.scale = w.get_visualizer_scale();
            s.config.position_x = w.get_position_x();
            s.config.position_y = w.get_position_y();

            let theme_colors = [
                ("primary", w.get_primary_color()),
                ("secondary", w.get_secondary_color()),
                ("accent", w.get_accent_color()),
                ("glow", w.get_glow_color()),
            ];
            for (label, c) in theme_colors {
                if !is_valid_hex(c.as_str()) {
                    push_toast(
                        &w,
                        &mut s,
                        ToastKind::Error,
                        format!("{label} color is not a valid hex value (\"{c}\"). Expected #RGB or #RRGGBB."),
                    );
                }
            }

            s.config.theme.primary_color = w.get_primary_color().to_string();
            s.config.theme.secondary_color = w.get_secondary_color().to_string();
            s.config.theme.accent_color = w.get_accent_color().to_string();
            s.config.theme.glow_color = w.get_glow_color().to_string();

            let bg_mode_id = label_to_id(w.get_bg_mode().as_str(), BG_MODE_PAIRS);
            if let Ok(bg_mode) = serde_json::from_str::<crate::config::BackgroundMode>(&format!(
                "\"{bg_mode_id}\""
            )) {
                s.config.background.mode = bg_mode;
            }
            s.config.background.solid_color = w.get_solid_color().to_string();
            s.config.background.gradient_start = w.get_gradient_start().to_string();
            s.config.background.gradient_end = w.get_gradient_end().to_string();
            s.config.background.overlay_opacity = w.get_overlay_opacity();
            s.config.background.image_opacity = Some(w.get_image_opacity());
            s.config.background.show_particles = w.get_show_particles();
            s.config.background.show_music_notes = Some(w.get_show_music_notes());

            s.config.background.grid_size = Some(w.get_grid_size());
            s.config.background.grid_line_width = Some(w.get_grid_line_width());
            s.config.background.grid_color = Some(w.get_grid_color().to_string());

            let p_style_id = label_to_id(w.get_particle_style().as_str(), PARTICLE_STYLE_PAIRS);
            if let Ok(st) = serde_json::from_str::<crate::config::ParticleStyle>(&format!("\"{p_style_id}\"")) {
                s.config.background.particle_style = Some(st);
            }

            let m_style_id = label_to_id(w.get_music_note_style().as_str(), MUSIC_NOTE_STYLE_PAIRS);
            if let Ok(st) = serde_json::from_str::<crate::config::MusicNoteStyle>(&format!("\"{m_style_id}\"")) {
                s.config.background.music_note_style = Some(st);
            }

            s.config.background.particle_size = Some(w.get_particle_size());
            s.config.background.particle_speed = Some(w.get_particle_speed());
            s.config.background.particle_count = Some(w.get_particle_count() as u32);
            s.config.background.particle_color = w.get_particle_color().to_string();

            s.config.background.music_note_size = Some(w.get_music_note_size());
            s.config.background.music_note_count = Some(w.get_music_note_count() as u32);
            s.config.background.music_note_sensitivity = Some(w.get_music_note_sensitivity());
            s.config.background.music_note_color = Some(w.get_music_note_color().to_string());

            s.config.background.star_count = Some(w.get_star_count() as u32);
            s.config.background.star_speed = Some(w.get_star_speed());
            s.config.background.star_brightness = Some(w.get_star_brightness());

            s.config.background.nebula_intensity = Some(w.get_nebula_intensity());
            s.config.background.nebula_speed = Some(w.get_nebula_speed());

            s.config.background.aurora_speed = Some(w.get_aurora_speed());
            s.config.background.aurora_amplitude = Some(w.get_aurora_amplitude());
            s.config.background.aurora_opacity = Some(w.get_aurora_opacity());

            s.config.background.grain_opacity = Some(w.get_grain_opacity());

            s.config.background.bokeh_count = Some(w.get_bokeh_count() as u32);
            s.config.background.bokeh_size = Some(w.get_bokeh_size());
            s.config.background.bokeh_opacity = Some(w.get_bokeh_opacity());

            s.config.background.psychedelic_speed = Some(w.get_psychedelic_speed());
            s.config.background.psychedelic_bands = Some(w.get_psychedelic_bands() as u32);
            s.config.background.psychedelic_line_width = Some(w.get_psychedelic_line_width());

            // Screen effects
            let fx_id = label_to_id(w.get_main_effect().as_str(), MAIN_EFFECT_PAIRS);
            if let Ok(fx) = serde_json::from_str::<ScreenEffect>(&format!("\"{fx_id}\"")) {
                s.config.screen_effects.main_effect = fx;
            }
            s.config.screen_effects.enabled = w.get_effects_enabled();
            s.config.screen_effects.shake_intensity = w.get_shake_intensity();
            s.config.screen_effects.shake_frequency = w.get_shake_frequency();
            s.config.screen_effects.shake_max_offset = w.get_shake_max_offset();
            s.config.screen_effects.shake_on_beat = w.get_shake_on_beat();
            s.config.screen_effects.glitch_intensity = w.get_glitch_intensity();
            s.config.screen_effects.pulse_intensity = w.get_pulse_intensity();
            s.config.screen_effects.strobe_intensity = w.get_strobe_intensity();
            s.config.screen_effects.scanline_opacity = w.get_scanline_opacity();
            s.config.screen_effects.chromatic_intensity = w.get_chromatic_intensity();
            s.config.screen_effects.zoom_intensity = w.get_zoom_intensity();
            s.config.screen_effects.invert_intensity = w.get_invert_intensity();
            s.config.screen_effects.bars_amount = w.get_bars_amount();
            s.config.screen_effects.shockwave_intensity = w.get_shockwave_intensity();
            s.config.screen_effects.pixelate_intensity = w.get_pixelate_intensity();
            s.config.screen_effects.tilt_intensity = w.get_tilt_intensity();
            s.config.screen_effects.heat_haze_intensity = w.get_heat_haze_intensity();
            s.config.screen_effects.hue_shift_intensity = w.get_hue_shift_intensity();
            s.config.screen_effects.background_only = Some(w.get_bg_only_effect());

            // Export settings
            if let Ok(ar) = serde_json::from_str::<AspectRatio>(&format!(
                "\"{}\"",
                w.get_export_aspect_ratio()
            )) {
                s.config.export.aspect_ratio = ar;
            }
            if let Ok(res) = serde_json::from_str::<ExportResolution>(&format!(
                "\"{}\"",
                w.get_export_resolution()
            )) {
                s.config.export.resolution = res;
            }
            s.config.export.fps = w.get_export_fps() as u32;
            if let Ok(fmt) = serde_json::from_str::<ExportFormat>(&format!(
                "\"{}\"",
                w.get_export_format()
            )) {
                s.config.export.format = fmt;
            }
            s.config.export.encoder = Some(label_to_id(w.get_export_encoder().as_str(), ENCODER_PAIRS));
            s.config.reactivity.fft_size = w.get_export_fft_size() as usize;

            s.config.text.song_title = w.get_track_title().to_string();
            s.config.text.artist_name = w.get_track_artist().to_string();
            s.config.text.cassette_label = w.get_cassette_label().to_string();
            s.config.text.title.text = w.get_track_title().to_string();
            s.config.text.artist.text = w.get_track_artist().to_string();
            s.config.text.show_title = w.get_show_title();
            s.config.text.show_artist = w.get_show_artist();

            s.config.text.title.enabled = true;
            s.config.text.title.opacity = 1.0;
            s.config.text.title.position_x = w.get_title_pos_x();
            s.config.text.title.position_y = w.get_title_pos_y();
            s.config.text.title.font_size = w.get_title_font_size();
            s.config.text.title.color = w.get_title_color().to_string();

            s.config.text.artist.enabled = true;
            s.config.text.artist.opacity = 1.0;
            s.config.text.artist.position_x = w.get_artist_pos_x();
            s.config.text.artist.position_y = w.get_artist_pos_y();
            s.config.text.artist.font_size = w.get_artist_font_size();
            s.config.text.artist.color = w.get_artist_color().to_string();
        }
    });

    // BIND CALLBACK: TOAST DISMISS (manual ✕ or auto-dismiss timer)
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_toast_dismissed(move |id| {
        let mut s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
        if s.dismiss_toast(id as u64) {
            if let Some(w) = window_handle.upgrade() {
                sync_toasts(&w, &mut s);
            }
        }
    });

    // BIND CALLBACK: START EXPORT
    let state_clone = state.clone();
    let window_handle = window.as_weak();
    window.on_start_export_clicked(move || {
        guarded("start_export", || {
        let (audio_path, mut config) = {
            let s = state_clone.lock().unwrap_or_else(|e| e.into_inner());
            (s.audio_path.clone(), s.config.clone())
        };

        // Always pull the live export settings from the UI so choices made in
        // the ExportModal (which never fires config-changed) reach the export.
        if let Some(w) = window_handle.upgrade() {
            if let Ok(ar) = serde_json::from_str::<AspectRatio>(&format!(
                "\"{}\"",
                w.get_export_aspect_ratio()
            )) {
                config.export.aspect_ratio = ar;
            }
            if let Ok(res) = serde_json::from_str::<ExportResolution>(&format!(
                "\"{}\"",
                w.get_export_resolution()
            )) {
                config.export.resolution = res;
            }
            config.export.fps = w.get_export_fps() as u32;
            if let Ok(fmt) = serde_json::from_str::<ExportFormat>(&format!(
                "\"{}\"",
                w.get_export_format()
            )) {
                config.export.format = fmt;
            }
            config.export.encoder = Some(label_to_id(w.get_export_encoder().as_str(), ENCODER_PAIRS));
            config.reactivity.fft_size = w.get_export_fft_size() as usize;
        }
        let include_audio = window_handle
            .upgrade()
            .map(|w| w.get_include_audio())
            .unwrap_or(true);

        let audio_path = match audio_path {
            Some(p) => p,
            None => {
                eprintln!("[Export] No audio file loaded to export");
                if let Some(w) = window_handle.upgrade() {
                    push_toast(
                        &w,
                        &mut state_clone.lock().unwrap_or_else(|e| e.into_inner()),
                        ToastKind::Error,
                        "Load an audio file before exporting",
                    );
                }
                return;
            }
        };

        // Match the save dialog to the selected output format.
        let is_webm = matches!(config.export.format, ExportFormat::Webm);
        let (file_name, filter_name, filter_ext) = if is_webm {
            ("visualizer_wave.webm", "WebM Video", "webm")
        } else {
            ("visualizer_wave.mp4", "MP4 Video", "mp4")
        };

        let output_path = match FileDialog::new()
            .set_file_name(file_name)
            .add_filter(filter_name, &[filter_ext])
            .save_file()
        {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };

        if let Some(w) = window_handle.upgrade() {
            w.set_is_exporting(true);
            w.set_export_status_text(slint::SharedString::from("Rendering & encoding MP4..."));
            w.set_export_progress_percent(10.0);
        }

        let window_weak = window_handle.clone();
        let state_for_toast = state_clone.clone();
        std::thread::spawn(move || {
            let res = crate::gpu_export::export_gpu(config, audio_path, output_path.clone(), include_audio);
            slint::invoke_from_event_loop(move || {
                if let Some(w) = window_weak.upgrade() {
                    w.set_is_exporting(false);
                    match res {
                        Ok(_) => {
                            w.set_export_status_text(slint::SharedString::from(format!(
                                "Export Saved: {}",
                                output_path
                            )));
                            w.set_export_progress_percent(100.0);
                            push_toast(
                                &w,
                                &mut state_for_toast.lock().unwrap_or_else(|e| e.into_inner()),
                                ToastKind::Success,
                                "Export complete — video saved",
                            );
                        }
                        Err(e) => {
                            w.set_export_status_text(slint::SharedString::from(format!(
                                "Error: {}",
                                e
                            )));
                            w.set_export_progress_percent(0.0);
                            push_toast(
                                &w,
                                &mut state_for_toast.lock().unwrap_or_else(|e| e.into_inner()),
                                ToastKind::Error,
                                format!("Export failed: {e}"),
                            );
                        }
                    }
                }
            })
            .unwrap();
        });
        });
    });

    // BIND CALLBACK: POP-OUT PREVIEW WINDOW
    // (legacy detachedPreviewService.openDetachedPreview — a second native
    // window showing only the live visualizer, synced by the render timer).
    let preview_weak = preview.map(|p| p.as_weak());
    window.on_toggle_preview_clicked(move || {
        let Some(p) = preview_weak.as_ref().and_then(|w| w.upgrade()) else {
            eprintln!("[Preview] Pop-out preview is not available");
            return;
        };
        if p.window().is_visible() {
            let _ = p.hide();
        } else if let Err(e) = p.show() {
            eprintln!("[Preview] Failed to show preview window: {}", e);
        }
    });

    // BIND CALLBACK: CUSTOM WINDOW CONTROLS
    // (legacy appWindow.minimize() / toggleMaximize() / close())
    let win_weak = window.as_weak();
    window.on_minimize_clicked(move || {
        if let Some(w) = win_weak.upgrade() {
            w.window().set_minimized(true);
        }
    });

    let win_weak = window.as_weak();
    window.on_maximize_clicked(move || {
        if let Some(w) = win_weak.upgrade() {
            let win = w.window();
            win.set_maximized(!win.is_maximized());
        }
    });

    let preview_weak = preview.map(|p| p.as_weak());
    let win_weak = window.as_weak();
    let state_for_close = state.clone();
    window.on_close_clicked(move || {
        // STOP AUDIO + MIC EXPLICITLY before hiding: cleanup must not rely on
        // the Drop chain (event loop end + Arc release). If the player child
        // survived to this point it would keep playing after the window is gone
        // — the reported "audio still playing after the app is closed". Stopping here
        // also covers the force-killed-process case, where Drop never runs.
        let mut s = state_for_close.lock().unwrap_or_else(|e| e.into_inner());
        s.audio_player.stop_for_shutdown();
        s.stop_listen();
        drop(s);

        // Close the pop-out preview too: Slint's event loop only quits when the
        // LAST window is closed, so leaving the preview open would keep the app
        // alive after the user clicked ✕ (legacy Tauri exits the whole app when
        // the main window closes).
        if let Some(p) = preview_weak.as_ref().and_then(|w| w.upgrade()) {
            let _ = p.hide();
        }
        if let Some(w) = win_weak.upgrade() {
            // Hiding the last window ends the Slint event loop (app exits),
            // matching the legacy close button.
            let _ = w.hide();
        }
    });

    // BIND CALLBACK: FRAMELESS WINDOW DRAG
    // (legacy `data-tauri-drag-region` on the whole header). Slint 1.17 has no
    // native begin-move API, so we track the pointer delta and reposition the
    // window ourselves. Works on X11 / Windows / macOS; Wayland ignores
    // set_position (documented Slint limitation) — there the window manager
    // titlebar-less drag rules apply.
    struct DragState {
        start_win: slint::LogicalPosition,
        start_ptr: (f32, f32),
    }
    let drag_state: Rc<RefCell<Option<DragState>>> = Rc::new(RefCell::new(None));

    let win_weak = window.as_weak();
    let drag = drag_state.clone();
    window.on_drag_pressed(move |x, y| {
        if let Some(w) = win_weak.upgrade() {
            let win = w.window();
            let pos = win.position().to_logical(win.scale_factor());
            *drag.borrow_mut() = Some(DragState {
                start_win: pos,
                start_ptr: (x, y),
            });
        }
    });

    let win_weak = window.as_weak();
    let drag = drag_state.clone();
    window.on_drag_moved(move |x, y| {
        if let Some(w) = win_weak.upgrade() {
            if let Some(st) = drag.borrow().as_ref() {
                let dx = x - st.start_ptr.0;
                let dy = y - st.start_ptr.1;
                let new_pos =
                    slint::LogicalPosition::new(st.start_win.x + dx, st.start_win.y + dy);
                w.window().set_position(slint::WindowPosition::Logical(new_pos));
            }
        }
    });

    let drag = drag_state.clone();
    window.on_drag_released(move || {
        *drag.borrow_mut() = None;
    });
}

/// Push a freshly loaded config into the Slint UI properties.
fn sync_ui_from_config(w: &crate::AppWindow, c: &crate::config::VisualizerConfig) {
    // serde renames are the canonical UI ids (e.g. "waveformFill", "customImage").
    w.set_style_val(slint::SharedString::from(serde_json::to_string(&c.style).unwrap_or_else(|_| "\"spectrum\"".into()).trim_matches('"')));
    w.set_bar_count(c.reactivity.bar_count as i32);
    w.set_bar_gap(c.reactivity.bar_gap);
    w.set_bar_width(c.reactivity.bar_width);
    w.set_bar_rounding(c.reactivity.bar_rounding);
    w.set_mirror_bars(c.reactivity.mirror_bars);
    w.set_sensitivity(c.reactivity.sensitivity);
    w.set_smoothing(c.reactivity.smoothing);
    w.set_bass_multiplier(c.reactivity.bass_multiplier);
    w.set_show_peaks(c.reactivity.show_peaks);
    w.set_peak_color(slint::SharedString::from(c.reactivity.peak_color.clone()));
    w.set_visualizer_scale(c.scale);
    w.set_position_x(c.position_x);
    w.set_position_y(c.position_y);

    w.set_theme_name(slint::SharedString::from(id_to_label(
        serde_json::to_string(&c.theme.name)
            .unwrap_or_else(|_| "\"custom\"".into())
            .trim_matches('"'),
        THEME_PAIRS,
    )));
    w.set_primary_color(slint::SharedString::from(c.theme.primary_color.clone()));
    w.set_secondary_color(slint::SharedString::from(c.theme.secondary_color.clone()));
    w.set_accent_color(slint::SharedString::from(c.theme.accent_color.clone()));
    w.set_glow_color(slint::SharedString::from(c.theme.glow_color.clone()));
    w.set_radial_center_image_uri(slint::SharedString::from(
        c.background.radial_center_image_uri.clone().unwrap_or_default(),
    ));

    let bg_mode = serde_json::to_string(&c.background.mode).unwrap_or_else(|_| "\"gradient\"".into());
    w.set_bg_mode(slint::SharedString::from(id_to_label(
        bg_mode.trim_matches('"'),
        BG_MODE_PAIRS,
    )));
    w.set_solid_color(slint::SharedString::from(c.background.solid_color.clone()));
    w.set_gradient_start(slint::SharedString::from(c.background.gradient_start.clone()));
    w.set_gradient_end(slint::SharedString::from(c.background.gradient_end.clone()));
    w.set_overlay_opacity(c.background.overlay_opacity);
    w.set_show_particles(c.background.show_particles);
    w.set_show_music_notes(c.background.show_music_notes.unwrap_or(false));

    let main_effect = serde_json::to_string(&c.screen_effects.main_effect).unwrap_or_else(|_| "\"none\"".into());
    w.set_main_effect(slint::SharedString::from(id_to_label(
        main_effect.trim_matches('"'),
        MAIN_EFFECT_PAIRS,
    )));
    w.set_effects_enabled(c.screen_effects.enabled);
    w.set_shake_intensity(c.screen_effects.shake_intensity);
    w.set_shake_frequency(c.screen_effects.shake_frequency);
    w.set_shake_max_offset(c.screen_effects.shake_max_offset);
    w.set_shake_on_beat(c.screen_effects.shake_on_beat);
    w.set_glitch_intensity(c.screen_effects.glitch_intensity);
    w.set_pulse_intensity(c.screen_effects.pulse_intensity);
    w.set_strobe_intensity(c.screen_effects.strobe_intensity);
    w.set_scanline_opacity(c.screen_effects.scanline_opacity);
    w.set_chromatic_intensity(c.screen_effects.chromatic_intensity);
    w.set_zoom_intensity(c.screen_effects.zoom_intensity);
    w.set_invert_intensity(c.screen_effects.invert_intensity);
    w.set_bars_amount(c.screen_effects.bars_amount);
    w.set_shockwave_intensity(c.screen_effects.shockwave_intensity);
    w.set_pixelate_intensity(c.screen_effects.pixelate_intensity);
    w.set_tilt_intensity(c.screen_effects.tilt_intensity);
    w.set_heat_haze_intensity(c.screen_effects.heat_haze_intensity);
    w.set_hue_shift_intensity(c.screen_effects.hue_shift_intensity);
    w.set_bg_only_effect(c.screen_effects.background_only.unwrap_or(true));

    let aspect = match c.export.aspect_ratio {
        AspectRatio::Widescreen => "16:9",
        AspectRatio::Portrait => "9:16",
        AspectRatio::Square => "1:1",
    };
    let resolution = match c.export.resolution {
        ExportResolution::P1080 => "1080p",
        ExportResolution::P720 => "720p",
        ExportResolution::K4 => "4K",
    };
    let format = match c.export.format {
        ExportFormat::Mp4 => "mp4",
        ExportFormat::Webm => "webm",
    };
    w.set_export_aspect_ratio(slint::SharedString::from(aspect));
    w.set_export_resolution(slint::SharedString::from(resolution));
    w.set_export_fps(c.export.fps as i32);
    w.set_export_format(slint::SharedString::from(format));
    w.set_export_encoder(slint::SharedString::from(id_to_label(
        &c.export.encoder.clone().unwrap_or_else(|| "auto".into()),
        ENCODER_PAIRS,
    )));
    w.set_export_fft_size(c.reactivity.fft_size as i32);
}
