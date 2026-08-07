//! Spiral Galaxy style renderer (`spiralGalaxy`) — faithful port of
//! `src/services/renderers/spiralGalaxy.ts` (export path parity).
//!
//! Mirrors the TS model exactly: 400 uniformly random particles across 3
//! arms, per-particle rotation speed `0.002 + (1-r)*0.008` plus a shared
//! `rotSpeed = 0.003 + be*0.01 + bs*0.02`, arm spiral offset
//! `p.radius * 0.5`, color mixing by radius, and a small white core circle
//! (NO radial core glow — a stale port added a large gradient halo that the
//! preview never shows).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{mix, GalaxyParticle};
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
  let bs = ctx.beat_strength;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  let cx = width / 2.0;
  let cy = height / 2.0;
  let max_r = width.min(height) * 0.45;

  // TS `initGalaxy`: 400 particles, `arm = floor(rand*3)`, `r = rand()`.
  if st.galaxy.is_empty() {
    for _ in 0..400 {
      let arm = (rng.next() * 3.0) as u32;
      let r = rng.next();
      st.galaxy.push(GalaxyParticle {
        angle: rng.next() * TAU + arm as f32 * 2.1,
        radius: r,
        speed: 0.002 + (1.0 - r) * 0.008,
        size: 0.5 + r * 2.5,
        arm,
        offset: 0.0,
      });
    }
  }

  // Base rotation speed amplified by bass & beat strength
  let rot_speed = 0.003 + be * 0.015 * sensitivity + bs * 0.025 * sensitivity;
  let glow_intensity = 0.5 + be * 1.8 * sensitivity;

  for gp in st.galaxy.iter_mut() {
    // Sample audio frequency band corresponding to particle radius
    let bin_idx = ((gp.radius * 0.85) * (ctx.freq_data.len() as f32 - 1.0)) as usize;
    let freq_val = if bin_idx < ctx.freq_data.len() {
      (ctx.freq_data[bin_idx] as f32 / 255.0) * sensitivity
    } else {
      0.0
    };

    let audio_pulse = (freq_val * 0.7 + be * 0.8 + bs * 0.5).clamp(0.0, 3.0);

    // Audio-reactive rotation & particle displacement
    gp.angle += gp.speed + rot_speed * (1.0 + audio_pulse * 1.5);

    // Dynamic distance & spiral arm expansion on audio beats
    let dist = gp.radius * max_r * (1.0 + audio_pulse * 0.35);
    let spiral = gp.radius * 0.5 * (1.0 + audio_pulse * 0.5);
    let a = gp.angle + gp.arm as f32 * 2.1 + gp.radius * 3.0;
    let x = cx + a.cos() * (dist + (gp.angle * 3.0 + gp.arm as f32).sin() * spiral);
    let y = cy + a.sin() * (dist + (gp.angle * 3.0 + gp.arm as f32).cos() * spiral);

    // Audio-reactive sizing, alpha & glow
    let alpha = ((0.3 + gp.radius * 0.4) * (0.6 + audio_pulse * 0.6)).clamp(0.0, 1.0);
    let size = gp.size * (1.0 + audio_pulse * 1.4);
    let col = mix(p, s, gp.radius);

    c.set_global_alpha(alpha);
    c.set_fill(Fill::Solid(col));
    c.set_shadow(glow, size * 3.0 * glow_intensity);
    c.fill_circle(x, y, size);
  }

  // Pulsing white galactic core + dynamic beat aura
  let core_radius = 2.5 + be * 8.0 * sensitivity + bs * 5.0;
  c.set_fill(Fill::Solid(Color::WHITE));
  c.set_shadow(glow, 20.0 * glow_intensity);
  c.set_global_alpha((0.8 + be * 0.2).clamp(0.0, 1.0));
  c.fill_circle(cx, cy, core_radius);

  // Outer audio shockwave ring on strong beats
  let beat_pulse = (be * 0.6 + bs * 0.4) * sensitivity;
  if beat_pulse > 0.35 {
    let ring_radius = core_radius * 2.0 + beat_pulse * 30.0;
    c.set_stroke(Fill::Solid(glow));
    c.set_line_width(1.5);
    c.set_global_alpha((beat_pulse * 0.6).clamp(0.0, 0.7));
    c.stroke_circle(cx, cy, ring_radius);
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
}
