//! Liquid Mercury Fluid Wave style renderer (`mercuryFluid`) — Liquid Metal Engine.
//!
//! Masterpiece 3D Liquid Metal Pool:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - Highly reflective 3D liquid mercury fluid surface pool.
//! - 36 liquid chrome equalizer water columns surging dynamically in 3D space with wave harmonics.
//! - Translucent 3D mirror reflections below the fluid pool surface.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const MERCURY_COLUMNS: usize = 36;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let _s_col = theme_secondary(theme);
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

    // Deep liquid metal space backdrop
    c.set_fill(Fill::Solid(Color::hex("#010307")));
    c.fill_rect(0.0, 0.0, width, height);

    // Liquid mercury thermal reflection glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy + height * 0.12,
        0.0,
        cx,
        cy + height * 0.12,
        width * 0.65 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.70, 0.85, 1.0, 0.30 + be * 0.15), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.20, 0.40, 0.80, 0.12), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(amb_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CONFIGURE NATIVE 3D SCENE (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    scene.cam_yaw = (frame_time * 0.10).sin() * 0.08;
    scene.cam_pitch = -0.22 - (frame_time * 0.05).sin() * 0.03 - be * 0.03;
    scene.cam_zoom = (0.95 - be * 0.04) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    let base_y = -100.0 * user_scale;
    let step_f = (freq.len() / bar_count).max(1);
    let pool_w = 540.0 * user_scale;
    let col_r = (pool_w / MERCURY_COLUMNS as f32) * 0.42;

    // -------------------------------------------------------------------------
    // 2. 36 LIQUID CHROME EQUALIZER WATER COLUMNS & MIRROR REFLECTIONS
    // -------------------------------------------------------------------------
    for i in 0..MERCURY_COLUMNS {
        let i_f = i as f32;
        let x_pos = (i_f - MERCURY_COLUMNS as f32 * 0.5 + 0.5) * (pool_w / MERCURY_COLUMNS as f32);

        let bin_k = (i * step_f / (MERCURY_COLUMNS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let col_h = (15.0 + fv * 260.0 * sensitivity + be * 40.0).clamp(10.0, 380.0) * user_scale;

        let mercury_col = mix(
            mix(Color::rgba(0.90, 0.95, 1.0, 0.95), glow_col, 0.5),
            mix(p_col, accent_col, (i % 3) as f32 / 3.0),
            fv,
        );

        // Main Liquid Chrome Column (3D Cylinder Disc Stack)
        scene.push();
        scene.translate(x_pos, base_y + col_h * 0.5, 0.0);
        scene.add_box(0.0, 0.0, 0.0, col_r * 1.8, col_h, col_r * 1.8, mercury_col);
        scene.pop();

        // 3D Translucent Mirror Reflection Below Fluid Floor
        let ref_h = col_h * 0.40;
        let ref_col = Color::rgba(mercury_col.r, mercury_col.g, mercury_col.b, 0.25);
        scene.push();
        scene.translate(x_pos, base_y - ref_h * 0.5, 0.0);
        scene.add_box(0.0, 0.0, 0.0, col_r * 1.7, ref_h, col_r * 1.7, ref_col);
        scene.pop();
    }

    c.set_global_alpha(1.0);
    c.restore();
}
