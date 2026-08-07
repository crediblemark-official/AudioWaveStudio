use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub enum VisualizerStyle {
  #[default]
  #[serde(rename = "spectrum")]
  Spectrum,
  #[serde(rename = "radial")]
  Radial,
  #[serde(rename = "oscilloscope")]
  Oscilloscope,
  #[serde(rename = "equalizer")]
  Equalizer,
  #[serde(rename = "minimal")]
  Minimal,
  #[serde(rename = "waveformFill")]
  WaveformFill,
  #[serde(rename = "circularBars")]
  CircularBars,
  #[serde(rename = "smoothSpectrum")]
  SmoothSpectrum,
  #[serde(rename = "pulseRings")]
  PulseRings,
  #[serde(rename = "vuMeter")]
  VuMeter,
  #[serde(rename = "auroraWave")]
  AuroraWave,
  #[serde(rename = "flameFire")]
  FlameFire,
  #[serde(rename = "spiralGalaxy")]
  SpiralGalaxy,
  #[serde(rename = "threeD")]
  ThreeD,
  #[serde(rename = "api3D")]
  Api3D,
  #[serde(rename = "neonCity3D")]
  NeonCity3D,
  #[serde(rename = "speaker3D")]
  Speaker3D,
  #[serde(rename = "speakerTrio")]
  SpeakerTrio,
  #[serde(rename = "speakerSplatter")]
  SpeakerSplatter,
  #[serde(rename = "radialRipple3D")]
  RadialRipple3D,
  #[serde(rename = "waterfall3D")]
  Waterfall3D,
  #[serde(rename = "cassetteTape")]
  CassetteTape,
  #[serde(rename = "vinylRecord")]
  VinylRecord,
  #[serde(rename = "turntable")]
  Turntable,
  #[serde(rename = "retroRadio")]
  RetroRadio,
  #[serde(rename = "djController")]
  DjController,
  #[serde(rename = "speakerExplosion")]
  SpeakerExplosion,
  #[serde(rename = "orbitSpike")]
  OrbitSpike,
  #[serde(rename = "cyberRing3D")]
  CyberRing3D,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum AspectRatio {
  #[default]
  #[serde(rename = "16:9")]
  Widescreen,
  #[serde(rename = "9:16")]
  Portrait,
  #[serde(rename = "1:1")]
  Square,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum ColorThemeName {
  #[serde(rename = "cyberpunk")]
  Cyberpunk,
  #[serde(rename = "synthwave")]
  Synthwave,
  #[serde(rename = "emerald")]
  Emerald,
  #[serde(rename = "violet")]
  Violet,
  #[serde(rename = "gold")]
  Gold,
  #[default]
  #[serde(rename = "custom")]
  Custom,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum MusicNoteStyle {
  #[default]
  #[serde(rename = "float")]
  Float,
  #[serde(rename = "bounce")]
  Bounce,
  #[serde(rename = "spiral")]
  Spiral,
  #[serde(rename = "wave")]
  Wave,
  #[serde(rename = "burst")]
  Burst,
  #[serde(rename = "confined")]
  Confined,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum ParticleStyle {
  #[default]
  #[serde(rename = "float")]
  Float,
  #[serde(rename = "bounce")]
  Bounce,
  #[serde(rename = "wave")]
  Wave,
  #[serde(rename = "static")]
  Static,
  #[serde(rename = "confined")]
  Confined,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum ScreenEffect {
  #[default]
  #[serde(rename = "none")]
  None,
  #[serde(rename = "shake")]
  Shake,
  #[serde(rename = "glitch")]
  Glitch,
  #[serde(rename = "vignette")]
  Vignette,
  #[serde(rename = "pulse")]
  Pulse,
  #[serde(rename = "spotlight")]
  Spotlight,
  #[serde(rename = "strobe")]
  Strobe,
  #[serde(rename = "scanline")]
  Scanline,
  #[serde(rename = "chromatic")]
  Chromatic,
  #[serde(rename = "zoom")]
  Zoom,
  #[serde(rename = "invert")]
  Invert,
  #[serde(rename = "bars")]
  Bars,
  #[serde(rename = "shockwave")]
  Shockwave,
  #[serde(rename = "pixelate")]
  Pixelate,
  #[serde(rename = "tilt")]
  Tilt,
  #[serde(rename = "heatHaze")]
  HeatHaze,
  #[serde(rename = "hueShift")]
  HueShift,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ColorTheme {
  pub name: ColorThemeName,
  pub label: String,
  pub primary_color: String,
  pub secondary_color: String,
  pub accent_color: String,
  pub glow_color: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum BackgroundFillType {
  #[default]
  #[serde(rename = "solid")]
  Solid,
  #[serde(rename = "gradient")]
  Gradient,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub enum BackgroundEffect {
  #[default]
  #[serde(rename = "none")]
  None,
  #[serde(rename = "grid")]
  Grid,
  #[serde(rename = "particles")]
  Particles,
  #[serde(rename = "musicNotes")]
  MusicNotes,
  #[serde(rename = "aurora")]
  Aurora,
  #[serde(rename = "noise")]
  Noise,
  #[serde(rename = "bokeh")]
  Bokeh,
  #[serde(rename = "starfield")]
  Starfield,
  #[serde(rename = "nebula")]
  Nebula,
  #[serde(rename = "psychedelic")]
  Psychedelic,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum BackgroundMode {
  #[default]
  #[serde(rename = "solid")]
  Solid,
  #[serde(rename = "gradient")]
  Gradient,
  #[serde(rename = "customImage")]
  CustomImage,
  #[serde(rename = "grid")]
  Grid,
  #[serde(rename = "aurora")]
  Aurora,
  #[serde(rename = "noise")]
  Noise,
  #[serde(rename = "bokeh")]
  Bokeh,
  #[serde(rename = "starfield")]
  Starfield,
  #[serde(rename = "nebula")]
  Nebula,
  #[serde(rename = "psychedelic")]
  Psychedelic,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct BackgroundSettings {
  pub mode: BackgroundMode,
  #[serde(default)]
  pub fill_type: Option<BackgroundFillType>,
  #[serde(default)]
  pub effect: Option<BackgroundEffect>,
  #[serde(default)]
  pub effects: Option<Vec<BackgroundEffect>>,
  pub solid_color: String,
  pub gradient_start: String,
  pub gradient_end: String,
  pub blur_amount: f32,
  pub overlay_opacity: f32,
  #[serde(default)]
  pub custom_image_uri: Option<String>,
  #[serde(default)]
  pub image_opacity: Option<f32>,
  #[serde(default)]
  pub grid_color: Option<String>,
  #[serde(default)]
  pub grid_size: Option<f32>,
  #[serde(default)]
  pub grid_line_width: Option<f32>,
  pub show_particles: bool,
  #[serde(default)]
  pub particle_style: Option<ParticleStyle>,
  pub particle_color: String,
  #[serde(default)]
  pub particle_size: Option<f32>,
  #[serde(default)]
  pub particle_speed: Option<f32>,
  #[serde(default)]
  pub particle_count: Option<u32>,
  #[serde(default)]
  pub show_music_notes: Option<bool>,
  #[serde(default)]
  pub show_fireworks: Option<bool>,
  #[serde(default)]
  pub show_matrix_rain: Option<bool>,
  #[serde(default)]
  pub show_fireflies: Option<bool>,
  #[serde(default)]
  pub show_sakura: Option<bool>,
  #[serde(default)]
  pub show_cyber_lightning: Option<bool>,
  #[serde(default)]
  pub music_note_style: Option<MusicNoteStyle>,
  #[serde(default)]
  pub music_note_color: Option<String>,
  #[serde(default)]
  pub radial_center_image_uri: Option<String>,
  #[serde(default)]
  pub music_note_density: Option<f32>,
  #[serde(default)]
  pub music_note_size: Option<f32>,
  #[serde(default)]
  pub music_note_count: Option<u32>,
  #[serde(default)]
  pub music_note_sensitivity: Option<f32>,
  #[serde(default)]
  pub star_count: Option<u32>,
  #[serde(default)]
  pub star_speed: Option<f32>,
  #[serde(default)]
  pub star_brightness: Option<f32>,
  #[serde(default)]
  pub nebula_intensity: Option<f32>,
  #[serde(default)]
  pub nebula_speed: Option<f32>,
  #[serde(default)]
  pub aurora_speed: Option<f32>,
  #[serde(default)]
  pub aurora_amplitude: Option<f32>,
  #[serde(default)]
  pub aurora_opacity: Option<f32>,
  #[serde(default)]
  pub grain_opacity: Option<f32>,
  #[serde(default)]
  pub bokeh_count: Option<u32>,
  #[serde(default)]
  pub bokeh_size: Option<f32>,
  #[serde(default)]
  pub bokeh_opacity: Option<f32>,
  #[serde(default)]
  pub psychedelic_speed: Option<f32>,
  #[serde(default)]
  pub psychedelic_bands: Option<u32>,
  #[serde(default)]
  pub psychedelic_line_width: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum TextTransform {
  #[default]
  #[serde(rename = "none")]
  None,
  #[serde(rename = "uppercase")]
  Uppercase,
  #[serde(rename = "lowercase")]
  Lowercase,
  #[serde(rename = "capitalize")]
  Capitalize,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum TextAlign {
  #[serde(rename = "left")]
  Left,
  #[default]
  #[serde(rename = "center")]
  Center,
  #[serde(rename = "right")]
  Right,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct TextBlock {
  pub id: String,
  pub text: String,
  pub enabled: bool,
  pub font_family: String,
  pub font_size: f32,
  pub font_weight: f32,
  pub italic: bool,
  pub color: String,
  pub use_gradient: bool,
  pub gradient_start: String,
  pub gradient_end: String,
  pub gradient_angle: f32,
  pub opacity: f32,
  pub letter_spacing: f32,
  pub transform: TextTransform,
  pub position_x: f32,
  pub position_y: f32,
  pub align: TextAlign,
  pub line_height: f32,
  pub max_width: f32,
  pub shadow: bool,
  pub shadow_blur: f32,
  pub shadow_offset_x: f32,
  pub shadow_offset_y: f32,
  pub glow_intensity: f32,
  pub outline: bool,
  pub outline_color: String,
  pub outline_width: f32,
  pub reactive_scale: f32,
  pub wave_effect: bool,
  pub fade_in: bool,
}

impl Default for TextBlock {
  fn default() -> Self {
    Self {
      id: "block_default".to_string(),
      text: String::new(),
      enabled: true,
      font_family: "Outfit".to_string(),
      font_size: 36.0,
      font_weight: 700.0,
      italic: false,
      color: "#ffffff".to_string(),
      use_gradient: false,
      gradient_start: "#ffffff".to_string(),
      gradient_end: "#00f0ff".to_string(),
      gradient_angle: 90.0,
      opacity: 1.0,
      letter_spacing: 0.0,
      transform: TextTransform::None,
      position_x: 50.0,
      position_y: 78.0,
      align: TextAlign::Center,
      line_height: 1.2,
      max_width: 0.0,
      shadow: false,
      shadow_blur: 0.0,
      shadow_offset_x: 0.0,
      shadow_offset_y: 0.0,
      glow_intensity: 0.0,
      outline: false,
      outline_color: "#000000".to_string(),
      outline_width: 0.0,
      reactive_scale: 0.0,
      wave_effect: false,
      fade_in: false,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct TextSettings {
  pub song_title: String,
  pub artist_name: String,
  pub cassette_label: String,
  pub show_title: bool,
  pub show_artist: bool,
  pub font_family: String,
  pub title: TextBlock,
  pub artist: TextBlock,
  pub blocks: Vec<TextBlock>,
}

impl Default for TextSettings {
  fn default() -> Self {
    Self {
      song_title: "AudioWave Visualizer".to_string(),
      artist_name: "CredibleMark Studio".to_string(),
      cassette_label: "AUDIOWAVE VOLUME #1".to_string(),
      show_title: true,
      show_artist: true,
      font_family: "Outfit".to_string(),
      title: TextBlock {
        id: "title_block".to_string(),
        text: "AudioWave Visualizer".to_string(),
        font_size: 36.0,
        font_weight: 700.0,
        position_x: 50.0,
        position_y: 78.0,
        color: "#ffffff".to_string(),
        opacity: 1.0,
        enabled: true,
        ..Default::default()
      },
      artist: TextBlock {
        id: "artist_block".to_string(),
        text: "CredibleMark Studio".to_string(),
        font_size: 20.0,
        font_weight: 400.0,
        position_x: 50.0,
        position_y: 86.0,
        color: "#a3a3a3".to_string(),
        opacity: 1.0,
        enabled: true,
        ..Default::default()
      },
      blocks: Vec::new(),
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct AudioReactivitySettings {
  pub fft_size: usize,
  pub sensitivity: f32,
  pub bass_multiplier: f32,
  pub bar_count: usize,
  pub bar_width: f32,
  pub bar_gap: f32,
  pub bar_rounding: f32,
  pub smoothing: f32,
  pub mirror_bars: bool,
  pub show_peaks: bool,
  pub peak_color: String,
  #[serde(default)]
  pub fire_width_ratio: Option<f32>,
  #[serde(default)]
  pub fire_height_scale: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum ExportResolution {
  #[serde(rename = "1080p")]
  P1080,
  #[default]
  #[serde(rename = "720p")]
  P720,
  #[serde(rename = "4K")]
  K4,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum ExportFormat {
  #[default]
  #[serde(rename = "mp4")]
  Mp4,
  #[serde(rename = "webm")]
  Webm,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ExportSettings {
  pub aspect_ratio: AspectRatio,
  pub resolution: ExportResolution,
  pub fps: u32,
  pub format: ExportFormat,
  /// Codec family preference for ffmpeg: "auto" | "h264" | "hevc" | "av1".
  pub encoder: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ScreenEffectsSettings {
  pub enabled: bool,
  pub background_only: Option<bool>,
  pub main_effect: ScreenEffect,
  pub shake_intensity: f32,
  pub shake_frequency: f32,
  pub shake_max_offset: f32,
  pub shake_on_beat: bool,
  pub glitch_intensity: f32,
  pub pulse_intensity: f32,
  pub spotlight_color: String,
  pub strobe_intensity: f32,
  pub scanline_opacity: f32,
  pub chromatic_intensity: f32,
  pub zoom_intensity: f32,
  pub invert_intensity: f32,
  pub bars_amount: f32,
  pub shockwave_intensity: f32,
  pub pixelate_intensity: f32,
  pub tilt_intensity: f32,
  pub heat_haze_intensity: f32,
  pub hue_shift_intensity: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct VisualizerConfig {
  pub style: VisualizerStyle,
  pub theme: ColorTheme,
  pub background: BackgroundSettings,
  pub text: TextSettings,
  pub reactivity: AudioReactivitySettings,
  pub export: ExportSettings,
  pub screen_effects: ScreenEffectsSettings,
  pub position_x: f32,
  pub position_y: f32,
  pub scale: f32,
}
