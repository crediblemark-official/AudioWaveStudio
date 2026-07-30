import React from 'react';
import { VisualizerConfig, ScreenEffect } from '../../types/visualizer';
import { CustomSelect } from '../CustomSelect';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

const effectOptions: { value: ScreenEffect; label: string; desc: string }[] = [
  { value: 'none', label: 'None', desc: 'No effect' },
  { value: 'shake', label: 'Screen Shake', desc: 'Canvas vibrates with the beat' },
  { value: 'glitch', label: 'Digital Glitch', desc: 'Random slice displacement & color artifacts' },

  { value: 'vignette', label: 'Pulsing Vignette', desc: 'Edges darken on bass hits' },
  { value: 'pulse', label: 'White Pulse', desc: 'Bright flash on each beat' },
  { value: 'spotlight', label: 'Stage Spotlight', desc: 'Concert light beams from above' },
];

export const EffectsTab: React.FC<Props> = ({ config, updateConfig }) => {
  const s = config.screenEffects;

  const handleChange = (key: keyof typeof s, value: unknown) => {
    updateConfig((prev) => ({
      ...prev,
      screenEffects: { ...prev.screenEffects, [key]: value },
    }));
  };

  const current = effectOptions.find((o) => o.value === s.mainEffect);
  const showIntensity = s.mainEffect !== 'none';

  return (
    <div className="tab-pane">
      <h3 className="section-title">Screen Effects</h3>

      <div className="control-group">
        <label className="checkbox-label">
          <input type="checkbox" checked={s.enabled}
            onChange={(e) => handleChange('enabled', e.target.checked)} />
          <span>Enable Effects</span>
        </label>
      </div>

      {s.enabled && (
        <>
          <div className="control-group">
            <label className="label-row">Effect Type</label>
            <CustomSelect value={s.mainEffect}
              onChange={(v) => handleChange('mainEffect', v)}
              options={effectOptions.map((o) => ({ value: o.value, label: o.label }))} />
            {current && current.desc && (
              <span className="hint-text">{current.desc}</span>
            )}
          </div>

          {showIntensity && s.mainEffect === 'glitch' && (
            <div className="control-group">
              <label className="label-row">
                <span>Glitch Intensity ({Math.round(s.glitchIntensity * 100)}%)</span>
              </label>
              <input type="range" min={0} max={1} step={0.05} value={s.glitchIntensity}
                onChange={(e) => handleChange('glitchIntensity', parseFloat(e.target.value))} className="input-range" />
            </div>
          )}

          {showIntensity && (s.mainEffect === 'vignette' || s.mainEffect === 'pulse') && (
            <div className="control-group">
              <label className="label-row">
                <span>Pulse Strength ({Math.round(s.pulseIntensity * 100)}%)</span>
              </label>
              <input type="range" min={0} max={1} step={0.05} value={s.pulseIntensity}
                onChange={(e) => handleChange('pulseIntensity', parseFloat(e.target.value))} className="input-range" />
            </div>
          )}
        </>
      )}
    </div>
  );
};
