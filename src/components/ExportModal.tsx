import React, { useEffect, useRef, useState } from 'react';
import { Film, FilmIcon, CheckCircle, AlertTriangle, Download, X, RefreshCw, Zap, Layers } from 'lucide-react';
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
  const [exportMethod, setExportMethod] = useState<ExportMethod>('screen_recording');
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

  return (
    <div className="modal-backdrop" onClick={(e) => { if (e.target === e.currentTarget) handleCancel(); }}>
      <div className="modal-card">
        <div className="modal-header">
          <div className="modal-title">
            <Film size={22} className="text-secondary" />
            <span>Export MP4 Video</span>
          </div>
          <button className="btn-icon" onClick={handleCancel} title="Close">
            <X size={20} />
          </button>
        </div>

        <div className="modal-body">
          {!hasStarted && (
            <div className="export-mode-select">
              <h4 className="status-heading">Export Settings</h4>

              <div className="export-method-select">
                <p className="status-sub" style={{ marginBottom: 8 }}>Method</p>
                <div className="export-mode-options">
                <button
                  className={`export-mode-card ${exportMethod === 'screen_recording' ? 'selected' : ''}`}
                  onClick={() => setExportMethod('screen_recording')}
                >
                  <Zap size={24} />
                  <span className="mode-label">Screen Recording</span>
                  <span className="mode-desc">Real-time capture from live canvas</span>
                </button>

                  <button
                    className={`export-mode-card ${exportMethod === 'frame_by_frame' ? 'selected' : ''}`}
                    onClick={() => setExportMethod('frame_by_frame')}
                  >
                    <Layers size={24} />
                    <span className="mode-label">Frame by Frame</span>
                    <span className="mode-desc">Precise per-frame render, slower but accurate</span>
                  </button>
                </div>
              </div>

              <div className="export-method-select" style={{ marginTop: 16 }}>
                <p className="status-sub" style={{ marginBottom: 8 }}>Audio</p>
                <div className="export-mode-options">
                  <button
                    className={`export-mode-card ${exportMode === 'with_audio' ? 'selected' : ''}`}
                    onClick={() => startExport('with_audio', exportMethod)}
                  >
                    <Film size={24} />
                    <span className="mode-label">With Audio</span>
                    <span className="mode-desc">Visualizer + song combined in MP4</span>
                  </button>

                  <button
                    className={`export-mode-card ${exportMode === 'visualizer_only' ? 'selected' : ''}`}
                    onClick={() => startExport('visualizer_only', exportMethod)}
                  >
                    <FilmIcon size={24} />
                    <span className="mode-label">Visualizer Only</span>
                    <span className="mode-desc">Silent visualizer video, no audio track</span>
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
                  ? 'Preparing...'
                  : progressState.status === 'muxing'
                  ? 'Converting to MP4...'
                  : progressState.status === 'rendering'
                  ? 'Rendering frames...'
                  : exportMode === 'with_audio'
                  ? 'Recording with Audio...'
                  : 'Recording Visualizer...'}
              </h4>
              <p className="status-sub">
                {progressState.status === 'muxing'
                  ? 'Combining video + audio with FFmpeg'
                  : `${exportMethod === 'screen_recording' ? 'Screen Recording' : 'Frame by Frame'} · ${config.export.fps} FPS · ${config.export.aspectRatio} ${config.export.resolution}`}
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
                  ? 'Your MP4 audio wave visualizer is ready to save.'
                  : 'Your silent MP4 visualizer is ready to save.'}
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
            <button className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
          ) : progressState.status === 'completed' ? (
            <>
              <button className="btn btn-secondary" onClick={onClose}>
                Done
              </button>
              <button
                className="btn btn-export"
                onClick={handleDownload}
                disabled={isSaving}
              >
                <Download size={18} />
                <span>{isSaving ? 'Saving...' : 'Save MP4 Video'}</span>
              </button>
            </>
          ) : progressState.status === 'error' ? (
            <button className="btn btn-secondary" onClick={handleCancel}>
              Close
            </button>
          ) : (
            <button className="btn btn-secondary" onClick={handleCancel}>
              Cancel Export
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
