//! Radial Spike Blade style renderer (`radialSpikeBlade`).
//!
//! Visual Concept:
//! - Hollow Teardrop Flare Blades using Solid Polygon Fills (`fill_polygon`) and tip spheres.
//! - ZERO thin line strokes.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.10, 0.05);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let blade_count = ctx.config.reactivity.bar_count.clamp(8, 128);
    let rot = frame_time * 0.08;

    // -------------------------------------------------------------------------
    // SOLID METALLIC FLARE BLADE POLYGONS (Zero Line Strokes!)
    // -------------------------------------------------------------------------
    for b in 0..blade_count {
        let t = b as f32 / blade_count as f32;
        let angle = t * TAU + rot;

        let sym_t = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
        let half = freq.len().saturating_sub(1);
        let off = ((s.sweep / std::f32::consts::TAU) * half as f32) as usize % half.max(1);
        let bin_k = (((sym_t * (half as f32 * 0.50)) as usize + off) % half.max(1)).min(half);
        let raw_v = freq[bin_k] as f32 / 255.0;

        let hf_boost = 1.0 + sym_t * 1.8;
        let fv = (raw_v * hf_boost * s.sensitivity * s.bass_mult + s.be * 0.30 + s.bs * 0.15
            + radial_common::beat_bump(&s, angle) * 0.6)
            .clamp(0.05, 3.5);

        let spike_h = (25.0 + fv * 160.0) * s.size_scale;
        let outer_r = s.inner_r + spike_h;
        let width_factor = (TAU / blade_count as f32) * (0.35 + fv * 0.10).min(0.48);
        let mid_r = s.inner_r + spike_h * 0.55;

        let (cos_a, sin_a) = angle.sin_cos();
        let (cos_l, sin_l) = (angle - width_factor).sin_cos();
        let (cos_r, sin_r) = (angle + width_factor).sin_cos();

        let p_base_l = (s.cx + cos_l * s.inner_r, s.cy + sin_l * s.inner_r);
        let p_base_r = (s.cx + cos_r * s.inner_r, s.cy + sin_r * s.inner_r);
        let p_mid_l = (s.cx + cos_l * mid_r, s.cy + sin_l * mid_r);
        let p_mid_r = (s.cx + cos_r * mid_r, s.cy + sin_r * mid_r);
        let p_tip = (s.cx + cos_a * outer_r, s.cy + sin_a * outer_r);

        let outer_poly = vec![p_base_l, p_mid_l, p_tip, p_mid_r, p_base_r];
        let blade_col = mix(mix(s.p_col, s.accent, sym_t), s.glow, (fv * 0.45).min(1.0));

        // Solid Blade Polygon Fill
        c.set_fill(Fill::Solid(blade_col.with_alpha(0.85)));
        c.set_shadow(blade_col, (12.0 + fv * 16.0) * s.user_scale);
        c.fill_polygon(&outer_poly);

        // Solid Tip Sphere Mote
        c.set_fill(Fill::Solid(mix(blade_col, Color::WHITE, 0.85)));
        c.set_shadow(s.glow, (10.0 + s.bs * 8.0) * s.user_scale);
        c.fill_circle(p_tip.0, p_tip.1, (2.8 + fv * 1.5) * s.user_scale);
    }

    radial_common::finish(c, ctx, &s);
}
