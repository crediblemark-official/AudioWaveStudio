use std::process::{Child, Command, Stdio};
use std::time::Instant;

pub struct AudioPlayer {
    file_path: Option<String>,
    duration_sec: f64,
    start_instant: Option<Instant>,
    accumulated_sec: f64,
    is_playing: bool,
    volume: f32,
    child_process: Option<Child>,
    /// When the playback child was last restarted for a live volume change.
    /// Restarts are throttled so dragging the volume slider does not churn
    /// (kill + re-spawn) the audio process on every tick.
    last_restart: Option<Instant>,
}

/// Minimum spacing between volume-change restarts of the playback child.
const RESTART_THROTTLE: std::time::Duration = std::time::Duration::from_millis(150);

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            file_path: None,
            duration_sec: 0.0,
            start_instant: None,
            accumulated_sec: 0.0,
            is_playing: false,
            volume: 0.8,
            child_process: None,
            last_restart: None,
        }
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPlayer {

    pub fn load_file(&mut self, path: &str, duration_sec: f64) -> Result<(), String> {
        self.stop();
        self.file_path = Some(path.to_string());
        self.duration_sec = duration_sec;
        self.accumulated_sec = 0.0;
        self.start_instant = None;
        self.is_playing = false;
        Ok(())
    }

    pub fn play(&mut self) -> Result<(), String> {
        let file_path = match &self.file_path {
            Some(p) => p.clone(),
            None => return Err("No audio file loaded".to_string()),
        };

        if self.is_playing {
            return Ok(());
        }

        // If the previous playback finished naturally (position sits at the
        // end), pressing play again restarts the track from the beginning.
        if self.duration_sec > 0.0 && self.accumulated_sec >= self.duration_sec {
            self.accumulated_sec = 0.0;
        }

        self.kill_process();

        let start_sec = self.accumulated_sec;
        let vol_pct = (self.volume * 100.0).round() as u32;

        // Launch system audio player process
        let vol_str = format!("{:.2}", self.volume);

        let ffplay_bin = crate::ffmpeg::resolve_ffplay(None).unwrap_or_else(|| "ffplay".to_string());
        let mut ffplay_cmd = Command::new(&ffplay_bin);
        hide_cmd(&mut ffplay_cmd);
        #[cfg(target_os = "linux")]
        ffplay_cmd.env("SDL_AUDIO_DRIVER", "pulse");

        let mut mpv_cmd = Command::new("mpv");
        hide_cmd(&mut mpv_cmd);

        let mut afplay_cmd = Command::new("afplay");
        hide_cmd(&mut afplay_cmd);

        let mut pwplay_cmd = Command::new("pw-play");
        hide_cmd(&mut pwplay_cmd);

        let mut paplay_cmd = Command::new("paplay");
        hide_cmd(&mut paplay_cmd);

        // Preferred backends first: ffplay (bundled/resolved) and mpv, both of
        // which can seek, set volume, and are headless. If neither is available
        // the OS-native fallbacks in `spawn_os_fallback` take over.
        let child = if let Ok(c) = ffplay_cmd
            .arg("-nodisp")
            .arg("-autoexit")
            .arg("-loglevel")
            .arg("quiet")
            .arg("-ss")
            .arg(format!("{:.2}", start_sec))
            .arg("-volume")
            .arg(vol_pct.to_string())
            .arg(&file_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            println!("[AudioPlayer] Playing via ffplay (start_sec={:.2}s): {}", start_sec, file_path);
            Some(c)
        } else if let Ok(c) = mpv_cmd
            .arg("--no-video")
            .arg(format!("--start={:.2}", start_sec))
            .arg(format!("--volume={}", vol_pct))
            .arg(&file_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            println!("[AudioPlayer] Playing via mpv (start_sec={:.2}s): {}", start_sec, file_path);
            Some(c)
        } else {
            spawn_os_fallback(
                afplay_cmd,
                pwplay_cmd,
                paplay_cmd,
                &file_path,
                start_sec,
                &vol_str,
                vol_pct,
            )
        };

        let Some(child) = child else {
            // No audio backend could be spawned — report the failure instead of
            // pretending playback started (the UI must not flip to "⏸" while
            // nothing is actually playing).
            self.child_process = None;
            self.start_instant = None;
            self.is_playing = false;
            return Err(
                "No audio player available — install ffplay or mpv".to_string(),
            );
        };

        self.child_process = Some(child);
        self.start_instant = Some(Instant::now());
        self.is_playing = true;
        Ok(())
    }

    pub fn pause(&mut self) {
        if !self.is_playing {
            return;
        }

        if let Some(instant) = self.start_instant.take() {
            self.accumulated_sec += instant.elapsed().as_secs_f64();
        }

        self.kill_process();
        self.is_playing = false;
    }

    pub fn stop(&mut self) {
        self.kill_process();
        self.is_playing = false;
        self.accumulated_sec = 0.0;
        self.start_instant = None;
    }

    /// Same as [`Self::stop`], but reaps the child more patiently. Used only on
    /// the app-shutdown path (close button / event-loop exit) where a second of
    /// UI-thread blocking is invisible and the player MUST not survive us.
    pub fn stop_for_shutdown(&mut self) {
        self.kill_process_deadline(std::time::Duration::from_millis(1200));
        self.is_playing = false;
        self.accumulated_sec = 0.0;
        self.start_instant = None;
    }

    pub fn seek(&mut self, target_sec: f64) {
        let was_playing = self.is_playing;
        let clamped = target_sec.clamp(0.0, self.duration_sec);
        if !was_playing {
            self.accumulated_sec = clamped;
            self.start_instant = None;
            return;
        }

        let now = Instant::now();
        if let Some(last) = self.last_restart {
            if now.duration_since(last) < RESTART_THROTTLE {
                self.accumulated_sec = clamped;
                return;
            }
        }
        self.last_restart = Some(now);

        self.stop();
        self.accumulated_sec = clamped;
        let _ = self.play();
    }

    pub fn set_volume(&mut self, vol: f32) {
        let new_vol = vol.clamp(0.0, 1.0);
        let changed = (self.volume - new_vol).abs() > 0.001;
        self.volume = new_vol;

        if !changed || !self.is_playing {
            return;
        }

        let now = Instant::now();
        if let Some(last) = self.last_restart {
            if now.duration_since(last) < RESTART_THROTTLE {
                return;
            }
        }
        self.last_restart = Some(now);

        let was_playing = self.is_playing;
        if let Some(instant) = self.start_instant.take() {
            self.accumulated_sec += instant.elapsed().as_secs_f64();
        }
        self.kill_process();
        if was_playing {
            let _ = self.play();
        }
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn get_duration_sec(&self) -> f64 {
        self.duration_sec
    }

    pub fn get_current_time_sec(&mut self) -> f64 {
        if self.is_playing {
            // Detect an unexpectedly dead child process (ffplay/mpv crashed or
            // was killed externally). Without this, the UI would keep showing
            // "playing" with advancing time while no audio is heard.
            let dead = self
                .child_process
                .as_mut()
                .map(|c| matches!(c.try_wait(), Ok(Some(_))))
                .unwrap_or(true);
            if dead {
                // Snapshot the current position so the seek bar freezes at the
                // last heard moment instead of jumping to 0 or duration.
                if let Some(instant) = self.start_instant {
                    self.accumulated_sec += instant.elapsed().as_secs_f64();
                }
                self.child_process = None;
                self.is_playing = false;
                self.start_instant = None;
                return self.accumulated_sec.min(self.duration_sec);
            }

            if let Some(instant) = self.start_instant {
                let current = self.accumulated_sec + instant.elapsed().as_secs_f64();
                if current >= self.duration_sec && self.duration_sec > 0.0 {
                    self.accumulated_sec = self.duration_sec;
                    self.kill_process();
                    self.is_playing = false;
                    self.start_instant = None;
                    return self.duration_sec;
                }
                return current;
            }
        }
        self.accumulated_sec.min(self.duration_sec)
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// PID of the playback child process, if one is currently running (used by
    /// tests to verify shutdown actually kills it).
    pub fn child_pid(&self) -> Option<u32> {
        self.child_process.as_ref().map(|c| c.id())
    }

    /// Kill and reap the playback child WITHOUT blocking the UI thread
    /// indefinitely: `Child::wait()` can hang for seconds if the audio process
    /// ignores the signal while blocked on a device, which froze the app on
    /// pause/stop/seek (perceived as a force-close). Reap within a short
    /// deadline, then give up (the zombie is reaped by init at exit).
    fn kill_process(&mut self) {
        // Interactive path (pause/stop/seek/load): keep it snappy at 60 FPS.
        self.kill_process_deadline(std::time::Duration::from_millis(300));
    }

    /// Retries SIGKILL while the child is dying, up to `deadline`. The retry
    /// matters on shutdown: a player can linger tens of ms after the first
    /// signal, and giving up too early leaves it alive — which is why audio
    /// could keep playing after the app closed.
    fn kill_process_deadline(&mut self, deadline: std::time::Duration) {
        if let Some(mut child) = self.child_process.take() {
            let end = std::time::Instant::now() + deadline;
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) if std::time::Instant::now() < end => {
                        let _ = child.kill(); // idempotent, re-signal while dying
                        std::thread::sleep(std::time::Duration::from_millis(25));
                    }
                    _ => break,
                }
            }
        }
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.kill_process();
    }
}

