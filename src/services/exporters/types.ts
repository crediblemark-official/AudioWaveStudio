import { VisualizerConfig } from '../../types/visualizer';

export type ExportMethod = 'hybrid' | 'offscreen' | 'screen_recording';

export interface ExportProgress {
  status: 'preparing' | 'recording' | 'muxing' | 'rendering' | 'completed' | 'cancelled' | 'error';
  progress: number;
  currentFrame: number;
  totalFrames: number;
  elapsedTime: number;
  errorMessage?: string;
  outputPath?: string;
}

export function getExportDimensions(config: VisualizerConfig): { width: number; height: number } {
  let width = 1920;
  let height = 1080;
  if (config.export.resolution === '720p') { width = 1280; height = 720; }
  else if (config.export.resolution === '4K') { width = 3840; height = 2160; }
  if (config.export.aspectRatio === '9:16') { const t = width; width = height; height = t; }
  else if (config.export.aspectRatio === '1:1') { height = width; }
  return { width, height };
}
