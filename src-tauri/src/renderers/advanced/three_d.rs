//! 3D perspective visualizer styles (`threeD`, `api3D`, `neonCity3D`).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};

use crate::renderers::RenderContext;

use super::{bin_sum, bright, mix, quadratic_wave, LightMote, Spark};

// ---------------------------------------------------------------------------
// threeD
// ---------------------------------------------------------------------------



#[allow(dead_code)]
struct BarInfo {
  _x: f32,
  _by: f32,
  _bh: f32,
  _bw: f32,
  _dy: f32,
  _val: f32,
}

pub fn three_d(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let g = crate::renderers::theme_glow(theme);
  let bar_count = 48.min(ctx.config.reactivity.bar_count.max(1));
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  let center_x = width / 2.0;
  let floor_y = height * 0.78;
  let persp = 0.3;
  let max_bar_w = 18.0;
  let gap = 2.0;
  let bar_step = max_bar_w + gap;
  let step = (ctx.freq_data.len() / bar_count).max(1);

  if st.motes.is_empty() {
    for _ in 0..50 {
      st.motes.push(LightMote {
        x: rng.next() * width,
        y: rng.next() * height * 0.7,
        vx: (rng.next() - 0.5) * 0.2,
        vy: -0.05 - rng.next() * 0.15,
        size: 1.0 + rng.next() * 2.5,
        alpha: 0.15 + rng.next() * 0.35,
        phase: rng.next() * TAU,
      });
    }
  }

  if bs > 0.1 && bs > st.three_d_prev_beat {
    let bar_hs: Vec<f32> = (0..bar_count).map(|i| bin_sum(ctx.freq_data, step, i) * sensitivity).collect();
    let spark_count = (8.0f32 + bs * 25.0).floor() as usize;
    for _ in 0..spark_count {
      let bi = (rng.next() * bar_count as f32) as usize % bar_count;
      let bh = bar_hs[bi] * height * 0.38;
      let x = center_x - (bar_count as f32 * bar_step) / 2.0 + bi as f32 * bar_step
        + (rng.next() - 0.5) * 12.0;
      let cd = (x - center_x) / center_x;
      let ps = 1.0 - cd.abs() * persp;
      let py = floor_y - bh * ps - 5.0;
      st.sparks.push(Spark {
        x,
        y: py,
        vx: (rng.next() - 0.5) * 4.0,
        vy: -3.0 - rng.next() * 5.0,
        life: 0.0,
        max_life: 25.0 + rng.next() * 40.0,
        size: 1.5 + rng.next() * 3.0,
        color: mix(p, s, rng.next()),
        decay: 0.96 + rng.next() * 0.03,
        trail: Vec::new(),
      });
    }
  }
  st.three_d_prev_beat = bs;

  let mut i = st.sparks.len();
  while i > 0 {
    i -= 1;
    let remove = {
      let sp = &mut st.sparks[i];
      sp.trail.push((sp.x, sp.y));
      if sp.trail.len() > 6 {
        sp.trail.remove(0);
      }
      sp.life += 1.0;
      sp.x += sp.vx;
      sp.vy *= sp.decay;
      sp.vy += 0.04;
      sp.y += sp.vy;
      sp.vx *= 0.99;
      sp.life > sp.max_life || sp.y > floor_y || sp.y < -30.0
    };
    if remove {
      st.sparks.remove(i);
    }
  }

  st.three_d_rot += 0.002 + be * 0.004;
  let rot = st.three_d_rot;

  for m in st.motes.iter_mut() {
    m.x += m.vx + (rot + m.phase).sin() * 0.15;
    m.y += m.vy + (rot * 0.5 + m.phase).cos() * 0.05;
    if m.y < -15.0 {
      m.y = height * 0.7;
      m.x = rng.next() * width;
    }
    if m.x < -15.0 {
      m.x = width + 15.0;
    }
    if m.x > width + 15.0 {
      m.x = -15.0;
    }
  }

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  let floor_grad = Fill::linear_gradient(0.0, floor_y, 0.0, height, &[
    (0.0, p.with_alpha(0.3)),
    (0.4, s.with_alpha(0.12)),
    (1.0, g.with_alpha(0.0)),
  ]);
  c.set_fill(floor_grad);
  c.fill_rect(0.0, floor_y, width, height - floor_y);

  c.set_stroke(Fill::Solid(g.with_alpha(0.05)));
  c.set_line_width(1.0);
  for gi in -35i32..=35 {
    let gx = center_x + gi as f32 * 12.0;
    if gx < 0.0 || gx > width {
      continue;
    }
    c.stroke_line(gx, floor_y + gi.unsigned_abs() as f32 * 0.25, gx, height);
  }

  let total_w = (max_bar_w + gap) * bar_count as f32;
  let start_x = center_x - total_w / 2.0;

  let mut bars: Vec<BarInfo> = Vec::with_capacity(bar_count);
  for i in 0..bar_count {
    let val = bin_sum(ctx.freq_data, step, i) * sensitivity;
    let bar_h = 2.0f32.max(val * height * 0.38);
    let x = start_x + i as f32 * (max_bar_w + gap);
    let cd = (x - center_x) / (total_w / 2.0);
    let ps = 1.0 - cd.abs() * persp;
    let bw = 2.0f32.max(max_bar_w * ps);
    let bh = bar_h * ps;
    let by = floor_y - bh;
    let depth = 1.0f32.max(bw * 0.4 * ps);
    let dx = depth * 0.7;
    let dy = depth * 0.5;
    bars.push(BarInfo { _x: x, _by: by, _bh: bh, _bw: bw, _dy: dy, _val: val });

    let freq_ratio = i as f32 / bar_count as f32;
    let base = mix(p, s, freq_ratio);
    let bright_f = 0.4 + val * 0.4;
    let bar_c = bright(base, bright_f);
    let top_c = bright(base, bright_f * 1.3);
    let side_c = bright(base, bright_f * 0.6);
    let bright_boost = 0.3 + be * 0.3 + if bs > 0.12 { bs * 0.5 } else { 0.0 };

    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_fill(Fill::Solid(bar_c));
    c.fill_rect(x, by, bw, bh);

    let pulse = 1.0 + be * 0.2 + (rot + i as f32 * 0.3).sin() * be * 0.05;

    c.set_fill(Fill::Solid(top_c));
    c.fill_polygon(&[
      (x, by),
      (x + dx, by - dy * pulse),
      (x + bw + dx, by - dy * pulse),
      (x + bw, by),
    ]);

    c.set_fill(Fill::Solid(side_c));
    c.fill_polygon(&[
      (x + bw, by),
      (x + bw + dx, by - dy * pulse),
      (x + bw + dx, floor_y - dy * pulse),
      (x + bw, floor_y),
    ]);

    if val > 0.3 {
      let cap_c = bright(mix(s, a, val), bright_boost);
      c.set_fill(Fill::Solid(cap_c));
      c.fill_polygon(&[
        (x, by),
        (x + dx, by - dy * pulse),
        (x + bw + dx, by - dy * pulse),
        (x + bw, by),
      ]);
    }

    let refl_c = Color::rgba(base.r * 0.5, base.g * 0.5, base.b * 0.5, 0.15 + val * 0.15);
    let refl_h = bh * 0.4;
    c.set_fill(Fill::Solid(refl_c));
    c.fill_rect(x, floor_y, bw, refl_h);
  }

  for m in &st.motes {
    let alpha = (m.alpha * (0.6 + (rot * 2.0 + m.phase).sin() * 0.4)).clamp(0.0, 1.0);
    c.set_global_alpha(alpha);
    c.set_fill(Fill::Solid(mix(s, a, (m.phase / TAU).clamp(0.0, 1.0))));
    c.set_shadow(g, m.size * 2.0);
    c.fill_circle(m.x, m.y, m.size);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);
  c.restore();
}

