//! Neon Bio-Hazard Pulse style renderer (`neonBiohazard`) — Cyber Toxic Engine.
//!
//! Features:
//! - 3D neon biohazard symbol in center pulsing to bass beats.
//! - Toxic green & electric cyan radial plasma spectrum bars.
//! - Concentric hazard warning rings with audio-reactive pulsing.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const PLASMA_BARS: usize = 60;

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

    let cx = width * 0.5;
    let cy = height * 0.5;
    let reference_size = width.min(height);
    let base_r = 90.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep toxic cyber backdrop
    c.set_fill(Fill::Solid(Color::hex("#020a04")));
    c.fill_rect(0.0, 0.0, width, height);

    // Toxic green ambient glow
    let toxic_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.5,
        cx,
        cy,
        base_r * 3.5,
        &[
            (0.0, Color::rgba(0.20, 1.0, 0.10, 0.25 + be * 0.20)),
            (0.40, Color::rgba(0.0, 0.85, 0.40, 0.12)),
            (0.80, Color::rgba(0.0, 0.30, 0.15, 0.04)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(toxic_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. TOXIC PLASMA RADIAL SPECTRUM BARS (360°)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..PLASMA_BARS {
        let angle = (i as f32 / PLASMA_BARS as f32) * TAU + frame_time * 0.10;
        let bin_k = (i * step_f / (PLASMA_BARS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let bar_h = 15.0 + fv * 160.0 * sensitivity + be * 40.0;
        let r0 = base_r * (1.0 + be * 0.12);
        let r1 = r0 + bar_h;

        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let bar_w = 6.0 + fv * 8.0;
        let px = -sin_a * (bar_w * 0.5);
        let py = cos_a * (bar_w * 0.5);

        let pts = vec![
            (x0 - px, y0 - py),
            (x0 + px, y0 + py),
            (x1 + px * 0.3, y1 + py * 0.3),
            (x1 - px * 0.3, y1 - px * 0.3),
        ];

        let bar_col = mix(
            Color::rgba(0.20, 1.0, 0.10, 0.90 + bs * 0.10),
            Color::rgba(0.0, 0.90, 0.85, 0.70),
            fv,
        );
        c.set_fill(Fill::Solid(bar_col));
        c.fill_polygon(&pts);
    }

    // -------------------------------------------------------------------------
    // 2. CONCENTRIC HAZARD WARNING RING
    // -------------------------------------------------------------------------
    let warn_r = base_r * (1.04 + be * 0.10);
    c.set_stroke(Fill::Solid(Color::hex("#39ff14")));
    c.set_line_width(3.0);
    c.set_shadow(Color::hex("#39ff14"), 16.0);
    c.stroke_circle(cx, cy, warn_r);

    // -------------------------------------------------------------------------
    // 3. NEON BIOHAZARD SYMBOL CORE
    // -------------------------------------------------------------------------
    let bio_r = base_r * (0.55 + be * 0.12);

    // Render 3 Biohazard Arcs
    for a_i in 0..3 {
        let a_ang = (a_i as f32 / 3.0) * TAU + frame_time * 0.15;
        let bx = cx + a_ang.cos() * (bio_r * 0.55);
        let by = cy + a_ang.sin() * (bio_r * 0.55);

        c.set_stroke(Fill::Solid(Color::hex("#39ff14")));
        c.set_line_width(4.5);
        c.stroke_circle(bx, by, bio_r * 0.50);
    }

    // Center Biohazard Core Ring
    c.set_stroke(Fill::Solid(Color::hex("#020a04")));
    c.set_line_width(6.0);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.stroke_circle(cx, cy, bio_r * 0.35);

    c.set_fill(Fill::Solid(Color::hex("#39ff14")));
    c.set_shadow(Color::hex("#39ff14"), 12.0);
    c.fill_circle(cx, cy, bio_r * 0.22);

    c.set_global_alpha(1.0);
    c.restore();
}
