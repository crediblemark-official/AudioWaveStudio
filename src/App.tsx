import React, { useEffect, useState } from 'react';
import { Navbar } from './components/Navbar';
import { VisualizerCanvas } from './components/VisualizerCanvas';
import { AudioPlayerBar } from './components/AudioPlayerBar';
import { ControlPanel } from './components/ControlPanel';
import { ExportModal } from './components/ExportModal';
import { DEFAULT_CONFIG, PRESETS } from './utils/presets';
import { SongMetadata, VisualizerConfig } from './types/visualizer';
import { audioEngine } from './services/audioEngine';
import { RefreshCw } from 'lucide-react';

export const App: React.FC = () => {
  const [config, setConfig] = useState<VisualizerConfig>(DEFAULT_CONFIG);
  const [songMeta, setSongMeta] = useState<SongMetadata | null>(null);
  const [isExportModalOpen, setIsExportModalOpen] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isFullscreen, setIsFullscreen] = useState<boolean>(false);

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
          artistName: meta.artist
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
          artistName: meta.artist
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
        onOpenExport={() => setIsExportModalOpen(true)}
      />

      <main className="main-viewport">
        <VisualizerCanvas config={config} onToggleFullscreen={toggleFullscreen} isFullscreen={isFullscreen} />
        {!isFullscreen && <ControlPanel config={config} onChangeConfig={setConfig} />}
      </main>

      {!isFullscreen && <AudioPlayerBar songMeta={songMeta} />}

      <ExportModal
        isOpen={isExportModalOpen}
        config={config}
        onClose={() => setIsExportModalOpen(false)}
      />
    </div>
  );
};

export default App;
