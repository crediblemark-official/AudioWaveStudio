import { VisualizerConfig } from '../types/visualizer';
import { audioEngine } from './audioEngine';
import { rustBridge } from './rustBridge';

export class CanvasRenderer {
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private useRustGpuPreview: boolean = false;
  private isRenderingRustFrame: boolean = false;

  private freqData: Uint8Array = new Uint8Array(512);
  private timeData: Uint8Array = new Uint8Array(512);
  private frameTime: number | null = null;

  public init(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) throw new Error('Failed to get 2D context from canvas');
    this.ctx = ctx;
  }

  public setCustomBackgroundImage(_uri?: string) {}
  public setRadialCenterImage(_uri?: string) {}
  public setRotationAngle(_angle: number) {}
  public setFrameTime(timeSec: number) {
    this.frameTime = timeSec;
  }
  public setExportData(_freqData: Uint8Array, _timeData: Uint8Array, _bassEnergy: number) {}
  public clearExportData() {
    this.frameTime = null;
  }
  public async preloadImages() {}

  public drawFrame(config: VisualizerConfig) {
    if (!this.canvas || !this.ctx) return;

    const width = this.canvas.width;
    const height = this.canvas.height;

    if (this.useRustGpuPreview && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
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
          .catch(() => {})
          .finally(() => {
            this.isRenderingRustFrame = false;
          });
      }
    }
  }
}

export const canvasRenderer = new CanvasRenderer();
