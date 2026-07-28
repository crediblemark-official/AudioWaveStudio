export type VisualizerStyle = 'spectrum' | 'radial' | 'oscilloscope' | 'equalizer' | 'minimal' | 'waveformFill' | 'circularBars' | 'smoothSpectrum';

export type AspectRatio = '16:9' | '9:16' | '1:1';

export type ColorThemeName = 'cyberpunk' | 'synthwave' | 'emerald' | 'violet' | 'gold' | 'custom';

export type MusicNoteStyle = 'float' | 'bounce' | 'spiral' | 'wave' | 'burst';

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
  particleColor: string;
  showMusicNotes?: boolean;
  musicNoteStyle?: MusicNoteStyle;
  musicNoteColor?: string;
  musicNoteDensity?: number;
  musicNoteSize?: number;
  musicNoteSpeed?: number;
  musicNoteCount?: number;
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
  position: 'center' | 'bottom-left' | 'bottom-center' | 'top-center';
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
}

export interface ThemePreset {
  id: string;
  name: string;
  description: string;
  config: Partial<VisualizerConfig>;
}
