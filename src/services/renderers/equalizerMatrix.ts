import { RenderContext } from './types';

export function renderEqualizerMatrix(ctx: RenderContext) {
  const { ctx: c, width, height, config, freqData } = ctx;
  const cols = Math.min(48, config.reactivity.barCount);
  const rows = 18;
  const availableW = width * 0.8;
  const blockW = availableW / cols - 4;
  const blockH = (height * 0.35) / rows - 3;
  const startX = (width - availableW) / 2;
  const startY = height * 0.6;

  const theme = config.theme;
  const step = Math.max(1, Math.floor(freqData.length / cols));

  for (let col = 0; col < cols; col++) {
    let val = 0;
    for (let j = 0; j < step; j++) {
      val += freqData[col * step + j] || 0;
    }
    val = (val / step / 255) * config.reactivity.sensitivity;
    const activeRows = Math.floor(val * rows);

    for (let r = 0; r < rows; r++) {
      const bx = startX + col * (blockW + 4);
      const by = startY - r * (blockH + 3);

      const isActive = r < activeRows;

      c.save();
      if (isActive) {
        c.shadowBlur = 8;
        c.shadowColor = theme.glowColor;
        c.fillStyle = r > rows * 0.8 ? theme.accentColor : r > rows * 0.5 ? theme.primaryColor : theme.secondaryColor;
      } else {
        c.globalAlpha = 0.12;
        c.fillStyle = '#ffffff';
      }

      c.fillRect(bx, by, blockW, blockH);
      c.restore();
    }
  }
}
