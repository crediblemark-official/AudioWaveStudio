import React, { useState } from 'react';
import { TextBlock } from '../../types/visualizer';
import { CustomSelect } from '../CustomSelect';
import { FontPicker } from '../FontPicker';
import { ChevronDown, Trash2, ArrowUp, ArrowDown } from 'lucide-react';

interface Props {
  title: string;
  block: TextBlock;
  onChange: (patch: Partial<TextBlock>) => void;
  collapsible?: boolean;
  defaultOpen?: boolean;
  showText?: boolean;
  showEnabled?: boolean;
  onRemove?: () => void;
  onMoveUp?: () => void;
  onMoveDown?: () => void;
}

function Slider({ label, value, min, max, step = 1, onChange, display }: {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (v: number) => void;
  display?: string;
}) {
  return (
    <div className="control-group">
      <label className="label-row">
        <span>{label}</span>
        <span>{display ?? value}</span>
      </label>
      <input type="range" min={min} max={max} step={step} value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))} className="input-range" />
    </div>
  );
}

function Toggle({ label, checked, onChange }: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="checkbox-label mb-2">
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
      <span>{label}</span>
    </label>
  );
}

function ColorField({ label, value, onChange }: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <div className="picker-item">
      <span>{label}</span>
      <input type="color" value={value} onChange={(e) => onChange(e.target.value)} />
    </div>
  );
}

