//! Particle visualizer styles (`flameFire`, `spiralGalaxy`).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};

use crate::renderers::RenderContext;

use super::{lerp, mix, FireParticle, GalaxyParticle};

// ---------------------------------------------------------------------------
// flameFire
// ---------------------------------------------------------------------------

pub fn flame_fire(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let sensitivity = ctx.config.reactivity.sensitivity;
  let theme = &ctx.config.theme;
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
        y: height - rng.next() * height * 0.05,
        vy: -(0.5 + rng.next() + be * 3.0),
        vx: (rng.next() - 0.5) * 0.5,
        size: 2.0 + rng.next() * 4.0 + be * 4.0,
        alpha: 0.5 + rng.next() * 0.5,
        life: 0.0,
        max_life: 40.0 + rng.next() * 30.0 + be * 30.0,
      });
    }
  }
  while st.fire.len() > 300 {
    st.fire.remove(0);
  }

  let mut i = st.fire.len();
  while i > 0 {
    i -= 1;
    let remove = {
      let fp = &mut st.fire[i];
      fp.life += 1.0;
      if fp.life >= fp.max_life {
        true
      } else {
        fp.x += fp.vx + (rng.next() - 0.5) * 0.3;
        fp.y += fp.vy;
        fp.vy += 0.02;
        fp.alpha *= 0.99;
        false
      }
    };
    if remove {
      st.fire.remove(i);
      continue;
    }
    let (fx, fy, size, alpha) = {
      let fp = &st.fire[i];
      let t = fp.life / fp.max_life;
      (fp.x, fp.y, fp.size * (1.0 - t * 0.7), fp.alpha * (1.0 - t))
    };
    let t = (st.fire[i].life / st.fire[i].max_life).min(1.0);
    let col = Color::rgba(
      lerp(p.r, s.r, t),
      lerp(p.g, s.g, t) * 0.5,
      lerp(p.b, s.b, t) * 0.2,
      1.0,
    );
    c.set_fill(Fill::Solid(col));
    c.set_global_alpha(alpha.clamp(0.0, 1.0));
    c.set_shadow(glow, 15.0);
    c.fill_circle(fx, fy, size);
  }

  let high_sum: f32 =
    ctx.freq_data.get(24..48).map(|w| w.iter().map(|&b| b as f32).sum::<f32>()).unwrap_or(0.0)
      / (24.0 * 255.0);
  if high_sum > 0.2 {
    let n = (high_sum * 5.0 * sensitivity).floor() as usize;
    for _ in 0..n {
      st.fire.push(FireParticle {
        x: rng.next() * width,
        y: height - 10.0,
        vy: -(1.0 + rng.next() * 2.0 + high_sum * 4.0),
        vx: (rng.next() - 0.5) * 1.5,
        size: 1.0 + rng.next() * 2.0,
        alpha: 1.0,
        life: 0.0,
        max_life: 15.0 + rng.next() * 10.0,
      });
    }
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
}

// ---------------------------------------------------------------------------
// spiralGalaxy
// ---------------------------------------------------------------------------

pub fn spiral_galaxy(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let glow = crate::renderers::theme_glow(theme);
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  if !st.galaxy_init {
    st.galaxy_init = true;
    for _ in 0..400 {
      let arm = (rng.next() * 3.0) as usize;
      let r = rng.next();
      st.galaxy.push(GalaxyParticle {
        angle: rng.next() * TAU + arm as f32 * 2.1,
        radius: r,
        speed: 0.002 + (1.0 - r) * 0.008,
        size: 0.5 + r * 2.5,
        arm: arm as u32,
      });
    }
  }

  let cx = width / 2.0;
  let cy = height / 2.0;
  let max_r = width.min(height) * 0.45;
  let rot_speed = 0.003 + be * 0.01 + bs * 0.02;
  let glow_intensity = 0.5 + be * 1.5;

  for gp in st.galaxy.iter_mut() {
    gp.angle += gp.speed + rot_speed;
    let dist = gp.radius * max_r;
    let spiral_offset = gp.radius * 0.5;
    let a = gp.angle + gp.arm as f32 * 2.1 + gp.radius * 3.0;
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
}
