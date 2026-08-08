//! Cyber Black Hole Horizon style renderer (`cyberBlackHole`) — Gravitational Accretion Visualizer.
//!
//! Features:
//! - Pitch-black event horizon core with dark gravitational lensing shadow.
//! - 360° rotating neon cyan & violet accretion disk.
//! - Audio-reactive Hawking radiation plasma jets & radial shockwave rings.
//! - Smooth, ultra-fluid spectrum tendrils pulling into the gravitational center.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const JET_COUNT: usize = 64;

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
    let base_radius = 85.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. DEEP GRAVITATIONAL ATMOSPHERIC BACKDROP
    // -------------------------------------------------------------------------
    c.set_fill(Fill::Solid(Color::hex("#020208")));
    c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx,
        cy,
        base_radius * 0.5,
        cx,
        cy,
        base_radius * 3.8,
        &[
            (0.0, Color::rgba(0.0, 0.70, 1.0, 0.22 + be * 0.15)),
            (0.40, Color::rgba(0.50, 0.0, 0.90, 0.12)),
            (0.80, Color::rgba(0.10, 0.0, 0.25, 0.04)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 2. ACCRETION DISK RADIAL FREQUENCY JETS (360°)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..JET_COUNT {
        let i_f = i as f32;
        let angle = (i_f / JET_COUNT as f32) * TAU + frame_time * 0.12;

        let bin_k = (i * step_f / (JET_COUNT / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let jet_len = 20.0 + fv * 180.0 * sensitivity + be * 45.0;
        let inner_r = base_radius * (1.0 + be * 0.10);
        let outer_r = inner_r + jet_len;

        let (sin_a, cos_a) = angle.sin_cos();

        let x0 = cx + cos_a * inner_r;
        let y0 = cy + sin_a * inner_r;
        let x1 = cx + cos_a * outer_r;
        let y1 = cy + sin_a * outer_r;

        let jet_w = 6.0 + fv * 8.0;
        let perp_x = -sin_a * (jet_w * 0.5);
        let perp_y = cos_a * (jet_w * 0.5);

        let jet_pts = vec![
            (x0 - perp_x, y0 - perp_y),
            (x0 + perp_x, y0 + perp_y),
            (x1 + perp_x * 0.2, y1 + perp_y * 0.2),
            (x1 - perp_x * 0.2, y1 - perp_y * 0.2),
        ];

        let jet_col = mix(
            Color::rgba(0.0, 0.90, 1.0, 0.90 + bs * 0.10),
            Color::rgba(0.70, 0.15, 1.0, 0.65),
            fv,
        );
        c.set_fill(Fill::Solid(jet_col));
        c.fill_polygon(&jet_pts);
    }

    // -------------------------------------------------------------------------
    // 3. EVENT HORIZON CORE & PHOTON SPHERE RING
    // -------------------------------------------------------------------------
    // Glowing Photon Sphere Ring
    let photon_r = base_radius * (1.04 + be * 0.08);
    c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.95, 1.0, 0.90)));
    c.set_line_width(3.5);
    c.set_shadow(Color::rgba(0.0, 0.85, 1.0, 0.95), 18.0);
    c.stroke_circle(cx, cy, photon_r);

    // Pitch-black Event Horizon Core
    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, base_radius * 0.98);

    // Subtle Inner Lensing Glow
    let inner_lensing = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        base_radius * 0.95,
        &[
            (0.0, Color::rgba(0.0, 0.0, 0.0, 1.0)),
            (0.75, Color::rgba(0.0, 0.20, 0.45, 0.25)),
            (1.0, Color::rgba(0.0, 0.90, 1.0, 0.85)),
        ],
    );
    c.set_fill(inner_lensing);
    c.fill_circle(cx, cy, base_radius * 0.95);

    c.set_global_alpha(1.0);
    c.restore();
}
