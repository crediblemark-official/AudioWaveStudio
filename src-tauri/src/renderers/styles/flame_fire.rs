//! Flame Fire style renderer (`flameFire`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::advanced::{mix, FireParticle};
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let sensitivity = ctx.config.reactivity.sensitivity;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let glow = crate::renderers::theme_glow(theme);
  let be: f32 = ctx.bass_energy;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  if !st.fire.is_empty() || be > 0.02 {
    let spawn = (2.0f32 + be * 8.0 * sensitivity).floor() as usize;
    for _ in 0..spawn {
      st.fire.push(FireParticle {
        x: rng.next() * width,
        y: height + 10.0,
        vx: (rng.next() - 0.5) * 1.5,
        vy: -2.0 - rng.next() * 3.5 - be * 4.0,
        size: 3.0 + rng.next() * 8.0 + be * 6.0,
        alpha: 1.0,
        life: 0.0,
        max_life: 40.0 + rng.next() * 50.0,
        heat: 0.8 + rng.next() * 0.2,
      });
    }
  }

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let mut i = st.fire.len();
  while i > 0 {
    i -= 1;
    let remove = {
      let p = &mut st.fire[i];
      p.life += 1.0;
      p.x += p.vx + (p.life * 0.05).sin() * 0.8;
      p.y += p.vy;
      p.size *= 0.97;
      p.life >= p.max_life || p.size < 0.5 || p.y < -20.0
    };

    if remove {
      st.fire.remove(i);
      continue;
    }

    let p_ref = &st.fire[i];
    let progress = p_ref.life / p_ref.max_life;
    let alpha = (1.0 - progress) * p_ref.heat;

    let color = if progress < 0.3 {
      mix(Color::WHITE, p, progress / 0.3)
    } else if progress < 0.7 {
      mix(p, s, (progress - 0.3) / 0.4)
    } else {
      mix(s, Color::hex("#220505"), (progress - 0.7) / 0.3)
    };

    c.set_global_alpha(alpha.clamp(0.0, 1.0));
    c.set_fill(Fill::Solid(color));
    c.set_shadow(glow, p_ref.size * 2.0);
    c.fill_circle(p_ref.x, p_ref.y, p_ref.size);
  }

  let bar_count = 64.min(ctx.config.reactivity.bar_count);
  let step = (ctx.freq_data.len() / bar_count).max(1);
  let bar_w = width / bar_count as f32;

  c.set_shadow(glow, 12.0);
  for k in 0..bar_count {
    let mut sum = 0usize;
    for j in 0..step {
      sum += *ctx.freq_data.get(k * step + j).unwrap_or(&0) as usize;
    }
    let val = (sum as f32 / (step as f32 * 255.0)) * sensitivity;
    let h = val * height * 0.35;
    let x = k as f32 * bar_w;

    let col = mix(p, s, k as f32 / bar_count as f32);
    c.set_fill(Fill::Solid(col.with_alpha(0.6)));
    c.fill_rect(x, height - h, bar_w - 1.0, h);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);
  c.restore();
}
