import React, { useState } from 'react';
import { Sliders, Palette, Image as ImageIcon, Type, Film, Tv2 } from 'lucide-react';
import { VisualizerConfig } from '../types/visualizer';
import { StyleTab } from './tabs/StyleTab';
import { ColorsTab } from './tabs/ColorsTab';
import { BackgroundTab } from './tabs/BackgroundTab';
import { TextTab } from './tabs/TextTab';
import { ExportTab } from './tabs/ExportTab';
import { EffectsTab } from './tabs/EffectsTab';

interface ControlPanelProps {
  config: VisualizerConfig;
  onChangeConfig: (newConfig: VisualizerConfig) => void;
}

type TabType = 'style' | 'colors' | 'background' | 'text' | 'export' | 'effects';

const tabs: { key: TabType; icon: React.ReactNode; label: string }[] = [
  { key: 'style', icon: <Sliders size={18} />, label: 'Style' },
  { key: 'colors', icon: <Palette size={18} />, label: 'Colors' },
  { key: 'background', icon: <ImageIcon size={18} />, label: 'Background' },
  { key: 'text', icon: <Type size={18} />, label: 'Text' },
  { key: 'effects', icon: <Tv2 size={18} />, label: 'Effects' },
  { key: 'export', icon: <Film size={18} />, label: 'Export' },
];

export const ControlPanel: React.FC<ControlPanelProps> = ({ config, onChangeConfig }) => {
  const [activeTab, setActiveTab] = useState<TabType>('style');

  const updateConfig = (updater: (prev: VisualizerConfig) => VisualizerConfig) => {
    onChangeConfig(updater(config));
  };

  const tabProps = { config, updateConfig };

  return (
    <aside className="control-panel">
      <nav className="panel-tabs">
        {tabs.map((t) => (
          <button key={t.key} className={`tab-btn ${activeTab === t.key ? 'active' : ''}`}
            onClick={() => setActiveTab(t.key)} title={t.label}>
            {t.icon}
            <span>{t.label}</span>
          </button>
        ))}
      </nav>

      <div className="panel-content">
        {activeTab === 'style' && <StyleTab {...tabProps} />}
        {activeTab === 'colors' && <ColorsTab {...tabProps} />}
        {activeTab === 'background' && <BackgroundTab {...tabProps} />}
        {activeTab === 'text' && <TextTab {...tabProps} />}
        {activeTab === 'effects' && <EffectsTab {...tabProps} />}
        {activeTab === 'export' && <ExportTab {...tabProps} />}
      </div>

      <div className="panel-footer" style={{ padding: '10px 16px', borderTop: '1px solid rgba(255, 215, 0, 0.12)', fontSize: 11, color: 'var(--text-muted)', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <span>AudioWave Studio v1.9.0</span>
        <a href="https://crediblemark.com" target="_blank" rel="noreferrer" style={{ color: '#ffd700', textDecoration: 'none', fontWeight: 500 }}>
          crediblemark.com
        </a>
      </div>
    </aside>
  );
};