impl AudioPlayer {
    /// Install a raw child handle without decoding (used by the integration test
    /// to simulate an active playback process on a never-ending FIFO).
    pub fn set_child_for_test(&mut self, child: Child, file_path: String) {
        self.child_process = Some(child);
        self.file_path = Some(file_path);
        self.is_playing = true;
        self.start_instant = Some(Instant::now());
    }

    /// Spawn pw-play on a path (a FIFO never ends, so the process stays alive
    /// until killed — used to verify the shutdown path reaps the child instead
    /// of leaving audio playing).
    pub fn play_pwplay_for_test(&mut self, path: &str) -> Result<(), String> {
        let mut cmd = Command::new("pw-play");
        hide_cmd(&mut cmd);
        if let Ok(c) = cmd
            .arg("--volume")
            .arg("0.5")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            self.set_child_for_test(c, path.to_string());
            return Ok(());
        }
        Err("pw-play not available".to_string())
    }
}

/// Try the OS-native fallback players, in priority order:
///
/// 1. `afplay` (built into macOS) — only when starting at 0:00, because afplay
///    has NO seek option. The old code passed the ffplay-only `-ss` flag to it,
///    which afplay rejects (exit 1), so on macOS playback silently produced no
///    sound while the UI showed "playing".
/// 2. `pw-play` (PipeWire) / `paplay` (PulseAudio) — Linux.
/// 3. Windows PowerShell MediaPlayer (see [`spawn_powershell_player`]).
fn spawn_os_fallback(
    mut afplay_cmd: Command,
    mut pwplay_cmd: Command,
    mut paplay_cmd: Command,
    file_path: &str,
    start_sec: f64,
    vol_str: &str,
    vol_pct: u32,
) -> Option<Child> {
    if start_sec <= 0.001 {
        if let Ok(c) = afplay_cmd
            .arg("-v")
            .arg(vol_str)
            .arg(file_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            println!("[AudioPlayer] Playing via afplay (start_sec={:.2}s): {}", start_sec, file_path);
            return Some(c);
        }
    }

    if let Ok(c) = pwplay_cmd
        .arg("--volume")
        .arg(vol_str)
        .arg(file_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        println!("[AudioPlayer] Playing via pw-play (start_sec={:.2}s): {}", start_sec, file_path);
        return Some(c);
    }

    if let Ok(c) = paplay_cmd
        .arg(file_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        println!("[AudioPlayer] Playing via paplay (start_sec={:.2}s): {}", start_sec, file_path);
        return Some(c);
    }

    if let Ok(c) = spawn_powershell_player(file_path, start_sec, vol_pct) {
        println!("[AudioPlayer] Playing via powershell (start_sec={:.2}s): {}", start_sec, file_path);
        return Some(c);
    }

    crate::logline!("[AudioPlayer] Failed to launch audio process for: {}", file_path);
    None
}

