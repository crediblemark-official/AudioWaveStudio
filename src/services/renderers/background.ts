import { RenderContext } from './types';
import {
  renderAurora,
  renderNoise,
  renderBokeh,
  renderStarfield,
  renderNebula,
  renderPsychedelic,
  renderGridEffect,
} from './background/index';

export * from './background/index';

function drawCoverImage(
  c: CanvasRenderingContext2D,
  img: HTMLImageElement,
  width: number,
  height: number,
  blur = 0,
  alpha = 1.0,
  margin = 0
): boolean {
  if (!img || !img.complete || img.naturalWidth === 0) return false;
  const imgRatio = img.naturalWidth / img.naturalHeight;
  const canvasRatio = width / height;
  let renderWidth = width;
  let renderHeight = height;
  let offsetX = 0;
  let offsetY = 0;

  if (imgRatio > canvasRatio) {
    renderWidth = height * imgRatio;
    offsetX = (width - renderWidth) / 2;
  } else {
    renderHeight = width / imgRatio;
    offsetY = (height - renderHeight) / 2;
  }

  const pad = margin + (blur > 0 ? blur * 2 : 0);

  c.save();
  if (alpha < 1) c.globalAlpha = Math.max(0, Math.min(1, alpha));
  if (blur > 0) c.filter = `blur(${Math.round(blur)}px)`;
  c.drawImage(img, offsetX - pad, offsetY - pad, renderWidth + pad * 2, renderHeight + pad * 2);
  c.filter = 'none';
  c.restore();
  return true;
}

export function renderBackground(ctx: RenderContext, shakeMargin = 0) {
  const { ctx: c, width, height, config, customImgElement } = ctx;
  const bg = config.background;
  const blur = bg.blurAmount || 0;

  // Determine fill type (gradient vs solid)
  const fillType = bg.fillType ?? (bg.mode === 'gradient' ? 'gradient' : 'solid');

  // Determine overlay effect
  let effect = bg.effect;
  if (!effect) {
    if (['grid', 'aurora', 'noise', 'bokeh', 'starfield', 'nebula', 'psychedelic'].includes(bg.mode)) {
      effect = bg.mode as any;
    } else {
      effect = 'none';
    }
  }

  // 1. Render Base Background Layer (Solid Color or Gradient)
  if (fillType === 'gradient') {
    const grad = c.createLinearGradient(0, 0, width, height);
    grad.addColorStop(0, bg.gradientStart || '#0f0c20');
    grad.addColorStop(1, bg.gradientEnd || '#06101e');
    c.fillStyle = grad;
    c.fillRect(-shakeMargin, -shakeMargin, width + shakeMargin * 2, height + shakeMargin * 2);
  } else {
    c.fillStyle = bg.solidColor || '#0b0c10';
    c.fillRect(-shakeMargin, -shakeMargin, width + shakeMargin * 2, height + shakeMargin * 2);
  }

  // 2. Render Custom Background Image Layer (if present)
  if (customImgElement) {
    const defaultOpacity = bg.mode === 'customImage' ? 1.0 : 0.7;
    const imageOpacity = bg.imageOpacity ?? defaultOpacity;
    drawCoverImage(c, customImgElement, width, height, blur, imageOpacity, shakeMargin);
  }

  // 3. Render Overlay Visual Effect Layers (Multi-Select Stacking Support)
  let activeEffects: string[] = bg.effects && bg.effects.length > 0
    ? bg.effects
    : (bg.effect && bg.effect !== 'none' ? [bg.effect] : []);

  if (activeEffects.length === 0 && ['grid', 'aurora', 'noise', 'bokeh', 'starfield', 'nebula', 'psychedelic', 'particles', 'musicNotes'].includes(bg.mode)) {
    activeEffects = [bg.mode];
  }

  for (const eff of activeEffects) {
    if (eff === 'starfield') renderStarfield(ctx);
    else if (eff === 'nebula') renderNebula(ctx);
    else if (eff === 'psychedelic') renderPsychedelic(ctx);
    else if (eff === 'aurora') renderAurora(ctx);
    else if (eff === 'noise') renderNoise(ctx);
    else if (eff === 'bokeh') renderBokeh(ctx);
    else if (eff === 'grid') renderGridEffect(ctx);
  }
}




