//! Spiral Galaxy style renderer (`spiralGalaxy`).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{mix, GalaxyParticle};
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let glow = crate::renderers::theme_glow(theme);
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  let cx = width / 2.0;
  let cy = height / 2.0;
  let max_radius = width.min(height) * 0.42;

  if st.galaxy.is_empty() {
    let arms = 3;
    let particles_per_arm = 80;
    for arm in 0..arms {
      for i in 0..particles_per_arm {
        let r = (i as f32 / particles_per_arm as f32).powf(0.7);
        let angle = r * TAU * 1.8 + arm as f32 * (TAU / arms as f32);
        let spread = (rng.next() - 0.5) * 0.35 * (1.0 - r * 0.5);
        st.galaxy.push(GalaxyParticle {
          arm,
          radius: r,
          angle: angle + spread,
          speed: 0.003 + (1.0 - r) * 0.008,
          size: 1.0 + (1.0 - r) * 3.5 + rng.next() * 1.5,
          offset: (rng.next() - 0.5) * 20.0,
        });
      }
    }
  }

  st.galaxy_rotation += 0.005 + be * 0.015 + bs * 0.01;
  let rot = st.galaxy_rotation;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let core_r = max_radius * 0.15 * (1.0 + be * 0.3);
  let core_grad = Fill::radial_gradient(cx, cy, 0.0, cx, cy, core_r * 2.0, &[
    (0.0, Color::WHITE.with_alpha(0.9)),
    (0.3, p.with_alpha(0.7)),
    (0.7, s.with_alpha(0.3)),
    (1.0, Color::TRANSPARENT),
  ]);
  c.set_fill(core_grad);
  c.set_shadow(glow, 25.0 * (1.0 + be));
  c.fill_circle(cx, cy, core_r * 2.0);

  let freq_count = ctx.freq_data.len();
  let step = (freq_count / st.galaxy.len().max(1)).max(1);

  let spiral_offset = be * 25.0 * sensitivity;
  let glow_intensity = 0.4 + be * 0.6;

  for (idx, gp) in st.galaxy.iter_mut().enumerate() {
    let freq_val = *ctx.freq_data.get((idx * step) % freq_count).unwrap_or(&0) as f32 / 255.0;
    let boost = freq_val * sensitivity * 0.4;
    gp.angle += gp.speed + boost * 0.01;

    let dist = gp.radius * max_radius * (1.0 + be * 0.12);
    let a = rot + gp.angle + gp.arm as f32 * 2.1 + gp.radius * 3.0;
    let x = cx + a.cos() * (dist + (gp.angle * 3.0 + gp.arm as f32).sin() * spiral_offset);
    let y = cy + a.sin() * (dist + (gp.angle * 3.0 + gp.arm as f32).cos() * spiral_offset);
    let alpha: f32 = (0.3 + gp.radius * 0.4) * (0.5 + be * 0.5);
    let size = gp.size * (1.0 + be * 0.5);
    let col = mix(p, s, gp.radius);
    c.set_global_alpha(alpha.clamp(0.0, 1.0));
    c.set_fill(Fill::Solid(col));
    c.set_shadow(glow, size * 3.0 * glow_intensity);
    c.fill_circle(x, y, size);
  }

  c.set_fill(Fill::Solid(Color::WHITE));
  c.set_shadow(glow, 20.0 * glow_intensity);
  c.set_global_alpha((0.8f32 + be * 0.2).clamp(0.0, 1.0));
  c.fill_circle(cx, cy, 2.0 + be * 4.0);

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
