//! Fire 3D style renderer (`api3D`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{bin_sum, quadratic_wave, Ember};
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let g = crate::renderers::theme_glow(theme);
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;
  st.api_time += 0.016;
  let time = st.api_time;

  let center_x = width / 2.0;
  let center_y = height / 2.0;
  let base_y = height * 0.52;

  if st.embers.len() < 80 && (be > 0.05 || bs > 0.08) {
    let count = (3.0f32 + be * 12.0).floor() as usize;
    for _ in 0..count {
      st.embers.push(Ember {
        x: center_x + (rng.next() - 0.5) * width * 0.8,
        y: base_y + (rng.next() - 0.5) * 40.0,
        vx: (rng.next() - 0.5) * 1.8,
        vy: -1.0 - rng.next() * 2.5 - be * 3.0,
        size: 1.0 + rng.next() * 2.5,
        life: 0.0,
        max_life: 30.0 + rng.next() * 40.0,
      });
    }
  }

  c.save();

  let fire_w_ratio = ctx.config.reactivity.fire_width_ratio.unwrap_or(0.94);
  let fire_h_scale = ctx.config.reactivity.fire_height_scale.unwrap_or(1.0);

  let point_count = 32usize;
  let step = (freq.len() / point_count).max(1);
  let mut disp = vec![0.0f32; point_count];
  for i in 0..point_count {
    let raw = bin_sum(freq, step, i) * sensitivity;
    let wave = (time * 3.0 + i as f32 * 0.35).sin() * 0.15;
    disp[i] = (raw + wave.max(0.0)) * height * 0.28 * fire_h_scale;
  }

  let usable_w = width * fire_w_ratio;
  let margin = (width - usable_w) / 2.0;
  let px_of = |idx: usize| margin + (idx as f32 / (point_count - 1) as f32) * usable_w;

  let hero_raw: Vec<(f32, f32)> =
    (0..point_count).map(|i| (px_of(i), center_y - disp[i])).collect();
  let hero = quadratic_wave(&hero_raw, 5);

  let fill_pts: Vec<(f32, f32)> = {
    let mut pts = hero.clone();
    pts.push((width, height));
    pts.push((0.0, height));
    pts
  };

  let fill_grad = Fill::linear_gradient(0.0, center_y - height * 0.28, 0.0, height, &[
    (0.0, p.with_alpha(0.35 + be * 0.2)),
    (0.5, s.with_alpha(0.12)),
    (1.0, Color::TRANSPARENT),
  ]);
  c.set_fill(fill_grad);
  c.fill_polygon(&fill_pts);

  c.set_shadow(g, 18.0 + be * 20.0);
  c.set_stroke(Fill::Solid(p));
  c.set_line_width(3.6);
  c.stroke_polyline(&hero);

  c.set_shadow(a, 8.0);
  c.set_stroke(Fill::Solid(a));
  c.set_line_width(1.8);
  c.stroke_polyline(&hero);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let raw_refl: Vec<(f32, f32)> =
    (0..point_count).map(|i| (px_of(i), center_y - disp[i] * 0.45)).collect();
  c.set_shadow(g, 12.0);
  c.set_stroke(Fill::Solid(s));
  c.set_global_alpha(0.2);
  c.set_line_width(2.2);
  c.stroke_polyline(&quadratic_wave(&raw_refl, 5));
  c.set_global_alpha(1.0);

  let mut i = st.embers.len();
  while i > 0 {
    i -= 1;
    let remove = {
      let e = &mut st.embers[i];
      e.life += 1.0;
      e.x += e.vx;
      e.y += e.vy;
      e.life / e.max_life >= 1.0
    };
    if remove {
      st.embers.remove(i);
      continue;
    }
    let (ex, ey, size, alpha) = {
      let e = &st.embers[i];
      let progress = e.life / e.max_life;
      (e.x, e.y, e.size * (1.0 - progress * 0.3), (1.0 - progress) * 0.85)
    };
    c.set_shadow(g, 6.0);
    c.set_fill(Fill::Solid(p));
    c.set_global_alpha(alpha.clamp(0.0, 1.0));
    c.fill_circle(ex, ey, size);
    c.set_global_alpha(1.0);
  }
  if st.embers.len() > 180 {
    let n = st.embers.len() - 180;
    st.embers.drain(0..n);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);
  c.restore();
}
