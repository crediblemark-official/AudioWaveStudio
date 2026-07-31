import { describe, it, expect } from 'vitest';
import { createTextBlock, migrateTextSettings } from './presets';
import { normalizeFont } from './fonts';

describe('createTextBlock', () => {
  it('fills all defaults', () => {
    const block = createTextBlock();
    expect(block.fontSize).toBe(24);
    expect(block.fontWeight).toBe(700);
    expect(block.fontFamily).toBe('');
    expect(block.useGradient).toBe(false);
    expect(block.opacity).toBe(1);
    expect(block.reactiveScale).toBe(0);
    expect(block.waveEffect).toBe(false);
    expect(typeof block.id).toBe('string');
    expect(block.id.length).toBeGreaterThan(0);
  });

  it('merges overrides', () => {
    const block = createTextBlock({ id: 'title', text: 'Hello', fontSize: 48 });
    expect(block.id).toBe('title');
    expect(block.text).toBe('Hello');
    expect(block.fontSize).toBe(48);
    expect(block.color).toBe('#ffffff');
  });
});

describe('migrateTextSettings', () => {
  it('migrates legacy text settings', () => {
    const legacy = {
      songTitle: 'Old Song',
      artistName: 'Old Artist',
      showTitle: true,
      showArtist: false,
      titleColor: '#ff0000',
      artistColor: '#00ff00',
      titleFontSize: 32,
      artistFontSize: 20,
      fontFamily: 'Arial',
      position: 'center',
      textPositionX: 20,
      textPositionY: 40,
      textShadow: true,
    } as unknown as Parameters<typeof migrateTextSettings>[0];

    const result = migrateTextSettings(legacy);

    expect(result.songTitle).toBe('Old Song');
    expect(result.title.id).toBe('title');
    expect(result.title.text).toBe('Old Song');
    expect(result.title.fontSize).toBe(32);
    expect(result.title.color).toBe('#ff0000');
    expect(result.title.positionX).toBe(20);
    expect(result.title.positionY).toBe(40);
    expect(result.title.shadow).toBe(true);
    expect(result.artist.id).toBe('artist');
    expect(result.artist.fontSize).toBe(20);
    expect(result.artist.color).toBe('#00ff00');
    expect(result.artist.positionX).toBe(20);
    expect(result.blocks).toEqual([]);
  });

  it('returns full defaults when text is undefined', () => {
    const result = migrateTextSettings(undefined);
    expect(result.title.text).toBe('Electrifying Night');
    expect(result.artist.text).toBe('Synthwave Producer');
    expect(result.blocks).toEqual([]);
  });

  it('normalizes partial blocks', () => {
    const result = migrateTextSettings({
      songTitle: 'New',
      artistName: 'Artist',
      showTitle: true,
      showArtist: true,
      title: { id: 'title', text: 'New' } as never,
    });
    expect(result.title.text).toBe('New');
    expect(result.title.fontSize).toBe(28);
    expect(result.artist.fontSize).toBe(16);
  });
});

describe('normalizeFont', () => {
  it('maps known labels to css stacks', () => {
    expect(normalizeFont('outfit')).toBe('"Outfit", "Inter", sans-serif');
    expect(normalizeFont('Inter')).toBe('"Inter", sans-serif');
  });

  it('handles custom font names', () => {
    expect(normalizeFont('Helvetica Neue')).toBe('"Helvetica Neue", sans-serif');
    expect(normalizeFont('monospace')).toBe('monospace');
    expect(normalizeFont('')).toBe('');
  });
});
