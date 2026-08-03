//! Flame Fire style renderer (`flameFire`) — faithful port of
//! `src/services/renderers/flameFire.ts` (export path parity).
//!
//! Mirrors the TS particle model exactly: particles spawn along the bottom
//! 5% of the screen, drift up with per-frame jitter, and mix primary →
//! secondary with channel-weighted rates (g at 0.5×, b at 0.2×). There are
//! NO spectrum bars in the TS style (a stale port drew 64 bars at the
//! bottom that never appear in the preview).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::FireParticle;
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let sensitivity = ctx.config.reactivity.sensitivity;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let glow = crate::renderers::theme_glow(theme);
  let be = ctx.bass_energy;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  // TS: `if (fireParticles.length > 0 || be > 0.02)` — spawn while any fire
  // exists (or a beat is active), so the effect never fully dies mid-export.
  if !st.fire.is_empty() || be > 0.02 {
    let spawn = (2.0f32 + be * 8.0 * sensitivity).floor() as usize;
    for _ in 0..spawn {
      st.fire.push(FireParticle {
        x: rng.next() * width,
        // TS: `y: height - Math.random() * height * 0.05` (bottom 5%).
        y: height - rng.next() * height * 0.05,
        vy: -(0.5 + rng.next() * 1.0 + be * 3.0),
        vx: (rng.next() - 0.5) * 0.5,
        size: 2.0 + rng.next() * 4.0 + be * 4.0,
        alpha: 0.5 + rng.next() * 0.5,
        life: 0.0,
        max_life: 40.0 + rng.next() * 30.0 + be * 30.0,
        heat: 0.0,
      });
    }
  }
  // TS: `const maxParticles = 300; while (...) fireParticles.shift();`
  while st.fire.len() > 300 {
    st.fire.remove(0);
  }

  let mut i = st.fire.len();
  while i > 0 {
    i -= 1;
    let remove = {
      let p = &mut st.fire[i];
      p.life += 1.0;
      if p.life >= p.max_life {
        true
      } else {
        // TS: `p.x += p.vx + (Math.random() - 0.5) * 0.3; p.y += p.vy;
        //      p.vy += 0.02; p.alpha *= 0.99;`
        p.x += p.vx + (rng.next() - 0.5) * 0.3;
        p.y += p.vy;
        p.vy += 0.02;
        p.alpha *= 0.99;
        false
      }
    };
    if remove {
      st.fire.remove(i);
      continue;
    }

    let (x, y, size, alpha, t) = {
      let p = &st.fire[i];
      let t = p.life / p.max_life;
      let size = p.size * (1.0 - t * 0.7);
      let alpha = p.alpha * (1.0 - t);
      (p.x, p.y, size, alpha, t)
    };

    // TS color mix: `mix = t; r = round(pR + (sR-pR)*mix);
    //   g = round(pG + (sG-pG)*mix*0.5); b = round(pB + (sB-pB)*mix*0.2)`.
    let col = Color::rgba(
      (p.r + (s.r - p.r) * t).clamp(0.0, 1.0),
      (p.g + (s.g - p.g) * t * 0.5).clamp(0.0, 1.0),
      (p.b + (s.b - p.b) * t * 0.2).clamp(0.0, 1.0),
      1.0,
    );
    c.set_global_alpha(alpha.clamp(0.0, 1.0));
    c.set_fill(Fill::Solid(col));
    c.set_shadow(glow, 15.0);
    c.fill_circle(x, y, size);
  }

  // TS high-frequency spark shower: bins 24..48, spawn only when > 0.2.
  let high_bins = 48.min(ctx.freq_data.len());
  let mut high_sum = 0usize;
  for k in 24..high_bins {
    high_sum += ctx.freq_data[k] as usize;
  }
  let high = high_sum as f32 / (24.0 * 255.0);
  if high > 0.2 {
    let spawn = (high * 5.0 * sensitivity).floor() as usize;
    for _ in 0..spawn {
      st.fire.push(FireParticle {
        x: rng.next() * width,
        y: height - 10.0,
        vy: -(1.0 + rng.next() * 2.0 + high * 4.0),
        vx: (rng.next() - 0.5) * 1.5,
        size: 1.0 + rng.next() * 2.0,
        alpha: 1.0,
        life: 0.0,
        max_life: 15.0 + rng.next() * 10.0,
        heat: 0.0,
      });
    }
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
}
