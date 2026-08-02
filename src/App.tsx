import React, { useEffect, useRef, useState } from 'react';
import { Navbar } from './components/Navbar';
import { VisualizerCanvas } from './components/VisualizerCanvas';
import { AudioPlayerBar } from './components/AudioPlayerBar';
import { ControlPanel } from './components/ControlPanel';
import { ExportModal } from './components/ExportModal';
import { HardwareModal } from './components/HardwareModal';
import { PRESETS, migrateTextSettings, loadSavedConfig, saveConfig } from './utils/presets';
import { SongMetadata, VisualizerConfig } from './types/visualizer';
import { rustBridge } from './services/rustBridge';
import { audioEngine } from './services/audioEngine';
import { canvasRenderer } from './services/canvasRenderer';
import { detachedPreviewService } from './services/detachedPreviewService';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { save, open } from '@tauri-apps/plugin-dialog';
import { RefreshCw, Headphones, Pin, PinOff, Maximize2, Minimize2, X } from 'lucide-react';

const isDetachedWindow = typeof window !== 'undefined' && window.location.search.includes('detached=true');

const DetachedPreviewView: React.FC = () => {
  const [config, setConfig] = useState<VisualizerConfig>(() => loadSavedConfig());
  const [isPinned, setIsPinned] = useState(true);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const configRef = useRef<VisualizerConfig>(config);
  configRef.current = config;

  useEffect(() => {
    invoke<boolean>('is_always_on_top_cmd')
      .then((pinned) => setIsPinned(pinned))
      .catch(() => {});
    try {
      const win = getCurrentWindow();
      win.isAlwaysOnTop().then((pinned) => setIsPinned(pinned)).catch(() => {});
      win.isFullscreen().then((fs) => setIsFullscreen(fs)).catch(() => {});
    } catch {}
  }, []);

  const handleStartDrag = (e: React.MouseEvent) => {
    if (e.button === 0) {
      try {
        getCurrentWindow().startDragging();
      } catch (err) {
        console.warn('[DetachedPreview] startDragging error:', err);
      }
    }
  };

  const togglePin = async () => {
    try {
      const res = await invoke<boolean>('toggle_detached_always_on_top');
      setIsPinned(res);
    } catch {
      try {
        const win = getCurrentWindow();
        const nextState = !isPinned;
        await win.setAlwaysOnTop(nextState);
        setIsPinned(nextState);
      } catch (err) {
        console.error('[DetachedPreview] Failed to toggle pin:', err);
      }
    }
  };

  const toggleFullscreen = async () => {
    try {
      const res = await invoke<boolean>('toggle_detached_fullscreen');
      setIsFullscreen(res);
    } catch {
      try {
        const win = getCurrentWindow();
        const next = !isFullscreen;
        await win.setFullscreen(next);
        setIsFullscreen(next);
      } catch {
        setIsFullscreen((prev) => !prev);
      }
    }
  };

  const closeWindow = async () => {
    try {
      const win = getCurrentWindow();
      await win.close();
    } catch {
      window.close();
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'F11') {
        e.preventDefault();
        toggleFullscreen();
      } else if (e.code === 'Escape' && isFullscreen) {
        toggleFullscreen();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isFullscreen]);

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
    const unsub = detachedPreviewService.listen(
      (newConfig) => {
        setConfig(newConfig);
      },
      (freqData, timeData) => {
        canvasRenderer.setExportData(freqData, timeData, 0.5);
      }
    );
    return unsub;
  }, []);

  useEffect(() => {
    let animId: number;
    const renderLoop = () => {
      try {
        canvasRenderer.drawFrame(configRef.current);
      } catch (e) {
        console.error('[DetachedPreview] drawFrame error:', e);
      }
      animId = requestAnimationFrame(renderLoop);
    };
    renderLoop();
    return () => cancelAnimationFrame(animId);
  }, []);

  return (
    <div
      data-tauri-drag-region
      onMouseDown={handleStartDrag}
      style={{
        position: 'relative',
        width: '100vw',
        height: '100vh',
        background: '#000',
        overflow: 'hidden',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        userSelect: 'none',
        cursor: 'grab',
      }}
      onDoubleClick={toggleFullscreen}
    >
      <canvas
        ref={canvasRef}
        width={1920}
        height={1080}
        style={{ width: '100%', height: '100%', objectFit: 'contain', pointerEvents: 'none' }}
      />
      <div className="canvas-overlay-controls" style={{ position: 'absolute', top: 12, right: 16, left: 'auto', display: 'flex', gap: 8, zIndex: 99 }} onMouseDown={(e) => e.stopPropagation()}>
        <button
          className={`btn-fullscreen ${isPinned ? 'active' : ''}`}
          onClick={(e) => { e.stopPropagation(); togglePin(); }}
          title={isPinned ? 'Unpin Always on Top' : 'Stay on Top (Sticky)'}
          style={{ background: isPinned ? 'rgba(0, 229, 255, 0.25)' : undefined, color: isPinned ? '#00e5ff' : undefined, borderColor: isPinned ? '#00e5ff' : undefined }}
        >
          {isPinned ? <PinOff size={16} /> : <Pin size={16} />}
        </button>
        <button
          className="btn-fullscreen"
          onClick={(e) => { e.stopPropagation(); toggleFullscreen(); }}
          title={isFullscreen ? 'Exit Fullscreen (F11)' : 'Fullscreen (F11)'}
        >
          {isFullscreen ? <Minimize2 size={16} /> : <Maximize2 size={16} />}
        </button>
        <button
          className="btn-fullscreen"
          onClick={(e) => { e.stopPropagation(); closeWindow(); }}
          title="Tutup Preview"
          style={{ background: 'rgba(255, 50, 50, 0.3)', borderColor: 'rgba(255, 50, 50, 0.5)' }}
        >
          <X size={16} />
        </button>
      </div>
    </div>
  );
};

