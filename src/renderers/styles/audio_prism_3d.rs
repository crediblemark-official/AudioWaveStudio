//! Neon Audio Prism 3D style renderer (`audioPrism3D`) — 3D Crystal Refraction Engine.
//!
//! Masterpiece 3D Crystal Refraction:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - Rotating 3D crystal prism core floating in 3D perspective space.
//! - 36 spectral audio rainbow rays refracting dynamically out of the crystal faces.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const REFRACTED_RAYS: usize = 36;

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
    let _bs = ctx.beat_strength.clamp(0.0, 1.0);
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
        width * 0.65 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.35 + be * 0.15), 0.5)),
            (0.40, mix(accent_col, Color::rgba(1.0, 0.10, 0.70, 0.15), 0.5)),
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

    scene.cam_yaw = frame_time * 0.30;
    scene.cam_pitch = -0.18 + (frame_time * 0.12).sin() * 0.08;
    scene.cam_zoom = (0.95 - be * 0.04) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    // A. Floating 3D Crystal Prism Core
    let prism_sz = (55.0 + be * 20.0 * sensitivity) * user_scale;
    scene.push();
    scene.rotate_y(frame_time * 0.8);
    scene.add_box(0.0, 0.0, 0.0, prism_sz, prism_sz * 1.5, prism_sz, mix(p_col, Color::rgba(1.0, 1.0, 1.0, 0.90), 0.6));
    scene.pop();

    // B. 36 3D Refracted Spectral Rainbow Rays
    let step_f = (freq.len() / bar_count).max(1);

    for r_i in 0..REFRACTED_RAYS {
        let r_f = r_i as f32;
        let angle = (r_f / REFRACTED_RAYS as f32) * TAU + frame_time * 0.25;

        let bin_k = (r_i * step_f / (REFRACTED_RAYS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let ray_len = (80.0 + fv * 200.0 * sensitivity + be * 40.0) * user_scale;
        let r0 = prism_sz * 0.8;
        let r_center = r0 + ray_len * 0.5;

        let rx = angle.cos() * r_center;
        let ry = (angle * 2.0).sin() * 20.0 * user_scale;
        let rz = angle.sin() * r_center;

        let ray_col = mix(
            mix(p_col, glow_col, r_f / REFRACTED_RAYS as f32),
            mix(accent_col, s_col, (r_i % 3) as f32 / 3.0),
            fv,
        );

        scene.push();
        scene.translate(rx, ry, rz);
        scene.rotate_y(angle);
        scene.add_box(0.0, 0.0, 0.0, 5.0 * user_scale, 5.0 * user_scale, ray_len, ray_col);
        scene.pop();
    }

    c.set_global_alpha(1.0);
    c.restore();
}
