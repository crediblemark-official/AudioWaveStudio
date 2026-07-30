import { RenderContext } from './types';

export function renderBackground(ctx: RenderContext) {
  const { ctx: c, width, height, config, customImgElement } = ctx;
  const bg = config.background;

  if (bg.mode === 'customImage' && customImgElement) {
    const blur = bg.blurAmount || 0;
    if (blur > 0) c.filter = `blur(${blur}px)`;
    c.drawImage(customImgElement, 0, 0, width, height);
    if (blur > 0) c.filter = 'none';
  } else if (bg.mode === 'gradient') {
    const grad = c.createLinearGradient(0, 0, width, height);
    grad.addColorStop(0, bg.gradientStart);
    grad.addColorStop(1, bg.gradientEnd);
    c.fillStyle = grad;
    c.fillRect(0, 0, width, height);
  } else if (bg.mode === 'grid') {
    c.fillStyle = bg.solidColor;
    c.fillRect(0, 0, width, height);
    const gridColor = bg.gridColor || '#ffffff';
    const gridSize = bg.gridSize || 40;
    const lineWidth = bg.gridLineWidth || 1;
    c.strokeStyle = gridColor;
    c.lineWidth = lineWidth;
    c.globalAlpha = 0.12;
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
    c.globalAlpha = 1;
  } else {
    c.fillStyle = bg.solidColor;
    c.fillRect(0, 0, width, height);
  }

}