export const App: React.FC = () => {
  if (isDetachedWindow) {
    return <DetachedPreviewView />;
  }

  const [config, setConfig] = useState<VisualizerConfig>(() => loadSavedConfig());
  const [songMeta, setSongMeta] = useState<SongMetadata | null>(null);
  const [isExportModalOpen, setIsExportModalOpen] = useState<boolean>(false);
  const [isHardwareModalOpen, setIsHardwareModalOpen] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isFullscreen, setIsFullscreen] = useState<boolean>(false);
  const [isListening, setIsListening] = useState<boolean>(false);
  const [listenError, setListenError] = useState<string>('');

  useEffect(() => {
    const t = setTimeout(() => saveConfig(config), 400);
    return () => clearTimeout(t);
  }, [config]);

  const handleStartListen = async (deviceId?: string) => {
    setIsLoading(true);
    setListenError('');
    try {
      await audioEngine.startListening(deviceId);
      setIsListening(true);
      setSongMeta(null);
    } catch (err) {
      setListenError(err instanceof Error ? err.message : 'Failed to access audio input');
      console.error('[App] Listen failed:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleStopListen = () => {
    audioEngine.stopListening();
    setIsListening(false);
    setListenError('');
  };

  const handleLoadSong = async (file: File) => {
    setIsLoading(true);
    try {
      const meta = await audioEngine.loadAudioFile(file);
      setSongMeta(meta);
      setConfig((prev) => ({
        ...prev,
        text: {
          ...prev.text,
          songTitle: meta.title,
          artistName: meta.artist,
          title: { ...prev.text.title, text: meta.title },
          artist: { ...prev.text.artist, text: meta.artist },
        },
      }));
    } catch (err) {
      console.error('Failed to load audio file:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLoadSongPath = async (filePath: string) => {
    setIsLoading(true);
    try {
      const meta = await audioEngine.loadAudioPath(filePath);
      setSongMeta(meta);
      setConfig((prev) => ({
        ...prev,
        text: {
          ...prev.text,
          songTitle: meta.title,
          artistName: meta.artist,
          title: { ...prev.text.title, text: meta.title },
          artist: { ...prev.text.artist, text: meta.artist },
        },
      }));
    } catch (err) {
      console.error('Failed to load audio path:', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      const tag = (e.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

      if (e.code === 'Escape' && isFullscreen) {
        setIsFullscreen(false);
        return;
      }
      if (e.code === 'F11') {
        e.preventDefault();
        setIsFullscreen((v) => !v);
        return;
      }

      switch (e.code) {
        case 'Space':
          e.preventDefault();
          if (audioEngine.getDuration() > 0) {
            if (audioEngine.getIsPlaying()) {
              audioEngine.pause();
            } else {
              audioEngine.play();
            }
          }
          break;
        case 'KeyS':
          if (audioEngine.getDuration() > 0) {
            e.preventDefault();
            audioEngine.stop();
          }
          break;
        case 'ArrowLeft':
          e.preventDefault();
          if (audioEngine.getDuration() > 0) audioEngine.seek(audioEngine.getCurrentTime() - 5);
          break;
        case 'ArrowRight':
          e.preventDefault();
          if (audioEngine.getDuration() > 0) audioEngine.seek(audioEngine.getCurrentTime() + 5);
          break;
        case 'ArrowUp':
          e.preventDefault();
          audioEngine.setVolume(Math.min(1, audioEngine.getVolume() + 0.1));
          break;
        case 'ArrowDown':
          e.preventDefault();
          audioEngine.setVolume(Math.max(0, audioEngine.getVolume() - 0.1));
          break;
        case 'KeyM':
          e.preventDefault();
          audioEngine.setVolume(audioEngine.getVolume() > 0 ? 0 : 0.8);
          break;
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isFullscreen]);

  const handleApplyPreset = (presetId: string) => {
    const preset = PRESETS.find((p) => p.id === presetId);
    if (preset && preset.config) {
      setConfig((prev) => ({
        ...prev,
        ...preset.config,
        background: {
          ...prev.background,
          ...preset.config.background
        },
        reactivity: {
          ...prev.reactivity,
          ...preset.config.reactivity
        }
      }));
    }
  };

  const handleSavePreset = async () => {
    try {
      const destPath = await save({
        defaultPath: 'my-preset.awpreset',
        filters: [{ name: 'AudioWave Preset', extensions: ['awpreset'] }],
      });
      if (!destPath) return;
      const json = JSON.stringify(config, null, 2);
      await rustBridge.writeTextFile(destPath, json);
    } catch (e) {
      console.error('[App] Failed to save preset:', e);
    }
  };

  const handleLoadPreset = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'AudioWave Preset', extensions: ['awpreset'] }],
      });
      if (!selected || typeof selected !== 'string') return;
      const b64 = await rustBridge.readFileB64(selected);
      const content = atob(b64);
      const presetConfig = JSON.parse(content) as VisualizerConfig;
      setConfig((prev) => ({
        ...prev,
        ...presetConfig,
        background: { ...prev.background, ...presetConfig.background },
        text: migrateTextSettings(presetConfig.text),
        reactivity: { ...prev.reactivity, ...presetConfig.reactivity },
        export: { ...prev.export, ...presetConfig.export },
        screenEffects: { ...prev.screenEffects, ...presetConfig.screenEffects },
      }));
    } catch (e) {
      console.error('[App] Failed to load preset:', e);
    }
  };

  const toggleFullscreen = () => setIsFullscreen((v) => !v);

  return (
    <div className={`app-layout ${isFullscreen ? 'fullscreen' : ''}`}>
      {isLoading && (
        <div className="loading-overlay">
          <div className="loading-content">
            <RefreshCw size={32} className="loading-spinner" />
            <span className="loading-text">Loading audio...</span>
          </div>
        </div>
      )}

      <Navbar
        config={config}
        songMeta={songMeta}
        onLoadSong={handleLoadSong}
        onLoadSongPath={handleLoadSongPath}
        onApplyPreset={handleApplyPreset}
        onSavePreset={handleSavePreset}
        onLoadPreset={handleLoadPreset}
        onOpenExport={() => setIsExportModalOpen(true)}
        onOpenHardware={() => setIsHardwareModalOpen(true)}
        isListening={isListening}
        onStartListen={handleStartListen}
        onStopListen={handleStopListen}
        listenError={listenError}
      />

      <main className="main-viewport">
        <VisualizerCanvas config={config} onToggleFullscreen={toggleFullscreen} isFullscreen={isFullscreen} />
        {!isFullscreen && <ControlPanel config={config} onChangeConfig={setConfig} />}
      </main>

      {!isFullscreen && !isListening && <AudioPlayerBar songMeta={songMeta} />}
      {!isFullscreen && isListening && (
        <div className="audio-player-bar listening-bar">
          <div className="player-track-info">
            <Headphones size={20} className="text-secondary" />
            <div className="track-details">
              <span className="track-title">Listening</span>
              <span className="track-artist">Capturing system audio via loopback</span>
            </div>
          </div>
          <div className="player-center">
            <button className="btn btn-primary" onClick={handleStopListen}>
              Stop Listening
            </button>
          </div>
        </div>
      )}

      <ExportModal
        isOpen={isExportModalOpen}
        config={config}
        onClose={() => setIsExportModalOpen(false)}
      />

      <HardwareModal
        isOpen={isHardwareModalOpen}
        onClose={() => setIsHardwareModalOpen(false)}
      />
    </div>
  );
};

export default App;
