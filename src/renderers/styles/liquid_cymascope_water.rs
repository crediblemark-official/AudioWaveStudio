//! Liquid Cymascope Water style renderer (`liquidCymascopeWater`).
//!
//! Visual Concept:
//! - A top-down photograph of a real CymaScope water dish: a translucent disc
//!   of liquid whose rim undulates softly in low harmonic modes while fine
//!   Chladni nodal caustic rays shimmer across the surface with the spectrum.
//! - Deliberately the OPPOSITE of `Acoustic Cymascope`: that style is a crisp
//!   neon ring mandala (hollow strokes); this one is a soft filled pool of
//!   water — blue, transparent, fluid.
//! - The water SURROUNDS the central logo disc — it is a wide annular dish
//!   whose inner edge is just outside the disc border.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const SURF_SEGS:   usize = 96;
const NODAL_RAYS:  usize = 10;
const RIPPLE_COUNT: usize = 4;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col  = theme_primary(theme);
    let s_col  = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let bass_mult    = ctx.config.reactivity.bass_multiplier;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = ctx.config.position_y * height * 0.5;
    let bar_count    = ctx.config.reactivity.bar_count.clamp(16, 96);

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.10 + bs * 0.05);

    // Bioluminescent water palette (fluid blue, transparent).
    let aqua      = mix(glow, Color::rgba(0.0, 0.9, 1.0, 1.0), 0.7);
    let deep_blue = mix(s_col, Color::rgba(0.02, 0.15, 0.35, 1.0), 0.6);
    let froth     = Color::rgba(0.9, 1.0, 1.0, 1.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // The water dish spans from just outside the central disc ring
    // to a wide outer edge, so it truly *surrounds* the radial logo disc.
    let outer_r   = base_r * 2.8;          // full dish outer radius
    let dish_inner = inner_r * 1.06;       // start just outside the disc border

    let step = ((freq.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
    let rot  = frame_time * 0.04;

    // -------------------------------------------------------------------------
    // WATER DISC — translucent top-down pool. The rim undulates in soft
    // harmonic modes. Starts at dish_inner (outside the central disc) and
    // extends to outer_r so it fully encircles the logo.
    // -------------------------------------------------------------------------
    let mut surf: Vec<(f32, f32)> = Vec::with_capacity(SURF_SEGS + 1);
    let mut total_val = 0.0f32;
    for i in 0..=SURF_SEGS {
        let t     = i as f32 / SURF_SEGS as f32;
        let angle = t * TAU + rot;

        let sym_t   = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
        let bin_idx = ((sym_t * (SURF_SEGS as f32 * 0.5)) as usize * step) % freq.len();
        let audio_v = bin_value(freq, step, bin_idx) * sensitivity;
        total_val  += audio_v;

        // Fluid surface: gentle 6-petal mode + slow wobble.
        let soft   = (angle * 6.0 + rot * 2.0).cos() * 0.5 + 0.5;
        let wob    = (angle * 2.0 - frame_time * 0.8).sin() * 0.5 + 0.5;
        let wave_h = (audio_v * 0.55 + be * 0.18).clamp(0.0, 2.0);

        // Rim undulates around outer_r — when silent rests at outer_r * 0.78
        let rim_base = outer_r * (0.78 + bs * 0.04);
        let r_cur    = rim_base + wave_h * (20.0 + soft * 36.0 + wob * 10.0) * user_scale;

        let (cos_a, sin_a) = angle.sin_cos();
        surf.push((cx + cos_a * r_cur, cy + sin_a * r_cur));
    }

    // Smooth the rim into a fluid petal contour.
    let mut rim: Vec<(f32, f32)> = Vec::new();
    for i in 0..SURF_SEGS {
        let p0  = surf[i];
        let p1  = surf[(i + 1) % SURF_SEGS];
        let mid = ((p0.0 + p1.0) * 0.5, (p0.1 + p1.1) * 0.5);
        let seg = GpuCanvas::sample_quadratic(
            if rim.is_empty() { p0 } else { *rim.last().unwrap() },
            p0, mid, 3,
        );
        if rim.is_empty() {
            rim.extend(seg);
        } else {
            rim.extend(seg.into_iter().skip(1));
        }
    }
    if let Some(&first) = rim.first() { rim.push(first); }

    // Translucent water fill: deep water near the disc, bright caustic at rim.
    let water_fill = Fill::radial_gradient(
        cx, cy, dish_inner,
        cx, cy, outer_r,
        &[
            (0.0,  deep_blue.with_alpha(0.30 + bs * 0.12)),
            (0.45, aqua.with_alpha(0.18 + be * 0.08)),
            (0.80, aqua.with_alpha(0.10)),
            (1.0,  aqua.with_alpha(0.02)),
        ],
    );
    c.set_fill(water_fill);
    c.set_shadow(aqua, (18.0 + bs * 12.0) * user_scale);
    c.fill_polygon(&rim);

    // Water rim gloss.
    c.set_stroke(Fill::Solid(aqua.with_alpha((0.50 + bs * 0.2).min(0.90))));
    c.set_line_width((2.0 + be * 1.5) * user_scale);
    c.set_shadow(aqua, (12.0 + bs * 8.0) * user_scale);
    c.stroke_polyline(&rim);

    // -------------------------------------------------------------------------
    // CHLADNI NODAL CAUSTIC etched on the surface: fine shimmering rays that
    // run from just outside the central disc all the way to the dish rim,
    // so they span the entire water annulus.
    // -------------------------------------------------------------------------
    let ray_alpha = (0.12 + total_val * 0.025 + be * 0.06).clamp(0.06, 0.45);
    for r_i in 0..NODAL_RAYS {
        let base_ang = r_i as f32 / NODAL_RAYS as f32 * TAU + rot * 0.7;
        let tilt     = if r_i % 2 == 0 { 0.0 } else { 0.28 };
        let mut ray: Vec<(f32, f32)> = Vec::with_capacity(25);
        for p in 0..=24 {
            let t   = p as f32 / 24.0;
            let ang = base_ang + (t - 0.5) * tilt;
            // Ray spans from just outside disc (dish_inner) to near the outer rim
            let r_ray = dish_inner + t * (outer_r * 0.82 - dish_inner);
            let w     = (t * 9.0 - frame_time * 3.0).sin() * (1.5 + be * 2.0) * user_scale;
            let rr    = (r_ray + w).max(dish_inner);
            let (cos_a, sin_a) = ang.sin_cos();
            ray.push((cx + cos_a * rr, cy + sin_a * rr));
        }
        let ray_col = mix(aqua, froth, 0.5).with_alpha(ray_alpha);
        c.set_stroke(Fill::Solid(ray_col));
        c.set_line_width(1.0 * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&ray);
    }

    // Expanding ghost interference rings — concentric, spanning the full water
    // annulus from dish_inner outward so they visibly surround the logo disc.
    for rp in 0..RIPPLE_COUNT {
        let rt      = ((frame_time * 0.30 + rp as f32 / RIPPLE_COUNT as f32) % 1.0).clamp(0.0, 1.0);
        let rr      = dish_inner + rt * (outer_r * 0.82 - dish_inner);
        let rp_alpha = ((1.0 - rt) * (0.28 + bs * 0.35) * (total_val * 0.25 + 0.18)).clamp(0.0, 0.38);
        c.set_stroke(Fill::Solid(aqua.with_alpha(rp_alpha)));
        c.set_line_width((1.8 - rt * 1.0).max(0.4) * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_circle(cx, cy, rr);
    }

    // Shimmering water motes scattered across the full annular dish.
    if total_val > 0.3 {
        for m in 0..20 {
            let mf  = m as f32;
            let ma  = (mf * 51.0).sin() * TAU + frame_time * 0.2;
            // Spread motes from inner edge to outer rim using golden ratio
            let mfrac = (mf * 0.618_034).fract();
            let mr    = dish_inner + mfrac * (outer_r * 0.78 - dish_inner);
            let m_sz  = (1.0 + be * 1.5) * user_scale;
            let m_alpha = (0.40 + bs * 0.25 + (mf * 7.3).sin() * 0.15).clamp(0.15, 0.85);
            c.set_fill(Fill::Solid(froth.with_alpha(m_alpha)));
            c.set_shadow(aqua, (5.0 + be * 3.0) * user_scale);
            c.fill_circle(cx + ma.cos() * mr, cy + ma.sin() * mr, m_sz);
        }
    }

    // Center Logo Disc (drawn on top so the water appears to surround it)
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(aqua));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(aqua, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = (p_col, accent);
    c.set_global_alpha(1.0);
    c.restore();
}
