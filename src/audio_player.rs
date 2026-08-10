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
        #[cfg(target_os = "linux")]
        ffplay_cmd.env("SDL_AUDIO_DRIVER", "pulse");

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
        } else if let Ok(c) = Command::new("mpv")
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
        } else if let Ok(c) = Command::new("afplay")
            .arg("-ss")
            .arg(format!("{:.2}", start_sec))
            .arg(&file_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            println!("[AudioPlayer] Playing via afplay (start_sec={:.2}s): {}", start_sec, file_path);
            Some(c)
        } else if let Ok(c) = Command::new("pw-play")
            .arg("--volume")
            .arg(&vol_str)
            .arg(&file_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            println!("[AudioPlayer] Playing via pw-play (start_sec={:.2}s): {}", start_sec, file_path);
            Some(c)
        } else if let Ok(c) = Command::new("paplay")
            .arg(&file_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            println!("[AudioPlayer] Playing via paplay (start_sec={:.2}s): {}", start_sec, file_path);
            Some(c)
        } else if let Ok(c) = spawn_powershell_player(&file_path, start_sec, vol_pct) {
            println!("[AudioPlayer] Playing via powershell (start_sec={:.2}s): {}", start_sec, file_path);
            Some(c)
        } else {
            eprintln!("[AudioPlayer] Failed to launch audio process for: {}", file_path);
            None
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
        if was_playing {
            self.pause();
        }
        self.accumulated_sec = target_sec.max(0.0).min(self.duration_sec);
        if was_playing {
            let _ = self.play();
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        let v = vol.clamp(0.0, 1.0);
        if (self.volume - v).abs() < 0.001 {
            return;
        }
        self.volume = v;

        // App-local volume: NEVER touch the system sink volume (the old
        // wpctl/pactl calls muted/un-muted the whole PipeWire/PulseAudio
        // default sink, so the app's slider controlled the entire OS).
        // External players only accept a volume at launch, so a live change
        // restarts playback at the same position with the new `-volume`.
        // Restarts are throttled (~6/s) so a drag doesn't churn the child.
        if self.is_playing {
            let now = Instant::now();
            let due = self
                .last_restart
                .map(|t| now.duration_since(t) >= RESTART_THROTTLE)
                .unwrap_or(true);
            if due {
                self.last_restart = Some(now);
                let pos = self.accumulated_sec + self.start_instant.map(|i| i.elapsed().as_secs_f64()).unwrap_or(0.0);
                self.pause();
                self.accumulated_sec = pos;
                let _ = self.play();
            }
        }
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn get_current_time_sec(&mut self) -> f64 {
        if self.is_playing {
            if let Some(instant) = self.start_instant {
                let current = self.accumulated_sec + instant.elapsed().as_secs_f64();
                if current >= self.duration_sec && self.duration_sec > 0.0 {
                    // Natural end of track: stop the child but keep the position
                    // pinned at the end so the progress bar stays at 100% (a
                    // plain `stop()` would reset accumulated_sec to 0 and make
                    // the bar snap back to the start on the next frame).
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

    pub fn get_duration_sec(&self) -> f64 {
        self.duration_sec
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
        if let Ok(c) = Command::new("pw-play")
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

/// Fallback audio player for Windows systems using PowerShell and Windows Media Player COM object.
fn spawn_powershell_player(file_path: &str, start_sec: f64, vol_pct: u32) -> std::io::Result<Child> {
    let escaped_path = file_path.replace('\'', "''");
    let script = format!(
        "$p = New-Object -ComObject WMPlayer.OCX; \
         $p.URL = '{escaped_path}'; \
         $p.settings.volume = {vol_pct}; \
         if ({start_sec:.2} -gt 0) {{ \
             for ($i = 0; $i -lt 20; $i++) {{ \
                 if ($p.openState -eq 13) {{ break }}; \
                 Start-Sleep -Milliseconds 50 \
             }}; \
             $p.controls.currentPosition = {start_sec:.2} \
         }}; \
         $p.controls.play(); \
         while ($p.playState -ne 1) {{ Start-Sleep -Milliseconds 200 }}"
    );
    Command::new("powershell")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-WindowStyle")
        .arg("Hidden")
        .arg("-Command")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

