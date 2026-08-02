import { RenderContext } from './types';

export function renderOscilloscopeVisualizer(ctx: RenderContext) {
  const { ctx: c, width, height, config, timeData } = ctx;
  const centerY = height * 0.52;
  const theme = config.theme;
  const len = timeData.length;
  const sliceWidth = width / (len - 1);

  const passes = [
    { alpha: 0.2, blur: 25, width: 8, color: theme.glowColor },
    { alpha: 0.6, blur: 15, width: 4, color: theme.secondaryColor },
    { alpha: 1.0, blur: 6, width: 2, color: theme.primaryColor }
  ];

  for (const pass of passes) {
    c.save();
    c.globalAlpha = pass.alpha;
    c.shadowBlur = pass.blur;
    c.shadowColor = pass.color;
    c.strokeStyle = pass.color;
    c.lineWidth = pass.width;
    c.beginPath();

    let x = 0;
    for (let i = 0; i < len; i++) {
      const v = (timeData[i] / 128.0) - 1.0;
      const y = centerY + v * (height * 0.3) * config.reactivity.sensitivity;
      if (i === 0) c.moveTo(x, y);
      else c.lineTo(x, y);
      x += sliceWidth;
    }
    c.stroke();
    c.restore();
  }
}
