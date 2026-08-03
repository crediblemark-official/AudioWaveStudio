/**
 * videoExporter.ts — Entry point for all export methods.
 * Actual export implementations live in src/services/exporters/*.ts.
 *
 * Available methods:
 *   hybrid (default)     — Rust computeSpectrum + canvas.toBlob JPEG. Fast, no AudioBuffer dep.
 *   offscreen            — Rust computeSpectrum + RGBA via IPC + Rust JPEG. Accurate, slower.
 *   screen_recording     — Live canvas capture. Fastest but AudioBuffer-dependent.
 */

import { VisualizerConfig } from '../types/visualizer';
import { save } from '@tauri-apps/plugin-dialog';
import { rustBridge } from './rustBridge';
import { ExportProgress, ExportMethod, getExportDimensions } from './exporters/types';
import { exportOffscreen } from './exporters/offscreen';
import { exportHybrid } from './exporters/hybrid';
import { exportScreenRecord } from './exporters/screen-record';

export type { ExportProgress, ExportMethod };
export { getExportDimensions };

export class VideoExporter {
  private isExporting: boolean = false;
  private animFrameId: number | null = null;

  public getIsExporting(): boolean {
    return this.isExporting;
  }

  public cancelExport() {
    this.isExporting = false;
    if (this.animFrameId !== null) {
      clearInterval(this.animFrameId);
      this.animFrameId = null;
    }
  }

  public async exportToVideo(
    sourceCanvas: HTMLCanvasElement,
    config: VisualizerConfig,
    includeAudio: boolean,
    method: ExportMethod,
    onProgress: (progress: ExportProgress) => void,
  ): Promise<Blob> {
    if (this.isExporting) throw new Error('An export is already in progress');
    this.isExporting = true;

    const isCancelled = () => !this.isExporting;
    const startTime = Date.now();

    try {
      let outputPath: string;

      switch (method) {
        case 'offscreen':
          outputPath = await exportOffscreen(config, includeAudio, onProgress, isCancelled);
          break;
        case 'screen_recording':
          outputPath = await exportScreenRecord(sourceCanvas, config, includeAudio, onProgress, isCancelled);
          break;
        case 'hybrid':
        default:
          outputPath = await exportHybrid(config, includeAudio, onProgress, isCancelled);
          break;
      }

      if (!this.isExporting) throw new Error('Export cancelled');

      this.isExporting = false;
      onProgress({ status: 'completed', progress: 100, currentFrame: 0, totalFrames: 0, elapsedTime: (Date.now() - startTime) / 1000, outputPath });
      return new Blob([]);
    } catch (err: unknown) {
      this.isExporting = false;
      const msg = err instanceof Error ? err.message : String(err);
      onProgress({ status: 'error', progress: 0, currentFrame: 0, totalFrames: 0, elapsedTime: (Date.now() - startTime) / 1000, errorMessage: `Export Error: ${msg}` });
      throw err;
    }
  }

  public async cleanupTempFile(filePath?: string) {
    if (!filePath) return;
    try {
      await rustBridge.deleteFile(filePath);
    } catch {
      // Best-effort cleanup
    }
  }

  public async saveToFile(sourcePath: string, defaultFilename: string): Promise<boolean> {
    const destPath = await save({
      defaultPath: defaultFilename,
      filters: [{ name: 'MP4 Video', extensions: ['mp4'] }],
    });
    if (!destPath) return false;
    await rustBridge.copyFileToPath(sourcePath, destPath);
    return true;
  }
}

export const videoExporter = new VideoExporter();
