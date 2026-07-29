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
import { renderPulseRings } from './renderers/pulseRings';
import { renderVuMeter } from './renderers/vuMeter';
import { renderAuroraWave } from './renderers/auroraWave';
import { renderFlameFire } from './renderers/flameFire';
import { renderSpiralGalaxy } from './renderers/spiralGalaxy';
import { renderThreeD } from './renderers/threeD';
import { renderApi3D } from './renderers/api3D';
import { renderNeonCity3D } from './renderers/neonCity3D';
import { renderSpeaker3D } from './renderers/speaker3D';
import { renderSpeakerTrio } from './renderers/speakerTrio';
import { renderSpeakerSplatter } from './renderers/speakerSplatter';
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
  private radialCenterImgElement: HTMLImageElement | null = null;

  private rotationAngle: number = 0;
  private bassEnergy: number = 0;
  private bassEnergyRaw: number = 0;
  private beatStrength: number = 0;
  private beatStrengthRaw: number = 0;
  private prevTargetBass: number = 0;
  private prevRawBass: number = 0;
  private bassFloor: number = 0;

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
    bassEnergyRaw: 0,
    beatStrength: 0,
    beatStrengthRaw: 0,
    peakData: this.peakData,
    particles: this.particles,
    musicNotes: this.musicNotes,
    customImgElement: null,
    radialCenterImgElement: null,
    rotationAngle: 0,
    exportFreqData: null,
    isPlaying: false,
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

  public setRadialCenterImage(uri?: string) {
    if (!uri) {
      this.radialCenterImgElement = null;
      return;
    }
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.src = uri;
    this.radialCenterImgElement = img;
  }

  public async preloadImages(): Promise<void> {
    const promises: Promise<void>[] = [];
    if (this.customImgElement && !this.customImgElement.complete) {
      promises.push(new Promise((resolve) => {
        this.customImgElement!.onload = () => resolve();
      }));
    }
    if (this.radialCenterImgElement && !this.radialCenterImgElement.complete) {
      promises.push(new Promise((resolve) => {
        this.radialCenterImgElement!.onload = () => resolve();
      }));
    }
    await Promise.all(promises);
  }

  public drawFrame(config: VisualizerConfig) {
    if (!this.canvas || !this.ctx) return;

    const width = this.canvas.width;
    const height = this.canvas.height;

    let targetBass: number;
    let rawBass: number;

    if (this.exportFreqData) {
      this.freqData = this.exportFreqData;
      this.timeData = this.exportTimeData!;
      this.bassEnergy = this.exportBassEnergy;
      this.bassEnergyRaw = this.exportBassEnergy;
      targetBass = this.exportBassEnergy;
      rawBass = targetBass;
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
       rawBass = bassSum / (bassBins * 255);
       targetBass = rawBass * (config.reactivity.bassMultiplier || 1.0) * (config.reactivity.sensitivity || 1.0);
      this.bassEnergy += (targetBass - this.bassEnergy) * 0.2;
      this.bassEnergyRaw += (rawBass - this.bassEnergyRaw) * 0.2;
    }

    if (targetBass < this.bassFloor) {
      this.bassFloor = targetBass;
    } else {
      this.bassFloor += (targetBass - this.bassFloor) * 0.0008;
    }
    const aboveFloor = Math.max(0, this.bassEnergy - this.bassFloor);

    const onset = Math.max(0, targetBass - this.prevTargetBass);
    this.prevTargetBass = targetBass;
    if (onset > 0.03) {
      this.beatStrength = Math.max(onset * 6, this.beatStrength * 0.6);
    } else {
      this.beatStrength *= 0.7;
    }

    const rawOnset = Math.max(0, rawBass - (this.prevRawBass || 0));
    this.prevRawBass = rawBass;
    this.beatStrengthRaw = rawOnset > 0.06
      ? Math.max(rawOnset * 5, this.beatStrengthRaw * 0.5)
      : this.beatStrengthRaw * 0.5;

    const r = this.rctx;
    r.width = width;
    r.height = height;
    r.config = config;
    r.freqData = this.freqData;
    r.timeData = this.timeData;
    r.bassEnergy = this.bassEnergy;
    r.bassEnergyRaw = this.bassEnergyRaw;
    r.beatStrength = this.beatStrength;
    r.beatStrengthRaw = this.beatStrengthRaw;
    r.peakData = this.peakData;
    r.particles = this.particles;
    r.musicNotes = this.musicNotes;
    r.customImgElement = this.customImgElement;
    r.radialCenterImgElement = this.radialCenterImgElement;
    r.rotationAngle = this.rotationAngle;
    r.exportFreqData = this.exportFreqData;
    r.isPlaying = audioEngine.getIsPlaying();

    const shakeOff = getShakeOffset(config.screenEffects, this.bassEnergy, this.beatStrength, aboveFloor);
    this.ctx.save();
    this.ctx.translate(shakeOff.x, shakeOff.y);

    renderBackground(r);

    const overlay = config.background.overlayOpacity || 0;
    if (overlay > 0) {
      this.ctx.fillStyle = `rgba(10, 10, 15, ${overlay})`;
      this.ctx.fillRect(0, 0, width, height);
    }

    this.ctx.save();
    const sx = config.scale ?? 1;
    if (sx !== 1) {
      this.ctx.translate(width * (1 - sx) / 2, height * (1 - sx) / 2);
      this.ctx.scale(sx, sx);
    }
    this.ctx.translate(config.positionX || 0, config.positionY || 0);
    switch (config.style) {
      case 'radial': renderRadialVisualizer(r); break;
      case 'oscilloscope': renderOscilloscopeVisualizer(r); break;
      case 'equalizer': renderEqualizerMatrix(r); break;
      case 'minimal': renderMinimalWaveVisualizer(r); break;
      case 'waveformFill': renderWaveformFill(r); break;
      case 'circularBars': renderCircularBars(r); break;
      case 'smoothSpectrum': renderSmoothSpectrum(r); break;
      case 'pulseRings': renderPulseRings(r); break;
      case 'vuMeter': renderVuMeter(r); break;
      case 'auroraWave': renderAuroraWave(r); break;
      case 'flameFire': renderFlameFire(r); break;
      case 'spiralGalaxy': renderSpiralGalaxy(r); break;
      case 'threeD': renderThreeD(r); break;
      case 'api3D': renderApi3D(r); break;
      case 'neonCity3D': renderNeonCity3D(r); break;
      case 'speaker3D': renderSpeaker3D(r); break;
      case 'speakerTrio': renderSpeakerTrio(r); break;
      case 'speakerSplatter': renderSpeakerSplatter(r); break;
      case 'spectrum':
      default: renderSpectrumBars(r); break;
    }
    this.ctx.restore();

    if (config.background.showParticles) {
      renderParticles(r);
    }
    if (config.background.showMusicNotes) {
      renderMusicNotes(r);
    }

    renderTextOverlay(r);

    this.ctx.restore();

    applyScreenEffects(this.canvas, this.ctx, config.screenEffects, this.beatStrength, aboveFloor);

    this.rotationAngle = r.rotationAngle;
  }
}

export const canvasRenderer = new CanvasRenderer();
