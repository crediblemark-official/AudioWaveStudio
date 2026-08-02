//! Radial Ring style renderer (`radial`).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let center_x = ctx.width / 2.0;
  let center_y = ctx.height * 0.48;
  let base_radius = ctx.width.min(ctx.height) * 0.18 + ctx.bass_energy * 18.0;
  let bar_count = ctx.config.reactivity.bar_count.min(96);
  let sensitivity = ctx.config.reactivity.sensitivity;

  if let Some(img) = &ctx.state.radial_center_image {
    let img_size = (base_radius - 5.0).max(0.0) * 2.0;
    let (iw, ih) = (img.w as f32, img.h as f32);
    if iw > 0.0 && ih > 0.0 {
      let s = (img_size / iw).min(img_size / ih);
      let w = iw * s;
      let h = ih * s;
      let ox = center_x - w / 2.0;
      let oy = center_y - h / 2.0;
      let layer_size = crate::gpu2d::LAYER_SIZE as f32;
      c.push_textured_quad(
        img.layer,
        ox,
        oy,
        w,
        h,
        [0.0, 0.0, iw / layer_size, ih / layer_size],
        Color::rgba(1.0, 1.0, 1.0, 1.0),
      );
    }
  } else {
    c.save();
    let disc_grad = Fill::radial_gradient(center_x, center_y, 5.0, center_x, center_y, base_radius, &[
      (0.0, theme_primary(theme)),
      (1.0, theme_secondary(theme)),
    ]);
    c.set_fill(disc_grad);
    c.fill_circle(center_x, center_y, (base_radius - 5.0).max(0.0));
    c.restore();
  }

  c.save();
  c.set_line_width(4.0);
  c.set_stroke(Fill::Solid(theme_accent(theme)));
  c.set_shadow(theme_glow(theme), 20.0);
  c.stroke_circle(center_x, center_y, base_radius);
  c.restore();

  let max_spike = ctx.width.min(ctx.height) * 0.25;
  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;

  c.save();
  c.set_shadow(theme_glow(theme), 12.0);

  for i in 0..bar_count {
    let val = (bin_value(ctx.freq_data, step, i) * sensitivity).min(1.0);
    let spike_h = val * max_spike;

    let angle = (i as f32 / bar_count as f32) * TAU + ctx.rotation_angle;
    let (sin, cos) = angle.sin_cos();

    let x1 = center_x + cos * base_radius;
    let y1 = center_y + sin * base_radius;
    let x2 = center_x + cos * (base_radius + spike_h);
    let y2 = center_y + sin * (base_radius + spike_h);

    let spike_grad = Fill::linear_gradient(x1, y1, x2, y2, &[
      (0.0, theme_primary(theme)),
      (1.0, theme_accent(theme)),
    ]);
    c.set_stroke(spike_grad);
    c.set_line_width(((TAU * 2.0 * base_radius) / bar_count as f32 - 3.0).max(2.0));
    c.stroke_line(x1, y1, x2, y2);
  }
  c.restore();
}
