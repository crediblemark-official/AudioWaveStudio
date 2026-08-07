//! Pulse Rings style renderer (`pulseRings`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  theme_glow, theme_primary, theme_secondary, PulseRing, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let center_x = ctx.width / 2.0;
  let center_y = ctx.height / 2.0;
  let max_dim = ctx.width.max(ctx.height) * 0.8;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let state = &mut ctx.state;
  if bs > 0.15 && bs > state.prev_beat {
    let count = 1 + (bs * 2.0).floor() as u32;
    for i in 0..count {
      let color = if i % 2 == 0 { theme_primary(theme) } else { theme_secondary(theme) };
      state.rings.push(PulseRing {
        radius: 10.0 + i as f32 * 20.0,
        max_radius: max_dim * (0.5 + state.rng.next() * 0.5),
        alpha: 0.4 + be * 0.3,
        speed: 2.0 + bs * 3.0 + state.rng.next() * 2.0,
        thickness: 2.0 + be * 4.0 + bs * 3.0,
        color,
      });
    }
  }
  state.prev_beat = bs;

  for i in (0..state.rings.len()).rev() {
    let r = &mut state.rings[i];
    r.radius += r.speed;
    r.alpha *= 0.985;

    if r.radius > r.max_radius || r.alpha < 0.01 {
      state.rings.remove(i);
      continue;
    }

    let denom = 0.4 + be * 0.3;
    let lw = if denom > 0.0 { r.thickness * (r.alpha / denom) } else { r.thickness };
    c.save();
    c.set_global_alpha(r.alpha);
    c.set_stroke(Fill::Solid(r.color));
    c.set_line_width(lw);
    c.set_shadow(theme_glow(theme), 15.0);
    c.stroke_circle(center_x, center_y, r.radius);
    c.restore();
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
}
