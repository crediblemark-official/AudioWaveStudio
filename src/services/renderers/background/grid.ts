import { RenderContext } from '../types';

export function renderGridEffect(ctx: RenderContext) {
  const { ctx: c, width, height, config } = ctx;
  const bg = config.background;
  const gridColor = bg.gridColor || '#ffffff';
  const gridSize = bg.gridSize || 40;
  const lineWidth = bg.gridLineWidth || 1;

  c.save();
  c.strokeStyle = gridColor;
  c.lineWidth = lineWidth;
  c.globalAlpha = 0.25;
  for (let x = 0; x <= width; x += gridSize) {
    c.beginPath();
    c.moveTo(x, 0);
    c.lineTo(x, height);
    c.stroke();
  }
  for (let y = 0; y <= height; y += gridSize) {
    c.beginPath();
    c.moveTo(0, y);
    c.lineTo(width, y);
    c.stroke();
  }
  c.restore();
}
