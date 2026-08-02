import React, { useEffect, useRef, useState } from 'react';
import { VisualizerConfig } from '../../types/visualizer';
import { CustomSelect } from '../CustomSelect';
import { TextBlockEditor } from './TextBlockEditor';
import { createTextBlock } from '../../utils/presets';
import { Plus } from 'lucide-react';

interface Props {
  config: VisualizerConfig;
  updateConfig: (updater: (prev: VisualizerConfig) => VisualizerConfig) => void;
}

function TextPreview({ config }: { config: VisualizerConfig }) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const c = canvas.getContext('2d');
    if (!c) return;
    c.fillStyle = '#0b0c10';
    c.fillRect(0, 0, canvas.width, canvas.height);

    c.textAlign = 'center';
    c.fillStyle = '#ffffff';
    c.font = `600 ${config.text.title.fontSize || 24}px ${config.text.fontFamily || 'sans-serif'}`;
    if (config.text.showTitle && config.text.songTitle) {
      c.fillText(config.text.songTitle, canvas.width / 2, canvas.height / 2);
    }
    if (config.text.showArtist && config.text.artistName) {
      c.font = `400 ${config.text.artist.fontSize || 16}px ${config.text.fontFamily || 'sans-serif'}`;
      c.fillText(config.text.artistName, canvas.width / 2, canvas.height / 2 + 30);
    }
  }, [config]);

  return (
    <div className="text-preview">
      <label className="label-row"><span>Live Preview</span></label>
      <canvas ref={ref} width={640} height={360} className="text-preview-canvas" />
    </div>
  );
}

const POSITION_PRESETS: Record<string, { title: { x: number; y: number; align: 'left' | 'center' | 'right' }; artist: { x: number; y: number; align: 'left' | 'center' | 'right' } }> = {
  'bottom-center': { title: { x: 50, y: 78, align: 'center' }, artist: { x: 50, y: 86, align: 'center' } },
  'top-center': { title: { x: 50, y: 12, align: 'center' }, artist: { x: 50, y: 20, align: 'center' } },
  'center': { title: { x: 50, y: 45, align: 'center' }, artist: { x: 50, y: 55, align: 'center' } },
  'bottom-left': { title: { x: 8, y: 78, align: 'left' }, artist: { x: 8, y: 86, align: 'left' } },
  'left-middle': { title: { x: 8, y: 45, align: 'left' }, artist: { x: 8, y: 55, align: 'left' } },
  'right-middle': { title: { x: 92, y: 45, align: 'right' }, artist: { x: 92, y: 55, align: 'right' } },
};

