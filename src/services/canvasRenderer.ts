import { VisualizerConfig } from '../types/visualizer';
import { audioEngine } from './audioEngine';
import { renderBackground } from './renderers/background';
import { renderSpectrumBars } from './renderers/spectrumBars';
import { renderRadialVisualizer } from './renderers/radial';
import { renderOscilloscopeVisualizer } from './renderers/oscilloscope';
import { renderEqualizerMatrix } from './renderers/equalizerMatrix';
import { renderMinimalWaveVisualizer } from './renderers/minimalWave';
import { renderWaveformFill } from './renderers/waveformFill';
import { renderCircularBars } from './renderers/circularBars';
import { renderSmoothSpectrum } from './renderers/smoothSpectrum';
import { renderMusicNotes } from './renderers/musicNotes';
import { renderTextOverlay } from './renderers/textOverlay';
import { renderParticles, initParticles } from './renderers/particles';
import { applyScreenEffects, getShakeOffset } from './renderers/screenEffects';
import type { RenderContext, MusicNote } from './renderers/types';

export class CanvasRenderer {
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;

  private freqData: Uint8Array = new Uint8Array(512);
  private timeData: Uint8Array = new Uint8Array(512);
  private peakData: number[] = [];

  private particles = initParticles();
  private musicNotes: MusicNote[] = [];
  private customImgElement: HTMLImageElement | null = null;

  private rotationAngle: number = 0;
  private bassEnergy: number = 0;

  private exportFreqData: Uint8Array | null = null;
  private exportTimeData: Uint8Array | null = null;
  private exportBassEnergy: number = 0;

  private rctx: RenderContext = {
    ctx: null as unknown as CanvasRenderingContext2D,
    width: 0,
    height: 0,
    config: null as unknown as VisualizerConfig,
    freqData: this.freqData,
    timeData: this.timeData,
    bassEnergy: 0,
    peakData: this.peakData,
    particles: this.particles,
    musicNotes: this.musicNotes,
    customImgElement: null,
    rotationAngle: 0,
    exportFreqData: null,
  };

  public setRotationAngle(angle: number) {
    this.rotationAngle = angle;
  }

  public setExportData(freqData: Uint8Array, timeData: Uint8Array, bassEnergy: number) {
    this.exportFreqData = freqData;
    this.exportTimeData = timeData;
    this.exportBassEnergy = bassEnergy;
  }

  public clearExportData() {
    this.exportFreqData = null;
    this.exportTimeData = null;
    this.exportBassEnergy = 0;
  }

  public init(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d', { alpha: false });
    this.rctx.ctx = this.ctx!;
  }

  public setCustomBackgroundImage(uri?: string) {
    if (!uri) {
      this.customImgElement = null;
      return;
    }
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.src = uri;
    this.customImgElement = img;
  }

  public async preloadImages(): Promise<void> {
    const promises: Promise<void>[] = [];
    if (this.customImgElement && !this.customImgElement.complete) {
      promises.push(new Promise((resolve) => {
        this.customImgElement!.onload = () => resolve();
      }));
    }
    await Promise.all(promises);
  }

  public drawFrame(config: VisualizerConfig) {
    if (!this.canvas || !this.ctx) return;

    const width = this.canvas.width;
    const height = this.canvas.height;

    if (this.exportFreqData) {
      this.freqData = this.exportFreqData;
      this.timeData = this.exportTimeData!;
      this.bassEnergy = this.exportBassEnergy;
    } else {
      const fftSize = config.reactivity.fftSize;
      if (this.freqData.length !== fftSize / 2) {
        this.freqData = new Uint8Array(fftSize / 2);
        this.timeData = new Uint8Array(fftSize / 2);
      }
      audioEngine.getFrequencyData(this.freqData);
      audioEngine.getTimeDomainData(this.timeData);

      let bassSum = 0;
      const bassBins = Math.min(16, this.freqData.length);
      for (let i = 0; i < bassBins; i++) {
        bassSum += this.freqData[i];
      }
      const targetBass = (bassSum / (bassBins * 255)) * config.reactivity.bassMultiplier;
      this.bassEnergy += (targetBass - this.bassEnergy) * 0.2;
    }

    const r = this.rctx;
    r.width = width;
    r.height = height;
    r.config = config;
    r.freqData = this.freqData;
    r.timeData = this.timeData;
    r.bassEnergy = this.bassEnergy;
    r.peakData = this.peakData;
    r.particles = this.particles;
    r.musicNotes = this.musicNotes;
    r.customImgElement = this.customImgElement;
    r.rotationAngle = this.rotationAngle;
    r.exportFreqData = this.exportFreqData;

    const shakeOff = getShakeOffset(config.screenEffects, this.bassEnergy);
    this.ctx.save();
    this.ctx.translate(shakeOff.x, shakeOff.y);

    renderBackground(r);

    if (config.background.showParticles) {
      renderParticles(r);
    }
    if (config.background.showMusicNotes) {
      renderMusicNotes(r);
    }

    this.ctx.save();
    switch (config.style) {
      case 'radial': renderRadialVisualizer(r); break;
      case 'oscilloscope': renderOscilloscopeVisualizer(r); break;
      case 'equalizer': renderEqualizerMatrix(r); break;
      case 'minimal': renderMinimalWaveVisualizer(r); break;
      case 'waveformFill': renderWaveformFill(r); break;
      case 'circularBars': renderCircularBars(r); break;
      case 'smoothSpectrum': renderSmoothSpectrum(r); break;
      case 'spectrum':
      default: renderSpectrumBars(r); break;
    }
    this.ctx.restore();

    renderTextOverlay(r);

    this.ctx.restore();

    applyScreenEffects(this.canvas, this.ctx, config.screenEffects, this.bassEnergy);

    this.rotationAngle = r.rotationAngle;
  }
}

export const canvasRenderer = new CanvasRenderer();
