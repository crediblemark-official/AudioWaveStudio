//! Neon Audio Prism 3D style renderer (`audioPrism3D`) — 3D Crystal Refraction Engine.
//!
//! Masterpiece 3D Crystal Refraction:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - Large rotating 3D crystalline pyramid prism core with specular glass reflections.
//! - 32 spectral rainbow audio light beams refracting dynamically out of the crystal faces.
//! - Audio-reactive rainbow light dispersion & crystal aura flares.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const REFRACTED_BEAMS: usize = 32;

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

    // Deep crystal space backdrop
    c.set_fill(Fill::Solid(Color::hex("#020108")));
    c.fill_rect(0.0, 0.0, width, height);

    // Crystal prism aura
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
    // 1. CONFIGURE NATIVE 3D SCENE (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    scene.cam_yaw = (frame_time * 0.15).sin() * 0.10;
    scene.cam_pitch = -0.16 + (frame_time * 0.08).sin() * 0.04;
    scene.cam_zoom = (0.95 - be * 0.05) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    // Floating 3D Crystalline Prism Core (Octahedron Glass Layers)
    let prism_sz = (90.0 + be * 25.0 * sensitivity) * user_scale;
    let rotation = frame_time * 0.60;

    for layer in 0..6 {
        let l_f = layer as f32;
        let l_sz = prism_sz * (1.0 - l_f * 0.14);
        let l_y = (l_f - 2.5) * 18.0 * user_scale;

        scene.push();
        scene.translate(0.0, l_y, 0.0);
        scene.rotate_y(rotation + l_f * 0.10);
        scene.add_box(
            0.0,
            0.0,
            0.0,
            l_sz,
            14.0 * user_scale,
            l_sz,
            mix(p_col, Color::rgba(0.95, 0.98, 1.0, 0.90), l_f / 6.0),
        );
        scene.pop();
    }

    // -------------------------------------------------------------------------
    // 2. 32 REFRACTED SPECTRAL RAINBOW LIGHT BEAMS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for b in 0..REFRACTED_BEAMS {
        let b_f = b as f32;
        let angle = (b_f / REFRACTED_BEAMS as f32) * TAU + frame_time * 0.12;

        let bin_k = (b * step_f / (REFRACTED_BEAMS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let ray_len = (30.0 + fv * 160.0 * sensitivity + be * 40.0) * user_scale;
        let r0 = prism_sz * 0.65;
        let r1 = r0 + ray_len;

        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let ray_col = mix(
            mix(p_col, glow_col, b_f / REFRACTED_BEAMS as f32),
            mix(accent_col, s_col, (b % 4) as f32 / 4.0),
            fv,
        );

        c.set_stroke(Fill::Solid(ray_col));
        c.set_line_width((3.0 + fv * 4.0) * user_scale);
        c.set_shadow(ray_col, (12.0 + fv * 8.0) * user_scale);
        c.stroke_line(x0, y0, x1, y1);
    }

    // Central Refraction Crystal Flare
    c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.95)));
    c.set_shadow(glow_col, (18.0 + bs * 10.0) * user_scale);
    c.fill_circle(cx, cy, 14.0 * user_scale);

    c.set_global_alpha(1.0);
    c.restore();
}
