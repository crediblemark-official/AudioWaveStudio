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
  { value: 'strobe', label: 'Strobe Flash', desc: 'Rapid white flashes synced to the music' },
  { value: 'scanline', label: 'CRT Scanlines', desc: 'Retro TV scanlines that darken with the beat' },
  { value: 'chromatic', label: 'Chromatic Aberration', desc: 'RGB color-split ghosting on bass' },
  { value: 'zoom', label: 'Zoom Pulse', desc: 'Canvas zooms in on each beat' },
  { value: 'invert', label: 'Invert Flash', desc: 'Colors briefly invert on bass hits' },
  { value: 'bars', label: 'Cinematic Bars', desc: 'Content pushes in with letterbox bars on bass' },
  { value: 'shockwave', label: 'Shockwave Ripple', desc: 'Radial ripple distortion from the center on each beat' },
  { value: 'pixelate', label: 'Pixelate', desc: 'Frame drops to chunky blocks on bass' },
  { value: 'tilt', label: 'Tilt Wobble', desc: 'Canvas tilts back & forth with the music' },
  { value: 'heatHaze', label: 'Heat Haze', desc: 'Wavy air distortion shimmering on bass' },
  { value: 'hueShift', label: 'Rainbow Hue Shift', desc: 'Colors drift through the spectrum' },
];

function Slider({
  label,
  value,
  min,
  max,
  step,
  onChange,
  hint,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (v: number) => void;
  hint?: string;
}) {
  return (
    <div className="control-group">
      <label className="label-row">
        <span>{label}</span>
      </label>
      <input type="range" min={min} max={max} step={step} value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))} className="input-range" />
      {hint && <span className="hint-text">{hint}</span>}
    </div>
  );
}

export const EffectsTab: React.FC<Props> = ({ config, updateConfig }) => {
  const s = config.screenEffects;

  const handleChange = (key: keyof typeof s, value: unknown) => {
    updateConfig((prev) => ({
      ...prev,
      screenEffects: { ...prev.screenEffects, [key]: value },
    }));
  };

  const current = effectOptions.find((o) => o.value === s.mainEffect);
  const show = s.mainEffect !== 'none';

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
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={s.backgroundOnly ?? true}
                onChange={(e) => handleChange('backgroundOnly', e.target.checked)}
              />
              <span>Apply Effects to Background Only (Keep Visualizer & Text on Outermost Top Layer)</span>
            </label>
          </div>

          <div className="control-group">
            <label className="label-row">Effect Type</label>
            <CustomSelect value={s.mainEffect}
              onChange={(v) => handleChange('mainEffect', v)}
              options={effectOptions.map((o) => ({ value: o.value, label: o.label }))} />
            {current && current.desc && (
              <span className="hint-text">{current.desc}</span>
            )}
          </div>

          {show && s.mainEffect === 'shake' && (
            <>
              <Slider label={`Shake Intensity (${Math.round(s.shakeIntensity * 100)}%)`}
                value={s.shakeIntensity} min={0} max={1} step={0.05}
                onChange={(v) => handleChange('shakeIntensity', v)}
                hint="Background moves with parallax for extra depth." />
              <Slider label={`Shake Frequency (${Math.round(s.shakeFrequency * 100)}%)`}
                value={s.shakeFrequency} min={0} max={1} step={0.05}
                onChange={(v) => handleChange('shakeFrequency', v)}
                hint="Low = slow sway, high = rapid jitter." />
              <Slider label={`Max Offset (${s.shakeMaxOffset}px)`}
                value={s.shakeMaxOffset} min={5} max={120} step={5}
                onChange={(v) => handleChange('shakeMaxOffset', v)} />
              <div className="control-group">
                <label className="checkbox-label">
                  <input type="checkbox" checked={s.shakeOnBeat}
                    onChange={(e) => handleChange('shakeOnBeat', e.target.checked)} />
                  <span>Shake only on beats</span>
                </label>
              </div>
            </>
          )}

          {show && s.mainEffect === 'glitch' && (
            <Slider label={`Glitch Intensity (${Math.round(s.glitchIntensity * 100)}%)`}
              value={s.glitchIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('glitchIntensity', v)} />
          )}

          {show && (s.mainEffect === 'vignette' || s.mainEffect === 'pulse') && (
            <Slider label={`Pulse Strength (${Math.round(s.pulseIntensity * 100)}%)`}
              value={s.pulseIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('pulseIntensity', v)} />
          )}

          {show && s.mainEffect === 'strobe' && (
            <Slider label={`Strobe Intensity (${Math.round(s.strobeIntensity * 100)}%)`}
              value={s.strobeIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('strobeIntensity', v)} />
          )}

          {show && s.mainEffect === 'scanline' && (
            <Slider label={`Scanline Opacity (${Math.round(s.scanlineOpacity * 100)}%)`}
              value={s.scanlineOpacity} min={0} max={0.5} step={0.01}
              onChange={(v) => handleChange('scanlineOpacity', v)} />
          )}

          {show && s.mainEffect === 'chromatic' && (
            <Slider label={`Chromatic Intensity (${Math.round(s.chromaticIntensity * 100)}%)`}
              value={s.chromaticIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('chromaticIntensity', v)} />
          )}

          {show && s.mainEffect === 'zoom' && (
            <Slider label={`Zoom Intensity (${Math.round(s.zoomIntensity * 100)}%)`}
              value={s.zoomIntensity} min={0} max={0.5} step={0.01}
              onChange={(v) => handleChange('zoomIntensity', v)} />
          )}

          {show && s.mainEffect === 'invert' && (
            <Slider label={`Invert Intensity (${Math.round(s.invertIntensity * 100)}%)`}
              value={s.invertIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('invertIntensity', v)} />
          )}

          {show && s.mainEffect === 'bars' && (
            <Slider label={`Bars Amount (${Math.round(s.barsAmount * 100)}%)`}
              value={s.barsAmount} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('barsAmount', v)} />
          )}

          {show && s.mainEffect === 'shockwave' && (
            <Slider label={`Shockwave Intensity (${Math.round(s.shockwaveIntensity * 100)}%)`}
              value={s.shockwaveIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('shockwaveIntensity', v)} />
          )}

          {show && s.mainEffect === 'pixelate' && (
            <Slider label={`Pixelate Intensity (${Math.round(s.pixelateIntensity * 100)}%)`}
              value={s.pixelateIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('pixelateIntensity', v)} />
          )}

          {show && s.mainEffect === 'tilt' && (
            <Slider label={`Tilt Intensity (${Math.round(s.tiltIntensity * 100)}%)`}
              value={s.tiltIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('tiltIntensity', v)} />
          )}

          {show && s.mainEffect === 'heatHaze' && (
            <Slider label={`Heat Haze Intensity (${Math.round(s.heatHazeIntensity * 100)}%)`}
              value={s.heatHazeIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('heatHazeIntensity', v)} />
          )}

          {show && s.mainEffect === 'hueShift' && (
            <Slider label={`Hue Shift Intensity (${Math.round(s.hueShiftIntensity * 100)}%)`}
              value={s.hueShiftIntensity} min={0} max={1} step={0.05}
              onChange={(v) => handleChange('hueShiftIntensity', v)} />
          )}
        </>
      )}
    </div>
  );
};
