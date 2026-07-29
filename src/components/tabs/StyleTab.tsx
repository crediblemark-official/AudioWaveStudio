import React, { useRef } from 'react';
import {
  Layers, Circle, Radio, Grid, Minus, AudioWaveform, CircleDot, LineChart, Image as ImageIcon,
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

  const radialImgInputRef = useRef<HTMLInputElement>(null);
  const prevRadialUrlRef = useRef<string>('');

  const handleRadialImageUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      if (prevRadialUrlRef.current) URL.revokeObjectURL(prevRadialUrlRef.current);
      const url = URL.createObjectURL(e.target.files[0]);
      prevRadialUrlRef.current = url;
      updateConfig((prev) => ({
        ...prev,
        background: { ...prev.background, radialCenterImageUri: url }
      }));
    }
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

      <h3 className="section-title">Position</h3>
      <div className="control-group">
        <label className="label-row"><span>Horizontal X ({config.positionX > 0 ? '+' : ''}{config.positionX})</span></label>
        <input type="range" min={-500} max={500} step={1} value={config.positionX}
          onChange={(e) => updateConfig((prev) => ({ ...prev, positionX: parseInt(e.target.value) }))} className="input-range" />
      </div>
      <div className="control-group">
        <label className="label-row"><span>Vertical Y ({config.positionY > 0 ? '+' : ''}{config.positionY})</span></label>
        <input type="range" min={-500} max={500} step={1} value={config.positionY}
          onChange={(e) => updateConfig((prev) => ({ ...prev, positionY: parseInt(e.target.value) }))} className="input-range" />
      </div>

      <h3 className="section-title">Radial Center Image</h3>
      <div className="control-group">
        <button className="btn btn-secondary w-full" onClick={() => radialImgInputRef.current?.click()}>
          <ImageIcon size={16} /><span>{config.background.radialCenterImageUri ? 'Change Center Image' : 'Choose Center Image'}</span>
        </button>
        <input type="file" ref={radialImgInputRef} onChange={handleRadialImageUpload} accept="image/*" className="hidden-input" />
        {config.background.radialCenterImageUri && (
          <button className="w-full mt-1" style={{ background: 'none', border: 'none', color: '#ff5555', cursor: 'pointer', fontSize: '0.8rem', padding: '4px 0' }} onClick={() => {
            if (prevRadialUrlRef.current) URL.revokeObjectURL(prevRadialUrlRef.current);
            updateConfig((prev) => ({ ...prev, background: { ...prev.background, radialCenterImageUri: undefined } }));
          }}>
            Remove Image
          </button>
        )}
      </div>
    </div>
  );
};
