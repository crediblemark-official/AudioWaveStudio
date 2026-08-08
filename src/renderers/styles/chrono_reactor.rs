//! Chrono Clockwork Reactor style renderer (`chronoReactor`) — Steampunk Sci-Fi Arc Reactor Engine.
//!
//! Features:
//! - 3D mechanical gear teeth & brass/copper metallic concentric rings.
//! - Pulsating cyan arc reactor energy core driven by bass excursion.
//! - 48 radial clockwork spectrum meters ticking to the audio beat.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const TEETH_COUNT: usize = 24;
const METERS_COUNT: usize = 48;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let _p = theme_primary(theme);
    let _s = theme_secondary(theme);
    let _accent = theme_accent(theme);
    let _glow = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 ;
    let cy = height * 0.5 ;
    let reference_size = width.min(height);
    let base_r = 85.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep dark metallic backdrop
    c.set_fill(Fill::Solid(Color::hex("#05080c")));
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. ROTATING BRASS & COPPER MECHANICAL GEAR (OUTER RING)
    // -------------------------------------------------------------------------
    let gear_r = base_r * (1.30 + be * 0.08);
    let gear_angle = frame_time * 0.25;

    c.set_fill(Fill::Solid(Color::hex("#121b28")));
    c.set_stroke(Fill::Solid(Color::hex("#00e5ff")));
    c.set_line_width(2.0);
    c.fill_circle(cx, cy, gear_r);
    c.stroke_circle(cx, cy, gear_r);

    // Render 24 gear teeth
    for g in 0..TEETH_COUNT {
        let a = (g as f32 / TEETH_COUNT as f32) * TAU + gear_angle;
        let (sin_a, cos_a) = a.sin_cos();

        let tx0 = cx + cos_a * gear_r;
        let ty0 = cy + sin_a * gear_r;
        let tx1 = cx + cos_a * (gear_r + 14.0);
        let ty1 = cy + sin_a * (gear_r + 14.0);

        let tooth_w = 8.0;
        let px = -sin_a * (tooth_w * 0.5);
        let py = cos_a * (tooth_w * 0.5);

        let t_pts = vec![
            (tx0 - px, ty0 - py),
            (tx0 + px, ty0 + py),
            (tx1 + px * 0.7, ty1 + py * 0.7),
            (tx1 - px * 0.7, ty1 - py * 0.7),
        ];

        c.set_fill(Fill::Solid(Color::hex("#1a293d")));
        c.fill_polygon(&t_pts);
        c.stroke_polyline(&t_pts);
    }

    // -------------------------------------------------------------------------
    // 2. RADIAL SPECTRUM CLOCKWORK METERS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..METERS_COUNT {
        let angle = (i as f32 / METERS_COUNT as f32) * TAU - frame_time * 0.15;
        let bin_k = (i * step_f / (METERS_COUNT / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let meter_h = 10.0 + fv * 120.0 * sensitivity + be * 30.0;
        let r0 = base_r * 0.85;
        let r1 = r0 + meter_h;

        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let meter_col = mix(
            Color::rgba(0.0, 0.90, 1.0, 0.85),
            Color::rgba(1.0, 0.75, 0.0, 0.95 + bs * 0.05),
            fv,
        );

        c.set_stroke(Fill::Solid(meter_col));
        c.set_line_width(3.0);
        c.stroke_line(x0, y0, x1, y1);
    }

    // -------------------------------------------------------------------------
    // 3. TONY STARK ARC REACTOR ENERGY CORE
    // -------------------------------------------------------------------------
    let core_r = base_r * (0.65 + be * 0.18);
    let core_grad = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 1.0)),
            (0.35, Color::rgba(0.0, 0.95, 1.0, 0.95)),
            (0.70, Color::rgba(0.0, 0.40, 0.85, 0.85)),
            (1.0, Color::hex("#05080c")),
        ],
    );
    c.set_fill(core_grad);
    c.set_shadow(Color::rgba(0.0, 0.90, 1.0, 0.95), 22.0);
    c.fill_circle(cx, cy, core_r);

    c.set_global_alpha(1.0);
    c.restore();
}
