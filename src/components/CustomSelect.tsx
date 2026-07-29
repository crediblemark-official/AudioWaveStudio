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
  icon?: React.ReactNode;
}

export const CustomSelect: React.FC<CustomSelectProps> = ({ value, onChange, options, className = '', icon }) => {
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
    <div className={`custom-select ${className}`} ref={ref}>
      <button
        type="button"
        className="custom-select-trigger"
        onClick={() => setOpen(!open)}
      >
        <div className="custom-select-label-wrapper">
          {icon && <span className="custom-select-icon">{icon}</span>}
          <span className="custom-select-label-text">{selected ? selected.label : 'Select...'}</span>
        </div>
        <svg className={`custom-select-arrow ${open ? 'open' : ''}`} width="12" height="12" viewBox="0 0 16 16" fill="#888">
          <path d="M8 11L3 6h10z" />
        </svg>
      </button>
      {open && (
        <div className="custom-select-options">
          {options.map((opt) => (
            <button
              key={opt.value}
              type="button"
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

