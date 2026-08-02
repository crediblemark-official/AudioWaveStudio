//! Neon City 3D style renderer (`neonCity3D`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

const HISTORY_DEPTH: usize = 12;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
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
