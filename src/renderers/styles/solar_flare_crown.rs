//! Solar Flare Crown style renderer (`solarFlareCrown`) — 360° Sun Corona Engine.
//!
//! Masterpiece Solar Prominence Corona:
//! - Blinding golden solar core with pulsing bass excursion.
//! - 64 organic fluid solar prominence arches & flame loops (NO needle spikes!).
//! - Bezier turbulence & solar prominence embers flying outward.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const FLARE_ARCHES: usize = 36;
const PROMINENCE_SPARKS: usize = 64;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let user_scale = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 - pos_offset_y;
    let reference_size = width.min(height);
    let base_r = 95.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    let corona_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.5,
        cx,
        cy,
        base_r * 3.6,
        &[
            (0.0, mix(glow_col, Color::rgba(1.0, 0.55, 0.05, 0.35 + be * 0.20), 0.5)),
            (0.45, mix(accent_col, Color::rgba(0.90, 0.20, 0.02, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.30, 0.03, 0.01, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(corona_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. ORGANIC SOLAR PROMINENCE ARCHES (CURVED LOOPS OVER SUN SURFACE)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for arch_i in 0..FLARE_ARCHES {
        let arch_f = arch_i as f32;
        let angle = (arch_f / FLARE_ARCHES as f32) * TAU + frame_time * 0.10;

        let bin_k = (arch_i * step_f / (FLARE_ARCHES / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let arch_r = (25.0 + fv * 140.0 * sensitivity + be * 40.0) * user_scale;

        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(17);
        for seg in 0..=16 {
            let t = seg as f32 / 16.0;
            let a_curr = angle + (t - 0.5) * 0.35;
            let r_curr = base_r + (t * std::f32::consts::PI).sin() * arch_r;

            let (sin_a, cos_a) = a_curr.sin_cos();
            pts.push((cx + cos_a * r_curr, cy + sin_a * r_curr));
        }

        let arch_col = mix(
            mix(Color::rgba(1.0, 0.95, 0.50, 0.95), glow_col, 0.5),
            mix(p_col, accent_col, fv),
            fv,
        );

        c.set_stroke(Fill::Solid(arch_col));
        c.set_line_width((2.5 + fv * 3.0) * user_scale);
        c.set_shadow(arch_col, (12.0 + fv * 10.0) * user_scale);
        c.stroke_polyline(&pts);
    }

    // -------------------------------------------------------------------------
    // 2. FLYING SOLAR PROMINENCE SPARKS & CORONA EMBERS
    // -------------------------------------------------------------------------
    for s_i in 0..PROMINENCE_SPARKS {
        let s_f = s_i as f32;
        let angle = (s_f / PROMINENCE_SPARKS as f32) * TAU - frame_time * 0.20;

        let bin_k = (s_i * step_f / (PROMINENCE_SPARKS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let spark_r = base_r * (1.10 + fv * 0.90 * sensitivity + be * 0.30) + (s_f * 5.0).sin() * (15.0 * user_scale);
        let spark_cx = cx + angle.cos() * spark_r;
        let spark_cy = cy + angle.sin() * spark_r;

        let spark_sz = (2.0 + (s_i % 3) as f32 * 1.5 + fv * 3.0) * user_scale;
        let spark_col = mix(Color::rgba(1.0, 1.0, 1.0, 0.95), glow_col, (s_i % 3) as f32 / 3.0);

        c.set_fill(Fill::Solid(spark_col));
        c.set_shadow(spark_col, (8.0 + fv * 6.0) * user_scale);
        c.fill_circle(spark_cx, spark_cy, spark_sz);
    }

    // -------------------------------------------------------------------------
    // 3. INTENSE GLOWING SOLAR CORE AT CENTER
    // -------------------------------------------------------------------------
    let core_r = base_r * (0.85 + be * 0.15 + bs * 0.05);
    let sun_core = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r,
        &[
            (0.0, Color::rgba(1.0, 1.0, 0.90, 0.98)),
            (0.35, Color::rgba(1.0, 0.65, 0.05, 0.90)),
            (0.75, mix(p_col, Color::hex("#d01500"), 0.8)),
            (1.0, Color::hex("#080201")),
        ],
    );

    c.set_fill(sun_core);
    c.set_shadow(Color::rgba(1.0, 0.50, 0.0, 0.95), (25.0 + bs * 15.0) * user_scale);
    c.fill_circle(cx, cy, core_r);

    // Center image on top of solar core (drawn last = on top of all flares)
    draw_radial_center_image(c, ctx, cx, cy, core_r * 0.75);

    c.set_global_alpha(1.0);
    c.restore();
}
