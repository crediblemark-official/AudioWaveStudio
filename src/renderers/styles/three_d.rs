//! Cinematic 3D Block Matrix style renderer (`threeD`) — Native 3D Retained Scene Engine.
//!
//! Upgraded Masterpiece:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - Dynamic 3D perspective camera with smooth yaw, pitch, and audio-reactive zoom.
//! - 48 volumetric 3D metallic spectrum boxes with 3D translucent mirror reflections.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const BLOCKS_3D_COUNT: usize = 48;

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

    // Deep 3D space backdrop
//     c.set_fill(Fill::Solid(Color::hex("#020308")));
//     c.fill_rect(0.0, 0.0, width, height);

    // Ambient thermal glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy + height * 0.10,
        0.0,
        cx,
        cy + height * 0.10,
        width * 0.65 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.85, 1.0, 0.22), 0.5)),
            (0.50, mix(p_col, Color::rgba(0.60, 0.0, 0.90, 0.08), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(amb_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CONFIGURE NATIVE 3D SCENE (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    scene.cam_yaw = (frame_time * 0.12).sin() * 0.10;
    scene.cam_pitch = -0.22 - (frame_time * 0.08).sin() * 0.04 - be * 0.03;
    scene.cam_zoom = (0.95 - be * 0.05) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    let base_y = -100.0 * user_scale;
    let step_f = (freq.len() / bar_count).max(1);
    let total_w = 400.0 * user_scale;
    let block_w = (total_w / BLOCKS_3D_COUNT as f32) * 0.80;

    for i in 0..BLOCKS_3D_COUNT {
        let i_f = i as f32;
        let x_pos = (i_f - BLOCKS_3D_COUNT as f32 * 0.5 + 0.5) * (total_w / BLOCKS_3D_COUNT as f32);

        let bin_k = (i * step_f / (BLOCKS_3D_COUNT / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let block_h = (20.0 + fv * 280.0 * sensitivity + be * 40.0).clamp(15.0, 400.0) * user_scale;

        let block_col = mix(
            mix(p_col, s_col, i_f / BLOCKS_3D_COUNT as f32),
            mix(accent_col, glow_col, fv),
            fv,
        );

        // Main Upper 3D Box
        scene.push();
        scene.translate(x_pos, base_y + block_h * 0.5, 0.0);
        scene.add_box(0.0, 0.0, 0.0, block_w, block_h, block_w, block_col);
        scene.pop();

        // Translucent 3D Reflection Box Below Floor
        let ref_h = block_h * 0.40;
        let ref_col = Color::rgba(block_col.r, block_col.g, block_col.b, 0.25);
        scene.push();
        scene.translate(x_pos, base_y - ref_h * 0.5, 0.0);
        scene.add_box(0.0, 0.0, 0.0, block_w * 0.95, ref_h, block_w * 0.95, ref_col);
        scene.pop();
    }

    c.set_global_alpha(1.0);
    c.restore();
}
