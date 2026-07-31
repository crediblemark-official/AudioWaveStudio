import { ColorTheme, TextBlock, TextSettings, ThemePreset, VisualizerConfig } from '../types/visualizer';

export function createTextBlock(overrides: Partial<TextBlock> = {}): TextBlock {
  return {
    id:
      typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
        ? crypto.randomUUID()
        : `block-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    text: '',
    enabled: true,
    fontFamily: '',
    fontSize: 24,
    fontWeight: 700,
    italic: false,
    color: '#ffffff',
    useGradient: false,
    gradientStart: '#ffffff',
    gradientEnd: '#00f0ff',
    gradientAngle: 0,
    opacity: 1,
    letterSpacing: 0,
    transform: 'none',
    positionX: 50,
    positionY: 78,
    align: 'center',
    lineHeight: 1.3,
    maxWidth: 0,
    shadow: false,
    shadowBlur: 12,
    shadowOffsetX: 0,
    shadowOffsetY: 0,
    glowIntensity: 0,
    outline: false,
    outlineColor: '#000000',
    outlineWidth: 2,
    reactiveScale: 0,
    waveEffect: false,
    fadeIn: false,
    ...overrides,
  };
}

function legacyValue(t: Record<string, unknown>, key: string, fallback: number | string | boolean): number | string | boolean {
  const v = t[key];
  return typeof v === 'number' || typeof v === 'string' || typeof v === 'boolean' ? v : fallback;
}

export function normalizeTextBlock(block?: Partial<TextBlock>, legacy?: Partial<TextBlock>): TextBlock {
  return createTextBlock({ ...(legacy || {}), ...(block || {}) });
}

export function migrateTextSettings(text?: Partial<TextSettings>): TextSettings {
  const fallback = DEFAULT_CONFIG.text;
  const t = (text || {}) as Partial<TextSettings> & Record<string, unknown>;
  const titleX = legacyValue(t, 'textPositionX', fallback.title.positionX) as number;
  const titleY = legacyValue(t, 'textPositionY', fallback.title.positionY) as number;

  return {
    ...fallback,
    ...t,
    songTitle: typeof t.songTitle === 'string' ? t.songTitle : fallback.songTitle,
    artistName: typeof t.artistName === 'string' ? t.artistName : fallback.artistName,
    showTitle: typeof t.showTitle === 'boolean' ? t.showTitle : fallback.showTitle,
    showArtist: typeof t.showArtist === 'boolean' ? t.showArtist : fallback.showArtist,
    fontFamily: typeof t.fontFamily === 'string' ? t.fontFamily : fallback.fontFamily,
    title: normalizeTextBlock(t.title as Partial<TextBlock> | undefined, {
      id: 'title',
      text: (typeof t.songTitle === 'string' ? t.songTitle : fallback.songTitle) || fallback.songTitle,
      fontSize: legacyValue(t, 'titleFontSize', fallback.title.fontSize) as number,
      color: legacyValue(t, 'titleColor', fallback.title.color) as string,
      positionX: titleX,
      positionY: titleY,
      shadow: legacyValue(t, 'textShadow', fallback.title.shadow) as boolean,
    }),
    artist: normalizeTextBlock(t.artist as Partial<TextBlock> | undefined, {
      id: 'artist',
      text: (typeof t.artistName === 'string' ? t.artistName : fallback.artistName) || fallback.artistName,
      fontSize: legacyValue(t, 'artistFontSize', fallback.artist.fontSize) as number,
      color: legacyValue(t, 'artistColor', fallback.artist.color) as string,
      positionX: titleX,
      positionY: Math.min(100, titleY + 6),
    }),
    blocks: Array.isArray(t.blocks) ? (t.blocks as Partial<TextBlock>[]).map((b) => normalizeTextBlock(b)) : [],
  };
}

const STORAGE_KEY = 'audiowave-config';

export function loadSavedConfig(): VisualizerConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_CONFIG;
    const saved = JSON.parse(raw) as Partial<VisualizerConfig>;
    return {
      ...DEFAULT_CONFIG,
      ...saved,
      background: { ...DEFAULT_CONFIG.background, ...(saved.background || {}) },
      text: migrateTextSettings(saved.text),
      reactivity: { ...DEFAULT_CONFIG.reactivity, ...(saved.reactivity || {}) },
      export: { ...DEFAULT_CONFIG.export, ...(saved.export || {}) },
      screenEffects: { ...DEFAULT_CONFIG.screenEffects, ...(saved.screenEffects || {}) },
    };
  } catch (e) {
    console.error('Failed to load saved config:', e);
    return DEFAULT_CONFIG;
  }
}

export function saveConfig(config: VisualizerConfig): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
  } catch (e) {
    console.error('Failed to save config:', e);
  }
}

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
    overlayOpacity: 0,
    gridColor: '#ffffff',
    gridSize: 40,
    gridLineWidth: 1,
    showParticles: true,
    particleStyle: 'float',
    particleColor: '#00f0ff',
    particleSize: 4,
    particleSpeed: 1.0,
    particleCount: 60,
    showMusicNotes: true,
    musicNoteStyle: 'float',
    musicNoteColor: '#ffe600',
    musicNoteDensity: 1.0,
    musicNoteSize: 60,
    musicNoteCount: 80,
    musicNoteSensitivity: 1.0
  },
  text: {
    songTitle: 'Electrifying Night',
    artistName: 'Synthwave Producer',
    showTitle: true,
    showArtist: true,
    fontFamily: '"Outfit", "Inter", sans-serif',
    title: createTextBlock({
      id: 'title',
      text: 'Electrifying Night',
      fontSize: 28,
      fontWeight: 700,
      color: '#ffffff',
      positionX: 50,
      positionY: 78,
      align: 'center',
      shadow: true,
    }),
    artist: createTextBlock({
      id: 'artist',
      text: 'Synthwave Producer',
      fontSize: 16,
      fontWeight: 500,
      color: '#00f0ff',
      positionX: 50,
      positionY: 86,
      align: 'center',
    }),
    blocks: [],
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
    peakColor: '#ffe600',
    fireWidthRatio: 0.94,
    fireHeightScale: 1.0
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
    shakeFrequency: 0.5,
    shakeMaxOffset: 40,
    shakeOnBeat: false,
    glitchIntensity: 0.4,

    pulseIntensity: 0.5,
    spotlightColor: '#FFD700',
    strobeIntensity: 0.5,
    scanlineOpacity: 0.12,
    chromaticIntensity: 0.3,
    zoomIntensity: 0.15,
    invertIntensity: 0.3,
    barsAmount: 0.5,
    shockwaveIntensity: 0.6,
    pixelateIntensity: 0.4,
    tiltIntensity: 0.3,
    heatHazeIntensity: 0.4,
    hueShiftIntensity: 0.3,
  },
  positionX: 0,
  positionY: 0,
  scale: 1,
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
        overlayOpacity: 0,
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
        overlayOpacity: 0,
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
        overlayOpacity: 0,
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
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#ffd700'
      }
    }
  },
  {
    id: 'spiral-galaxy',
    name: 'Spiral Galaxy',
    description: 'Particle galaxy orbiting in a spiral with the music.',
    config: {
      style: 'spiralGalaxy',
      theme: COLOR_THEMES.synthwave,
      background: {
        mode: 'solid',
        solidColor: '#020210',
        gradientStart: '#020210',
        gradientEnd: '#080828',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#4444ff'
      }
    }
  },
  {
    id: 'flame-fire',
    name: 'Flame Fire',
    description: 'Dynamic fire particles that rise with the bass.',
    config: {
      style: 'flameFire',
      theme: COLOR_THEMES.gold,
      background: {
        mode: 'solid',
        solidColor: '#0a0500',
        gradientStart: '#0a0500',
        gradientEnd: '#1a0a00',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#ff4400'
      }
    }
  },
  {
    id: 'aurora-wave',
    name: 'Aurora Wave',
    description: 'Flowing gradient aurora waves that undulate with the music.',
    config: {
      style: 'auroraWave',
      theme: COLOR_THEMES.synthwave,
      background: {
        mode: 'solid',
        solidColor: '#050510',
        gradientStart: '#050510',
        gradientEnd: '#0a0a20',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#ff00aa'
      }
    }
  },
  {
    id: 'vu-meter',
    name: 'VU Meter',
    description: 'Classic analog VU meters with bouncing needles.',
    config: {
      style: 'vuMeter',
      theme: COLOR_THEMES.emerald,
      background: {
        mode: 'solid',
        solidColor: '#080808',
        gradientStart: '#080808',
        gradientEnd: '#0a0a0a',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#00ff88'
      }
    }
  },
  {
    id: 'pulse-rings',
    name: 'Pulse Rings',
    description: 'Concentric rings that pulse outward with the beat.',
    config: {
      style: 'pulseRings',
      theme: COLOR_THEMES.synthwave,
      background: {
        mode: 'solid',
        solidColor: '#050510',
        gradientStart: '#050510',
        gradientEnd: '#0a0a20',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#ff00aa'
      }
    }
  },
  {
    id: 'neon-city-3d',
    name: 'Neon City 3D',
    description: '3D cyberpunk skyline: matrix depth bars with neon spectrum gradient and glass floor reflection.',
    config: {
      style: 'neonCity3D',
      theme: COLOR_THEMES.cyberpunk,
      background: {
        mode: 'solid',
        solidColor: '#000000',
        gradientStart: '#000000',
        gradientEnd: '#050510',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#00f0ff'
      }
    }
  },
  {
    id: 'api-3d',
    name: 'Fire 3D',
    description: '3D volumetric fire with dancing flame columns and flying embers.',
    config: {
      style: 'api3D',
      theme: COLOR_THEMES.cyberpunk,
      background: {
        mode: 'solid',
        solidColor: '#050515',
        gradientStart: '#050515',
        gradientEnd: '#0a0a25',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#00f0ff'
      }
    }
  },
  {
    id: '3d-blocks',
    name: '3D Blocks',
    description: '3D extruded bars with perspective floor depth and bass-reactive glow.',
    config: {
      style: 'threeD',
      theme: COLOR_THEMES.cyberpunk,
      background: {
        mode: 'solid',
        solidColor: '#050510',
        gradientStart: '#050510',
        gradientEnd: '#0a0a20',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#00f0ff'
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
        overlayOpacity: 0,
        showParticles: true,
        particleColor: '#ffffff'
      }
    }
  },
  {
    id: 'speaker-3d',
    name: 'Realistic 3D Speaker',
    description: 'Realistic 3D subwoofer speaker cone with dynamic bass pulse vibration and fiery spectrum bars.',
    config: {
      style: 'speaker3D',
      theme: COLOR_THEMES.synthwave,
      background: {
        mode: 'solid',
        solidColor: '#0a0a0c',
        gradientStart: '#0e0b14',
        gradientEnd: '#06050a',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: true,
        particleColor: '#ff7700'
      }
    }
  },
  {
    id: 'speaker-trio',
    name: 'Triple Speaker Studio',
    description: 'Triple 3D subwoofer setup with halftone dot spectrum matrix and dancing floating music notes.',
    config: {
      style: 'speakerTrio',
      theme: COLOR_THEMES.cyberpunk,
      background: {
        mode: 'solid',
        solidColor: '#f4f5fa',
        gradientStart: '#ffffff',
        gradientEnd: '#e6e8f2',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#00f0ff'
      }
    }
  },
  {
    id: 'speaker-splatter',
    name: 'Grunge Paint Splatter Speaker',
    description: 'Urban grunge paint splatter explosion with cyan & magenta halftone dot bursts and triple speaker cluster.',
    config: {
      style: 'speakerSplatter',
      theme: COLOR_THEMES.cyberpunk,
      background: {
        mode: 'solid',
        solidColor: '#f8f9fc',
        gradientStart: '#ffffff',
        gradientEnd: '#ebeef5',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#ff007f'
      }
    }
  },
  {
    id: 'waveform-fill',
    name: 'Waveform Fill',
    description: 'Solid waveform fill that traces the audio signal with a bold colored silhouette.',
    config: {
      style: 'waveformFill',
      theme: COLOR_THEMES.cyberpunk,
      background: {
        mode: 'solid',
        solidColor: '#080810',
        gradientStart: '#080810',
        gradientEnd: '#150a21',
        blurAmount: 8,
        overlayOpacity: 0,
        showParticles: false,
        particleColor: '#00f0ff'
      }
    }
  },
  {
    id: 'circular-bars',
    name: 'Circular Bars',
    description: 'Radial ring of bars arranged in a full circle with bass-reactive center glow.',
    config: {
      style: 'circularBars',
      theme: COLOR_THEMES.synthwave,
      background: {
        mode: 'gradient',
        solidColor: '#070515',
        gradientStart: '#070515',
        gradientEnd: '#0f0a20',
        blurAmount: 0,
        overlayOpacity: 0,
        showParticles: true,
        particleColor: '#ff00aa'
      }
    }
  },
  {
    id: 'smooth-spectrum',
    name: 'Smooth Spectrum',
    description: 'Buttery smooth continuous frequency curve with soft glow and gradient fill.',
    config: {
      style: 'smoothSpectrum',
      theme: COLOR_THEMES.emerald,
      background: {
        mode: 'gradient',
        solidColor: '#051210',
        gradientStart: '#051210',
        gradientEnd: '#0a2520',
        blurAmount: 12,
        overlayOpacity: 0,
        showParticles: true,
        particleColor: '#00ff88'
      }
    }
  }
];