/// Suppress console window creation on Windows platform.
fn hide_cmd(cmd: &mut Command) -> &mut Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

/// Native audio player for Windows systems using .NET WPF System.Windows.Media.MediaPlayer.
fn spawn_powershell_player(file_path: &str, start_sec: f64, vol_pct: u32) -> std::io::Result<Child> {
    let forward_slash_path = file_path.replace('\\', "/");
    let escaped_path = forward_slash_path.replace('\'', "''");
    let start_ms = (start_sec * 1000.0).round() as u64;

    let script = format!(
        "Add-Type -AssemblyName PresentationCore; \
         $path = [System.IO.Path]::GetFullPath('{escaped_path}'); \
         $player = New-Object System.Windows.Media.MediaPlayer; \
         $player.Open([System.Uri]::new($path)); \
         $player.Volume = {vol_pct} / 100.0; \
         if ({start_ms} -gt 0) {{ $player.Position = [Timespan]::FromMilliseconds({start_ms}) }}; \
         $player.Play(); \
         while ($true) {{ Start-Sleep -Seconds 1 }}"
    );

    let mut cmd = Command::new("powershell");
    cmd.arg("-NoProfile")
       .arg("-NonInteractive")
       .arg("-ExecutionPolicy")
       .arg("Bypass")
       .arg("-WindowStyle")
       .arg("Hidden")
       .arg("-Command")
       .arg(script)
       .stdin(Stdio::null())
       .stdout(Stdio::null())
       .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    cmd.spawn()
}
