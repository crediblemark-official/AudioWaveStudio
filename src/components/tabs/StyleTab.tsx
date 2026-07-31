import React, { useRef } from 'react';
import {
  Layers, Circle, Radio, Grid, Minus, AudioWaveform, CircleDot, LineChart, Disc, Image as ImageIcon,
} from 'lucide-react';
import { VisualizerConfig, VisualizerStyle } from '../../types/visualizer';
import { fileToDataUrl } from '../../utils/imageUtils';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

function BarChartIcon({ size }: { size: number }) {
  return <Layers size={size} />;
}

const barStyles: VisualizerStyle[] = ['spectrum', 'radial', 'equalizer', 'minimal', 'smoothSpectrum', 'circularBars', 'threeD', 'api3D', 'neonCity3D', 'speaker3D', 'speakerTrio', 'speakerSplatter'];
const sensitivityStyles: VisualizerStyle[] = ['spectrum', 'radial', 'equalizer', 'minimal', 'smoothSpectrum', 'circularBars', 'oscilloscope', 'waveformFill', 'vuMeter', 'flameFire', 'threeD', 'api3D', 'neonCity3D', 'speaker3D', 'speakerTrio', 'speakerSplatter'];

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

  const handleRadialImageUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files && e.target.files[0];
    if (file) {
      try {
        const dataUrl = await fileToDataUrl(file);
        updateConfig((prev) => ({
          ...prev,
          background: { ...prev.background, radialCenterImageUri: dataUrl }
        }));
      } catch (err) {
        console.error('Failed to read radial center image:', err);
      }
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
        <button className={`style-card ${config.style === 'pulseRings' ? 'selected' : ''}`} onClick={() => handleStyleChange('pulseRings')}>
          <Radio size={24} /><span>Pulse Rings</span>
        </button>
        <button className={`style-card ${config.style === 'vuMeter' ? 'selected' : ''}`} onClick={() => handleStyleChange('vuMeter')}>
          <CircleDot size={24} /><span>VU Meter</span>
        </button>
        <button className={`style-card ${config.style === 'auroraWave' ? 'selected' : ''}`} onClick={() => handleStyleChange('auroraWave')}>
          <Layers size={24} /><span>Aurora Wave</span>
        </button>
        <button className={`style-card ${config.style === 'flameFire' ? 'selected' : ''}`} onClick={() => handleStyleChange('flameFire')}>
          <Circle size={24} /><span>Flame Fire</span>
        </button>
        <button className={`style-card ${config.style === 'spiralGalaxy' ? 'selected' : ''}`} onClick={() => handleStyleChange('spiralGalaxy')}>
          <CircleDot size={24} /><span>Spiral Galaxy</span>
        </button>
        <button className={`style-card ${config.style === 'threeD' ? 'selected' : ''}`} onClick={() => handleStyleChange('threeD')}>
          <Grid size={24} /><span>3D Blocks</span>
        </button>
        <button className={`style-card ${config.style === 'api3D' ? 'selected' : ''}`} onClick={() => handleStyleChange('api3D')}>
          <CircleDot size={24} /><span>Fire 3D</span>
        </button>
        <button className={`style-card ${config.style === 'neonCity3D' ? 'selected' : ''}`} onClick={() => handleStyleChange('neonCity3D')}>
          <Grid size={24} /><span>Neon City 3D</span>
        </button>
        <button className={`style-card ${config.style === 'speaker3D' ? 'selected' : ''}`} onClick={() => handleStyleChange('speaker3D')}>
          <Radio size={24} /><span>Speaker 3D</span>
        </button>
        <button className={`style-card ${config.style === 'speakerTrio' ? 'selected' : ''}`} onClick={() => handleStyleChange('speakerTrio')}>
          <Radio size={24} /><span>Speaker Trio</span>
        </button>
        <button className={`style-card ${config.style === 'speakerSplatter' ? 'selected' : ''}`} onClick={() => handleStyleChange('speakerSplatter')}>
          <Disc size={24} /><span>Speaker Splatter</span>
        </button>
      </div>

      <hr className="section-divider" />

      {barStyles.includes(config.style) && (
        <div className="control-group mt-4">
          <label className="label-row"><span>Bar Count ({config.reactivity.barCount})</span></label>
          <input type="range" min={16} max={128} step={4} value={config.reactivity.barCount}
            onChange={(e) => handleReactivityChange('barCount', parseInt(e.target.value))} className="input-range" />
        </div>
      )}

      {sensitivityStyles.includes(config.style) && (
        <div className="control-group">
          <label className="label-row"><span>Sensitivity ({config.reactivity.sensitivity.toFixed(1)}x)</span></label>
          <input type="range" min={0.5} max={2.5} step={0.1} value={config.reactivity.sensitivity}
            onChange={(e) => handleReactivityChange('sensitivity', parseFloat(e.target.value))} className="input-range" />
        </div>
      )}

      <div className="control-group">
        <label className="label-row"><span>Bass Pulse Boost ({config.reactivity.bassMultiplier.toFixed(1)}x)</span></label>
        <input type="range" min={1.0} max={3.0} step={0.1} value={config.reactivity.bassMultiplier}
          onChange={(e) => handleReactivityChange('bassMultiplier', parseFloat(e.target.value))} className="input-range" />
      </div>

      {config.style === 'api3D' && (
        <>
          <h3 className="section-title mt-3">Fire 3D Dimension Controls</h3>
          <div className="control-group">
            <label className="label-row">
              <span>Fire Wave Width / Lebar Api ({Math.round((config.reactivity.fireWidthRatio ?? 0.94) * 100)}%)</span>
            </label>
            <input
              type="range"
              min={0.3}
              max={1.0}
              step={0.02}
              value={config.reactivity.fireWidthRatio ?? 0.94}
              onChange={(e) => handleReactivityChange('fireWidthRatio', parseFloat(e.target.value))}
              className="input-range"
            />
          </div>
          <div className="control-group">
            <label className="label-row">
              <span>Fire Wave Height / Tinggi Api ({(config.reactivity.fireHeightScale ?? 1.0).toFixed(1)}x)</span>
            </label>
            <input
              type="range"
              min={0.3}
              max={2.5}
              step={0.1}
              value={config.reactivity.fireHeightScale ?? 1.0}
              onChange={(e) => handleReactivityChange('fireHeightScale', parseFloat(e.target.value))}
              className="input-range"
            />
          </div>
        </>
      )}

      <h3 className="section-title">Scale & Position</h3>
      <div className="control-group">
        <label className="label-row"><span>Scale ({Math.round(config.scale * 100)}%)</span></label>
        <input type="range" min={0.1} max={2.0} step={0.05} value={config.scale}
          onChange={(e) => updateConfig((prev) => ({ ...prev, scale: parseFloat(e.target.value) }))} className="input-range" />
      </div>
      <div className="control-group">
        <label className="label-row"><span>Horizontal X ({config.positionX > 0 ? '+' : ''}{config.positionX})</span></label>
        <input type="range" min={-1200} max={1200} step={1} value={config.positionX}
          onChange={(e) => updateConfig((prev) => ({ ...prev, positionX: parseInt(e.target.value) }))} className="input-range" />
      </div>
      <div className="control-group">
        <label className="label-row"><span>Vertical Y ({config.positionY > 0 ? '+' : ''}{config.positionY})</span></label>
        <input type="range" min={-1200} max={1200} step={1} value={config.positionY}
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
            updateConfig((prev) => ({ ...prev, background: { ...prev.background, radialCenterImageUri: undefined } }));
          }}>
            Remove Image
          </button>
        )}
      </div>
    </div>
  );
};
