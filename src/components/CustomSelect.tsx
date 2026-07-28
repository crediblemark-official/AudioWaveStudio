import React, { useState, useRef, useEffect } from 'react';

interface SelectOption {
  value: string;
  label: string;
}

interface CustomSelectProps {
  value: string;
  onChange: (value: string) => void;
  options: SelectOption[];
  className?: string;
}

export const CustomSelect: React.FC<CustomSelectProps> = ({ value, onChange, options, className = '' }) => {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const selected = options.find((o) => o.value === value);
  const handleSelect = (val: string) => {
    onChange(val);
    setOpen(false);
  };

  return (
    <div className={`custom-select ${className}`} style={{ position: 'relative', width: '100%' }} ref={ref}>
      <button
        className="custom-select-trigger"
        style={{
          width: '100%',
          padding: '8px 12px 8px 12px',
          paddingRight: '30px',
          background: '#10121b',
          border: '1px solid rgba(255,255,255,0.08)',
          borderRadius: '6px',
          color: '#e0e0e0',
          fontSize: '0.85rem',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          outline: 'none',
          fontFamily: 'inherit',
        }}
        onClick={() => setOpen(!open)}
      >
        <span>{selected ? selected.label : 'Select...'}</span>
        <svg className="custom-select-arrow" width="12" height="12" viewBox="0 0 16 16" fill="#888">
          <path d="M8 11L3 6h10z" />
        </svg>
      </button>
      {open && (
        <div className="custom-select-options">
          {options.map((opt) => (
            <button
              key={opt.value}
              className={`custom-select-option ${opt.value === value ? 'selected' : ''}`}
              onClick={() => handleSelect(opt.value)}
            >
              {opt.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
