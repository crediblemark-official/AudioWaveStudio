import React, { useRef, useEffect, useState } from 'react';
import { VisualizerConfig, MusicNoteStyle, ParticleStyle, BackgroundEffect, BackgroundFillType } from '../../types/visualizer';
import { CustomSelect } from '../CustomSelect';
import { Sparkles, Music, Upload, Square, Layers, Grid3x3, Wind, GripHorizontal, CircleDot, Stars, Cloud, Compass, Trash2, CheckCircle2, Sliders } from 'lucide-react';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

export const BackgroundTab: React.FC<Props> = ({ config, updateConfig }) => {
  const bgImageInputRef = useRef<HTMLInputElement>(null);
  const prevBgUrlRef = useRef<string>('');
  const [selectedSettingsTab, setSelectedSettingsTab] = useState<BackgroundEffect | null>(null);

  useEffect(() => {
    return () => {
      if (prevBgUrlRef.current) {
        URL.revokeObjectURL(prevBgUrlRef.current);
      }
    };
  }, []);

  const fillType: BackgroundFillType = config.background.fillType ?? (config.background.mode === 'gradient' ? 'gradient' : 'solid');
  
  // Multi-select active effects array
  const activeEffects: BackgroundEffect[] = config.background.effects ?? (
    config.background.effect && config.background.effect !== 'none'
      ? [config.background.effect]
      : (['grid', 'aurora', 'noise', 'bokeh', 'starfield', 'nebula', 'psychedelic', 'particles', 'musicNotes'].includes(config.background.mode)
          ? [config.background.mode as BackgroundEffect]
          : [])
  );

  const currentTab = (selectedSettingsTab && activeEffects.includes(selectedSettingsTab))
    ? selectedSettingsTab
    : (activeEffects[0] || null);

  const toggleEffect = (eff: BackgroundEffect) => {
    updateConfig((prev) => {
      const current = prev.background.effects ?? (
        prev.background.effect && prev.background.effect !== 'none'
          ? [prev.background.effect]
          : (['grid', 'aurora', 'noise', 'bokeh', 'starfield', 'nebula', 'psychedelic', 'particles', 'musicNotes'].includes(prev.background.mode)
              ? [prev.background.mode as BackgroundEffect]
              : [])
      );
      const isSelected = current.includes(eff);
      const nextEffects = isSelected ? current.filter((e) => e !== eff) : [...current, eff];
      
      if (!isSelected) {
        setSelectedSettingsTab(eff);
      }
      
      return {
        ...prev,
        background: {
          ...prev.background,
          effects: nextEffects,
          effect: nextEffects[0] || 'none',
          showParticles: nextEffects.includes('particles'),
          showMusicNotes: nextEffects.includes('musicNotes'),
        }
      };
    });
  };

  const clearAllEffects = () => {
    setSelectedSettingsTab(null);
    updateConfig((prev) => ({
      ...prev,
      background: {
        ...prev.background,
        effects: [],
        effect: 'none',
        showParticles: false,
        showMusicNotes: false,
      }
    }));
  };

  const handleFillTypeChange = (ft: BackgroundFillType) => {
    updateConfig((prev) => ({
      ...prev,
      background: {
        ...prev.background,
        fillType: ft,
        mode: ft === 'gradient' ? 'gradient' : (prev.background.customImageUri ? 'customImage' : 'solid')
      }
    }));
  };

  const handleBgImageUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      if (prevBgUrlRef.current) URL.revokeObjectURL(prevBgUrlRef.current);
      const url = URL.createObjectURL(e.target.files[0]);
      prevBgUrlRef.current = url;
      updateConfig((prev) => ({
        ...prev,
        background: { ...prev.background, customImageUri: url }
      }));
      e.target.value = '';
    }
  };

  const handleRemoveBgImage = () => {
    if (prevBgUrlRef.current) {
      URL.revokeObjectURL(prevBgUrlRef.current);
      prevBgUrlRef.current = '';
    }
    updateConfig((prev) => ({
      ...prev,
      background: { ...prev.background, customImageUri: undefined }
    }));
  };

  const effectList: { value: BackgroundEffect; label: string; icon: React.ReactNode }[] = [
    { value: 'grid', label: 'Grid', icon: <Grid3x3 size={15} /> },
    { value: 'particles', label: 'Particles', icon: <Sparkles size={15} /> },
    { value: 'musicNotes', label: 'Music Notes', icon: <Music size={15} /> },
    { value: 'starfield', label: 'Starfield', icon: <Stars size={15} /> },
    { value: 'nebula', label: 'Nebula', icon: <Cloud size={15} /> },
    { value: 'aurora', label: 'Aurora', icon: <Wind size={15} /> },
    { value: 'noise', label: 'Film Grain', icon: <GripHorizontal size={15} /> },
    { value: 'bokeh', label: 'Bokeh', icon: <CircleDot size={15} /> },
    { value: 'psychedelic', label: 'Psychedelic', icon: <Compass size={15} /> },
  ];

  return (
    <div className="tab-pane">
      {/* 1. Base Fill Section */}
      <h3 className="section-title">Base Background Fill</h3>
      <div className="btn-group">
        <button
          className={`btn-toggle ${fillType === 'solid' ? 'active' : ''}`}
          onClick={() => handleFillTypeChange('solid')}
        >
          <Square size={14} className="mr-1 inline" /> Solid Color
        </button>
        <button
          className={`btn-toggle ${fillType === 'gradient' ? 'active' : ''}`}
          onClick={() => handleFillTypeChange('gradient')}
        >
          <Layers size={14} className="mr-1 inline" /> Gradient
        </button>
      </div>

      {fillType === 'solid' ? (
        <div className="control-group mt-3">
          <label className="label-row">
            <span>Background Color</span>
            <input
              type="color"
              value={config.background.solidColor || '#0b0c10'}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, solidColor: e.target.value } }))}
            />
          </label>
        </div>
      ) : (
        <div className="color-pickers mt-3">
          <div className="picker-item">
            <span>Gradient Start</span>
            <input
              type="color"
              value={config.background.gradientStart || '#0f0c20'}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, gradientStart: e.target.value } }))}
            />
          </div>
          <div className="picker-item">
            <span>Gradient End</span>
            <input
              type="color"
              value={config.background.gradientEnd || '#06101e'}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, gradientEnd: e.target.value } }))}
            />
          </div>
        </div>
      )}

      {/* 2. Custom Background Image Section */}
      <h3 className="section-title mt-4">Background Image</h3>
      <div className="control-group">
        <div className="flex gap-2 mt-1">
          <button className="btn btn-secondary flex-1" onClick={() => bgImageInputRef.current?.click()}>
            <Upload size={16} />
            <span>{config.background.customImageUri ? 'Change Image' : 'Choose Background Image'}</span>
          </button>
          {config.background.customImageUri && (
            <button className="btn btn-secondary text-danger" onClick={handleRemoveBgImage} title="Remove Image">
              <Trash2 size={16} />
            </button>
          )}
        </div>
        <input type="file" ref={bgImageInputRef} onChange={handleBgImageUpload} accept="image/*" className="hidden-input" />
      </div>

      {config.background.customImageUri && (
        <>
          <div className="control-group mt-2">
            <label className="label-row">
              <span>Image Opacity ({Math.round((config.background.imageOpacity ?? (config.background.mode === 'customImage' ? 1.0 : 0.7)) * 100)}%)</span>
            </label>
            <input
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={config.background.imageOpacity ?? (config.background.mode === 'customImage' ? 1.0 : 0.7)}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, imageOpacity: parseFloat(e.target.value) } }))}
              className="input-range"
            />
          </div>
          <div className="control-group mt-2">
            <label className="label-row"><span>Background Blur ({config.background.blurAmount || 0}px)</span></label>
            <input
              type="range"
              min={0}
              max={20}
              step={1}
              value={config.background.blurAmount || 0}
              onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, blurAmount: parseInt(e.target.value) } }))}
              className="input-range"
            />
          </div>
        </>
      )}

      {/* 3. Overlay Effect Section (Multi-Select Stacking) */}
      <div className="flex items-center justify-between mt-4">
        <h3 className="section-title my-0">Overlay Effects (Combine Multiple)</h3>
        {activeEffects.length > 0 && (
          <button className="btn btn-link text-xs text-danger p-0" onClick={clearAllEffects}>
            Clear All ({activeEffects.length})
          </button>
        )}
      </div>

      <div className="color-pickers mt-2" style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px' }}>
        {effectList.map((item) => {
          const isActive = activeEffects.includes(item.value);
          return (
            <button
              key={item.value}
              type="button"
              className={`btn-toggle ${isActive ? 'active' : ''}`}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '5px',
                padding: '8px 4px',
                fontSize: '0.75rem',
                border: isActive ? '1px solid var(--accent-color, #00f0ff)' : '1px solid rgba(255,255,255,0.1)',
                background: isActive ? 'rgba(0, 240, 255, 0.15)' : 'rgba(255,255,255,0.03)',
                color: isActive ? '#ffffff' : 'var(--text-secondary, #a0aec0)',
                borderRadius: '6px',
                cursor: 'pointer',
              }}
              onClick={() => toggleEffect(item.value)}
            >
              {item.icon}
              <span>{item.label}</span>
              {isActive && <CheckCircle2 size={12} style={{ marginLeft: 'auto', color: '#00f0ff' }} />}
            </button>
          );
        })}
      </div>

      {/* Unified Settings Panel with Single-Line Dropdown Selector */}
      {activeEffects.length > 0 && (
        <div className="mt-3">
          {activeEffects.length > 1 ? (
            <div className="control-group mb-3">
              <label className="label-row flex items-center justify-between text-xs font-semibold" style={{ color: '#00f0ff' }}>
                <span className="flex items-center gap-1.5">
                  <Sliders size={14} /> Configure Effect:
                </span>
              </label>
              <CustomSelect
                value={currentTab || activeEffects[0]}
                options={activeEffects.map((eff) => {
                  const item = effectList.find((e) => e.value === eff);
                  return {
                    value: eff,
                    label: `${item?.label || eff} Settings`,
                  };
                })}
                onChange={(val) => setSelectedSettingsTab(val as BackgroundEffect)}
              />
            </div>
          ) : (
            <div className="flex items-center gap-2 mb-2 text-xs font-bold" style={{ color: '#00f0ff' }}>
              <Sliders size={14} />
              <span>{effectList.find((e) => e.value === currentTab)?.label || 'Effect'} Settings</span>
            </div>
          )}

          {/* Grid Settings */}
          {currentTab === 'grid' && (
            <div>
              <div className="control-group mt-2">
                <label className="label-row"><span>Grid Color</span></label>
                <input
                  type="color"
                  value={config.background.gridColor || '#ffffff'}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, gridColor: e.target.value } }))}
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Grid Size ({config.background.gridSize || 40}px)</span></label>
                <input
                  type="range"
                  min={10}
                  max={120}
                  step={5}
                  value={config.background.gridSize || 40}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, gridSize: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Line Width ({config.background.gridLineWidth || 1}px)</span></label>
                <input
                  type="range"
                  min={1}
                  max={6}
                  step={1}
                  value={config.background.gridLineWidth || 1}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, gridLineWidth: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
            </div>
          )}

          {/* Floating Particles Settings */}
          {currentTab === 'particles' && (
            <div>
              <div className="control-group mt-2">
                <label className="label-row">Movement Style</label>
                <CustomSelect
                  value={config.background.particleStyle ?? 'float'}
                  options={[
                    { value: 'float', label: 'Float' },
                    { value: 'bounce', label: 'Bounce' },
                    { value: 'wave', label: 'Wave' },
                    { value: 'confined', label: 'Confined' },
                    { value: 'static', label: 'Static' },
                  ]}
                  onChange={(val) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, particleStyle: val as ParticleStyle } }))}
                />
              </div>
              <div className="control-group mt-2">
                <div className="picker-item">
                  <span>Color</span>
                  <input
                    type="color"
                    value={config.background.particleColor || '#00f0ff'}
                    onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, particleColor: e.target.value } }))}
                  />
                </div>
              </div>
              <div className="control-group">
                <label className="label-row"><span>Particle Size ({config.background.particleSize ?? 4}px)</span></label>
                <input
                  type="range"
                  min={1}
                  max={12}
                  step={1}
                  value={config.background.particleSize ?? 4}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, particleSize: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Reactivity Speed ({((config.background.particleSpeed ?? 1.0) * 100).toFixed(0)}%)</span></label>
                <input
                  type="range"
                  min={0.2}
                  max={3}
                  step={0.1}
                  value={config.background.particleSpeed ?? 1.0}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, particleSpeed: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Particle Count ({config.background.particleCount ?? 60})</span></label>
                <input
                  type="range"
                  min={10}
                  max={150}
                  step={5}
                  value={config.background.particleCount ?? 60}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, particleCount: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
            </div>
          )}

          {/* Floating Music Notes Settings */}
          {currentTab === 'musicNotes' && (
            <div>
              <div className="control-group mt-2">
                <div className="picker-item">
                  <span>Note Color</span>
                  <input
                    type="color"
                    value={config.background.musicNoteColor ?? '#ffe600'}
                    onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteColor: e.target.value } }))}
                  />
                </div>
              </div>
              <div className="control-group mt-2">
                <label className="label-row"><span>Movement Style</span></label>
                <CustomSelect
                  value={config.background.musicNoteStyle ?? 'float'}
                  options={[
                    { value: 'float', label: 'Floating' },
                    { value: 'bounce', label: 'Bouncing' },
                    { value: 'spiral', label: 'Spiral' },
                    { value: 'wave', label: 'Sinusoidal' },
                    { value: 'burst', label: 'Burst' },
                    { value: 'confined', label: 'Confined' },
                  ]}
                  onChange={(val) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteStyle: val as MusicNoteStyle } }))}
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Note Density ({Math.round((config.background.musicNoteDensity ?? 1.0) * 100)}%)</span></label>
                <input
                  type="range"
                  min={0.1}
                  max={1}
                  step={0.05}
                  value={config.background.musicNoteDensity ?? 1.0}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteDensity: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Note Size ({config.background.musicNoteSize ?? 60}px)</span></label>
                <input
                  type="range"
                  min={16}
                  max={120}
                  step={4}
                  value={config.background.musicNoteSize ?? 60}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteSize: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Max Count ({config.background.musicNoteCount ?? 80})</span></label>
                <input
                  type="range"
                  min={10}
                  max={80}
                  step={5}
                  value={config.background.musicNoteCount ?? 80}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteCount: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Beat Sensitivity ({Math.round((config.background.musicNoteSensitivity ?? 1.0) * 100)}%)</span></label>
                <input
                  type="range"
                  min={0}
                  max={2}
                  step={0.1}
                  value={config.background.musicNoteSensitivity ?? 1.0}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, musicNoteSensitivity: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
            </div>
          )}

          {/* Starfield Settings */}
          {currentTab === 'starfield' && (
            <div>
              <div className="control-group mt-2">
                <label className="label-row"><span>Star Count ({config.background.starCount ?? 160})</span></label>
                <input
                  type="range"
                  min={30}
                  max={300}
                  step={10}
                  value={config.background.starCount ?? 160}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, starCount: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Twinkle Speed ({((config.background.starSpeed ?? 1.0) * 100).toFixed(0)}%)</span></label>
                <input
                  type="range"
                  min={0.2}
                  max={3.0}
                  step={0.1}
                  value={config.background.starSpeed ?? 1.0}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, starSpeed: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Brightness ({Math.round((config.background.starBrightness ?? 1.0) * 100)}%)</span></label>
                <input
                  type="range"
                  min={0.2}
                  max={1.5}
                  step={0.05}
                  value={config.background.starBrightness ?? 1.0}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, starBrightness: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
            </div>
          )}

          {/* Nebula Settings */}
          {currentTab === 'nebula' && (
            <div>
              <div className="control-group mt-2">
                <label className="label-row"><span>Nebula Intensity ({Math.round((config.background.nebulaIntensity ?? 0.6) * 100)}%)</span></label>
                <input
                  type="range"
                  min={0.1}
                  max={1.0}
                  step={0.05}
                  value={config.background.nebulaIntensity ?? 0.6}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, nebulaIntensity: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Animation Speed ({((config.background.nebulaSpeed ?? 1.0) * 100).toFixed(0)}%)</span></label>
                <input
                  type="range"
                  min={0.2}
                  max={3.0}
                  step={0.1}
                  value={config.background.nebulaSpeed ?? 1.0}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, nebulaSpeed: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
            </div>
          )}

          {/* Aurora Settings */}
          {currentTab === 'aurora' && (
            <div>
              <div className="control-group mt-2">
                <label className="label-row"><span>Wave Speed ({((config.background.auroraSpeed ?? 1.0) * 100).toFixed(0)}%)</span></label>
                <input
                  type="range"
                  min={0.2}
                  max={3.0}
                  step={0.1}
                  value={config.background.auroraSpeed ?? 1.0}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, auroraSpeed: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Wave Height ({config.background.auroraAmplitude ?? 50}px)</span></label>
                <input
                  type="range"
                  min={10}
                  max={120}
                  step={5}
                  value={config.background.auroraAmplitude ?? 50}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, auroraAmplitude: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Aurora Opacity ({Math.round((config.background.auroraOpacity ?? 0.25) * 100)}%)</span></label>
                <input
                  type="range"
                  min={0.05}
                  max={0.6}
                  step={0.05}
                  value={config.background.auroraOpacity ?? 0.25}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, auroraOpacity: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
            </div>
          )}

          {/* Film Grain Settings */}
          {currentTab === 'noise' && (
            <div>
              <div className="control-group mt-2">
                <label className="label-row"><span>Grain Opacity ({Math.round((config.background.grainOpacity ?? 0.08) * 100)}%)</span></label>
                <input
                  type="range"
                  min={0.01}
                  max={0.3}
                  step={0.01}
                  value={config.background.grainOpacity ?? 0.08}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, grainOpacity: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
            </div>
          )}

          {/* Bokeh Settings */}
          {currentTab === 'bokeh' && (
            <div>
              <div className="control-group mt-2">
                <label className="label-row"><span>Orb Count ({config.background.bokehCount ?? 18})</span></label>
                <input
                  type="range"
                  min={5}
                  max={40}
                  step={1}
                  value={config.background.bokehCount ?? 18}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, bokehCount: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Orb Size ({config.background.bokehSize ?? 30}px)</span></label>
                <input
                  type="range"
                  min={10}
                  max={80}
                  step={2}
                  value={config.background.bokehSize ?? 30}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, bokehSize: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Bokeh Opacity ({Math.round((config.background.bokehOpacity ?? 0.3) * 100)}%)</span></label>
                <input
                  type="range"
                  min={0.05}
                  max={0.6}
                  step={0.05}
                  value={config.background.bokehOpacity ?? 0.3}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, bokehOpacity: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
            </div>
          )}

          {/* Psychedelic Settings */}
          {currentTab === 'psychedelic' && (
            <div>
              <div className="control-group mt-2">
                <label className="label-row"><span>Spiral Speed ({((config.background.psychedelicSpeed ?? 1.0) * 100).toFixed(0)}%)</span></label>
                <input
                  type="range"
                  min={0.2}
                  max={3.0}
                  step={0.1}
                  value={config.background.psychedelicSpeed ?? 1.0}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, psychedelicSpeed: parseFloat(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Ring Count ({config.background.psychedelicBands ?? 24})</span></label>
                <input
                  type="range"
                  min={10}
                  max={50}
                  step={2}
                  value={config.background.psychedelicBands ?? 24}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, psychedelicBands: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
              <div className="control-group">
                <label className="label-row"><span>Line Width ({config.background.psychedelicLineWidth ?? 4}px)</span></label>
                <input
                  type="range"
                  min={1}
                  max={12}
                  step={1}
                  value={config.background.psychedelicLineWidth ?? 4}
                  onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, psychedelicLineWidth: parseInt(e.target.value) } }))}
                  className="input-range"
                />
              </div>
            </div>
          )}
        </div>
      )}

      {/* 4. Canvas Darkening */}
      <h3 className="section-title mt-4">Canvas Darkening</h3>
      <div className="control-group">
        <label className="label-row"><span>Overlay Darkening ({Math.round(config.background.overlayOpacity * 100)}%)</span></label>
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={config.background.overlayOpacity}
          onChange={(e) => updateConfig((prev) => ({ ...prev, background: { ...prev.background, overlayOpacity: parseFloat(e.target.value) } }))}
          className="input-range"
        />
      </div>
    </div>
  );
};




