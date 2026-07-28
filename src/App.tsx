import React, { useState } from 'react';
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

  return (
    <div className="app-layout">
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
        <VisualizerCanvas config={config} />
        <ControlPanel config={config} onChangeConfig={setConfig} />
      </main>

      <AudioPlayerBar songMeta={songMeta} />

      <ExportModal
        isOpen={isExportModalOpen}
        config={config}
        onClose={() => setIsExportModalOpen(false)}
      />
    </div>
  );
};

export default App;
