import { VisualizerConfig } from '../types/visualizer';
import { audioEngine } from './audioEngine';
import { rustBridge } from './rustBridge';
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
import { renderMusicNotes, renderParticles, initParticles } from './renderers/background/index';
import { renderTextOverlay } from './renderers/textOverlay';
import { applyScreenEffects, getShakeOffset } from './renderers/screenEffects';
import type { RenderContext, MusicNote } from './renderers/types';

const BACKGROUND_SHAKE_MULT = 1.8;

export class CanvasRenderer {
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private useRustGpuPreview: boolean = true;
  private isRenderingRustFrame: boolean = false;

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

  private frameTime: number | null = null;

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
    frameTime: 0,
  };

  public setRotationAngle(angle: number) {
    this.rotationAngle = angle;
  }

  public setFrameTime(timeSec: number) {
    this.frameTime = timeSec;
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
    this.frameTime = null;
  }

  public init(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) throw new Error('Failed to get 2D context from canvas');
    this.ctx = ctx;
    this.rctx.ctx = ctx;
  }

  public setCustomBackgroundImage(uri?: string) {
    if (!uri) {
      this.customImgElement = null;
      return;
    }
    if (this.customImgElement && this.customImgElement.src === uri) {
      return;
    }
    const img = new Image();
    if (uri.startsWith('http://') || uri.startsWith('https://')) {
      img.crossOrigin = 'anonymous';
    }
    img.onload = () => { /* loaded ok */ };
    img.onerror = (e) => { console.warn('[CanvasRenderer] Custom background image load error:', e); };
    img.src = uri;
    this.customImgElement = img;
  }

  public setRadialCenterImage(uri?: string) {
    if (!uri) {
      this.radialCenterImgElement = null;
      return;
    }
    if (this.radialCenterImgElement && this.radialCenterImgElement.src === uri) {
      return;
    }
    const img = new Image();
    if (uri.startsWith('http://') || uri.startsWith('https://')) {
      img.crossOrigin = 'anonymous';
    }
    img.onload = () => { /* loaded ok */ };
    img.onerror = (e) => { console.warn('[CanvasRenderer] Radial center image load error:', e); };
    img.src = uri;
    this.radialCenterImgElement = img;
  }

  public async preloadImages(): Promise<void> {
    const promises: Promise<void>[] = [];
    if (this.customImgElement && !this.customImgElement.complete) {
      promises.push(
        new Promise((resolve) => {
          if (!this.customImgElement) return resolve();
          this.customImgElement.onload = () => resolve();
          this.customImgElement.onerror = () => resolve();
        })
      );
    }
    if (this.radialCenterImgElement && !this.radialCenterImgElement.complete) {
      promises.push(
        new Promise((resolve) => {
          if (!this.radialCenterImgElement) return resolve();
          this.radialCenterImgElement.onload = () => resolve();
          this.radialCenterImgElement.onerror = () => resolve();
        })
      );
    }
    if (promises.length === 0) return;
    const timeout = new Promise<void>((resolve) => setTimeout(resolve, 3000));
    await Promise.race([Promise.all(promises), timeout]);
  }

  public drawFrame(config: VisualizerConfig) {
    if (!this.canvas || !this.ctx) return;

    const width = this.canvas.width;
    const height = this.canvas.height;

    if (this.useRustGpuPreview && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window && !this.exportFreqData) {
      if (!this.isRenderingRustFrame) {
        this.isRenderingRustFrame = true;
        const currentFrameTime = this.frameTime ?? audioEngine.getCurrentTime();
        const fftSize = config.reactivity.fftSize || 1024;
        if (this.freqData.length !== fftSize / 2) {
          this.freqData = new Uint8Array(fftSize / 2);
          this.timeData = new Uint8Array(fftSize / 2);
        }
        audioEngine.getFrequencyData(this.freqData);
        audioEngine.getTimeDomainData(this.timeData);

        rustBridge
          .renderRustPreviewFrame(config, this.freqData, this.timeData, currentFrameTime, width, height)
          .then((rgbaBytes) => {
            if (this.ctx && rgbaBytes && rgbaBytes.length === width * height * 4) {
              const clamped = new Uint8ClampedArray(rgbaBytes.length);
              clamped.set(rgbaBytes);
              const imgData = new ImageData(clamped, width, height);
              this.ctx.putImageData(imgData, 0, 0);
            }
          })
          .catch(() => {
            // Fallback to TS canvas 2D if GPU readback is temporarily busy
          })
          .finally(() => {
            this.isRenderingRustFrame = false;
          });
      }
      return;
    }

    let targetBass: number;
    let rawBass: number;

    if (this.exportFreqData) {
      this.freqData = this.exportFreqData;
      this.timeData = this.exportTimeData!;
      rawBass = this.exportBassEnergy;
      targetBass = rawBass * (config.reactivity.bassMultiplier || 1.0) * (config.reactivity.sensitivity || 1.0);
      this.bassEnergy += (targetBass - this.bassEnergy) * 0.2;
      this.bassEnergyRaw += (rawBass - this.bassEnergyRaw) * 0.2;
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
    r.isPlaying = this.exportFreqData ? true : audioEngine.getIsPlaying();
    r.frameTime = this.frameTime ?? audioEngine.getCurrentTime();
    const frameTime = r.frameTime;

    const shakeOff = getShakeOffset(config.screenEffects, this.bassEnergy, this.beatStrength, aboveFloor, frameTime);
    const bgShakeOff = {
      x: Math.round(shakeOff.x * BACKGROUND_SHAKE_MULT),
      y: Math.round(shakeOff.y * BACKGROUND_SHAKE_MULT),
    };
    const shakeMargin = Math.ceil(Math.hypot(bgShakeOff.x, bgShakeOff.y));

    this.ctx.save();
    this.ctx.translate(bgShakeOff.x, bgShakeOff.y);
    renderBackground(r, shakeMargin);
    this.ctx.restore();

    const overlay = config.background.overlayOpacity || 0;
    if (overlay > 0) {
      this.ctx.save();
      this.ctx.translate(shakeOff.x, shakeOff.y);
      this.ctx.fillStyle = `rgba(10, 10, 15, ${overlay})`;
      this.ctx.fillRect(-shakeMargin, -shakeMargin, width + shakeMargin * 2, height + shakeMargin * 2);
      this.ctx.restore();
    }

    this.ctx.save();
    this.ctx.translate(shakeOff.x, shakeOff.y);
    const sx = config.scale ?? 1;
    const posX = config.positionX || 0;
    const posY = config.positionY || 0;

    this.ctx.translate(width / 2 + posX, height / 2 + posY);
    if (sx !== 1) {
      this.ctx.scale(sx, sx);
    }
    this.ctx.translate(-width / 2, -height / 2);
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

    applyScreenEffects(this.canvas, this.ctx, config.screenEffects, this.beatStrength, aboveFloor, frameTime);

    this.rotationAngle = r.rotationAngle;
  }
}

export const canvasRenderer = new CanvasRenderer();
