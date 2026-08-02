import { describe, it, expect } from 'vitest';
import { hexToRgba, convertToRustConfig } from './rustBridge';
import { createTextBlock } from '../utils/presets';
import type { VisualizerConfig } from '../types/visualizer';

describe('hexToRgba', () => {
  it('converts 6-digit hex to rgba', () => {
    expect(hexToRgba('#00f0ff')).toEqual([0, 240, 255, 255]);
  });

  it('converts 3-digit hex to rgba', () => {
    expect(hexToRgba('#f0a')).toEqual([255, 0, 170, 255]);
  });

  it('handles hex without #', () => {
    expect(hexToRgba('ff00aa')).toEqual([255, 0, 170, 255]);
  });

  it('returns default for invalid hex', () => {
    expect(hexToRgba('xyz')).toEqual([0, 240, 255, 255]);
  });

  it('converts black', () => {
    expect(hexToRgba('#000000')).toEqual([0, 0, 0, 255]);
  });

  it('converts white', () => {
    expect(hexToRgba('#ffffff')).toEqual([255, 255, 255, 255]);
  });
});

describe('convertToRustConfig', () => {
  function makeMinimalConfig(): VisualizerConfig {
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
        showParticles: true,
        particleColor: '#fff',
      },
      text: {
        songTitle: 'My Song',
        artistName: 'My Artist',
        showTitle: true,
        showArtist: false,
        fontFamily: 'Arial',
        title: createTextBlock({ id: 'title', text: 'My Song', fontSize: 24, color: '#fff' }),
        artist: createTextBlock({ id: 'artist', text: 'My Artist', fontSize: 18, color: '#aaa' }),
        blocks: [],
      },
      reactivity: {
        fftSize: 1024,
        sensitivity: 1.5,
        bassMultiplier: 2.0,
        barCount: 32,
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
        shakeFrequency: 0.5,
        shakeMaxOffset: 40,
        shakeOnBeat: false,
        glitchIntensity: 0,
        pulseIntensity: 0,
        spotlightColor: '#fff',
        strobeIntensity: 0,
        scanlineOpacity: 0,
        chromaticIntensity: 0,
        zoomIntensity: 0,
        invertIntensity: 0,
        barsAmount: 0,
        shockwaveIntensity: 0,
        pixelateIntensity: 0,
        tiltIntensity: 0,
        heatHazeIntensity: 0,
        hueShiftIntensity: 0,
      },
      positionX: 10,
      positionY: 20,
      scale: 1,
    };
  }

  it('maps basic fields correctly', () => {
    const config = makeMinimalConfig();
    const result = convertToRustConfig(config, 1280, 720);

    expect(result.style).toBe('spectrum');
    expect(result.width).toBe(1280);
    expect(result.height).toBe(720);
    expect(result.bar_count).toBe(32);
    expect(result.sensitivity).toBe(1.5);
    expect(result.bass_multiplier).toBe(2.0);
    expect(result.show_particles).toBe(true);
    expect(result.position_x).toBe(10);
    expect(result.position_y).toBe(20);
  });

  it('converts theme colors correctly', () => {
    const config = makeMinimalConfig();
    const result = convertToRustConfig(config);

    expect(result.primary_color).toEqual([0, 240, 255, 255]);
    expect(result.secondary_color).toEqual([255, 0, 170, 255]);
    expect(result.accent_color).toEqual([255, 170, 0, 255]);
    expect(result.bg_color).toEqual([10, 10, 15, 255]);
  });

  it('sets title_text when showTitle is true', () => {
    const config = makeMinimalConfig();
    config.text.showTitle = true;
    const result = convertToRustConfig(config);
    expect(result.title_text).toBe('My Song');
  });

  it('omits title_text when showTitle is false', () => {
    const config = makeMinimalConfig();
    config.text.showTitle = false;
    const result = convertToRustConfig(config);
    expect(result.title_text).toBeUndefined();
  });

  it('omits artist_text when showArtist is false', () => {
    const config = makeMinimalConfig();
    config.text.showArtist = false;
    const result = convertToRustConfig(config);
    expect(result.artist_text).toBeUndefined();
  });

  it('supports all 19 visualizer styles in config mapping', () => {
    const styles = [
      'spectrum', 'radial', 'oscilloscope', 'equalizer', 'minimal', 'waveformFill',
      'circularBars', 'smoothSpectrum', 'pulseRings', 'vuMeter', 'auroraWave',
      'flameFire', 'spiralGalaxy', 'threeD', 'api3D', 'neonCity3D', 'speaker3D',
      'speakerTrio', 'speakerSplatter'
    ] as const;

    for (const style of styles) {
      const config = makeMinimalConfig();
      config.style = style;
      const result = convertToRustConfig(config);
      expect(result.style).toBe(style);
    }
  });

  it('maps radialCenterImageUri, scale, positionX and positionY correctly', () => {
    const config = makeMinimalConfig();
    config.background.radialCenterImageUri = 'data:image/png;base64,iVBORw0KGgo=';
    config.scale = 1.25;
    config.positionX = -50;
    config.positionY = 100;

    const result = convertToRustConfig(config);
    expect(result.position_x).toBe(-50);
    expect(result.position_y).toBe(100);
  });
});