export const TextTab: React.FC<Props> = ({ config, updateConfig }) => {
  const text = config.text;
  const [positionPreset, setPositionPreset] = useState('bottom-center');

  const handleTextChange = (key: keyof typeof text, value: unknown) => {
    updateConfig((prev) => ({ ...prev, text: { ...prev.text, [key]: value } }));
  };

  const updateTitleBlock = (patch: Partial<typeof text.title>) => {
    updateConfig((prev) => ({ ...prev, text: { ...prev.text, title: { ...prev.text.title, ...patch } } }));
  };

  const updateArtistBlock = (patch: Partial<typeof text.artist>) => {
    updateConfig((prev) => ({ ...prev, text: { ...prev.text, artist: { ...prev.text.artist, ...patch } } }));
  };

  const updateCustomBlock = (id: string, patch: Partial<typeof text.blocks[number]>) => {
    updateConfig((prev) => ({
      ...prev,
      text: {
        ...prev.text,
        blocks: prev.text.blocks.map((b) => (b.id === id ? { ...b, ...patch } : b)),
      },
    }));
  };

  const addBlock = () => {
    const block = createTextBlock();
    updateConfig((prev) => ({
      ...prev,
      text: { ...prev.text, blocks: [...prev.text.blocks, block] },
    }));
  };

  const removeBlock = (id: string) => {
    updateConfig((prev) => ({
      ...prev,
      text: { ...prev.text, blocks: prev.text.blocks.filter((b) => b.id !== id) },
    }));
  };

  const moveBlock = (id: string, dir: -1 | 1) => {
    updateConfig((prev) => {
      const blocks = [...prev.text.blocks];
      const idx = blocks.findIndex((b) => b.id === id);
      const target = idx + dir;
      if (idx < 0 || target < 0 || target >= blocks.length) return prev;
      [blocks[idx], blocks[target]] = [blocks[target], blocks[idx]];
      return { ...prev, text: { ...prev.text, blocks } };
    });
  };

  return (
    <div className="tab-pane">
      <h3 className="section-title">Song Metadata & Typography</h3>

      <div className="control-group">
        <label className="checkbox-label mb-2">
          <input type="checkbox" checked={text.showTitle}
            onChange={(e) => handleTextChange('showTitle', e.target.checked)} />
          <span>Display Song Title</span>
        </label>
        <input type="text" value={text.songTitle}
          onChange={(e) => updateConfig((prev) => ({
            ...prev,
            text: {
              ...prev.text,
              songTitle: e.target.value,
              title: { ...prev.text.title, text: e.target.value },
            },
          }))}
          placeholder="Song Title" className="input-text" />
      </div>

      <div className="control-group">
        <label className="checkbox-label mb-2">
          <input type="checkbox" checked={text.showArtist}
            onChange={(e) => handleTextChange('showArtist', e.target.checked)} />
          <span>Display Artist Name</span>
        </label>
        <input type="text" value={text.artistName}
          onChange={(e) => updateConfig((prev) => ({
            ...prev,
            text: {
              ...prev.text,
              artistName: e.target.value,
              artist: { ...prev.text.artist, text: e.target.value },
            },
          }))}
          placeholder="Artist Name" className="input-text" />
      </div>

      <hr className="section-divider" />

      <TextPreview config={config} />

      <hr className="section-divider" />

      <div className="control-group">
        <label className="label-row">Quick Position Preset</label>
        <CustomSelect value={positionPreset}
          onChange={(v) => {
            setPositionPreset(v);
            const p = POSITION_PRESETS[v];
            if (!p) return;
            updateConfig((prev) => ({
              ...prev,
              text: {
                ...prev.text,
                title: { ...prev.text.title, positionX: p.title.x, positionY: p.title.y, align: p.title.align },
                artist: { ...prev.text.artist, positionX: p.artist.x, positionY: p.artist.y, align: p.artist.align },
              },
            }));
          }}
          options={Object.keys(POSITION_PRESETS).map((value) => ({
            value,
            label: value.charAt(0).toUpperCase() + value.slice(1).replace('-', ' '),
          }))} />
        <span className="hint-text">Applies to title &amp; artist. Fine-tune each block below.</span>
      </div>

      <hr className="section-divider" />

      <h3 className="section-title">Title Style</h3>
      <TextBlockEditor title="Song Title" block={text.title} onChange={updateTitleBlock} collapsible defaultOpen={false} />

      <h3 className="section-title">Artist Style</h3>
      <TextBlockEditor title="Artist Name" block={text.artist} onChange={updateArtistBlock} collapsible defaultOpen={false} />

      <hr className="section-divider" />

      <div className="section-header-row">
        <h3 className="section-title">Additional Text Blocks</h3>
      </div>
      {text.blocks.map((block, idx) => (
        <TextBlockEditor
          key={block.id}
          title={block.text.trim() ? block.text.slice(0, 24) : 'Untitled Block'}
          block={block}
          onChange={(patch) => updateCustomBlock(block.id, patch)}
          collapsible
          defaultOpen={idx === text.blocks.length - 1}
          showText
          showEnabled
          onRemove={() => removeBlock(block.id)}
          onMoveUp={idx > 0 ? () => moveBlock(block.id, -1) : undefined}
          onMoveDown={idx < text.blocks.length - 1 ? () => moveBlock(block.id, 1) : undefined}
        />
      ))}
      <button type="button" className="btn-add" onClick={addBlock}>
        <Plus size={14} /> Add Text Block
      </button>
    </div>
  );
};
