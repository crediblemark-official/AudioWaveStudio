use audiowave_studio_lib::config::*;

#[test]
fn deserializes_camel_case_config() {
  let json = r##"{
    "style": "spectrum",
    "theme": { "name": "cyberpunk", "label": "Cyberpunk Neon", "primaryColor": "#00f0ff", "secondaryColor": "#ff007f", "accentColor": "#ffe600", "glowColor": "#00f0ff" },
    "background": { "mode": "solid", "solidColor": "#0b0c10", "gradientStart": "#0f0c20", "gradientEnd": "#06101e", "blurAmount": 8, "overlayOpacity": 0, "showParticles": true, "particleColor": "#00f0ff" },
    "text": { "songTitle": "Test", "artistName": "Artist", "showTitle": false, "showArtist": false, "fontFamily": "monospace", "title": {}, "artist": {}, "blocks": [] },
    "reactivity": { "fftSize": 1024, "sensitivity": 1.0, "bassMultiplier": 1.0, "barCount": 64, "barWidth": 0, "barGap": 4, "barRounding": 4, "smoothing": 0.8, "mirrorBars": false, "showPeaks": true, "peakColor": "#ffffff" },
    "export": { "aspectRatio": "16:9", "resolution": "720p", "fps": 60, "format": "mp4" },
    "screenEffects": { "enabled": true, "mainEffect": "shake", "shakeIntensity": 1.0, "shakeFrequency": 0.5, "shakeMaxOffset": 8, "shakeOnBeat": true, "glitchIntensity": 0.5, "pulseIntensity": 0.3, "spotlightColor": "#ffffff", "strobeIntensity": 0.5, "scanlineOpacity": 0.15, "chromaticIntensity": 0.5, "zoomIntensity": 0.1, "invertIntensity": 0.5, "barsAmount": 0.3, "shockwaveIntensity": 0.5, "pixelateIntensity": 0.5, "tiltIntensity": 0.5, "heatHazeIntensity": 0.5, "hueShiftIntensity": 0.5 },
    "positionX": 0,
    "positionY": 0,
    "scale": 1.0
  }"##;
  let cfg: VisualizerConfig = serde_json::from_str(json).expect("camelCase config must deserialize");
  assert_eq!(cfg.background.solid_color, "#0b0c10");
  assert_eq!(cfg.background.gradient_start, "#0f0c20");
  assert_eq!(cfg.reactivity.fft_size, 1024);
  assert_eq!(cfg.export.fps, 60);
  assert!(cfg.screen_effects.enabled);
  assert!(cfg.background.show_particles);
  assert!(matches!(cfg.style, VisualizerStyle::Spectrum));
  assert!(matches!(cfg.background.mode, BackgroundMode::Solid));
}

#[test]
fn missing_fields_fill_defaults() {
  let json = r#"{"style":"spectrum","theme":{},"background":{"mode":"solid"},"text":{},"reactivity":{},"export":{},"screenEffects":{},"positionX":0,"positionY":0,"scale":1}"#;
  let cfg: VisualizerConfig = serde_json::from_str(json).expect("missing fields must default");
  assert!(matches!(cfg.background.mode, BackgroundMode::Solid));
  assert_eq!(cfg.background.solid_color, "");
  assert!(!cfg.screen_effects.enabled);
  assert_eq!(cfg.export.fps, 0);
}
