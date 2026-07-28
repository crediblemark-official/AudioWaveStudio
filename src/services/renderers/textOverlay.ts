import { RenderContext } from './types';

export function renderTextOverlay(ctx: RenderContext) {
  const { ctx: c, width, height, config } = ctx;
  const txt = config.text;
  if (!txt.showTitle && !txt.showArtist) return;

  c.save();
  c.textAlign = 'center';

  let x = width / 2;
  let yTitle = height * 0.82;
  let yArtist = height * 0.87;

  if (txt.position === 'top-center') {
    yTitle = height * 0.12;
    yArtist = height * 0.18;
  } else if (txt.position === 'center') {
    yTitle = height * 0.46;
    yArtist = height * 0.53;
  } else if (txt.position === 'bottom-left') {
    c.textAlign = 'left';
    x = width * 0.08;
    yTitle = height * 0.82;
    yArtist = height * 0.88;
  }

  if (txt.textShadow) {
    c.shadowBlur = 12;
    c.shadowColor = config.theme.glowColor;
  }

  if (txt.showTitle && txt.songTitle) {
    c.font = `700 ${txt.titleFontSize}px "Outfit", "Inter", sans-serif`;
    c.fillStyle = txt.titleColor;
    c.fillText(txt.songTitle, x, yTitle);
  }

  if (txt.showArtist && txt.artistName) {
    c.font = `500 ${txt.artistFontSize}px "Inter", sans-serif`;
    c.fillStyle = txt.artistColor;
    c.fillText(txt.artistName, x, yArtist);
  }

  c.restore();
}
