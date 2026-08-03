//! Aurora Wave style renderer (`auroraWave`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  theme_primary, theme_secondary, RenderContext,
};

struct AuroraLayer {
  offset: f32,
  speed: f32,
  freq: f32,
  amp: f32,
  color: Color,
  alpha: f32,
  y_base: f32,
}

fn with_alpha(color: Color, a: f32) -> Color {
  Color { r: color.r, g: color.g, b: color.b, a }
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let be = ctx.bass_energy;

  // TS increments the module-level `t` at the START of renderAuroraWave
  // (auroraWave.ts:18), before any sampling. Mirror that so the wave phase
  // (and its `>10000` wrap) lines up frame-for-frame with the preview.
  let next_t = ctx.state.aurora_t + 0.008;
  ctx.state.aurora_t = if next_t > 10000.0 { next_t - 10000.0 } else { next_t };
  let t = ctx.state.aurora_t;
  let bass_amp = 0.1 + be * 0.4;

  let primary = theme_primary(theme);
  let secondary = theme_secondary(theme);
  let accent = Color::hex(&theme.accent_color);

  let layers = [
    AuroraLayer { offset: 0.0, speed: 0.5, freq: 0.003, amp: 0.06, color: primary, alpha: 0.25, y_base: ctx.height * 0.3 },
    AuroraLayer { offset: 2.0, speed: 0.7, freq: 0.005, amp: 0.04, color: secondary, alpha: 0.2, y_base: ctx.height * 0.45 },
    AuroraLayer { offset: 4.0, speed: 0.3, freq: 0.002, amp: 0.08, color: accent, alpha: 0.15, y_base: ctx.height * 0.35 },
  ];

  for layer in &layers {
    let mut points: Vec<(f32, f32)> = Vec::new();
    let mut x = 0.0f32;
    while x <= ctx.width {
      // TS samples ONE bin: `freqData[freqIdx] || 0` (auroraWave.ts:35). No
      // window averaging — averaging 5 bins changed every layer's amp.
      let freq_idx = ((x / ctx.width) * ctx.freq_data.len() as f32).floor() as usize;
      let f_val = ctx.freq_data.get(freq_idx).copied().unwrap_or(0) as f32;

      let wave = (x * layer.freq + t * layer.speed + layer.offset).sin();
      let wave2 = (x * layer.freq * 2.3 + t * layer.speed * 1.7 + layer.offset + 1.5).sin();
      let amp = layer.amp * (1.0 + bass_amp * 2.0) * (1.0 + (f_val / 255.0) * 0.5);
      let y = layer.y_base + wave * amp * ctx.height + wave2 * amp * ctx.height * 0.5;
      points.push((x, y));
      x += 4.0;
    }

    // TS connects the sampled points with straight lineTo segments — the
    // fill polygon and the stroke use the SAME raw points (auroraWave.ts:43-47
    // and :64-74). No quadratic smoothing.
    let mut poly: Vec<(f32, f32)> = Vec::with_capacity(points.len() + 2);
    poly.push((0.0, ctx.height));
    poly.extend_from_slice(&points);
    poly.push((ctx.width, ctx.height));

    let alpha_hex = (layer.alpha * 255.0).round() as u32;
    let grad = Fill::linear_gradient(0.0, 0.0, ctx.width, 0.0, &[
      (0.0, with_alpha(layer.color, 0.0)),
      (0.3, with_alpha(layer.color, alpha_hex as f32 / 255.0)),
      (0.7, with_alpha(layer.color, alpha_hex as f32 / 255.0)),
      (1.0, with_alpha(layer.color, 0.0)),
    ]);
    c.save();
    c.set_global_alpha(layer.alpha * (0.5 + be * 0.5));
    c.set_fill(grad);
    c.fill_polygon(&poly);
    c.restore();

    c.save();
    c.set_stroke(Fill::Solid(layer.color));
    c.set_line_width(2.0);
    c.set_global_alpha(layer.alpha * (0.8 + be * 0.2));
    c.set_shadow(layer.color, 20.0);
    c.stroke_polyline(&points);
    c.restore();
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
}
