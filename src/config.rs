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
  #[serde(rename = "cyberHorizon")]
  CyberHorizon,
  #[serde(rename = "cyberBlackHole")]
  CyberBlackHole,
  #[serde(rename = "supernovaBurst")]
  SupernovaBurst,
  #[serde(rename = "quantumEye")]
  QuantumEye,
  #[serde(rename = "chronoReactor")]
  ChronoReactor,
  #[serde(rename = "solarFlareCrown")]
  SolarFlareCrown,
  #[serde(rename = "warpDrive")]
  WarpDrive,
  #[serde(rename = "neonBiohazard")]
  NeonBiohazard,
  #[serde(rename = "neonLotus")]
  NeonLotus,
  #[serde(rename = "holographicVinyl")]
  HolographicVinyl,
  #[serde(rename = "acousticCymascope")]
  AcousticCymascope,
  #[serde(rename = "synthwaveHighway3D")]
  SynthwaveHighway3D,
  #[serde(rename = "mercuryFluid")]
  MercuryFluid,
  #[serde(rename = "neonMetropolis3D")]
  NeonMetropolis3D,
  #[serde(rename = "matrixRain")]
  MatrixRain,
  #[serde(rename = "quantumRibbon")]
  QuantumRibbon,
  #[serde(rename = "audioPrism3D")]
  AudioPrism3D,
  #[serde(rename = "vaporwaveDeck3D")]
  VaporwaveDeck3D,
  #[serde(rename = "nebulaCloud3D")]
  NebulaCloud3D,
  #[serde(rename = "cyberTunnel3D")]
  CyberTunnel3D,
  #[serde(rename = "laserWall")]
  LaserWall,
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
  #[serde(rename = "hologramStage")]
  HologramStage,
  #[serde(rename = "dualPortalBridge")]
  DualPortalBridge,
  #[serde(rename = "particleWave3D")]
  ParticleWave3D,
  #[serde(rename = "cylinderMatrix3D")]
  CylinderMatrix3D,
  #[serde(rename = "dualWaveHorizon")]
  DualWaveHorizon,
  #[serde(rename = "waveformSeismograph")]
  WaveformSeismograph,
  #[serde(rename = "waveformDualTube")]
  WaveformDualTube,
  #[serde(rename = "waveformVoxelTerrain")]
  WaveformVoxelTerrain,
  #[serde(rename = "waveformSineComb")]
  WaveformSineComb,
  #[serde(rename = "waveformHarmonicWeb")]
  WaveformHarmonicWeb,
  #[serde(rename = "waveformSteppedArcade")]
  WaveformSteppedArcade,
  #[serde(rename = "waveformBarcodePulse")]
  WaveformBarcodePulse,
  #[serde(rename = "waveformCurtainBeams")]
  WaveformCurtainBeams,
  #[serde(rename = "waveformOscillatingRings")]
  WaveformOscillatingRings,
  #[serde(rename = "waveformTopographicRibbon")]
  WaveformTopographicRibbon,
  #[serde(rename = "radialSpikeBlade")]
  RadialSpikeBlade,
  #[serde(rename = "radialPlasmaAura")]
  RadialPlasmaAura,
  #[serde(rename = "radialCyberRune")]
  RadialCyberRune,
  #[serde(rename = "radialNeonOrbiter")]
  RadialNeonOrbiter,
  #[serde(rename = "radialStarlightHalo")]
  RadialStarlightHalo,
  #[serde(rename = "radialVortexSpiral")]
  RadialVortexSpiral,
  #[serde(rename = "radialBioShuriken")]
  RadialBioShuriken,
  #[serde(rename = "radialHexCore")]
  RadialHexCore,
  #[serde(rename = "radialSonicMandala")]
  RadialSonicMandala,
  #[serde(rename = "radialLaserCurtain")]
  RadialLaserCurtain,
  #[serde(rename = "radialSolarFlareBurst")]
  RadialSolarFlareBurst,
  #[serde(rename = "radialApertureIris")]
  RadialApertureIris,
  #[serde(rename = "radialRadarSweep")]
  RadialRadarSweep,
  #[serde(rename = "radialGearMechanism")]
  RadialGearMechanism,
  #[serde(rename = "radialFireworksBurst")]
  RadialFireworksBurst,
  #[serde(rename = "radialKaleidoscope")]
  RadialKaleidoscope,
  #[serde(rename = "radialOrrery")]
  RadialOrrery,
  #[serde(rename = "radialClockwork")]
  RadialClockwork,
  #[serde(rename = "radialGeodesicWeb")]
  RadialGeodesicWeb,
  #[serde(rename = "pulsingPillRing")]
  PulsingPillRing,
  #[serde(rename = "pulsingLiquidAura")]
  PulsingLiquidAura,
  #[serde(rename = "pulsingDualRing")]
  PulsingDualRing,
  #[serde(rename = "pulsingShockwave")]
  PulsingShockwave,
  #[serde(rename = "pulsingNeonArcade")]
  PulsingNeonArcade,
  #[serde(rename = "pulsingLaserWeb")]
  PulsingLaserWeb,
  #[serde(rename = "pulsingCosmicDust")]
  PulsingCosmicDust,
  #[serde(rename = "pulsingCyberShield")]
  PulsingCyberShield,
  #[serde(rename = "pulsingSunburstCorona")]
  PulsingSunburstCorona,
  #[serde(rename = "pulsingBarcodePill")]
  PulsingBarcodePill,
  #[serde(rename = "SaturnHalo")]
  SaturnHalo,
  #[serde(rename = "StarHexagon")]
  StarHexagon,
  #[serde(rename = "QuantumCloud")]
  QuantumCloud,
  #[serde(rename = "HyperdriveTunnel")]
  HyperdriveTunnel,
  #[serde(rename = "NebulaRing")]
  NebulaRing,
  #[serde(rename = "TacticalHud")]
  TacticalHud,
  #[serde(rename = "CrystalPrism")]
  CrystalPrism,
  #[serde(rename = "SynthwaveSun")]
  SynthwaveSun,
  #[serde(rename = "BiomorphicBloom")]
  BiomorphicBloom,
  #[serde(rename = "InfinityLoop")]
  InfinityLoop,
  #[serde(rename = "liquidTriLobeAura")]
  LiquidTriLobeAura,
  #[serde(rename = "liquidFerrofluidSpikes")]
  LiquidFerrofluidSpikes,
  #[serde(rename = "liquidMoltenMercury")]
  LiquidMoltenMercury,
  #[serde(rename = "liquidConcentricDrop")]
  LiquidConcentricDrop,
  #[serde(rename = "liquidJellyfishTentacles")]
  LiquidJellyfishTentacles,
  #[serde(rename = "liquidOilSlick")]
  LiquidOilSlick,
  #[serde(rename = "liquidVortexSwirl")]
  LiquidVortexSwirl,
  #[serde(rename = "liquidMetaballLava")]
  LiquidMetaballLava,
  #[serde(rename = "liquidToxicSlime")]
  LiquidToxicSlime,
  #[serde(rename = "liquidCymascopeWater")]
  LiquidCymascopeWater,
  #[serde(rename = "liquidBioluminescentPlasma")]
  LiquidBioluminescentPlasma,
  #[serde(rename = "liquidPlasmaBlob3D")]
  LiquidPlasmaBlob3D,
  #[serde(rename = "liquidChromaticViscosity")]
  LiquidChromaticViscosity,
  #[serde(rename = "liquidHydroElectricArcs")]
  LiquidHydroElectricArcs,
  #[serde(rename = "liquidBioluminescentPlankton")]
  LiquidBioluminescentPlankton,
  #[serde(rename = "liquidRadioactiveIsotope")]
  LiquidRadioactiveIsotope,
  #[serde(rename = "liquidNeonCyberGoo")]
  LiquidNeonCyberGoo,
  #[serde(rename = "liquidMoltenGoldStream")]
  LiquidMoltenGoldStream,
  #[serde(rename = "liquidMagmaCrustCore")]
  LiquidMagmaCrustCore,
  #[serde(rename = "liquidQuantumFluid")]
  LiquidQuantumFluid,
  #[serde(rename = "glassBoxQuantumPlasma")]
  GlassBoxQuantumPlasma,
  #[serde(rename = "glassBoxNeonSpectrum")]
  GlassBoxNeonSpectrum,
  #[serde(rename = "glassBoxCyberGrid")]
  GlassBoxCyberGrid,
  #[serde(rename = "glassBoxBioluminescentJellyfish")]
  GlassBoxBioluminescentJellyfish,
  #[serde(rename = "glassBoxMoltenLava")]
  GlassBoxMoltenLava,
  #[serde(rename = "glassBoxLaserMatrix")]
  GlassBoxLaserMatrix,
  #[serde(rename = "glassBoxLiquidChrome")]
  GlassBoxLiquidChrome,
  #[serde(rename = "glassBoxCosmicNebula")]
  GlassBoxCosmicNebula,
  #[serde(rename = "glassBoxHologramCore")]
  GlassBoxHologramCore,
  #[serde(rename = "glassBoxMatrixRain")]
  GlassBoxMatrixRain,
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
  #[serde(rename = "glassCrack")]
  GlassCrack,
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
  pub fireworks_count: Option<u32>,
  #[serde(default)]
  pub fireworks_size: Option<f32>,
  #[serde(default)]
  pub fireworks_speed: Option<f32>,
  #[serde(default)]
  pub fireworks_depth: Option<f32>,
  #[serde(default)]
  pub fireworks_color: Option<String>,
  #[serde(default)]
  pub matrix_rain_count: Option<u32>,
  #[serde(default)]
  pub matrix_rain_size: Option<f32>,
  #[serde(default)]
  pub matrix_rain_speed: Option<f32>,
  #[serde(default)]
  pub matrix_rain_depth: Option<f32>,
  #[serde(default)]
  pub matrix_rain_color: Option<String>,
  #[serde(default)]
  pub fireflies_count: Option<u32>,
  #[serde(default)]
  pub fireflies_size: Option<f32>,
  #[serde(default)]
  pub fireflies_speed: Option<f32>,
  #[serde(default)]
  pub fireflies_depth: Option<f32>,
  #[serde(default)]
  pub fireflies_color: Option<String>,
  #[serde(default)]
  pub sakura_count: Option<u32>,
  #[serde(default)]
  pub sakura_size: Option<f32>,
  #[serde(default)]
  pub sakura_speed: Option<f32>,
  #[serde(default)]
  pub sakura_depth: Option<f32>,
  #[serde(default)]
  pub sakura_color: Option<String>,
  #[serde(default)]
  pub cyber_lightning_count: Option<u32>,
  #[serde(default)]
  pub cyber_lightning_size: Option<f32>,
  #[serde(default)]
  pub cyber_lightning_speed: Option<f32>,
  #[serde(default)]
  pub cyber_lightning_depth: Option<f32>,
  #[serde(default)]
  pub cyber_lightning_color: Option<String>,
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
        position_y: 81.0,
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
        position_y: 88.0,
        color: "#a3a3a3".to_string(),
        opacity: 1.0,
        enabled: true,
        ..Default::default()
      },
      blocks: Vec::new(),
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

