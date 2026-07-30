import { describe, it, expect } from 'vitest';
import { getExportDimensions } from './videoExporter';
import type { VisualizerConfig } from '../types/visualizer';

function makeConfig(overrides?: Partial<VisualizerConfig>): VisualizerConfig {
  return {
    style: 'spectrum',
    theme: {
      name: 'custom',
      label: 'Custom',
      primaryColor: '#00f0ff',
      secondaryColor: '#ff00aa',
      accentColor: '#ffaa00',
      glowColor: '#00f0ff',
    },
    background: {
      mode: 'solid',
      solidColor: '#0a0a0f',
      gradientStart: '#000',
      gradientEnd: '#111',
      blurAmount: 0,
      overlayOpacity: 0,
      showParticles: false,
      particleColor: '#fff',
    },
    text: {
      songTitle: 'Test Song',
      artistName: 'Test Artist',
      showTitle: false,
      showArtist: false,
      titleColor: '#fff',
      artistColor: '#aaa',
      titleFontSize: 24,
      artistFontSize: 18,
      fontFamily: 'Arial',
      position: 'center',
      textPositionX: 0,
      textPositionY: 0,
      textShadow: false,
    },
    reactivity: {
      fftSize: 1024,
      sensitivity: 1,
      bassMultiplier: 1,
      barCount: 64,
      barWidth: 4,
      barGap: 2,
      barRounding: 2,
      smoothing: 0.8,
      mirrorBars: true,
      showPeaks: true,
      peakColor: '#fff',
    },
    export: {
      aspectRatio: '16:9',
      resolution: '1080p',
      fps: 60,
      format: 'mp4',
    },
    screenEffects: {
      enabled: false,
      mainEffect: 'none',
      shakeIntensity: 0,
      glitchIntensity: 0,
      pulseIntensity: 0,
      spotlightColor: '#fff',
    },
    positionX: 0,
    positionY: 0,
    scale: 1,
    ...overrides,
  };
}

describe('getExportDimensions', () => {
  it('returns 1920x1080 for 1080p 16:9', () => {
    const config = makeConfig({ export: { aspectRatio: '16:9', resolution: '1080p', fps: 60, format: 'mp4' } });
    expect(getExportDimensions(config)).toEqual({ width: 1920, height: 1080 });
  });

  it('returns 1280x720 for 720p 16:9', () => {
    const config = makeConfig({ export: { aspectRatio: '16:9', resolution: '720p', fps: 60, format: 'mp4' } });
    expect(getExportDimensions(config)).toEqual({ width: 1280, height: 720 });
  });

  it('returns 3840x2160 for 4K 16:9', () => {
    const config = makeConfig({ export: { aspectRatio: '16:9', resolution: '4K', fps: 60, format: 'mp4' } });
    expect(getExportDimensions(config)).toEqual({ width: 3840, height: 2160 });
  });

  it('swaps dimensions for 9:16 (portrait)', () => {
    const config = makeConfig({ export: { aspectRatio: '9:16', resolution: '1080p', fps: 60, format: 'mp4' } });
    expect(getExportDimensions(config)).toEqual({ width: 1080, height: 1920 });
  });

  it('returns square for 1:1 (height = width)', () => {
    const config = makeConfig({ export: { aspectRatio: '1:1', resolution: '1080p', fps: 60, format: 'mp4' } });
    expect(getExportDimensions(config)).toEqual({ width: 1920, height: 1920 });
  });

  it('720p 9:16 portrait', () => {
    const config = makeConfig({ export: { aspectRatio: '9:16', resolution: '720p', fps: 60, format: 'mp4' } });
    expect(getExportDimensions(config)).toEqual({ width: 720, height: 1280 });
  });

  it('4K 1:1 square', () => {
    const config = makeConfig({ export: { aspectRatio: '1:1', resolution: '4K', fps: 60, format: 'mp4' } });
    expect(getExportDimensions(config)).toEqual({ width: 3840, height: 3840 });
  });
});
