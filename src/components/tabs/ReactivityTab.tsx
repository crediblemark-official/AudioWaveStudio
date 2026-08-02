import React from 'react';
import { VisualizerConfig } from '../../types/visualizer';
import { CustomSelect } from '../CustomSelect';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

export const ReactivityTab: React.FC<Props> = ({ config, updateConfig }) => {
  const handleReactivityChange = (key: keyof typeof config.reactivity, value: unknown) => {
    updateConfig((prev) => ({ ...prev, reactivity: { ...prev.reactivity, [key]: value } }));
  };

  return (
    <div className="tab-pane">
      <h3 className="section-title">Audio Processing Options</h3>

      <div className="control-group">
        <label className="label-row">FFT Frequency Resolution</label>
        <CustomSelect value={String(config.reactivity.fftSize)}
          onChange={(v) => handleReactivityChange('fftSize', parseInt(v))}
          options={[
            { value: '256', label: '256 Bins (Fast)' },
            { value: '512', label: '512 Bins (Balanced)' },
            { value: '1024', label: '1024 Bins (High Precision)' },
            { value: '2048', label: '2048 Bins (Ultra Detail)' },
          ]} />
      </div>

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

      <div className="control-group mt-3">
        <label className="label-row"><span>Smoothing ({config.reactivity.smoothing.toFixed(2)})</span></label>
        <input type="range" min={0.1} max={0.95} step={0.05} value={config.reactivity.smoothing}
          onChange={(e) => handleReactivityChange('smoothing', parseFloat(e.target.value))} className="input-range" />
      </div>
    </div>
  );
};