// ---------------------------------------------------------------------------
// api3D
// ---------------------------------------------------------------------------

pub fn api_3d(c: &mut GpuCanvas, ctx: &mut RenderContext) {
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
      st.embers.push(super::Ember {
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

  let point_count = 32usize;
  let step = (freq.len() / point_count).max(1);
  let mut disp = vec![0.0f32; point_count];
  for i in 0..point_count {
    let raw = bin_sum(freq, step, i) * sensitivity;
    let wave = (time * 3.0 + i as f32 * 0.35).sin() * 0.15;
    disp[i] = (raw + wave.max(0.0)) * height * 0.28;
  }

  let margin = width * 0.08;
  let usable_w = width - margin * 2.0;
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

// ---------------------------------------------------------------------------
// neonCity3D
// ---------------------------------------------------------------------------

const HISTORY_DEPTH: usize = 12;

pub fn neon_city_3d(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let bar_count = 64.min(36.max(ctx.config.reactivity.bar_count));
  let sensitivity = ctx.config.reactivity.sensitivity;
  let st = &mut ctx.state.advanced;

  let center_x = width / 2.0;
  let floor_y = height * 0.58;
  let step = (ctx.freq_data.len() / bar_count).max(1);

  if st.frame_history.first().map(|f| f.len()) != Some(ctx.freq_data.len()) {
    st.frame_history.clear();
  }
  st.frame_history.insert(0, ctx.freq_data.to_vec());
  if st.frame_history.len() > HISTORY_DEPTH {
    st.frame_history.pop();
  }

  let rows = st.frame_history.len();
  let cols = bar_count;
  let total_available_w = width * 0.88;
  let gap = 2.0;
  let max_bar_w = 4.0f32.max(18.0f32.min((total_available_w - cols as f32 * gap) / cols as f32));
  let total_w = cols as f32 * (max_bar_w + gap);
  let start_x = center_x - total_w / 2.0;

  let vals: Vec<Vec<f32>> = st
    .frame_history
    .iter()
    .map(|data| {
      (0..cols)
        .map(|i| {
          let mut sum = 0usize;
          for j in 0..step {
            sum += *data.get(i * step + j).unwrap_or(&0) as usize;
          }
          (sum as f32 / (step as f32 * 255.0)) * sensitivity
        })
        .collect()
    })
    .collect();

  let get_color = |ratio: f32, bright_val: f32| {
    let base = mix(p, s, ratio);
    let f = 0.5 + bright_val * 0.5;
    Color::rgba(
      (base.r * f).min(1.0),
      (base.g * f).min(1.0),
      (base.b * f).min(1.0),
      1.0,
    )
  };

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  for row in (0..rows).rev() {
    let depth_ratio = row as f32 / rows as f32;
    let z_offset = (rows - 1 - row) as f32 * 8.0;
    let row_y = floor_y - z_offset * 0.5;
    let scale = 1.0 - depth_ratio * 0.35;
    let row_alpha = 1.0 - depth_ratio * 0.45;

    for i in 0..cols {
      let val = vals[row][i];
      if val < 0.005 {
        continue;
      }
      let col = get_color(i as f32 / cols as f32, val);
      let max_h = height * 0.45 * scale;
      let bh = 2.0f32.max(val * max_h);
      let bw = 2.0f32.max(max_bar_w * scale);
      let x = start_x + i as f32 * (max_bar_w + gap) * scale + (1.0 - scale) * (total_w / 2.0);
      let by = row_y - bh;
      let dx = 1.0f32.max(bw * 0.4);
      let dy = 1.0f32.max(bw * 0.3);

      let front = col.with_alpha(row_alpha * 0.85);
      let top = Color::rgba(
        (col.r + 40.0 / 255.0).min(1.0),
        (col.g + 40.0 / 255.0).min(1.0),
        (col.b + 40.0 / 255.0).min(1.0),
        row_alpha,
      );
      let side = Color::rgba(col.r * 0.5, col.g * 0.5, col.b * 0.5, row_alpha * 0.75);
      let stroke_col = Color::rgba(
        (col.r + 60.0 / 255.0).min(1.0),
        (col.g + 60.0 / 255.0).min(1.0),
        (col.b + 60.0 / 255.0).min(1.0),
        row_alpha * 0.6,
      );

      c.set_fill(Fill::Solid(front));
      c.fill_rect(x, by, bw, bh);
      c.set_stroke(Fill::Solid(stroke_col));
      c.set_line_width(0.7);
      c.stroke_rect(x, by, bw, bh);

      c.set_fill(Fill::Solid(top));
      c.fill_polygon(&[
        (x, by),
        (x + dx, by - dy),
        (x + bw + dx, by - dy),
        (x + bw, by),
      ]);

      c.set_fill(Fill::Solid(side));
      c.fill_polygon(&[
        (x + bw, by),
        (x + bw + dx, by - dy),
        (x + bw + dx, row_y - dy),
        (x + bw, row_y),
      ]);

      if val > 0.45 && row == 0 {
        let beam = Fill::linear_gradient(0.0, by, 0.0, 0.0, &[
          (0.0, col.with_alpha(val * 0.35)),
          (1.0, col.with_alpha(0.0)),
        ]);
        c.set_fill(beam);
        c.fill_rect(x - 1.0, 0.0, bw + 2.0, by);
      }
    }
  }

  let h_span = (height - floor_y).max(1.0);
  for row in 0..rows {
    let depth_ratio = row as f32 / rows as f32;
    let z_offset = (rows - 1 - row) as f32 * 8.0;
    let row_y = floor_y + z_offset * 0.3;
    let scale = 1.0 - depth_ratio * 0.35;
    let ref_alpha = ((0.35 - depth_ratio * 0.2) * (1.0 - (row_y - floor_y) / h_span)).max(0.05);

    for i in 0..cols {
      let val = vals[row][i];
      if val < 0.01 {
        continue;
      }
      let col = get_color(i as f32 / cols as f32, val);
      let max_h = height * 0.38 * scale;
      let bh = 2.0f32.max(val * max_h * 0.8);
      let bw = 2.0f32.max(max_bar_w * scale);
      let x = start_x + i as f32 * (max_bar_w + gap) * scale + (1.0 - scale) * (total_w / 2.0);
      let ref_by = row_y;
      let dx = 1.0f32.max(bw * 0.4);
      let dy = 1.0f32.max(bw * 0.3);

      let front = col.with_alpha(ref_alpha * 0.6);
      let bottom = Color::rgba(col.r * 0.7, col.g * 0.7, col.b * 0.7, ref_alpha * 0.4);
      let side = Color::rgba(col.r * 0.3, col.g * 0.3, col.b * 0.3, ref_alpha * 0.4);

      c.set_fill(Fill::Solid(front));
      c.fill_rect(x, ref_by, bw, bh);

      c.set_fill(Fill::Solid(bottom));
      c.fill_polygon(&[
        (x, ref_by + bh),
        (x + dx, ref_by + bh + dy),
        (x + bw + dx, ref_by + bh + dy),
        (x + bw, ref_by + bh),
      ]);

      c.set_fill(Fill::Solid(side));
      c.fill_polygon(&[
        (x + bw, ref_by),
        (x + bw + dx, ref_by + dy),
        (x + bw + dx, ref_by + bh + dy),
        (x + bw, ref_by + bh),
      ]);
    }
  }

  c.restore();
}
