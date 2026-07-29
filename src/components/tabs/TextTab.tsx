import React from 'react';
import { VisualizerConfig } from '../../types/visualizer';
import { CustomSelect } from '../CustomSelect';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

export const TextTab: React.FC<Props> = ({ config, updateConfig }) => {
  const handleTextChange = (key: keyof typeof config.text, value: unknown) => {
    updateConfig((prev) => ({ ...prev, text: { ...prev.text, [key]: value } }));
  };

  return (
    <div className="tab-pane">
      <h3 className="section-title">Song Metadata & Typography</h3>

      <div className="control-group">
        <label className="checkbox-label mb-2">
          <input type="checkbox" checked={config.text.showTitle}
            onChange={(e) => handleTextChange('showTitle', e.target.checked)} />
          <span>Display Song Title</span>
        </label>
        <input type="text" value={config.text.songTitle}
          onChange={(e) => handleTextChange('songTitle', e.target.value)}
          placeholder="Song Title" className="input-text" />
      </div>

      <div className="control-group">
        <label className="checkbox-label mb-2">
          <input type="checkbox" checked={config.text.showArtist}
            onChange={(e) => handleTextChange('showArtist', e.target.checked)} />
          <span>Display Artist Name</span>
        </label>
        <input type="text" value={config.text.artistName}
          onChange={(e) => handleTextChange('artistName', e.target.value)}
          placeholder="Artist Name" className="input-text" />
      </div>

      <div className="control-group">
        <label className="label-row">Font Family</label>
        <CustomSelect value={config.text.fontFamily}
          onChange={(v) => handleTextChange('fontFamily', v)}
          options={[
            { value: '"Outfit", "Inter", sans-serif', label: 'Outfit' },
            { value: '"Inter", sans-serif', label: 'Inter' },
            { value: '"Space Grotesk", sans-serif', label: 'Space Grotesk' },
            { value: '"JetBrains Mono", monospace', label: 'JetBrains Mono' },
            { value: '"Georgia", serif', label: 'Georgia' },
            { value: 'system-ui, sans-serif', label: 'System UI' },
          ]} />
      </div>

      <div className="control-group">
        <label className="label-row">Text Preset</label>
        <CustomSelect value={config.text.position}
          onChange={(v) => {
            const presets: Record<string, { x: number; y: number }> = {
              'bottom-center': { x: 50, y: 82 },
              'top-center': { x: 50, y: 15 },
              'center': { x: 50, y: 48 },
              'bottom-left': { x: 8, y: 82 },
            };
            const p = presets[v] || { x: 50, y: 82 };
            updateConfig((prev) => ({
              ...prev,
              text: { ...prev.text, position: v as typeof prev.text.position, textPositionX: p.x, textPositionY: p.y }
            }));
          }}
          options={[
            { value: 'bottom-center', label: 'Bottom Center' },
            { value: 'top-center', label: 'Top Center' },
            { value: 'center', label: 'Center Overlay' },
            { value: 'bottom-left', label: 'Bottom Left' },
          ]} />
      </div>
      <div className="control-group">
        <label className="label-row"><span>Horizontal X ({config.text.textPositionX}%)</span></label>
        <input type="range" min={0} max={100} step={1} value={config.text.textPositionX}
          onChange={(e) => handleTextChange('textPositionX', parseInt(e.target.value))} className="input-range" />
      </div>
      <div className="control-group">
        <label className="label-row"><span>Vertical Y ({config.text.textPositionY}%)</span></label>
        <input type="range" min={0} max={100} step={1} value={config.text.textPositionY}
          onChange={(e) => handleTextChange('textPositionY', parseInt(e.target.value))} className="input-range" />
      </div>

      <div className="control-group">
        <label className="label-row"><span>Title Font Size ({config.text.titleFontSize}px)</span></label>
        <input type="range" min={16} max={48} step={2} value={config.text.titleFontSize}
          onChange={(e) => handleTextChange('titleFontSize', parseInt(e.target.value))} className="input-range" />
      </div>

      <div className="control-group">
        <label className="label-row"><span>Artist Font Size ({config.text.artistFontSize}px)</span></label>
        <input type="range" min={10} max={36} step={2} value={config.text.artistFontSize}
          onChange={(e) => handleTextChange('artistFontSize', parseInt(e.target.value))} className="input-range" />
      </div>

      <div className="control-group">
        <label className="checkbox-label">
          <input type="checkbox" checked={config.text.textShadow}
            onChange={(e) => handleTextChange('textShadow', e.target.checked)} />
          <span>Text Drop Shadow</span>
        </label>
      </div>

      <div className="color-pickers">
        <div className="picker-item">
          <span>Title Color</span>
          <input type="color" value={config.text.titleColor}
            onChange={(e) => handleTextChange('titleColor', e.target.value)} />
        </div>
        <div className="picker-item">
          <span>Artist Color</span>
          <input type="color" value={config.text.artistColor}
            onChange={(e) => handleTextChange('artistColor', e.target.value)} />
        </div>
      </div>
    </div>
  );
};
