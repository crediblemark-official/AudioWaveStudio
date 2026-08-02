import React, { useCallback, useEffect, useRef } from 'react';
import { Maximize2, Minimize2, ExternalLink } from 'lucide-react';
import { VisualizerConfig } from '../types/visualizer';
import { canvasRenderer } from '../services/canvasRenderer';
import { audioEngine } from '../services/audioEngine';
import { detachedPreviewService } from '../services/detachedPreviewService';

interface VisualizerCanvasProps {
  config: VisualizerConfig;
  onToggleFullscreen: () => void;
  isFullscreen: boolean;
}

function getCanvasSize(resolution: string, aspectRatio: string): { width: number; height: number } {
  let width = 1920;
  let height = 1080;
  if (resolution === '720p') { width = 1280; height = 720; }
  else if (resolution === '4K') { width = 3840; height = 2160; }
  if (aspectRatio === '9:16') { const t = width; width = height; height = t; }
  else if (aspectRatio === '1:1') { height = width; }
  return { width, height };
}

export const VisualizerCanvas: React.FC<VisualizerCanvasProps> = ({ config, onToggleFullscreen, isFullscreen }) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const configRef = useRef<VisualizerConfig>(config);
  configRef.current = config;

  const canvasSize = getCanvasSize(config.export.resolution, config.export.aspectRatio);

  useEffect(() => {
    if (canvasRef.current) {
      canvasRenderer.init(canvasRef.current);
    }
  }, []);

  useEffect(() => {
    canvasRenderer.setCustomBackgroundImage(config.background.customImageUri);
  }, [config.background.customImageUri]);

  useEffect(() => {
    canvasRenderer.setRadialCenterImage(config.background.radialCenterImageUri);
  }, [config.background.radialCenterImageUri]);

  useEffect(() => {
    audioEngine.setFftSize(config.reactivity.fftSize || 1024);
  }, [config.reactivity.fftSize]);

  useEffect(() => {
    audioEngine.setSmoothing(config.reactivity.smoothing ?? 0.8);
  }, [config.reactivity.smoothing]);

  useEffect(() => {
    detachedPreviewService.broadcastConfig(config);
  }, [config]);

  const freqBuffer = useRef(new Uint8Array(512));
  const timeBuffer = useRef(new Uint8Array(512));

  useEffect(() => {
    let animId: number;
    const renderLoop = () => {
      try {
        canvasRenderer.drawFrame(configRef.current);
        detachedPreviewService.broadcastAudioFrame(
          audioEngine.getFrequencyData(freqBuffer.current),
          audioEngine.getTimeDomainData(timeBuffer.current)
        );
      } catch (e) {
        console.error('[VisualizerCanvas] drawFrame error:', e);
      }
      animId = requestAnimationFrame(renderLoop);
    };
    renderLoop();
    return () => cancelAnimationFrame(animId);
  }, []);

  const getAspectRatioClass = () => {
    switch (config.export.aspectRatio) {
      case '9:16':
        return 'aspect-9-16';
      case '1:1':
        return 'aspect-1-1';
      case '16:9':
      default:
        return 'aspect-16-9';
    }
  };

  const handleDoubleClick = useCallback(() => {
    onToggleFullscreen();
  }, [onToggleFullscreen]);

  return (
    <div className="canvas-wrapper" onDoubleClick={handleDoubleClick}>
      <div className={`canvas-container ${getAspectRatioClass()}`}>
        <canvas
          ref={canvasRef}
          width={canvasSize.width}
          height={canvasSize.height}
          className="visualizer-canvas"
        />
        <div className="canvas-overlay-controls">
          <button
            className="btn-fullscreen"
            onClick={(e) => { e.stopPropagation(); detachedPreviewService.openDetachedPreview(); }}
            title="Pisah Preview Ke Window/Monitor Lain"
          >
            <ExternalLink size={16} />
          </button>
          <button
            className="btn-fullscreen"
            onClick={(e) => { e.stopPropagation(); onToggleFullscreen(); }}
            title={isFullscreen ? 'Exit Fullscreen (F11)' : 'Fullscreen (F11)'}
          >
            {isFullscreen ? <Minimize2 size={16} /> : <Maximize2 size={16} />}
          </button>
        </div>
        {!isFullscreen && (
          <div className="aspect-badge">
            {config.export.aspectRatio} ({config.export.resolution})
          </div>
        )}
      </div>
    </div>
  );
};