export const TextBlockEditor: React.FC<Props> = ({
  title,
  block,
  onChange,
  collapsible = false,
  defaultOpen = true,
  showText = false,
  showEnabled = false,
  onRemove,
  onMoveUp,
  onMoveDown,
}) => {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="text-block-card">
      <div className="text-block-header">
        {collapsible ? (
          <button type="button" className="text-block-title" onClick={() => setOpen(!open)}>
            <ChevronDown size={14} className={`text-block-chevron ${open ? 'open' : ''}`} />
            <span>{title}</span>
          </button>
        ) : (
          <span className="text-block-title-static">{title}</span>
        )}
        <div className="btn-group text-block-actions">
          {onMoveUp && <button type="button" className="btn-icon" onClick={onMoveUp} title="Move up"><ArrowUp size={14} /></button>}
          {onMoveDown && <button type="button" className="btn-icon" onClick={onMoveDown} title="Move down"><ArrowDown size={14} /></button>}
          {onRemove && <button type="button" className="btn-icon btn-icon-danger" onClick={onRemove} title="Remove block"><Trash2 size={14} /></button>}
        </div>
      </div>

      {(!collapsible || open) && (
        <div className="text-block-body">
          {showEnabled && (
            <div className="control-group">
              <Toggle label="Visible" checked={block.enabled} onChange={(v) => onChange({ enabled: v })} />
            </div>
          )}

          {showText && (
            <div className="control-group">
              <label className="label-row">Text (supports line breaks)</label>
              <textarea className="input-text input-textarea" rows={2}
                value={block.text}
                placeholder="Type your text..."
                onChange={(e) => onChange({ text: e.target.value })} />
            </div>
          )}

          <div className="text-block-subtitle">Layout</div>
          <Slider label="Horizontal X" value={block.positionX} min={0} max={100} onChange={(v) => onChange({ positionX: v })} display={`${block.positionX}%`} />
          <Slider label="Vertical Y" value={block.positionY} min={0} max={100} onChange={(v) => onChange({ positionY: v })} display={`${block.positionY}%`} />
          <div className="control-group">
            <label className="label-row">Alignment</label>
            <CustomSelect value={block.align}
              onChange={(v) => onChange({ align: v as TextBlock['align'] })}
              options={[
                { value: 'left', label: 'Left' },
                { value: 'center', label: 'Center' },
                { value: 'right', label: 'Right' },
              ]} />
          </div>
          <Slider label="Line Height" value={block.lineHeight} min={0.8} max={2.5} step={0.05} onChange={(v) => onChange({ lineHeight: v })} />
          <Slider label="Wrap Width" value={block.maxWidth} min={0} max={100} onChange={(v) => onChange({ maxWidth: v })} display={block.maxWidth === 0 ? 'Auto' : `${block.maxWidth}%`} />

          <div className="text-block-subtitle">Typography</div>
          <FontPicker value={block.fontFamily} onChange={(v) => onChange({ fontFamily: v })} emptyLabel="Default Font" />
          <Slider label="Font Size" value={block.fontSize} min={12} max={96} onChange={(v) => onChange({ fontSize: v })} display={`${block.fontSize}px`} />
          <Slider label="Font Weight" value={block.fontWeight} min={100} max={900} step={100} onChange={(v) => onChange({ fontWeight: v })} />
          <Toggle label="Italic" checked={block.italic} onChange={(v) => onChange({ italic: v })} />
          <Slider label="Letter Spacing" value={block.letterSpacing} min={0} max={24} onChange={(v) => onChange({ letterSpacing: v })} display={`${block.letterSpacing}px`} />
          <div className="control-group">
            <label className="label-row">Text Transform</label>
            <CustomSelect value={block.transform}
              onChange={(v) => onChange({ transform: v as TextBlock['transform'] })}
              options={[
                { value: 'none', label: 'None' },
                { value: 'uppercase', label: 'UPPERCASE' },
                { value: 'lowercase', label: 'lowercase' },
                { value: 'capitalize', label: 'Capitalize' },
              ]} />
          </div>
          <Slider label="Opacity" value={block.opacity} min={0} max={1} step={0.05} onChange={(v) => onChange({ opacity: v })} display={`${Math.round(block.opacity * 100)}%`} />

          <div className="text-block-subtitle">Fill</div>
          <div className="color-pickers">
            <ColorField label="Text Color" value={block.color} onChange={(v) => onChange({ color: v })} />
          </div>
          <Toggle label="Gradient Fill" checked={block.useGradient} onChange={(v) => onChange({ useGradient: v })} />
          {block.useGradient && (
            <>
              <div className="color-pickers">
                <ColorField label="Gradient Start" value={block.gradientStart} onChange={(v) => onChange({ gradientStart: v })} />
                <ColorField label="Gradient End" value={block.gradientEnd} onChange={(v) => onChange({ gradientEnd: v })} />
              </div>
              <Slider label="Gradient Angle" value={block.gradientAngle} min={0} max={360} onChange={(v) => onChange({ gradientAngle: v })} display={`${block.gradientAngle}°`} />
            </>
          )}

          <div className="text-block-subtitle">Effects</div>
          <Toggle label="Drop Shadow" checked={block.shadow} onChange={(v) => onChange({ shadow: v })} />
          {block.shadow && (
            <>
              <Slider label="Shadow Blur" value={block.shadowBlur} min={0} max={40} onChange={(v) => onChange({ shadowBlur: v })} />
              <Slider label="Shadow Offset X" value={block.shadowOffsetX} min={-30} max={30} onChange={(v) => onChange({ shadowOffsetX: v })} />
              <Slider label="Shadow Offset Y" value={block.shadowOffsetY} min={-30} max={30} onChange={(v) => onChange({ shadowOffsetY: v })} />
            </>
          )}
          <Slider label="Glow Intensity" value={block.glowIntensity} min={0} max={60} onChange={(v) => onChange({ glowIntensity: v })} display={block.glowIntensity === 0 ? 'Off' : `${block.glowIntensity}px`} />
          <Toggle label="Outline / Stroke" checked={block.outline} onChange={(v) => onChange({ outline: v })} />
          {block.outline && (
            <>
              <Slider label="Outline Width" value={block.outlineWidth} min={0.5} max={12} step={0.5} onChange={(v) => onChange({ outlineWidth: v })} display={`${block.outlineWidth}px`} />
              <div className="color-pickers">
                <ColorField label="Outline Color" value={block.outlineColor} onChange={(v) => onChange({ outlineColor: v })} />
              </div>
            </>
          )}

          <div className="text-block-subtitle">Animation</div>
          <Slider label="Bass Reaction" value={block.reactiveScale} min={0} max={1} step={0.05} onChange={(v) => onChange({ reactiveScale: v })} display={block.reactiveScale === 0 ? 'Off' : `${Math.round(block.reactiveScale * 100)}%`} />
          <Toggle label="Wave Effect" checked={block.waveEffect} onChange={(v) => onChange({ waveEffect: v })} />
          <Toggle label="Fade In on Play" checked={block.fadeIn} onChange={(v) => onChange({ fadeIn: v })} />
        </div>
      )}
    </div>
  );
};
