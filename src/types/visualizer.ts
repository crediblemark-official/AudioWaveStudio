export type VisualizerStyle = 'spectrum' | 'radial' | 'oscilloscope' | 'equalizer' | 'minimal' | 'waveformFill' | 'circularBars' | 'smoothSpectrum' | 'pulseRings' | 'vuMeter' | 'auroraWave' | 'flameFire' | 'spiralGalaxy' | 'threeD' | 'api3D' | 'neonCity3D' | 'speaker3D' | 'speakerTrio' | 'speakerSplatter';

export type AspectRatio = '16:9' | '9:16' | '1:1';

export type ColorThemeName = 'cyberpunk' | 'synthwave' | 'emerald' | 'violet' | 'gold' | 'custom';

export type MusicNoteStyle = 'float' | 'bounce' | 'spiral' | 'wave' | 'burst' | 'confined';
export type ParticleStyle = 'float' | 'bounce' | 'wave' | 'static' | 'confined';

export type ScreenEffect = 'none' | 'shake' | 'glitch' | 'chromatic' | 'vignette' | 'pulse';

export interface ColorTheme {
  name: ColorThemeName;
  label: string;
  primaryColor: string;
  secondaryColor: string;
  accentColor: string;
  glowColor: string;
}

export interface BackgroundSettings {
  mode: 'solid' | 'gradient' | 'customImage';
  solidColor: string;
  gradientStart: string;
  gradientEnd: string;
  blurAmount: number; // 0 to 20 px
  overlayOpacity: number; // 0 to 1
  customImageUri?: string;
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
}

export interface TextSettings {
  songTitle: string;
  artistName: string;
  showTitle: boolean;
  showArtist: boolean;
  titleColor: string;
  artistColor: string;
  titleFontSize: number;
  artistFontSize: number;
  fontFamily: string;
  position: 'center' | 'bottom-left' | 'bottom-center' | 'top-center';
  textPositionX: number;
  textPositionY: number;
  textShadow: boolean;
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
  mainEffect: ScreenEffect;
  shakeIntensity: number;
  glitchIntensity: number;
  chromaticIntensity: number;
  pulseIntensity: number;
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
