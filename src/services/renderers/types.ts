import { VisualizerConfig } from '../../types/visualizer';

export interface Particle {
  x: number;
  y: number;
  radius: number;
  vx: number;
  vy: number;
  alpha: number;
  color: string;
  phase?: number;
}

export interface MusicNote {
  x: number;
  y: number;
  vx: number;
  vy: number;
  size: number;
  alpha: number;
  rotation: number;
  symbol: string;
  life: number;
  maxLife: number;
  baseX: number;
  phase: number;
}

export interface RenderContext {
  ctx: CanvasRenderingContext2D;
  width: number;
  height: number;
  config: VisualizerConfig;
  freqData: Uint8Array;
  timeData: Uint8Array;
  bassEnergy: number;
  bassEnergyRaw: number;
  beatStrength: number;
  beatStrengthRaw: number;
  peakData: number[];
  particles: Particle[];
  musicNotes: MusicNote[];
  customImgElement: HTMLImageElement | null;
  radialCenterImgElement: HTMLImageElement | null;
  rotationAngle: number;
  exportFreqData: Uint8Array | null;
  isPlaying: boolean;
  frameTime: number; // seconds (frame-accurate during export, playback time live)
}
