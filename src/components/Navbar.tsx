import React, { useRef, useState, useEffect } from 'react';
import { Activity, Upload, Film, Sparkles, Save, FolderOpen, Minus, Square, X, Headphones, StopCircle } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { PRESETS } from '../utils/presets';
import { SongMetadata, VisualizerConfig } from '../types/visualizer';
import { CustomSelect } from './CustomSelect';

const appWindow = getCurrentWindow();

interface NavbarProps {
  config: VisualizerConfig;
  songMeta: SongMetadata | null;
  onLoadSong: (file: File) => void;
  onLoadSongPath: (path: string) => void;
  onApplyPreset: (presetId: string) => void;
  onSavePreset: () => void;
  onLoadPreset: () => void;
  onOpenExport: () => void;
  isListening: boolean;
  onStartListen: (deviceId?: string) => void;
  onStopListen: () => void;
  listenError: string;
}

export const Navbar: React.FC<NavbarProps> = ({
  songMeta,
  onLoadSong,
  onLoadSongPath,
  onApplyPreset,
  onSavePreset,
  onLoadPreset,
  onOpenExport,
  isListening,
  onStartListen,
  onStopListen,
  listenError,
}) => {
  const audioInputRef = useRef<HTMLInputElement>(null);
  const [presetValue, setPresetValue] = React.useState('');
  const [audioDevices, setAudioDevices] = useState<MediaDeviceInfo[]>([]);
  const [showDevicePicker, setShowDevicePicker] = useState(false);

  useEffect(() => {
    if (showDevicePicker) {
      navigator.mediaDevices.enumerateDevices().then(devices => {
        setAudioDevices(devices.filter(d => d.kind === 'audioinput'));
      }).catch(() => {});
    }
  }, [showDevicePicker]);

  const handleListenClick = async () => {
    const devices = await navigator.mediaDevices.enumerateDevices().catch(() => []);
    const audioInputs = devices.filter(d => d.kind === 'audioinput');
    if (audioInputs.length === 0) {
      onStartListen();
    } else if (audioInputs.length === 1) {
      onStartListen(audioInputs[0].deviceId);
    } else {
      setAudioDevices(audioInputs);
      setShowDevicePicker(true);
    }
  };

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
    <header className="navbar" data-tauri-drag-region>
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
            disabled={isListening}
          >
            <Upload size={16} />
            <span>{songMeta ? 'Change Song' : 'Open Audio File'}</span>
          </button>
          <button
            className={`btn ${isListening ? 'btn-danger' : 'btn-secondary'}`}
            onClick={isListening ? onStopListen : handleListenClick}
          >
            {isListening ? <StopCircle size={16} /> : <Headphones size={16} />}
            <span>{isListening ? 'Stop Listen' : 'Listen'}</span>
          </button>
        </div>

        {showDevicePicker && (
          <div className="device-picker-overlay" onClick={() => setShowDevicePicker(false)}>
            <div className="device-picker" onClick={e => e.stopPropagation()}>
              <h4>Select Audio Input</h4>
              {audioDevices.map(d => (
                <button key={d.deviceId} className="device-option" onClick={() => {
                  onStartListen(d.deviceId);
                  setShowDevicePicker(false);
                }}>
                  {d.label || `Input ${d.deviceId.slice(0, 8)}...`}
                </button>
              ))}
              <button className="device-option cancel" onClick={() => setShowDevicePicker(false)}>
                Cancel
              </button>
            </div>
          </div>
        )}

        {listenError && (
          <div className="listen-error-toast">
            <span>Listen failed: {listenError}</span>
          </div>
        )}
      </div>

      <div className="navbar-right">
        {/* Preset Selector */}
        <div className="preset-selector-wrapper">
          <CustomSelect
            value={presetValue}
            icon={<Sparkles size={16} className="preset-icon" />}
            onChange={(v) => {
              if (!v) return;
              setPresetValue('');
              onApplyPreset(v);
            }}
            options={[
              { value: '', label: 'Preset Themes' },
              ...PRESETS.map((p) => ({ value: p.id, label: p.name })),
            ]}
          />
        </div>

        {/* Save / Load Preset Buttons */}
        <div className="preset-actions">
          <button className="btn btn-icon" onClick={onSavePreset} title="Save Preset">
            <Save size={16} />
          </button>
          <button className="btn btn-icon" onClick={onLoadPreset} title="Load Preset">
            <FolderOpen size={16} />
          </button>
        </div>

        {/* Export MP4 Button */}
        <button className="btn btn-export" onClick={onOpenExport}>
          <Film size={18} />
          <span>Export MP4 Video</span>
        </button>

        <div className="window-controls">
          <button className="btn-winctrl" onClick={() => appWindow.minimize()} title="Minimize">
            <Minus size={14} />
          </button>
          <button className="btn-winctrl" onClick={() => appWindow.toggleMaximize()} title="Maximize">
            <Square size={12} />
          </button>
          <button className="btn-winctrl btn-winctrl-close" onClick={() => appWindow.close()} title="Close">
            <X size={14} />
          </button>
        </div>
      </div>
    </header>
  );
};
