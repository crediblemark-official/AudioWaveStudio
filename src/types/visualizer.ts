export type VisualizerStyle = 'spectrum' | 'radial' | 'oscilloscope' | 'equalizer' | 'minimal' | 'waveformFill' | 'circularBars' | 'smoothSpectrum' | 'pulseRings' | 'vuMeter' | 'auroraWave' | 'flameFire' | 'spiralGalaxy' | 'threeD' | 'api3D' | 'neonCity3D' | 'speaker3D' | 'speakerTrio' | 'speakerSplatter';

export type AspectRatio = '16:9' | '9:16' | '1:1';

export type ColorThemeName = 'cyberpunk' | 'synthwave' | 'emerald' | 'violet' | 'gold' | 'custom';

export type MusicNoteStyle = 'float' | 'bounce' | 'spiral' | 'wave' | 'burst' | 'confined';
export type ParticleStyle = 'float' | 'bounce' | 'wave' | 'static' | 'confined';

export type ScreenEffect = 'none' | 'shake' | 'glitch' | 'vignette' | 'pulse' | 'spotlight' | 'strobe' | 'scanline' | 'chromatic' | 'zoom' | 'invert' | 'bars' | 'shockwave' | 'pixelate' | 'tilt' | 'heatHaze' | 'hueShift';

export interface ColorTheme {
  name: ColorThemeName;
  label: string;
  primaryColor: string;
  secondaryColor: string;
  accentColor: string;
  glowColor: string;
}

export type BackgroundFillType = 'solid' | 'gradient';
export type BackgroundEffect = 'none' | 'grid' | 'particles' | 'musicNotes' | 'aurora' | 'noise' | 'bokeh' | 'starfield' | 'nebula' | 'psychedelic';

export interface BackgroundSettings {
  mode: 'solid' | 'gradient' | 'customImage' | 'grid' | 'aurora' | 'noise' | 'bokeh' | 'starfield' | 'nebula' | 'psychedelic';
  fillType?: BackgroundFillType;
  effect?: BackgroundEffect;
  effects?: BackgroundEffect[];
  solidColor: string;
  gradientStart: string;
  gradientEnd: string;
  blurAmount: number; // 0 to 20 px
  overlayOpacity: number; // 0 to 1
  customImageUri?: string;
  imageOpacity?: number; // 0 to 1
  gridColor?: string;
  gridSize?: number;
  gridLineWidth?: number;
  showParticles: boolean;
  particleStyle?: ParticleStyle;
  particleColor: string;
  particleSize?: number;
  particleSpeed?: number;
  particleCount?: number;
  showMusicNotes?: boolean;
  musicNoteStyle?: MusicNoteStyle;
  musicNoteColor?: string;
  radialCenterImageUri?: string;
  musicNoteDensity?: number;
  musicNoteSize?: number;
  musicNoteCount?: number;
  musicNoteSensitivity?: number;
  // Starfield settings
  starCount?: number;
  starSpeed?: number;
  starBrightness?: number;
  // Nebula settings
  nebulaIntensity?: number;
  nebulaSpeed?: number;
  // Aurora settings
  auroraSpeed?: number;
  auroraAmplitude?: number;
  auroraOpacity?: number;
  // Film Grain Noise settings
  grainOpacity?: number;
  // Bokeh settings
  bokehCount?: number;
  bokehSize?: number;
  bokehOpacity?: number;
  // Psychedelic settings
  psychedelicSpeed?: number;
  psychedelicBands?: number;
  psychedelicLineWidth?: number;
}

export type TextTransform = 'none' | 'uppercase' | 'lowercase' | 'capitalize';
export type TextAlign = 'left' | 'center' | 'right';

export interface TextBlock {
  id: string;
  text: string;
  enabled: boolean;
  fontFamily: string; // CSS font stack; '' = inherit global font
  fontSize: number;
  fontWeight: number;
  italic: boolean;
  color: string;
  useGradient: boolean;
  gradientStart: string;
  gradientEnd: string;
  gradientAngle: number; // degrees, 0 = left→right
  opacity: number; // 0 to 1
  letterSpacing: number; // px
  transform: TextTransform;
  positionX: number; // % of canvas width (anchor: left edge / center / right edge per align)
  positionY: number; // % of canvas height (baseline of first line)
  align: TextAlign;
  lineHeight: number; // multiplier of font size
  maxWidth: number; // % of canvas width for wrapping; 0 = no wrap
  shadow: boolean;
  shadowBlur: number; // px
  shadowOffsetX: number; // px
  shadowOffsetY: number; // px
  glowIntensity: number; // extra glow blur in px (0 = off)
  outline: boolean;
  outlineColor: string;
  outlineWidth: number; // px
  reactiveScale: number; // 0 to 1, how much text scales with bass
  waveEffect: boolean; // per-character wave animation
  fadeIn: boolean; // fade in on play
}

export interface TextSettings {
  songTitle: string;
  artistName: string;
  showTitle: boolean;
  showArtist: boolean;
  fontFamily: string;
  title: TextBlock;
  artist: TextBlock;
  blocks: TextBlock[]; // additional custom text blocks
}

export interface AudioReactivitySettings {
  fftSize: number; // 256, 512, 1024, 2048
  sensitivity: number; // 0.5 to 2.5
  bassMultiplier: number; // 1.0 to 3.0 (reactive pulse)
  barCount: number; // 16 to 128
  barWidth: number; // px or ratio
  barGap: number; // px
  barRounding: number; // px
  smoothing: number; // 0.1 to 0.95
  mirrorBars: boolean;
  showPeaks: boolean;
  peakColor: string;
  fireWidthRatio?: number;
  fireHeightScale?: number;
}

export interface ExportSettings {
  aspectRatio: AspectRatio;
  resolution: '1080p' | '720p' | '4K';
  fps: 30 | 60;
  format: 'mp4' | 'webm';
}

export interface ScreenEffectsSettings {
  enabled: boolean;
  backgroundOnly?: boolean; // If true, screen effects apply only to background layer, leaving visualizer on top
  mainEffect: ScreenEffect;
  shakeIntensity: number;
  shakeFrequency: number;
  shakeMaxOffset: number;
  shakeOnBeat: boolean;
  glitchIntensity: number;
  pulseIntensity: number;
  spotlightColor: string;
  strobeIntensity: number;
  scanlineOpacity: number;
  chromaticIntensity: number;
  zoomIntensity: number;
  invertIntensity: number;
  barsAmount: number;
  shockwaveIntensity: number;
  pixelateIntensity: number;
  tiltIntensity: number;
  heatHazeIntensity: number;
  hueShiftIntensity: number;
}

export interface SongMetadata {
  fileName: string;
  title: string;
  artist: string;
  duration: number;
  audioUrl?: string;
}

export interface VisualizerConfig {
  style: VisualizerStyle;
  theme: ColorTheme;
  background: BackgroundSettings;
  text: TextSettings;
  reactivity: AudioReactivitySettings;
  export: ExportSettings;
  screenEffects: ScreenEffectsSettings;
  positionX: number;
  positionY: number;
  scale: number;
}

export interface ThemePreset {
  id: string;
  name: string;
  description: string;
  config: Partial<VisualizerConfig>;
}
