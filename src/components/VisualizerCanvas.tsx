import React, { useCallback, useEffect, useRef } from 'react';
import { Maximize2, Minimize2 } from 'lucide-react';
import { VisualizerConfig } from '../types/visualizer';
import { canvasRenderer } from '../services/canvasRenderer';
import { audioEngine } from '../services/audioEngine';

interface VisualizerCanvasProps {
  config: VisualizerConfig;
  onToggleFullscreen: () => void;
  isFullscreen: boolean;
}

export const VisualizerCanvas: React.FC<VisualizerCanvasProps> = ({ config, onToggleFullscreen, isFullscreen }) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const configRef = useRef<VisualizerConfig>(config);
  configRef.current = config;

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
    let animId: number;
    const renderLoop = () => {
      try {
        canvasRenderer.drawFrame(configRef.current);
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
          width={1280}
          height={config.export.aspectRatio === '9:16' ? 2275 : config.export.aspectRatio === '1:1' ? 1280 : 720}
          className="visualizer-canvas"
        />
        <button
          className="btn-fullscreen"
          onClick={(e) => { e.stopPropagation(); onToggleFullscreen(); }}
          title={isFullscreen ? 'Exit Fullscreen (F11)' : 'Fullscreen (F11)'}
        >
          {isFullscreen ? <Minimize2 size={16} /> : <Maximize2 size={16} />}
        </button>
        {!isFullscreen && (
          <div className="aspect-badge">
            {config.export.aspectRatio} ({config.export.resolution})
          </div>
        )}
      </div>
    </div>
  );
};
