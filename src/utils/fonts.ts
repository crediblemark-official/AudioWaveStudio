export interface FontOption {
  value: string; // CSS font-family stack
  label: string;
}

export const FONT_OPTIONS: FontOption[] = [
  { value: '"Outfit", "Inter", sans-serif', label: 'Outfit' },
  { value: '"Inter", sans-serif', label: 'Inter' },
  { value: '"Montserrat", sans-serif', label: 'Montserrat' },
  { value: '"Poppins", sans-serif', label: 'Poppins' },
  { value: '"Space Grotesk", sans-serif', label: 'Space Grotesk' },
  { value: '"Oswald", sans-serif', label: 'Oswald' },
  { value: '"Bebas Neue", sans-serif', label: 'Bebas Neue' },
  { value: '"Righteous", sans-serif', label: 'Righteous' },
  { value: '"Orbitron", sans-serif', label: 'Orbitron' },
  { value: '"Playfair Display", serif', label: 'Playfair Display' },
  { value: '"Cinzel", serif', label: 'Cinzel' },
  { value: '"Merriweather", serif', label: 'Merriweather' },
  { value: '"Pacifico", cursive', label: 'Pacifico' },
  { value: '"Dancing Script", cursive', label: 'Dancing Script' },
  { value: '"Caveat", cursive', label: 'Caveat' },
  { value: '"JetBrains Mono", monospace', label: 'JetBrains Mono' },
  { value: 'Georgia, serif', label: 'Georgia' },
  { value: 'system-ui, sans-serif', label: 'System UI' },
];

export function fontLabel(value: string): string | null {
  const match = FONT_OPTIONS.find((f) => f.value === value);
  return match ? match.label : null;
}

export function normalizeFont(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) return '';
  const match = FONT_OPTIONS.find((f) => f.label.toLowerCase() === trimmed.toLowerCase());
  if (match) return match.value;
  if (/^["']/.test(trimmed)) return trimmed;
  if (/\s/.test(trimmed)) return `"${trimmed}", sans-serif`;
  return trimmed;
}
