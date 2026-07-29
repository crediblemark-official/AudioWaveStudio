import { RenderContext } from './types';

export function renderTextOverlay(ctx: RenderContext) {
  const { ctx: c, width, height, config } = ctx;
  const txt = config.text;
  if (!txt.showTitle && !txt.showArtist) return;

  c.save();
  c.textAlign = 'center';

  let x = (txt.textPositionX / 100) * width;
  let yTitle = (txt.textPositionY / 100) * height;
  let yArtist = yTitle + (txt.titleFontSize * 0.4) + txt.artistFontSize;

  if (txt.position === 'bottom-left') {
    c.textAlign = 'left';
  }

  if (txt.textShadow) {
    c.shadowBlur = 12;
    c.shadowColor = config.theme.glowColor;
  }

  const font = txt.fontFamily || '"Outfit", "Inter", sans-serif';

  if (txt.showTitle && txt.songTitle) {
    c.font = `700 ${txt.titleFontSize}px ${font}`;
    c.fillStyle = txt.titleColor;
    c.fillText(txt.songTitle, x, yTitle);
  }

  if (txt.showArtist && txt.artistName) {
    c.font = `500 ${txt.artistFontSize}px ${font}`;
    c.fillStyle = txt.artistColor;
    c.fillText(txt.artistName, x, yArtist);
  }

  c.restore();
}
