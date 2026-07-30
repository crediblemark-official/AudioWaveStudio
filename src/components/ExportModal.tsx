import React, { useEffect, useRef, useState } from 'react';
import { Film, CheckCircle, AlertTriangle, Download, X, RefreshCw, Zap, Layers, Camera, Volume2, VolumeX, Sparkles } from 'lucide-react';
import { VisualizerConfig } from '../types/visualizer';
import { ExportProgress, ExportMethod, videoExporter } from '../services/videoExporter';

export type ExportMode = 'with_audio' | 'visualizer_only';

interface ExportModalProps {
  isOpen: boolean;
  config: VisualizerConfig;
  onClose: () => void;
}

export const ExportModal: React.FC<ExportModalProps> = ({ isOpen, config, onClose }) => {
  const [exportMode, setExportMode] = useState<ExportMode>('with_audio');
  const [exportMethod, setExportMethod] = useState<ExportMethod>('hybrid');
  const [hasStarted, setHasStarted] = useState<boolean>(false);
  const [isSaving, setIsSaving] = useState<boolean>(false);
  const mountedRef = useRef(true);
  const [progressState, setProgressState] = useState<ExportProgress>({
    status: 'preparing',
    progress: 0,
    currentFrame: 0,
    totalFrames: 0,
    elapsedTime: 0
  });

  const startExport = (mode: ExportMode, method: ExportMethod) => {
    const sourceCanvas = document.querySelector('.visualizer-canvas') as HTMLCanvasElement;
    if (!sourceCanvas) return;

    setHasStarted(true);
    setExportMode(mode);
    setExportMethod(method);
    setProgressState({
      status: 'preparing',
      progress: 0,
      currentFrame: 0,
      totalFrames: 0,
      elapsedTime: 0
    });

    videoExporter
      .exportToVideo(sourceCanvas, config, mode === 'with_audio', method, (progress) => {
        if (mountedRef.current) setProgressState(progress);
      })
      .catch((err) => {
        console.error('Export error:', err);
      });
  };

  useEffect(() => {
    mountedRef.current = true;
    if (!isOpen) {
      setHasStarted(false);
      setIsSaving(false);
    }
    return () => {
      mountedRef.current = false;
      videoExporter.cancelExport();
    };
  }, [isOpen]);

  if (!isOpen) return null;

  const handleCancel = () => {
    videoExporter.cancelExport();
    onClose();
  };

  const handleDownload = async () => {
    if (!progressState.outputPath) return;
    setIsSaving(true);
    try {
      const title = config.text.songTitle || 'audiowave_visualizer';
      const cleanTitle = title.toLowerCase().replace(/[^a-z0-9]/g, '_');
      const suffix = exportMode === 'with_audio' ? '_visualizer' : '_visualizer_no_audio';
      await videoExporter.saveToFile(progressState.outputPath, `${cleanTitle}${suffix}.mp4`);
    } catch (err) {
      console.error('Save error:', err);
    } finally {
      setIsSaving(false);
    }
  };

  const methodIcon = (m: ExportMethod) => {
    if (m === 'hybrid') return <Zap size={22} className="text-secondary" />;
    if (m === 'offscreen') return <Layers size={22} />;
    return <Camera size={22} />;
  };

  const methodLabel = (m: ExportMethod) => {
    if (m === 'hybrid') return 'Hybrid';
    if (m === 'offscreen') return 'Offscreen';
    return 'Screen Capture';
  };

  const methodBadge = (m: ExportMethod) => {
    if (m === 'hybrid') return 'Fast & Reliable';
    if (m === 'offscreen') return 'Most Accurate';
    return 'Live Recording';
  };

  const methodDesc = (m: ExportMethod) => {
    if (m === 'hybrid') return 'Rust spectrum + browser encoding. Fast & full duration.';
    if (m === 'offscreen') return 'Rust spectrum + Rust JPEG. Precise per-frame.';
    return 'Captures live canvas. Requires real-time audio playback.';
  };

  return (
    <div className="modal-backdrop" onClick={(e) => { if (e.target === e.currentTarget) handleCancel(); }}>
      <div className="modal-card">
        <div className="modal-header">
          <div className="modal-title">
            <Film size={20} className="text-secondary" />
            <span>Export MP4 Video</span>
          </div>
          <button className="btn-icon" onClick={handleCancel} title="Close">
            <X size={18} />
          </button>
        </div>

        <div className="modal-body">
          {!hasStarted && (
            <div className="export-mode-select">
              <div className="export-section">
                <label className="export-section-label">Export Method</label>
                <div className="export-mode-options">
                  {(['hybrid', 'offscreen', 'screen_recording'] as ExportMethod[]).map((m) => (
                    <button
                      key={m}
                      type="button"
                      className={`export-mode-card ${exportMethod === m ? 'selected' : ''}`}
                      onClick={() => setExportMethod(m)}
                    >
                      {m === 'hybrid' && <span className="recommended-badge">Recommended</span>}
                      <div className="card-header-icon">
                        {methodIcon(m)}
                      </div>
                      <span className="mode-label">{methodLabel(m)}</span>
                      <span className="mode-tag">{methodBadge(m)}</span>
                      <span className="mode-desc">{methodDesc(m)}</span>
                    </button>
                  ))}
                </div>
              </div>

              <div className="export-section mt-4">
                <label className="export-section-label">Audio Options</label>
                <div className="export-mode-options">
                  <button
                    type="button"
                    className={`export-mode-card ${exportMode === 'with_audio' ? 'selected' : ''}`}
                    onClick={() => setExportMode('with_audio')}
                  >
                    <div className="card-header-icon">
                      <Volume2 size={22} className="text-secondary" />
                    </div>
                    <span className="mode-label">With Audio</span>
                    <span className="mode-desc">Include audio track in MP4</span>
                  </button>

                  <button
                    type="button"
                    className={`export-mode-card ${exportMode === 'visualizer_only' ? 'selected' : ''}`}
                    onClick={() => setExportMode('visualizer_only')}
                  >
                    <div className="card-header-icon">
                      <VolumeX size={22} />
                    </div>
                    <span className="mode-label">Visualizer Only</span>
                    <span className="mode-desc">Silent video without audio</span>
                  </button>
                </div>
              </div>
            </div>
          )}

          {hasStarted && (progressState.status === 'preparing' || progressState.status === 'recording' || progressState.status === 'muxing' || progressState.status === 'rendering') && (
            <div className="export-status">
              <div className="spinner-wrapper">
                <RefreshCw size={36} className="spin-icon" />
              </div>
              <h4 className="status-heading">
                {progressState.status === 'preparing'
                  ? 'Preparing Export...'
                  : progressState.status === 'muxing'
                  ? 'Encoding MP4 Video...'
                  : progressState.status === 'rendering'
                  ? 'Rendering Frames...'
                  : exportMode === 'with_audio'
                  ? 'Recording with Audio...'
                  : 'Recording Visualizer...'}
              </h4>
              <p className="status-sub">
                {methodLabel(exportMethod)} · {config.export.fps} FPS · {config.export.aspectRatio} ({config.export.resolution})
              </p>

              <div className="progress-track">
                <div
                  className="progress-fill"
                  style={{ width: `${Math.min(100, Math.max(0, progressState.progress))}%` }}
                />
              </div>

              <div className="progress-stats">
                <span>{progressState.progress.toFixed(1)}% Completed</span>
                <span>
                  Frame {progressState.currentFrame} / {progressState.totalFrames}
                </span>
                <span>
                  {Math.floor(progressState.elapsedTime / 60)}:{String(Math.floor(progressState.elapsedTime % 60)).padStart(2, '0')}
                </span>
              </div>
            </div>
          )}

          {hasStarted && progressState.status === 'completed' && (
            <div className="export-status success">
              <CheckCircle size={48} className="text-emerald mb-2" />
              <h4 className="status-heading">Export Completed Successfully!</h4>
              <p className="status-sub">
                {exportMode === 'with_audio'
                  ? 'Your MP4 video with audio is ready.'
                  : 'Your silent visualizer video is ready.'}
              </p>
            </div>
          )}

          {hasStarted && progressState.status === 'error' && (
            <div className="export-status error">
              <AlertTriangle size={48} className="text-danger mb-2" />
              <h4 className="status-heading">Export Encountered an Error</h4>
              <p className="status-sub">
                {progressState.errorMessage || 'Failed to render or decode audio track.'}
              </p>
            </div>
          )}
        </div>

        <div className="modal-footer">
          {!hasStarted ? (
            <>
              <button type="button" className="btn btn-secondary" onClick={onClose}>
                Cancel
              </button>
              <button
                type="button"
                className="btn btn-export"
                onClick={() => startExport(exportMode, exportMethod)}
              >
                <Sparkles size={16} />
                <span>Start Export</span>
              </button>
            </>
          ) : progressState.status === 'completed' ? (
            <>
              <button type="button" className="btn btn-secondary" onClick={onClose}>
                Done
              </button>
              <button
                type="button"
                className="btn btn-export"
                onClick={handleDownload}
                disabled={isSaving}
              >
                <Download size={18} />
                <span>{isSaving ? 'Saving...' : 'Save MP4 Video'}</span>
              </button>
            </>
          ) : progressState.status === 'error' ? (
            <button type="button" className="btn btn-secondary" onClick={handleCancel}>
              Close
            </button>
          ) : (
            <button type="button" className="btn btn-secondary" onClick={handleCancel}>
              Cancel Export
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

