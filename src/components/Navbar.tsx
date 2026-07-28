import React, { useRef } from 'react';
import { Activity, Upload, Film, Sparkles } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { PRESETS } from '../utils/presets';
import { SongMetadata, VisualizerConfig } from '../types/visualizer';
import { CustomSelect } from './CustomSelect';

interface NavbarProps {
  config: VisualizerConfig;
  songMeta: SongMetadata | null;
  onLoadSong: (file: File) => void;
  onLoadSongPath: (path: string) => void;
  onApplyPreset: (presetId: string) => void;
  onOpenExport: () => void;
}

export const Navbar: React.FC<NavbarProps> = ({
  songMeta,
  onLoadSong,
  onLoadSongPath,
  onApplyPreset,
  onOpenExport
}) => {
  const audioInputRef = useRef<HTMLInputElement>(null);
  const [presetValue, setPresetValue] = React.useState('');

  const handleOpenAudioDialog = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Audio Files',
          extensions: ['mp3', 'wav', 'flac', 'm4a', 'ogg', 'aac']
        }]
      });

      if (selected && typeof selected === 'string') {
        onLoadSongPath(selected);
      }
    } catch {
      audioInputRef.current?.click();
    }
  };

  const handleAudioFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      const file = e.target.files[0];
      const nativePath = (file as unknown as { path?: string }).path;
      if (nativePath) {
        onLoadSongPath(nativePath);
      } else {
        onLoadSong(file);
      }
    }
  };

  return (
    <header className="navbar">
      <div className="navbar-left">
        <div className="brand">
          <div className="brand-icon">
            <Activity className="icon-pulse" size={24} />
          </div>
          <div className="brand-text">
            <span className="brand-name">AudioWave</span>
            <span className="brand-tag">Studio</span>
          </div>
        </div>

        {/* Hidden inputs */}
        <input
          type="file"
          ref={audioInputRef}
          onChange={handleAudioFileChange}
          accept="audio/*,.mp3,.wav,.flac,.m4a,.ogg,.aac"
          className="hidden-input"
        />

        <div className="action-buttons">
          <button
            className="btn btn-primary"
            onClick={handleOpenAudioDialog}
          >
            <Upload size={16} />
            <span>{songMeta ? 'Change Song' : 'Open Audio File'}</span>
          </button>
        </div>
      </div>

      <div className="navbar-right">
        {/* Preset Selector */}
        <div className="preset-selector">
          <Sparkles size={16} className="preset-icon" />
          <CustomSelect
            value={presetValue}
            onChange={(v) => {
              if (!v) return;
              setPresetValue(v);
              onApplyPreset(v);
            }}
            options={[
              { value: '', label: 'Preset Themes' },
              ...PRESETS.map((p) => ({ value: p.id, label: p.name })),
            ]}
          />
        </div>

        {/* Export MP4 Button */}
        <button className="btn btn-export" onClick={onOpenExport}>
          <Film size={18} />
          <span>Export MP4 Video</span>
        </button>
      </div>
    </header>
  );
};
