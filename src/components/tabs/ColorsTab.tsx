import React from 'react';
import { VisualizerConfig, ColorThemeName } from '../../types/visualizer';
import { COLOR_THEMES } from '../../utils/presets';
import { CustomSelect } from '../CustomSelect';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

export const ColorsTab: React.FC<Props> = ({ config, updateConfig }) => {
  const handleThemeChange = (themeKey: ColorThemeName) => {
    if (themeKey === 'custom') return;
    const theme = COLOR_THEMES[themeKey];
    if (theme) {
      updateConfig((prev) => ({ ...prev, theme }));
    }
  };

  const handleCustomColorChange = (key: keyof typeof config.theme, value: string) => {
    updateConfig((prev) => ({
      ...prev,
      theme: { ...prev.theme, name: 'custom', label: 'Custom', [key]: value }
    }));
  };

  const handleReactivityChange = (key: keyof typeof config.reactivity, value: unknown) => {
    updateConfig((prev) => ({ ...prev, reactivity: { ...prev.reactivity, [key]: value } }));
  };

  return (
    <div className="tab-pane">
      <h3 className="section-title">Color Palette Presets</h3>
      <div className="control-group">
        <CustomSelect
          value={config.theme.name === 'custom' ? '' : config.theme.name}
          onChange={(v) => handleThemeChange(v as ColorThemeName)}
          options={Object.values(COLOR_THEMES).map((t) => ({ value: t.name, label: t.label }))}
        />
      </div>

      <h3 className="section-title mt-4">Custom Colors</h3>
      <div className="color-pickers">
        <div className="picker-item">
          <span>Primary</span>
          <input type="color" value={config.theme.primaryColor}
            onChange={(e) => handleCustomColorChange('primaryColor', e.target.value)} />
        </div>
        <div className="picker-item">
          <span>Secondary</span>
          <input type="color" value={config.theme.secondaryColor}
            onChange={(e) => handleCustomColorChange('secondaryColor', e.target.value)} />
        </div>
        <div className="picker-item">
          <span>Accent</span>
          <input type="color" value={config.theme.accentColor}
            onChange={(e) => handleCustomColorChange('accentColor', e.target.value)} />
        </div>
        <div className="picker-item">
          <span>Neon Glow</span>
          <input type="color" value={config.theme.glowColor}
            onChange={(e) => handleCustomColorChange('glowColor', e.target.value)} />
        </div>
      </div>

      <h3 className="section-title mt-4">Peak Markers</h3>
      <div className="control-group">
        <label className="checkbox-label">
          <input type="checkbox" checked={config.reactivity.showPeaks}
            onChange={(e) => handleReactivityChange('showPeaks', e.target.checked)} />
          <span>Show Dynamic Peak Drop Markers</span>
        </label>
      </div>
      {config.reactivity.showPeaks && (
        <div className="picker-item mt-2">
          <span>Peak Marker Color</span>
          <input type="color" value={config.reactivity.peakColor}
            onChange={(e) => handleReactivityChange('peakColor', e.target.value)} />
        </div>
      )}
    </div>
  );
};
