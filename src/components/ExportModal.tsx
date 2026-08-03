import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { Film, CheckCircle, AlertTriangle, Download, RefreshCw, Zap, Layers, Camera, Volume2, VolumeX, Sparkles, Wrench, Cpu, FolderOpen, FolderPlus, Trash2, ExternalLink, X } from 'lucide-react';
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
  const [renderEngine, setRenderEngine] = useState<'gpu' | 'canvas'>('canvas');
  const [encoder, setEncoder] = useState<'auto' | 'h264' | 'hevc' | 'av1'>('auto');
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

    const exportConfig: VisualizerConfig = {
      ...config,
      export: {
        ...config.export,
        renderEngine,
        encoder,
      },
    };

    videoExporter
      .exportToVideo(sourceCanvas, exportConfig, mode === 'with_audio', method, (progress) => {
        if (mountedRef.current) setProgressState(progress);
      })
      .catch((err) => {
        console.error('Export error:', err);
      });
  };

  const [outputFolder, setOutputFolder] = useState<string>(() => localStorage.getItem('audiowave_output_folder') || '');

  const handlePickOutputFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Pilih Folder Destination Ekspor Video',
      });
      if (typeof selected === 'string' && selected) {
        setOutputFolder(selected);
        localStorage.setItem('audiowave_output_folder', selected);
      }
    } catch (e) {
      console.error('Pick output folder error:', e);
    }
  };

  const handleClearOutputFolder = () => {
    setOutputFolder('');
    localStorage.removeItem('audiowave_output_folder');
  };

  const handleCloseModal = () => {
    if (progressState.outputPath) {
      videoExporter.cleanupTempFile(progressState.outputPath);
    }
    setHasStarted(false);
    onClose();
  };

  useEffect(() => {
    mountedRef.current = true;
    if (!isOpen) {
      if (progressState.outputPath) {
        videoExporter.cleanupTempFile(progressState.outputPath);
      }
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
    if (progressState.outputPath) {
      videoExporter.cleanupTempFile(progressState.outputPath);
    }
    setHasStarted(false);
  };

  const handleDownload = async () => {
    if (!progressState.outputPath) return;
    setIsSaving(true);
    try {
      const title = config.text.songTitle || 'audiowave_visualizer';
      const cleanTitle = title.toLowerCase().replace(/[^a-z0-9]/g, '_');
      const suffix = exportMode === 'with_audio' ? '_visualizer' : '_visualizer_no_audio';
      const fileName = `${cleanTitle}${suffix}.mp4`;
      const defaultFilename = outputFolder ? `${outputFolder}/${fileName}` : fileName;
      await videoExporter.saveToFile(progressState.outputPath, defaultFilename);
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
      <div className="modal-card modal-landscape">
        <div className="modal-header">
          <div className="modal-title">
            <Film size={20} className="text-secondary" />
            <span>Export MP4 Video</span>
          </div>
          <button
            type="button"
            className="btn-icon"
            onClick={handleCloseModal}
            title="Close"
            style={{
              background: 'transparent',
              border: 'none',
              color: '#94a3b8',
              cursor: 'pointer',
              padding: 4,
              borderRadius: 6,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.15s ease'
            }}
          >
            <X size={18} />
          </button>
        </div>

        <div className="modal-body" style={{ maxHeight: '78vh', overflowY: 'auto' }}>
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
            <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
              {/* Row 1: EXPORT METHOD (Primary Visual Cards) */}
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

              {/* Row 2: AUDIO OPTIONS & RENDER ENGINE (Segmented Pill Controls Side by Side) */}
              <div className="modal-landscape-grid" style={{ gridTemplateColumns: '1fr 1fr', gap: 16 }}>
                <div className="export-section">
                  <label className="export-section-label">Audio Options</label>
                  <div className="export-pill-group">
                    <button
                      type="button"
                      className={`export-pill-button ${exportMode === 'with_audio' ? 'selected' : ''}`}
                      onClick={() => setExportMode('with_audio')}
                    >
                      <Volume2 size={16} className={exportMode === 'with_audio' ? 'text-secondary' : ''} />
                      <span>With Audio</span>
                      <span style={{ fontSize: '0.68rem', opacity: 0.7 }}>(Audio + Video)</span>
                    </button>

                    <button
                      type="button"
                      className={`export-pill-button ${exportMode === 'visualizer_only' ? 'selected' : ''}`}
                      onClick={() => setExportMode('visualizer_only')}
                    >
                      <VolumeX size={16} />
                      <span>Visualizer Only</span>
                      <span style={{ fontSize: '0.68rem', opacity: 0.7 }}>(Silent)</span>
                    </button>
                  </div>
                </div>

                <div className="export-section">
                  <label className="export-section-label">Render Engine</label>
                  <div className="export-pill-group">
                    <button
                      type="button"
                      className={`export-pill-button ${renderEngine === 'canvas' ? 'selected' : ''}`}
                      onClick={() => setRenderEngine('canvas')}
                    >
                      <Layers size={16} className={renderEngine === 'canvas' ? 'text-secondary' : ''} />
                      <span>Pixel Perfect</span>
                      <span style={{ fontSize: '0.68rem', opacity: 0.7 }}>(Canvas2D)</span>
                    </button>

                    <button
                      type="button"
                      className={`export-pill-button ${renderEngine === 'gpu' ? 'selected' : ''}`}
                      onClick={() => setRenderEngine('gpu')}
                    >
                      <Zap size={16} />
                      <span>Super Fast</span>
                      <span style={{ fontSize: '0.68rem', opacity: 0.7 }}>(Rust WGPU)</span>
                    </button>
                  </div>
                </div>
              </div>

              {/* Row 3: CODEC / ENCODER (Full width 4-segmented pill bar) */}
              <div className="export-section">
                <label className="export-section-label">Codec / Encoder</label>
                <div className="export-pill-group">
                  <button
                    type="button"
                    className={`export-pill-button ${encoder === 'auto' ? 'selected' : ''}`}
                    onClick={() => setEncoder('auto')}
                  >
                    <Zap size={15} className={encoder === 'auto' ? 'text-secondary' : ''} />
                    <span>Auto</span>
                    <span style={{ fontSize: '0.65rem', padding: '1px 5px', borderRadius: 4, background: 'rgba(255, 215, 0, 0.25)', color: '#FED700', fontWeight: 700 }}>RECOMMENDED</span>
                  </button>

                  <button
                    type="button"
                    className={`export-pill-button ${encoder === 'h264' ? 'selected' : ''}`}
                    onClick={() => setEncoder('h264')}
                  >
                    <Film size={15} />
                    <span>H.264</span>
                    <span style={{ fontSize: '0.68rem', opacity: 0.7 }}>(Universal)</span>
                  </button>

                  <button
                    type="button"
                    className={`export-pill-button ${encoder === 'hevc' ? 'selected' : ''}`}
                    onClick={() => setEncoder('hevc')}
                  >
                    <Layers size={15} />
                    <span>HEVC (H.265)</span>
                    <span style={{ fontSize: '0.68rem', opacity: 0.7 }}>(~40% Lebih Kecil)</span>
                  </button>

                  <button
                    type="button"
                    className={`export-pill-button ${encoder === 'av1' ? 'selected' : ''}`}
                    onClick={() => setEncoder('av1')}
                  >
                    <Cpu size={15} />
                    <span>AV1</span>
                    <span style={{ fontSize: '0.68rem', opacity: 0.7 }}>(Paling Hemat)</span>
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
                      background: 'rgba(56, 189, 248, 0.15)',
                      color: '#38bdf8',
                      border: '1px solid rgba(56, 189, 248, 0.3)',
                    }}
                  >
                    <Zap size={14} />
                    <span>⚡ GPU Hardware Acceleration Active ({hwInfo.recommended_encoder})</span>
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

              {/* Crediblemark Ad Banner */}
              <a
                href="https://crediblemark.com"
                target="_blank"
                rel="noopener noreferrer"
                style={{
                  marginTop: 16,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  gap: 16,
                  padding: '14px 18px',
                  borderRadius: 12,
                  background: 'linear-gradient(135deg, rgba(254, 215, 0, 0.08) 0%, rgba(10, 10, 10, 0.95) 60%, rgba(56, 189, 248, 0.08) 100%)',
                  border: '1px solid rgba(254, 215, 0, 0.3)',
                  boxShadow: '0 8px 24px rgba(0, 0, 0, 0.4), inset 0 0 15px rgba(254, 215, 0, 0.05)',
                  textDecoration: 'none',
                  transition: 'all 0.25s ease',
                  cursor: 'pointer',
                }}
                className="crediblemark-ad-banner"
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 14, minWidth: 0 }}>
                  <img
                    src="https://ur9p9ubjnfqwdnjf.public.blob.vercel-storage.com/logos/1781276538440-ICON_CREDIBLEMARK.webp"
                    alt="Crediblemark Logo"
                    style={{ width: 36, height: 36, objectFit: 'contain', flexShrink: 0, filter: 'drop-shadow(0 0 8px rgba(254, 215, 0, 0.5))' }}
                  />
                  <div style={{ display: 'flex', flexDirection: 'column', minWidth: 0, textAlign: 'left' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 2 }}>
                      <span style={{ fontWeight: 800, fontSize: '0.92rem', color: '#ffffff', letterSpacing: '-0.01em' }}>
                        Crediblemark
                      </span>
                      <span
                        style={{
                          fontSize: '0.62rem',
                          fontWeight: 800,
                          padding: '2px 7px',
                          borderRadius: 10,
                          background: 'linear-gradient(135deg, #FFB800, #FED700)',
                          color: '#000000',
                          textTransform: 'uppercase',
                          letterSpacing: '0.05em',
                        }}
                      >
                        SPONSORED
                      </span>
                    </div>
                    <span style={{ fontSize: '0.78rem', color: '#e2e8f0', fontWeight: 600, lineHeight: 1.3 }}>
                      Designer of Your Business Digital System • Custom Web & App Development
                    </span>
                    <span style={{ fontSize: '0.72rem', color: '#94a3b8', marginTop: 2 }}>
                      Understand the Problem. Build the Solution. 100% Code Ownership.
                    </span>
                  </div>
                </div>

                <div
                  style={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: 6,
                    padding: '8px 16px',
                    borderRadius: 20,
                    background: 'linear-gradient(135deg, #FFB800, #FED700)',
                    color: '#000000',
                    fontWeight: 800,
                    fontSize: '0.78rem',
                    flexShrink: 0,
                    boxShadow: '0 4px 14px rgba(254, 215, 0, 0.3)',
                    whiteSpace: 'nowrap',
                  }}
                >
                  <span>Free Consultation</span>
                  <ExternalLink size={14} />
                </div>
              </a>
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

        {/* MODAL FOOTER - PLACED AT THE VERY BOTTOM */}
        <div className="modal-footer">
          {!hasStarted ? (
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, flex: 1, minWidth: 0, paddingRight: 16 }}>
              <FolderOpen size={18} className="text-secondary" style={{ flexShrink: 0 }} />
              <div style={{ display: 'flex', flexDirection: 'column', minWidth: 0 }}>
                <span style={{ fontSize: '0.78rem', fontWeight: 600, color: 'var(--text-primary)' }}>Default Save Folder</span>
                <span style={{ fontSize: '0.72rem', color: '#94a3b8', textOverflow: 'ellipsis', overflow: 'hidden', whiteSpace: 'nowrap' }} title={outputFolder}>
                  {outputFolder || 'Not set (Save dialog will prompt location)'}
                </span>
              </div>
              <div style={{ display: 'flex', gap: 6, flexShrink: 0, marginLeft: 4 }}>
                <button
                  type="button"
                  className="btn btn-sm btn-secondary"
                  onClick={handlePickOutputFolder}
                  title={outputFolder ? 'Ubah Folder Default' : 'Pilih Folder Default'}
                  style={{ padding: '6px 10px', display: 'inline-flex', alignItems: 'center', justifyContent: 'center' }}
                >
                  <FolderPlus size={15} />
                </button>
                {outputFolder && (
                  <button
                    type="button"
                    className="btn btn-sm btn-secondary"
                    onClick={handleClearOutputFolder}
                    title="Reset Default Folder"
                    style={{ padding: '6px 10px', display: 'inline-flex', alignItems: 'center', justifyContent: 'center' }}
                  >
                    <Trash2 size={15} />
                  </button>
                )}
              </div>
            </div>
          ) : (
            <div />
          )}

          <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexShrink: 0 }}>
            {!hasStarted ? (
              <>
                <button type="button" className="btn btn-secondary" onClick={handleCloseModal}>
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
                <button type="button" className="btn btn-secondary" onClick={handleCloseModal}>
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
    </div>
  );
};