impl Default for AudioReactivitySettings {
  /// Mirror the Slint control-panel slider defaults (app_window.slint) so a
  /// FRESH config — app launch, or an old/partial preset that omits these
  /// fields — is immediately usable instead of zeroed (sensitivity 0 would
  /// flatten every frequency bin to silence, making the visualizer look
  /// dead/unresponsive until the user touches a control and fires
  /// `config-changed`).
  fn default() -> Self {
    AudioReactivitySettings {
      fft_size: 1024,
      sensitivity: 1.2,
      bass_multiplier: 1.5,
      bar_count: 64,
      bar_width: 6.0,
      bar_gap: 2.0,
      bar_rounding: 2.0,
      smoothing: 0.8,
      mirror_bars: false,
      show_peaks: false,
      peak_color: "#ffffff".to_string(),
      fire_width_ratio: Some(0.94),
      fire_height_scale: Some(1.0),
    }
  }
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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
  pub vignette_intensity: f32,
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
  pub glass_crack_intensity: f32,
}

impl Default for ScreenEffectsSettings {
  fn default() -> Self {
    // Mirror the legacy defaults (presets.ts) and the Slint control-panel
    // slider defaults so a fresh config has usable effect values.
    ScreenEffectsSettings {
      enabled: true,
      background_only: None,
      main_effect: ScreenEffect::None,
      shake_intensity: 1.0,
      shake_frequency: 0.5,
      shake_max_offset: 40.0,
      shake_on_beat: false,
      glitch_intensity: 0.5,
      pulse_intensity: 0.7,
      vignette_intensity: 0.7,
      spotlight_color: "#ffd700".to_string(),
      strobe_intensity: 0.8,
      scanline_opacity: 0.2,
      chromatic_intensity: 0.5,
      zoom_intensity: 0.2,
      invert_intensity: 0.5,
      bars_amount: 0.6,
      shockwave_intensity: 0.6,
      pixelate_intensity: 0.6,
      tilt_intensity: 0.6,
      heat_haze_intensity: 0.6,
      hue_shift_intensity: 0.6,
      glass_crack_intensity: 0.6,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

impl Default for VisualizerConfig {
  /// Custom default so `scale` starts at 1.0 (the derived Default was 0.0,
  /// which the renderers clamp to a barely-visible 0.1) and `reactivity`
  /// picks up the UI-matching defaults above instead of all zeros.
  fn default() -> Self {
    VisualizerConfig {
      style: VisualizerStyle::Spectrum,
      theme: ColorTheme::default(),
      background: BackgroundSettings::default(),
      text: TextSettings::default(),
      reactivity: AudioReactivitySettings::default(),
      export: ExportSettings::default(),
      screen_effects: ScreenEffectsSettings::default(),
      position_x: 0.0,
      position_y: 0.0,
      scale: 1.0,
    }
  }
}

#[cfg(test)]
mod defaults_tests {
  use super::*;

  #[test]
  fn reactivity_defaults_are_usable_not_zeroed() {
    // The visualizer must be responsive from the first frame, before the user
    // touches any control. A zeroed sensitivity/scale made every frequency bin
    // collapse to silence ("visualizer not responding to the music").
    let r = AudioReactivitySettings::default();
    assert!(r.sensitivity > 0.0, "sensitivity must be > 0");
    assert!(r.smoothing > 0.0);
    assert!(r.bass_multiplier > 0.0);
    assert!(r.bar_count >= 16);
    assert!(r.fft_size >= 64);
  }

  #[test]
  fn config_default_scale_is_visible() {
    let c = VisualizerConfig::default();
    assert_eq!(c.scale, 1.0);
    assert_eq!(c.style, VisualizerStyle::Spectrum);
    assert!(c.reactivity.sensitivity > 0.0);
  }

  #[test]
  fn serde_default_fills_reactivity_for_legacy_presets() {
    // A preset that omits the reactivity block (legacy export) must still
    // deserialize to a usable config, not zeros.
    let json = r#"{"style":"radial"}"#;
    let c: VisualizerConfig = serde_json::from_str(json).unwrap();
    assert_eq!(c.style, VisualizerStyle::Radial);
    assert!(c.reactivity.sensitivity > 0.0);
    assert_eq!(c.scale, 1.0);
  }
}
