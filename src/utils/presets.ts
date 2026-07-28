import { ColorTheme, ThemePreset, VisualizerConfig } from '../types/visualizer';

export const COLOR_THEMES: Record<string, ColorTheme> = {
  cyberpunk: {
    name: 'cyberpunk',
    label: 'Cyberpunk Neon',
    primaryColor: '#00f0ff',
    secondaryColor: '#ff007f',
    accentColor: '#ffe600',
    glowColor: '#00f0ff'
  },
  synthwave: {
    name: 'synthwave',
    label: 'Sunset Synthwave',
    primaryColor: '#ff7700',
    secondaryColor: '#bf00ff',
    accentColor: '#ff0055',
    glowColor: '#ff007f'
  },
  emerald: {
    name: 'emerald',
    label: 'Emerald Aurora',
    primaryColor: '#00ff88',
    secondaryColor: '#00bfff',
    accentColor: '#7000ff',
    glowColor: '#00ff88'
  },
  violet: {
    name: 'violet',
    label: 'Deep Violet',
    primaryColor: '#8a2be2',
    secondaryColor: '#da70d6',
    accentColor: '#00ffff',
    glowColor: '#8a2be2'
  },
  gold: {
    name: 'gold',
    label: 'Solar Gold',
    primaryColor: '#ffd700',
    secondaryColor: '#ff8c00',
    accentColor: '#ffffff',
    glowColor: '#ffd700'
  }
};

export const DEFAULT_CONFIG: VisualizerConfig = {
  style: 'spectrum',
  theme: COLOR_THEMES.cyberpunk,
  background: {
    mode: 'solid',
    solidColor: '#0b0c10',
    gradientStart: '#0f0c20',
    gradientEnd: '#06101e',
    blurAmount: 8,
    overlayOpacity: 0.7,
    showParticles: true,
    particleColor: '#00f0ff',
    showMusicNotes: true,
    musicNoteStyle: 'float',
    musicNoteColor: '#ffe600',
    musicNoteDensity: 0.5,
    musicNoteSize: 24,
    musicNoteSpeed: 1.0,
    musicNoteCount: 40
  },
  text: {
    songTitle: 'Electrifying Night',
    artistName: 'Synthwave Producer',
    showTitle: true,
    showArtist: true,
    titleColor: '#ffffff',
    artistColor: '#00f0ff',
    titleFontSize: 28,
    artistFontSize: 16,
    position: 'bottom-center',
    textShadow: true
  },
  reactivity: {
    fftSize: 1024,
    sensitivity: 1.2,
    bassMultiplier: 1.8,
    barCount: 48,
    barWidth: 6,
    barGap: 4,
    barRounding: 4,
    smoothing: 0.8,
    mirrorBars: false,
    showPeaks: true,
    peakColor: '#ffe600'
  },
  export: {
    aspectRatio: '16:9',
    resolution: '1080p',
    fps: 60,
    format: 'mp4'
  },
  screenEffects: {
    enabled: false,
    mainEffect: 'shake',
    shakeIntensity: 0.3,
    glitchIntensity: 0.4,
    chromaticIntensity: 0.3,
    pulseIntensity: 0.5,
  }
};

export const PRESETS: ThemePreset[] = [
  {
    id: 'cyberpunk-spectrum',
    name: 'Cyberpunk Spectrum',
    description: 'Neon cyan frequency bars with dynamic bass glow and floating particles.',
    config: {
      style: 'spectrum',
      theme: COLOR_THEMES.cyberpunk,
      background: {
        mode: 'solid',
        solidColor: '#080810',
        gradientStart: '#080810',
        gradientEnd: '#150a21',
        blurAmount: 8,
        overlayOpacity: 0.7,
        showParticles: true,
        particleColor: '#00f0ff'
      }
    }
  },
  {
    id: 'radial-aurora',
    name: 'Radial Aurora Ring',
    description: 'Circular audio ring with pulsing particle burst.',
    config: {
      style: 'radial',
      theme: COLOR_THEMES.emerald,
      background: {
        mode: 'gradient',
        solidColor: '#051210',
        gradientStart: '#051210',
        gradientEnd: '#0a2520',
        blurAmount: 12,
        overlayOpacity: 0.75,
        showParticles: true,
        particleColor: '#00ff88'
      }
    }
  },
  {
    id: 'neon-oscilloscope',
    name: 'Neon Oscilloscope',
    description: 'Multi-layer glowing bezier wave curves with bloom effects.',
    config: {
      style: 'oscilloscope',
      theme: COLOR_THEMES.synthwave,
      background: {
        mode: 'gradient',
        solidColor: '#120516',
        gradientStart: '#180520',
        gradientEnd: '#090210',
        blurAmount: 6,
        overlayOpacity: 0.8,
        showParticles: true,
        particleColor: '#ff0055'
      }
    }
  },
  {
    id: 'retro-matrix',
    name: 'Retro Equalizer Matrix',
    description: 'Pixel matrix equalizer with peak drop indicators.',
    config: {
      style: 'equalizer',
      theme: COLOR_THEMES.gold,
      background: {
        mode: 'solid',
        solidColor: '#0f0e08',
        gradientStart: '#0f0e08',
        gradientEnd: '#1a1805',
        blurAmount: 0,
        overlayOpacity: 0.8,
        showParticles: false,
        particleColor: '#ffd700'
      }
    }
  },
  {
    id: 'minimal-studio',
    name: 'Minimal Studio Wave',
    description: 'Clean modern pill bars with elegant typography.',
    config: {
      style: 'minimal',
      theme: COLOR_THEMES.violet,
      background: {
        mode: 'solid',
        solidColor: '#0d0d12',
        gradientStart: '#0d0d12',
        gradientEnd: '#14141f',
        blurAmount: 0,
        overlayOpacity: 0.9,
        showParticles: true,
        particleColor: '#ffffff'
      }
    }
  }
];
