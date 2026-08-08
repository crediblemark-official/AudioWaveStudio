//! Neon City 3D style renderer (`neonCity3D`) — native 3D port.
//!
//! Each spectrum-history frame becomes a real extruded box. The rows sit on
//! stepped terraces that rise and recede into the screen (−z), so the layering,
//! perspective shrink and the mirror reflections below each terrace are genuine
//! 3D (depth-tested in wgpu) instead of the old scaled 2D polygons.

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
  let be = ctx.bass_energy;
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

  // Camera: slow lateral drift with a gentle beat kick, pitched slightly down
  // over the skyline so the terrace steps and reflection pools read as depth.
  let scene = &mut ctx.scene3d;
  scene.cam_yaw = (ctx.frame_time * 0.04).sin() * 0.05 + be * 0.05;
  scene.cam_pitch = -0.05 - be * 0.02;
  scene.cam_zoom = 1.0;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // World y of the front terrace floor (origin = frame centre, y-up).
  let world_floor = height / 2.0 - floor_y;
  let rise = 6.0; // terrace step height between history rows
  let depth_step = 12.0; // world units between rows along −z

  for row in (0..rows).rev() {
    let depth_t = row as f32 / rows as f32;
    let row_depth = (rows - 1 - row) as f32 * depth_step;
    let terrace_y = world_floor + (rows - 1 - row) as f32 * rise;
    let row_alpha = 1.0 - depth_t * 0.45;

    for i in 0..cols {
      let val = vals[row][i];
      if val < 0.005 {
        continue;
      }
      let col = get_color(i as f32 / cols as f32, val);
      let bh = 2.0f32.max(val * height * 0.45);
      let bw = 2.0f32.max(max_bar_w);
      let depth = 1.0f32.max(bw * 0.4);
      let x = start_x + i as f32 * (max_bar_w + gap);

      // Building: front face at z = −row_depth, standing on its terrace.
      scene.add_box(
        x - center_x,
        terrace_y + bh * 0.5,
        -row_depth - depth * 0.5,
        bw,
        bh,
        depth,
        col.with_alpha(row_alpha),
      );

      // Mirror reflection below the terrace, receding in step with the box.
      let ref_col = Color::rgba(col.r * 0.4, col.g * 0.4, col.b * 0.4, row_alpha * 0.3);
      scene.add_box(
        x - center_x,
        terrace_y - bh * 0.5,
        -row_depth - depth * 0.5,
        bw,
        bh,
        depth,
        ref_col,
      );

      // Neon "data beam" on the front-most active tower (screen-space glow).
      if val > 0.45 && row == 0 {
        let by = floor_y - bh;
        let beam = Fill::linear_gradient(0.0, by, 0.0, 0.0, &[
          (0.0, col.with_alpha(val * 0.35)),
          (1.0, col.with_alpha(0.0)),
        ]);
        c.set_fill(beam);
        c.fill_rect(x - 1.0, 0.0, bw + 2.0, by);
      }
    }
  }

  c.restore();
}
