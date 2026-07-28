import React from 'react';
import {
  Layers, Circle, Radio, Grid, Minus, AudioWaveform, CircleDot, LineChart,
} from 'lucide-react';
import { VisualizerConfig, VisualizerStyle } from '../../types/visualizer';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

function BarChartIcon({ size }: { size: number }) {
  return <Layers size={size} />;
}

export const StyleTab: React.FC<Props> = ({ config, updateConfig }) => {
  const handleStyleChange = (style: VisualizerStyle) => {
    updateConfig((prev) => ({ ...prev, style }));
  };
  const handleReactivityChange = (key: keyof typeof config.reactivity, value: unknown) => {
    updateConfig((prev) => ({
      ...prev,
      reactivity: { ...prev.reactivity, [key]: value }
    }));
  };

  return (
    <div className="tab-pane">
      <h3 className="section-title">Visualizer Style</h3>
      <div className="style-grid">
        <button className={`style-card ${config.style === 'spectrum' ? 'selected' : ''}`} onClick={() => handleStyleChange('spectrum')}>
          <BarChartIcon size={24} /><span>Spectrum Bars</span>
        </button>
        <button className={`style-card ${config.style === 'radial' ? 'selected' : ''}`} onClick={() => handleStyleChange('radial')}>
          <Circle size={24} /><span>Radial Ring</span>
        </button>
        <button className={`style-card ${config.style === 'oscilloscope' ? 'selected' : ''}`} onClick={() => handleStyleChange('oscilloscope')}>
          <Radio size={24} /><span>Oscilloscope</span>
        </button>
        <button className={`style-card ${config.style === 'equalizer' ? 'selected' : ''}`} onClick={() => handleStyleChange('equalizer')}>
          <Grid size={24} /><span>Matrix Equalizer</span>
        </button>
        <button className={`style-card ${config.style === 'minimal' ? 'selected' : ''}`} onClick={() => handleStyleChange('minimal')}>
          <Minus size={24} /><span>Minimal Wave</span>
        </button>
        <button className={`style-card ${config.style === 'waveformFill' ? 'selected' : ''}`} onClick={() => handleStyleChange('waveformFill')}>
          <AudioWaveform size={24} /><span>Waveform Fill</span>
        </button>
        <button className={`style-card ${config.style === 'circularBars' ? 'selected' : ''}`} onClick={() => handleStyleChange('circularBars')}>
          <CircleDot size={24} /><span>Circular Bars</span>
        </button>
        <button className={`style-card ${config.style === 'smoothSpectrum' ? 'selected' : ''}`} onClick={() => handleStyleChange('smoothSpectrum')}>
          <LineChart size={24} /><span>Smooth Spectrum</span>
        </button>
      </div>

      <div className="control-group mt-4">
        <label className="label-row"><span>Bar Count ({config.reactivity.barCount})</span></label>
        <input type="range" min={16} max={128} step={4} value={config.reactivity.barCount}
          onChange={(e) => handleReactivityChange('barCount', parseInt(e.target.value))} className="input-range" />
      </div>

      <div className="control-group">
        <label className="label-row"><span>Sensitivity ({config.reactivity.sensitivity.toFixed(1)}x)</span></label>
        <input type="range" min={0.5} max={2.5} step={0.1} value={config.reactivity.sensitivity}
          onChange={(e) => handleReactivityChange('sensitivity', parseFloat(e.target.value))} className="input-range" />
      </div>

      <div className="control-group">
        <label className="label-row"><span>Bass Pulse Boost ({config.reactivity.bassMultiplier.toFixed(1)}x)</span></label>
        <input type="range" min={1.0} max={3.0} step={0.1} value={config.reactivity.bassMultiplier}
          onChange={(e) => handleReactivityChange('bassMultiplier', parseFloat(e.target.value))} className="input-range" />
      </div>
    </div>
  );
};
