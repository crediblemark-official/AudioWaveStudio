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
        <label className="label-row">Text Position</label>
        <CustomSelect value={config.text.position}
          onChange={(v) => handleTextChange('position', v)}
          options={[
            { value: 'bottom-center', label: 'Bottom Center' },
            { value: 'top-center', label: 'Top Center' },
            { value: 'center', label: 'Center Overlay' },
            { value: 'bottom-left', label: 'Bottom Left' },
          ]} />
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
