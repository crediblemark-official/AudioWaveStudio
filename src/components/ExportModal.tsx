import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Film, CheckCircle, AlertTriangle, Download, RefreshCw, Zap, Layers, Camera, Volume2, VolumeX, Sparkles, Wrench, Cpu } from 'lucide-react';
import { VisualizerConfig } from '../types/visualizer';
import { ExportProgress, ExportMethod, videoExporter } from '../services/videoExporter';
import { rustBridge } from '../services/rustBridge';

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
  const [ffmpegAvailable, setFfmpegAvailable] = useState<boolean>(true);
  const [ffmpegAutoInstall, setFfmpegAutoInstall] = useState<boolean>(false);
  const [ffmpegInstalling, setFfmpegInstalling] = useState<boolean>(false);
  const [ffmpegPhase, setFfmpegPhase] = useState<string>('');
  const [ffmpegError, setFfmpegError] = useState<string>('');
  const [hwInfo, setHwInfo] = useState<any>(null);
  const [memoryInfo, setMemoryInfo] = useState<{ used_mb: number; total_mb: number; used_percent: number } | null>(null);
  const mountedRef = useRef(true);
  const [progressState, setProgressState] = useState<ExportProgress>({
    status: 'preparing',
    progress: 0,
    currentFrame: 0,
    totalFrames: 0,
    elapsedTime: 0
  });

  useEffect(() => {
    if (isOpen) {
      invoke('check_hardware').then((info) => {
        if (mountedRef.current) setHwInfo(info);
      }).catch(() => {});
    }
  }, [isOpen]);

  useEffect(() => {
    let timer: any;
    if (isOpen) {
      const fetchMem = () => {
        invoke('get_system_memory_cmd').then((mem: any) => {
          if (mountedRef.current && mem) setMemoryInfo(mem);
        }).catch(() => {});
      };
      fetchMem();
      timer = setInterval(fetchMem, 1000);
    }
    return () => clearInterval(timer);
  }, [isOpen, hasStarted]);

  useEffect(() => {
    if (isOpen) {
      setFfmpegInstalling(false);
      setFfmpegPhase('');
      setFfmpegError('');
      setFfmpegAutoInstall(false);
      rustBridge
        .checkFfmpeg()
        .then((ok) => {
          if (!mountedRef.current) return;
          setFfmpegAvailable(ok);
          if (!ok) {
            rustBridge.ffmpegAutoInstallSupported().then((s) => {
              if (mountedRef.current) setFfmpegAutoInstall(s);
            });
          }
        })
        .catch(() => {
          if (mountedRef.current) setFfmpegAvailable(true);
        });
    }
  }, [isOpen]);

  const handleInstallFfmpeg = async () => {
    setFfmpegInstalling(true);
    setFfmpegError('');
    setFfmpegPhase('downloading');
    try {
      await rustBridge.installFfmpeg((phase) => {
        if (mountedRef.current) setFfmpegPhase(phase);
      });
      setFfmpegAvailable(true);
    } catch (err) {
      if (mountedRef.current) {
        setFfmpegError(err instanceof Error ? err.message : String(err));
        setFfmpegPhase('');
      }
    } finally {
      if (mountedRef.current) setFfmpegInstalling(false);
    }
  };

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

  const handleCancelExport = () => {
    videoExporter.cancelExport();
    setHasStarted(false);
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
    <div className="modal-backdrop">
      <div className="modal-card">
        <div className="modal-header">
          <div className="modal-title">
            <Film size={20} className="text-secondary" />
            <span>Export MP4 Video</span>
          </div>
        </div>

        <div className="modal-body">
          {!ffmpegAvailable && !hasStarted && (
            <div className="export-ffmpeg-warning">
              <div className="warning-icon">
                <AlertTriangle size={20} />
              </div>
              <div className="warning-content">
                <span className="warning-title">FFmpeg is not installed</span>
                <span className="warning-desc">
                  Export requires FFmpeg to encode the MP4 video.
                  {ffmpegAutoInstall && !ffmpegInstalling && ' Install it now with one click.'}
                </span>
                {ffmpegPhase === 'done' && <span className="warning-success">FFmpeg installed successfully. You can now export.</span>}
                {ffmpegError && <span className="warning-error">{ffmpegError}</span>}
              </div>
              {ffmpegAutoInstall && !ffmpegInstalling && ffmpegPhase !== 'done' && (
                <button type="button" className="btn btn-export btn-sm" onClick={handleInstallFfmpeg}>
                  <Wrench size={16} />
                  <span>Install FFmpeg</span>
                </button>
              )}
              {ffmpegInstalling && (
                <div className="warning-progress">
                  <RefreshCw size={16} className="spin-icon" />
                  <span>
                    {ffmpegPhase === 'downloading'
                      ? 'Downloading FFmpeg (~100 MB)...'
                      : ffmpegPhase === 'extracting'
                      ? 'Extracting...'
                      : 'Installing...'}
                  </span>
                </div>
              )}
            </div>
          )}

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

              {/* Hardware GPU/CPU Indicator Badge */}
              <div style={{ display: 'flex', justifyContent: 'center', marginTop: '10px' }}>
                {hwInfo && hwInfo.recommended_encoder !== 'libx264' ? (
                  <span
                    style={{
                      display: 'inline-flex',
                      alignItems: 'center',
                      gap: '6px',
                      fontSize: '0.78rem',
                      fontWeight: 600,
                      padding: '4px 14px',
                      borderRadius: '20px',
                      background: 'rgba(34, 197, 94, 0.15)',
                      color: '#4ade80',
                      border: '1px solid rgba(34, 197, 94, 0.3)',
                    }}
                  >
                    <Zap size={14} />
                    <span>⚡ Mode GPU Hardware: {hwInfo.recommended_label.replace('⚡ GPU Accelerated (', '').replace(')', '')}</span>
                  </span>
                ) : (
                  <span
                    style={{
                      display: 'inline-flex',
                      alignItems: 'center',
                      gap: '6px',
                      fontSize: '0.78rem',
                      fontWeight: 600,
                      padding: '4px 14px',
                      borderRadius: '20px',
                      background: 'rgba(234, 179, 8, 0.15)',
                      color: '#facc15',
                      border: '1px solid rgba(234, 179, 8, 0.3)',
                    }}
                  >
                    <Cpu size={14} />
                    <span>💻 Mode CPU Software: x264</span>
                  </span>
                )}
              </div>

              <div className="progress-track" style={{ marginTop: '14px' }}>
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

              {/* Live RAM Usage Metric */}
              {memoryInfo && (
                <div
                  style={{
                    marginTop: '12px',
                    padding: '8px 14px',
                    borderRadius: '8px',
                    background: 'rgba(15, 23, 42, 0.6)',
                    border: '1px solid rgba(255, 255, 255, 0.06)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    fontSize: '0.78rem',
                    color: '#94a3b8',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <Cpu size={14} style={{ color: '#38bdf8' }} />
                    <span>Penggunaan RAM Sistem:</span>
                  </div>
                  <span style={{ fontWeight: 600, color: memoryInfo.used_percent > 85 ? '#f87171' : '#38bdf8' }}>
                    {(memoryInfo.used_mb / 1024).toFixed(1)} GB / {(memoryInfo.total_mb / 1024).toFixed(1)} GB ({memoryInfo.used_percent.toFixed(1)}%)
                  </span>
                </div>
              )}
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
            <button type="button" className="btn btn-secondary" onClick={handleCancelExport}>
              Close
            </button>
          ) : (
            <button type="button" className="btn btn-secondary" onClick={handleCancelExport}>
              Cancel Export
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

