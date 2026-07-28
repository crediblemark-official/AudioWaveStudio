import { RenderContext } from './types';

export function renderBackground(ctx: RenderContext) {
  const { ctx: c, width, height, config, bassEnergy, customImgElement } = ctx;
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
  } else {
    c.fillStyle = bg.solidColor;
    c.fillRect(0, 0, width, height);
  }

  const overlay = bg.overlayOpacity || 0;
  if (overlay > 0) {
    c.fillStyle = `rgba(10, 10, 15, ${overlay})`;
    c.fillRect(0, 0, width, height);
  }

  if (bassEnergy > 0.05) {
    c.fillStyle = `rgba(255,255,255,${Math.min(bassEnergy * 0.35, 0.35)})`;
    c.beginPath();
    c.arc(width / 2, height / 2, Math.max(width, height) * 0.45, 0, Math.PI * 2);
    c.fill();
  }
}
