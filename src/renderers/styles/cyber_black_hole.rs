//! Cyber Black Hole Horizon style renderer (`cyberBlackHole`) — Gravitational Accretion Engine.
//!
//! Masterpiece Black Hole Event Horizon:
//! - Pitch-black central Event Horizon shadow core with blinding white-hot Photon Ring.
//! - 8 3D gravitational lensing accretion plasma rings swirling around the event horizon (NO needle spikes!).
//! - Audio-reactive Hawking radiation plasma pulses & gravitational redshift color gradients.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const ACCRETION_RINGS: usize = 8;

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
    let cy = height * 0.5 + pos_offset_y;
    let reference_size = width.min(height);
    let base_radius = 115.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Gravitational Lensing Thermal Glow
    let bg_glow = Fill::radial_gradient(
        cx,
        cy,
        base_radius * 0.5,
        cx,
        cy,
        base_radius * 3.8,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.85, 1.0, 0.35 + be * 0.15), 0.5)),
            (0.40, mix(p_col, Color::rgba(0.60, 0.0, 0.90, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.10, 0.0, 0.25, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. 8 GRAVITATIONAL ACCRETION PLASMA RINGS (SWIRLING LENSING DISK)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for r_i in 1..=ACCRETION_RINGS {
        let r_f = r_i as f32;
        let bin_k = (r_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let rx = base_radius * (0.60 + r_f * 0.22 + fv * 0.06 * sensitivity);
        let ry = rx * 0.38; // Gravitational tilt perspective

        let ring_col = mix(
            mix(p_col, glow_col, r_f / ACCRETION_RINGS as f32),
            mix(accent_col, Color::rgba(1.0, 0.15, 0.80, 0.90), fv),
            fv,
        );

        c.save();
        c.translate(cx, cy);
        c.rotate(r_f * 0.18 + frame_time * 0.25);

        c.set_fill(Fill::Solid(Color::rgba(ring_col.r, ring_col.g, ring_col.b, 0.40)));
        c.set_stroke(Fill::Solid(ring_col));
        c.set_line_width((3.0 + fv * 4.0) * user_scale);
        c.set_shadow(ring_col, (14.0 + fv * 10.0) * user_scale);
        c.fill_ellipse(0.0, 0.0, rx, ry);
        c.restore();
    }

    // -------------------------------------------------------------------------
    // 2. BLINDING WHITE-HOT PHOTON RING
    // -------------------------------------------------------------------------
    let photon_r = base_radius * (0.52 + be * 0.06);
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.95)));
    c.set_line_width(3.5 * user_scale);
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.90), (16.0 + bs * 8.0) * user_scale);
    c.stroke_circle(cx, cy, photon_r);

    // -------------------------------------------------------------------------
    // 3. PITCH-BLACK EVENT HORIZON SHADOW CORE
    // -------------------------------------------------------------------------
    let horizon_r = base_radius * 0.48;
    c.set_fill(Fill::Solid(Color::hex("#000002")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, horizon_r);

    // Center image inside the event horizon (drawn last = on top)
    draw_radial_center_image(c, ctx, cx, cy, horizon_r * 0.88);

    c.set_global_alpha(1.0);
    c.restore();
}
