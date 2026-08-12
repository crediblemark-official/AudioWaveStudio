//! Pulsing Sunburst Corona style renderer (`pulsingSunburstCorona`).
//!
//! Explosive sun/star burst — deliberately distinct from the crystal-prism
//! facets of `CrystalPrism`:
//! - Spiral-swept starburst rays: long/short spikes with a bright inner core
//!   and soft corona halo, skewing slightly as they radiate (spinning-sun feel).
//! - Beat-driven expanding corona shockwave rings.
//! - Radiant photosphere core glow instead of a flat black disc + neon ring.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col  = theme_primary(theme);
    let _s_col = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let bass_mult    = ctx.config.reactivity.bass_multiplier;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = ctx.config.position_y * height * 0.5;
    let bar_count    = ctx.config.reactivity.bar_count.clamp(18, 56);

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.10 + bs * 0.05);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Pseudo-random sweep per beat (consistent with the radial family).
    let sweep = super::radial_common::sweep_angle(ctx.beat_count);

    let num_rays = bar_count;
    let step = ((freq.len() as f32) / num_rays as f32).floor().max(1.0) as usize;
    let rot = frame_time * 0.08;
    let sweep_off = ((sweep / TAU) * num_rays as f32) as usize % num_rays.max(1);

    // -------------------------------------------------------------------------
    // 1. SPIRAL-SWEPT SUN RAYS (alternating long/short starburst spikes)
    // -------------------------------------------------------------------------
    for i in 0..num_rays {
        let t = i as f32 / num_rays as f32;
        // Slight spiral skew gives the burst a swept, spinning-sun feel.
        let angle = t * TAU + rot + t * 0.35;
        let is_long = i % 2 == 0;

        let idx = (i + sweep_off) % num_rays.max(1);
        let audio_v = bin_value(freq, step, idx) * sensitivity;

        let mut d = (angle - sweep).rem_euclid(TAU);
        if d > std::f32::consts::PI {
            d = TAU - d;
        }
        let bump = (1.0 - ((d / 0.5).powi(2)).exp().min(1.0)) * bs.min(1.5) * 0.5;

        let base_wave = (frame_time * 1.7 + i as f32 * 0.9).sin() * 0.15 + 0.22;
        let val = (base_wave + audio_v * 0.9 + be * 0.20 + bump).clamp(0.10, 1.8);

        let (len, half_w) = if is_long {
            ((50.0 + val * 120.0) * user_scale, 0.030 + val * 0.020)
        } else {
            ((20.0 + val * 55.0) * user_scale, 0.050 + val * 0.030)
        };

        let base_r_use = inner_r * 1.02;
        let (cos_a, sin_a) = angle.sin_cos();
        let (cos_l, sin_l) = (angle - half_w).sin_cos();
        let (cos_r, sin_r) = (angle + half_w).sin_cos();

        let base_l = (cx + cos_l * base_r_use, cy + sin_l * base_r_use);
        let base_r_pt = (cx + cos_r * base_r_use, cy + sin_r * base_r_use);
        let tip = (cx + cos_a * (base_r_use + len), cy + sin_a * (base_r_use + len));

        // Hot ray: warm blend from glow (root) to accent (tip).
        let ray_col = mix(glow, accent, (t * 0.5 + val * 0.15).clamp(0.0, 1.0));
        let halo_col = mix(ray_col, p_col, 0.35);

        // Outer soft corona halo of the ray
        c.set_fill(Fill::Solid(halo_col.with_alpha(0.28 + val * 0.12)));
        c.set_shadow(ray_col, (10.0 + val * 10.0) * user_scale);
        c.fill_polygon(&[base_l, tip, base_r_pt]);

        // Inner hot core ray (narrower, brighter)
        let hw2 = half_w * 0.5;
        let (cos_l2, sin_l2) = (angle - hw2).sin_cos();
        let (cos_r2, sin_r2) = (angle + hw2).sin_cos();
        let core_l = (cx + cos_l2 * base_r_use, cy + sin_l2 * base_r_use);
        let core_r_pt = (cx + cos_r2 * base_r_use, cy + sin_r2 * base_r_use);
        let core_tip = (cx + cos_a * (base_r_use + len * 0.92), cy + sin_a * (base_r_use + len * 0.92));

        c.set_fill(Fill::Solid(mix(ray_col, Color::WHITE, 0.45).with_alpha(0.85)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_polygon(&[core_l, core_tip, core_r_pt]);

        // Sparkle tip mote
        c.set_fill(Fill::Solid(mix(ray_col, Color::WHITE, 0.8)));
        c.set_shadow(glow, (10.0 + bs * 8.0) * user_scale);
        c.fill_circle(tip.0, tip.1, (1.6 + val * 1.6) * user_scale);
    }

    // -------------------------------------------------------------------------
    // 2. BEAT-DRIVEN EXPANDING CORONA RINGS (shockwave fronts)
    // -------------------------------------------------------------------------
    for ri in 0..3 {
        let rf = ri as f32;
        let ring_r = inner_r + (rf * 26.0 + be * 8.0 + bs * (rf + 1.0) * 20.0) * user_scale;
        let ring_col = mix(glow, accent, rf / 3.0);
        let alpha = (0.45 - rf * 0.10) * (0.6 + be * 0.4).min(1.0);
        c.set_stroke(Fill::Solid(ring_col.with_alpha(alpha)));
        c.set_line_width((2.4 - rf * 0.5) * user_scale);
        c.set_shadow(ring_col, (12.0 - rf * 2.0) * user_scale);
        c.stroke_circle(cx, cy, ring_r.max(inner_r + 2.0));
    }

    // -------------------------------------------------------------------------
    // 3. RADIANT PHOTOSPHERE CORE (warm bloom that pulses with bass)
    // -------------------------------------------------------------------------
    let core_r = inner_r * (0.85 + be * 0.10 + bs * 0.06);
    c.set_fill(Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r * 2.4,
        &[
            (0.0, mix(Color::WHITE, glow, 0.25)),
            (0.30, glow.with_alpha(0.8)),
            (0.70, mix(glow, accent, 0.5).with_alpha(0.25)),
            (1.0, Color::TRANSPARENT),
        ],
    ));
    c.fill_circle(cx, cy, core_r * 2.4);

    c.set_fill(Fill::Solid(mix(Color::WHITE, glow, 0.4)));
    c.set_shadow(glow, (18.0 + bs * 14.0) * user_scale);
    c.fill_circle(cx, cy, core_r * 0.55);

    draw_radial_center_image(c, ctx, cx, cy, core_r * 0.5);

    c.set_global_alpha(1.0);
    c.restore();
}
