//! Neon Audio Prism 3D style renderer (`audioPrism3D`) — Optical Dispersion Prism Engine.
//!
//! Masterpiece Optical Prism Refraction:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - White laser beam entering the 3D glass crystal prism from the left.
//! - Continuous audio-reactive rainbow dispersion fan refracting out of the prism to the right (NO surrounding sticks!).
//! - Rotating 3D crystal pyramid core with glass specular reflections & inner spectrum dispersion.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const DISPERSION_BEAMS: usize = 32;

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

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep optic space backdrop
    c.set_fill(Fill::Solid(Color::hex("#020108")));
    c.fill_rect(0.0, 0.0, width, height);

    // Prism optics atmospheric glow
    let prism_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.70 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.35 + be * 0.20), 0.5)),
            (0.40, mix(accent_col, Color::rgba(1.0, 0.10, 0.70, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.10, 0.0, 0.30, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(prism_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. INCOMING INTENSE WHITE LASER BEAM (FROM LEFT TO PRISM)
    // -------------------------------------------------------------------------
    let prism_x = cx;
    let prism_y = cy;
    let beam_in_start_x = 0.0;
    let beam_in_start_y = cy + (frame_time * 0.5).sin() * (15.0 * user_scale);

    // Outer Laser Beam Glow
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.40)));
    c.set_line_width((16.0 + be * 8.0) * user_scale);
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.80), 20.0 * user_scale);
    c.stroke_line(beam_in_start_x, beam_in_start_y, prism_x - 30.0 * user_scale, prism_y);

    // Intense White Core Beam
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.98)));
    c.set_line_width((4.0 + be * 3.0) * user_scale);
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.95), 10.0 * user_scale);
    c.stroke_line(beam_in_start_x, beam_in_start_y, prism_x - 30.0 * user_scale, prism_y);

    // -------------------------------------------------------------------------
    // 2. CONFIGURE NATIVE 3D CRYSTAL PRISM (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    scene.cam_yaw = (frame_time * 0.15).sin() * 0.10;
    scene.cam_pitch = -0.16 + (frame_time * 0.08).sin() * 0.04;
    scene.cam_zoom = (0.95 - be * 0.05) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    // Floating 3D Crystalline Prism Pyramid Core
    let prism_sz = (110.0 + be * 25.0 * sensitivity) * user_scale;
    let rotation = frame_time * 0.40;

    for layer in 0..6 {
        let l_f = layer as f32;
        let l_sz = prism_sz * (1.0 - l_f * 0.14);
        let l_y = (l_f - 2.5) * 20.0 * user_scale;

        scene.push();
        scene.translate(0.0, l_y, 0.0);
        scene.rotate_y(rotation + l_f * 0.10);
        scene.add_box(
            0.0,
            0.0,
            0.0,
            l_sz,
            16.0 * user_scale,
            l_sz,
            mix(p_col, Color::rgba(0.95, 0.98, 1.0, 0.92), l_f / 6.0),
        );
        scene.pop();
    }

    // -------------------------------------------------------------------------
    // 3. OUTGOING CONTINUOUS AUDIO-REACTIVE RAINBOW DISPERSION FAN (TO RIGHT)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);
    let disp_start_x = prism_x + 30.0 * user_scale;
    let disp_end_x = width;

    for b in 0..DISPERSION_BEAMS {
        let b_f = b as f32;
        let t_spread = (b_f / (DISPERSION_BEAMS - 1) as f32) - 0.5; // -0.5 to +0.5 fan spread

        let bin_k = (b * step_f / (DISPERSION_BEAMS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let end_y = cy + t_spread * (height * 0.85 * user_scale) + (t_spread * 12.0).sin() * (fv * 60.0 * sensitivity);

        // Rainbow spectral hue gradient across the fan
        let ray_col = mix(
            mix(glow_col, Color::rgba(1.0, 0.10, 0.50, 0.90), t_spread + 0.5),
            mix(p_col, accent_col, fv),
            fv,
        );

        c.set_stroke(Fill::Solid(ray_col));
        c.set_line_width((4.0 + fv * 6.0) * user_scale);
        c.set_shadow(ray_col, (12.0 + fv * 8.0) * user_scale);
        c.stroke_line(disp_start_x, prism_y, disp_end_x, end_y);
    }

    // Central Refraction Crystal Point Flare
    c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.98)));
    c.set_shadow(glow_col, (24.0 + bs * 12.0) * user_scale);
    c.fill_circle(prism_x, prism_y, 16.0 * user_scale);

    c.set_global_alpha(1.0);
    c.restore();
}
