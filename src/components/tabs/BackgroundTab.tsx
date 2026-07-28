import React, { useRef } from 'react';
import { VisualizerConfig, MusicNoteStyle } from '../../types/visualizer';
import { CustomSelect } from '../CustomSelect';
import { Sparkles, Music, Upload } from 'lucide-react';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

export const BackgroundTab: React.FC<Props> = ({ config, updateConfig }) => {
  const bgImageInputRef = useRef<HTMLInputElement>(null);
  const prevBgUrlRef = useRef<string>('');

  const handleBgModeChange = (mode: typeof config.background.mode) => {
    updateConfig((prev) => ({ ...prev, background: { ...prev.background, mode } }));
  };

  const handleBgImageUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      if (prevBgUrlRef.current) URL.revokeObjectURL(prevBgUrlRef.current);
      const url = URL.createObjectURL(e.target.files[0]);
      prevBgUrlRef.current = url;
      updateConfig((prev) => ({
        ...prev,
        background: { ...prev.background, mode: 'customImage', customImageUri: url }
      }));
    }
  };

  return (
    <div className="tab-pane">
      <h3 className="section-title">Background Style</h3>
      <div className="btn-group">
        {(['solid', 'gradient', 'customImage'] as const).map((m) => (
          <button key={m} className={`btn-toggle ${config.background.mode === m ? 'active' : ''}`}
            onClick={() => handleBgModeChange(m)}>
            {m === 'customImage' ? 'Custom Image' : m.charAt(0).toUpperCase() + m.slice(1)}
          </button>
        ))}
      </div>

      {config.background.mode === 'solid' && (
        <div className="control-group mt-3">
          <label className="label-row">
            <span>Background Color</span>
            <input type="color" value={config.background.solidColor}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, solidColor: e.target.value } }))} />
          </label>
        </div>
      )}

      {config.background.mode === 'gradient' && (
        <div className="color-pickers mt-3">
          <div className="picker-item">
            <span>Gradient Start</span>
            <input type="color" value={config.background.gradientStart}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, gradientStart: e.target.value } }))} />
          </div>
          <div className="picker-item">
            <span>Gradient End</span>
            <input type="color" value={config.background.gradientEnd}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, gradientEnd: e.target.value } }))} />
          </div>
        </div>
      )}

      {config.background.mode === 'customImage' && (
        <div className="control-group mt-3">
          <button className="btn btn-secondary w-full" onClick={() => bgImageInputRef.current?.click()}>
            <Upload size={16} /><span>Choose Background Image</span>
          </button>
          <input type="file" ref={bgImageInputRef} onChange={handleBgImageUpload} accept="image/*" className="hidden-input" />
        </div>
      )}

      <div className="control-group mt-3">
        <label className="label-row"><span>Overlay Darkening ({Math.round(config.background.overlayOpacity * 100)}%)</span></label>
        <input type="range" min={0} max={1} step={0.05} value={config.background.overlayOpacity}
          onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, overlayOpacity: parseFloat(e.target.value) } }))}
          className="input-range" />
      </div>

      <div className="control-group mt-3">
        <label className="checkbox-label">
          <input type="checkbox" checked={config.background.showParticles}
            onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, showParticles: e.target.checked } }))} />
          <Sparkles size={16} className="text-secondary mr-1" />
          <span>Show Ambient Floating Particles</span>
        </label>
      </div>

      {config.background.showParticles && (
        <div className="control-group mt-2">
          <div className="picker-item">
            <span>Particle Color</span>
            <input type="color" value={config.background.particleColor}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, particleColor: e.target.value } }))} />
          </div>
        </div>
      )}

      {config.background.mode === 'customImage' && (
        <div className="control-group mt-2">
          <label className="label-row"><span>Background Blur ({config.background.blurAmount}px)</span></label>
          <input type="range" min={0} max={20} step={1} value={config.background.blurAmount}
            onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, blurAmount: parseInt(e.target.value) } }))}
            className="input-range" />
        </div>
      )}

      <div className="control-group mt-3">
        <label className="checkbox-label">
          <input type="checkbox" checked={config.background.showMusicNotes ?? false}
            onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, showMusicNotes: e.target.checked } }))} />
          <Music size={16} className="text-secondary mr-1" />
          <span>Floating Music Notes</span>
        </label>
      </div>

      {(config.background.showMusicNotes ?? false) && (
        <>
          <div className="control-group mt-2">
            <div className="picker-item">
              <span>Note Color</span>
              <input type="color" value={config.background.musicNoteColor ?? '#ffe600'}
                onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteColor: e.target.value } }))} />
            </div>
          </div>
          <div className="control-group mt-2">
            <label className="label-row"><span>Movement Style</span></label>
            <CustomSelect value={config.background.musicNoteStyle ?? 'float'}
              options={[
                { value: 'float', label: 'Floating' },
                { value: 'bounce', label: 'Bouncing' },
                { value: 'spiral', label: 'Spiral' },
                { value: 'wave', label: 'Sinusoidal' },
                { value: 'burst', label: 'Burst' },
              ]}
              onChange={(val) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteStyle: val as MusicNoteStyle } }))} />
          </div>
          <div className="control-group">
            <label className="label-row"><span>Note Density ({Math.round((config.background.musicNoteDensity ?? 0.5) * 100)}%)</span></label>
            <input type="range" min={0} max={1} step={0.05} value={config.background.musicNoteDensity ?? 0.5}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteDensity: parseFloat(e.target.value) } }))}
              className="input-range" />
          </div>
          <div className="control-group">
            <label className="label-row"><span>Note Size ({config.background.musicNoteSize ?? 24}px)</span></label>
            <input type="range" min={8} max={64} step={2} value={config.background.musicNoteSize ?? 24}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteSize: parseInt(e.target.value) } }))}
              className="input-range" />
          </div>
          <div className="control-group">
            <label className="label-row"><span>Float Speed ({((config.background.musicNoteSpeed ?? 1.0) * 100).toFixed(0)}%)</span></label>
            <input type="range" min={0.2} max={3} step={0.1} value={config.background.musicNoteSpeed ?? 1.0}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteSpeed: parseFloat(e.target.value) } }))}
              className="input-range" />
          </div>
          <div className="control-group">
            <label className="label-row"><span>Max Count ({config.background.musicNoteCount ?? 40})</span></label>
            <input type="range" min={5} max={80} step={5} value={config.background.musicNoteCount ?? 40}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteCount: parseInt(e.target.value) } }))}
              className="input-range" />
          </div>
        </>
      )}
    </div>
  );
};
