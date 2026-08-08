//! Supernova Shockwave Burst style renderer (`supernovaBurst`) — Stellar Explosion Engine.
//!
//! Features:
//! - Blinding white dwarf star core with expanding radial plasma shockwaves.
//! - 360° gold & cyan spectrum ray bursts reacting to audio frequency spikes.
//! - 50+ floating starlight embers accelerating outward into space.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const RAY_COUNT: usize = 72;
const STAR_EMBERS: usize = 50;

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
    let base_r = 75.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep cosmic space backdrop
    c.set_fill(Fill::Solid(Color::hex("#03020a")));
    c.fill_rect(0.0, 0.0, width, height);

    // Radial supernova glow aura
    let super_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.2,
        cx,
        cy,
        base_r * 4.2,
        &[
            (0.0, Color::rgba(1.0, 0.90, 0.40, 0.35 + be * 0.25)),
            (0.35, Color::rgba(0.0, 0.85, 1.0, 0.15)),
            (0.75, Color::rgba(0.40, 0.0, 0.80, 0.05)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(super_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. EXPANDING RADIAL SHOCKWAVE RINGS
    // -------------------------------------------------------------------------
    for shock_i in 0..3 {
        let shock_t = ((frame_time * 0.6 + shock_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let shock_r = base_r + shock_t * (width * 0.45 + be * 50.0);
        let shock_alpha = (1.0 - shock_t).powi(2) * (0.65 + bs * 0.30);

        let shock_col = mix(
            Color::rgba(1.0, 0.90, 0.30, shock_alpha),
            Color::rgba(0.0, 0.80, 1.0, shock_alpha * 0.5),
            shock_t,
        );
        c.set_stroke(Fill::Solid(shock_col));
        c.set_line_width(4.0 * (1.0 - shock_t) + 1.0);
        c.stroke_circle(cx, cy, shock_r);
    }

    // -------------------------------------------------------------------------
    // 2. 360° PLASMA RAY BURSTS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..RAY_COUNT {
        let angle = (i as f32 / RAY_COUNT as f32) * TAU + frame_time * 0.08;
        let bin_k = (i * step_f / (RAY_COUNT / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let ray_h = 15.0 + fv * 160.0 * sensitivity + be * 40.0;
        let r0 = base_r * (1.0 + be * 0.12);
        let r1 = r0 + ray_h;

        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let ray_w = 5.0 + fv * 7.0;
        let px = -sin_a * (ray_w * 0.5);
        let py = cos_a * (ray_w * 0.5);

        let pts = vec![
            (x0 - px, y0 - py),
            (x0 + px, y0 + py),
            (x1 + px * 0.2, y1 + py * 0.2),
            (x1 - px * 0.2, y1 - px * 0.2),
        ];

        let ray_col = mix(
            Color::rgba(1.0, 0.95, 0.60, 0.90),
            Color::rgba(0.0, 0.85, 1.0, 0.70),
            i as f32 / RAY_COUNT as f32,
        );
        c.set_fill(Fill::Solid(ray_col));
        c.fill_polygon(&pts);
    }

    // -------------------------------------------------------------------------
    // 3. BLINDING WHITE DWARF STAR CORE
    // -------------------------------------------------------------------------
    let core_r = base_r * (0.85 + be * 0.20 + bs * 0.10);
    let core_grad = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 1.0)),
            (0.40, Color::rgba(1.0, 0.92, 0.60, 0.95)),
            (0.75, Color::rgba(0.0, 0.80, 1.0, 0.80)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(core_grad);
    c.set_shadow(Color::rgba(1.0, 0.90, 0.50, 1.0), 24.0);
    c.fill_circle(cx, cy, core_r);

    // -------------------------------------------------------------------------
    // 4. FLOATING STARLIGHT EMBERS
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    for e_i in 0..STAR_EMBERS {
        let e_t = ((frame_time * 0.35 + e_i as f32 * 0.07) % 1.0).clamp(0.0, 1.0);
        let er = base_r + e_t * (width * 0.40);
        let e_ang = e_i as f32 * 0.8 + frame_time * 0.2;

        let ex = cx + e_ang.cos() * er;
        let ey = cy + e_ang.sin() * er;

        let esz = (4.5 * (1.0 - e_t) + 1.0).clamp(1.0, 6.0);
        let ecol = Color::rgba(1.0, 0.90, 0.50, (1.0 - e_t) * 0.85);

        c.set_fill(Fill::Solid(ecol));
        c.fill_ellipse(ex, ey, esz, esz);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
