import React, { useEffect, useRef, useState } from 'react';
import { FONT_OPTIONS, fontLabel, normalizeFont } from '../utils/fonts';

interface Props {
  value: string; // CSS font-family stack; '' = inherit
  onChange: (value: string) => void;
  emptyLabel?: string;
}

function displayLabel(value: string, emptyLabel: string): string {
  if (!value) return emptyLabel;
  const known = fontLabel(value);
  if (known) return known;
  const cleaned = value.replace(/^["']/, '').replace(/["']\s*,.*$/, '').trim();
  return cleaned || value;
}

export const FontPicker: React.FC<Props> = ({ value, onChange, emptyLabel = 'Default Font' }) => {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const ref = useRef<HTMLDivElement>(null);

  const label = displayLabel(value, emptyLabel);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const q = query.trim().toLowerCase();
  const filtered = q
    ? FONT_OPTIONS.filter((f) => f.label.toLowerCase().includes(q))
    : FONT_OPTIONS;
  const exactMatch = q.length > 0 && filtered.some((f) => f.label.toLowerCase() === q);
  const hasCustom = q.length > 0 && !exactMatch;

  const commit = (v: string) => {
    onChange(v);
    setOpen(false);
    setQuery('');
  };

  return (
    <div className="control-group">
      <label className="label-row"><span>Font Family</span></label>
      <div className="font-picker" ref={ref}>
        <button
          type="button"
          className="custom-select-trigger"
          onClick={() => {
            setOpen((o) => !o);
            setQuery('');
          }}
        >
          <span className="custom-select-label-text" style={{ fontFamily: value || 'inherit' }}>{label}</span>
          <svg className={`custom-select-arrow ${open ? 'open' : ''}`} width="12" height="12" viewBox="0 0 16 16" fill="#888">
            <path d="M8 11L3 6h10z" />
          </svg>
        </button>

        {open && (
          <div className="custom-select-options font-picker-options">
            <div className="font-picker-search-wrap">
              <input
                type="text"
                className="input-text font-picker-search"
                autoFocus
                value={query}
                placeholder="Search or type a custom font..."
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Escape') setOpen(false);
                  if (e.key === 'Enter' && hasCustom) commit(normalizeFont(query));
                }}
              />
            </div>
            <div className="font-picker-list">
              <button
                type="button"
                className="custom-select-option"
                onClick={() => commit('')}
              >
                {emptyLabel} (inherit)
              </button>
              {filtered.map((f) => (
                <button
                  key={f.value}
                  type="button"
                  className={`custom-select-option ${f.value === value ? 'selected' : ''}`}
                  style={{ fontFamily: f.value }}
                  onClick={() => commit(f.value)}
                >
                  {f.label}
                </button>
              ))}
              {hasCustom && (
                <button
                  type="button"
                  className="custom-select-option font-picker-custom"
                  onClick={() => commit(normalizeFont(query))}
                >
                  Use custom font: <em>{query}</em>
                </button>
              )}
              {!hasCustom && filtered.length === 0 && (
                <button
                  type="button"
                  className="custom-select-option font-picker-custom"
                  onClick={() => commit(normalizeFont(query))}
                >
                  Use custom font: <em>{query}</em>
                </button>
              )}
            </div>
          </div>
        )}
      </div>
      <span className="font-preview" style={{ fontFamily: value || '"Outfit", "Inter", sans-serif' }}>
        {value ? `${label}` : 'Default'}: AaBbCc 123
      </span>
      {!value && <span className="hint-text">{emptyLabel} — uses the app&apos;s default font (Outfit).</span>}
    </div>
  );
};
