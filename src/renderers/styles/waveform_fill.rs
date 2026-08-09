//! Waveform Fill style renderer (`waveformFill`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let user_scale = ctx.config.scale.clamp(0.1, 5.0);
  let pos_offset_y = -ctx.config.position_y * ctx.height * 0.5;
  let center_y = ctx.height * 0.5 + pos_offset_y + ctx.height * 0.05;
  let len = ctx.time_data.len();
  if len < 2 {
    return;
  }
  let slice_width = ctx.width / (len as f32 - 1.0);
  let sensitivity = ctx.config.reactivity.sensitivity;

  let mut pts: Vec<(f32, f32)> = Vec::with_capacity(len);
  for i in 0..len {
    let v = ctx.time_data[i] as f32 / 128.0 - 1.0;
    let y = center_y + v * (ctx.height * 0.28) * sensitivity * user_scale;
    pts.push((i as f32 * slice_width, y));
  }

  let mirror = ctx.config.reactivity.mirror_bars;

  let fill_grad = Fill::linear_gradient(0.0, 0.0, 0.0, ctx.height, &[
    (0.0, theme_primary(theme)),
    (0.5, theme_secondary(theme)),
    (1.0, Color::TRANSPARENT),
  ]);

  // Fill glow approximation: canvas `fill()` with shadowBlur=20 paints the
  // fill region convolved with a Gaussian of sigma=10 (shadowBlur/2), so the
  // shadow hugs the ENTIRE fill silhouette — a uniform ~0.45*glowColor wash
  // deep inside the fill (measured against Skia), falling off as
  // 0.5*erfc(d/(10*sqrt2)) outside the wave edge and clipping at the canvas
  // border. Reproduce by rasterizing the fill region N times at
  // Gaussian-sampled offsets (radial inverse-CDF + golden-angle spiral, the
  // same technique as the text glow): the copy-sum converges to the true
  // convolution. A coarser polyline (every 4th point) is fine — the blur
  // hides the geometry detail. The wave stroke's own shadowBlur=10 glow is
  // handled by stroke_polyline separately below.
  let glow = theme_glow(theme);
  const GLOW_N: u32 = 96;
  const GLOW_SIGMA: f32 = 10.0;
  const GLOW_INTERIOR: f32 = 0.9;
  if glow.a > 0.0 {
    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);
    // Main fill glow (drawn behind the fill, like canvas shadow-then-fill).
    // The shadow is scaled by the fill's alpha channel (canvas draws the
    // shadow with the content's coverage), which fades to 0 toward the bottom
    // gradient stop — so the copies use a gradient with the same alpha fade.
    let coarse: Vec<(f32, f32)> = pts.iter().step_by(4).copied().collect();
    let per_copy = GLOW_INTERIOR / GLOW_N as f32;
    let main_glow = Fill::linear_gradient(0.0, 0.0, 0.0, ctx.height, &[
      (0.0, glow.with_alpha(per_copy)),
      (0.5, glow.with_alpha(per_copy)),
      (1.0, Color::rgba(glow.r, glow.g, glow.b, 0.0)),
    ]);
    for i in 0..GLOW_N {
      let r = GLOW_SIGMA
        * (-2.0 * (1.0 - (i as f32 + 0.5) / GLOW_N as f32).ln()).sqrt();
      let theta = i as f32 * 2.399963;
      c.save();
      c.translate(r * theta.cos(), r * theta.sin());
      c.set_fill(main_glow.clone());
      c.fill_polyline_to_base(&coarse, ctx.height);
      c.restore();
    }
    c.restore();
  }

  c.save();
  c.set_fill(fill_grad);
  // Quad strips to the bottom edge — exactly the region a canvas fill() of
  // [wave pts, (width, h), (0, h)] paints (non-zero winding), without the
  // fan-from-corner overflow that fill_polygon would produce on non-
  // star-shaped waves (dips below the anchor at high sensitivity).
  c.fill_polyline_to_base(&pts, ctx.height);

  if mirror {
    let mirror_pts: Vec<(f32, f32)> = pts
      .iter()
      .map(|&(x, y)| (x, center_y - (y - center_y)))
      .collect();
    // Mirror fills up to the TOP edge — quad strips to y=0 tile exactly the
    // region between the mirrored wave and the top edge (TS lineTo(width,0)
    // + lineTo(0,0) + closePath).

    // TS keeps shadowBlur=20 active for the mirror fill too (canvas state
    // persists between the two fill() calls), drawn with the mirror's 0.5
    // globalAlpha — Gaussian copies at half the interior alpha, over the main
    // fill (TS draws this shadow after the first fill).
    if glow.a > 0.0 {
      let coarse_m: Vec<(f32, f32)> = mirror_pts.iter().step_by(4).copied().collect();
      let mirror_glow = Fill::linear_gradient(0.0, 0.0, 0.0, ctx.height, &[
        (0.0, glow.with_alpha(GLOW_INTERIOR * 0.5 / GLOW_N as f32)),
        (0.5, glow.with_alpha(GLOW_INTERIOR * 0.5 / GLOW_N as f32)),
        (1.0, Color::rgba(glow.r, glow.g, glow.b, 0.0)),
      ]);
      for i in 0..GLOW_N {
        let r = GLOW_SIGMA
          * (-2.0 * (1.0 - (i as f32 + 0.5) / GLOW_N as f32).ln()).sqrt();
        let theta = i as f32 * 2.399963;
        c.save();
        c.translate(r * theta.cos(), r * theta.sin());
        c.set_fill(mirror_glow.clone());
        c.fill_polyline_to_base(&coarse_m, 0.0);
        c.restore();
      }
    }

    c.set_global_alpha(0.5);
    c.fill_polyline_to_base(&mirror_pts, 0.0);
    c.set_global_alpha(1.0);
  }

  c.restore();

  c.save();
  c.set_stroke(Fill::Solid(theme_accent(theme)));
  c.set_line_width(2.0);
  c.set_shadow(theme_glow(theme), 10.0);
  c.stroke_polyline(&pts);

  if mirror {
    let mirror_pts: Vec<(f32, f32)> = pts
      .iter()
      .map(|&(x, y)| (x, center_y - (y - center_y)))
      .collect();
    c.set_global_alpha(0.6);
    c.stroke_polyline(&mirror_pts);
    c.set_global_alpha(1.0);
  }

  c.restore();
}
