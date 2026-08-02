import React from 'react';
import { VisualizerConfig, AspectRatio } from '../../types/visualizer';
import { CustomSelect } from '../CustomSelect';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

export const ExportTab: React.FC<Props> = ({ config, updateConfig }) => {
  const handleExportSettingChange = (key: keyof typeof config.export, value: unknown) => {
    updateConfig((prev) => ({ ...prev, export: { ...prev.export, [key]: value } }));
  };

  const handleReactivityChange = (key: keyof typeof config.reactivity, value: unknown) => {
    updateConfig((prev) => ({ ...prev, reactivity: { ...prev.reactivity, [key]: value } }));
  };

  return (
    <div className="tab-pane">
      <h3 className="section-title">Video Export Settings</h3>

      <div className="control-group">
        <label className="label-row">Aspect Ratio (Format)</label>
        <div className="btn-group">
          {(['16:9', '9:16', '1:1'] as AspectRatio[]).map((ar) => (
            <button key={ar} className={`btn-toggle ${config.export.aspectRatio === ar ? 'active' : ''}`}
              onClick={() => handleExportSettingChange('aspectRatio', ar)}>
              {ar === '16:9' ? '16:9 Widescreen' : ar === '9:16' ? '9:16 Reel/Shorts' : '1:1 Square'}
            </button>
          ))}
        </div>
      </div>

      <div className="control-group">
        <label className="label-row">Target Resolution</label>
        <CustomSelect value={config.export.resolution}
          onChange={(v) => handleExportSettingChange('resolution', v)}
          options={[
            { value: '1080p', label: '1080p Full HD (1920x1080)' },
            { value: '720p', label: '720p HD (1280x720)' },
            { value: '4K', label: '4K Ultra HD (3840x2160)' },
          ]} />
      </div>

      <div className="control-group">
        <label className="label-row">Frame Rate</label>
        <div className="btn-group">
          <button className={`btn-toggle ${config.export.fps === 30 ? 'active' : ''}`}
            onClick={() => handleExportSettingChange('fps', 30)}>30 FPS</button>
          <button className={`btn-toggle ${config.export.fps === 60 ? 'active' : ''}`}
            onClick={() => handleExportSettingChange('fps', 60)}>60 FPS (Ultra Smooth)</button>
        </div>
      </div>

      <div className="control-group">
        <label className="label-row">Output Format</label>
        <div className="btn-group">
          <button className={`btn-toggle ${config.export.format === 'mp4' ? 'active' : ''}`}
            onClick={() => handleExportSettingChange('format', 'mp4')}>MP4 (H.264)</button>
          <button className={`btn-toggle ${config.export.format === 'webm' ? 'active' : ''}`}
            onClick={() => handleExportSettingChange('format', 'webm')}>WebM</button>
        </div>
      </div>

      <h3 className="section-title mt-4">Audio Analysis</h3>
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
        <span className="hint-text">Higher precision produces more detailed visualizations but uses more processing power</span>
      </div>
    </div>
  );
};
