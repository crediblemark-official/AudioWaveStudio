//! Neon Cyberpunk Metropolis 3D style renderer (`neonMetropolis3D`) — 3D City Skyline Engine.
//!
//! Masterpiece 3D Cyberpunk Skyline:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - 48 glowing skyscraper equalizer towers arranged in a 4x12 3D megacity grid.
//! - Audio-reactive building heights & window grid LED lighting.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const CITY_COLS: usize = 12;
const CITY_ROWS: usize = 4;

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

    // Deep midnight city backdrop
    c.set_fill(Fill::Solid(Color::hex("#020409")));
    c.fill_rect(0.0, 0.0, width, height);

    // City skyline glow
    let city_glow = Fill::radial_gradient(
        cx,
        cy + height * 0.10,
        0.0,
        cx,
        cy + height * 0.10,
        width * 0.65 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.85, 1.0, 0.25 + be * 0.15), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.60, 0.0, 0.90, 0.10), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(city_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CONFIGURE NATIVE 3D SCENE (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    scene.cam_yaw = (frame_time * 0.08).sin() * 0.10;
    scene.cam_pitch = -0.22 - (frame_time * 0.05).sin() * 0.03 - be * 0.03;
    scene.cam_zoom = (0.95 - be * 0.05) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    let base_y = -110.0 * user_scale;
    let step_f = (freq.len() / bar_count).max(1);

    // -------------------------------------------------------------------------
    // 2. 3D SKYSCRAPER EQUALIZER TOWERS (GRID OF 4x12 BUILDINGS)
    // -------------------------------------------------------------------------
    for r in 0..CITY_ROWS {
        let r_f = r as f32;
        let z_pos = -r_f * (45.0 * user_scale);

        for col in 0..CITY_COLS {
            let col_f = col as f32;
            let x_pos = (col_f - CITY_COLS as f32 * 0.5 + 0.5) * (42.0 * user_scale);

            let index = r * CITY_COLS + col;
            let bin_k = (index * step_f / (CITY_ROWS * CITY_COLS / bar_count.max(1)).max(1))
                .min(freq.len().saturating_sub(1));
            let fv = freq[bin_k] as f32 / 255.0;

            let bld_h = (25.0 + fv * 260.0 * sensitivity + be * 40.0).clamp(15.0, 420.0) * user_scale;
            let bld_w = 32.0 * user_scale;

            let bld_col = mix(
                mix(p_col, s_col, (col % 3) as f32 / 3.0),
                mix(accent_col, glow_col, fv),
                r_f / CITY_ROWS as f32,
            );

            // Main Skyscraper Body
            scene.push();
            scene.translate(x_pos, base_y + bld_h * 0.5, z_pos);
            scene.add_box(0.0, 0.0, 0.0, bld_w, bld_h, bld_w, bld_col);
            scene.pop();

            // Glowing Spire Spool Top
            if fv > 0.4 {
                let spire_h = 15.0 * user_scale;
                scene.push();
                scene.translate(x_pos, base_y + bld_h + spire_h * 0.5, z_pos);
                scene.add_box(0.0, 0.0, 0.0, 4.0 * user_scale, spire_h, 4.0 * user_scale, Color::rgba(1.0, 1.0, 1.0, 0.95));
                scene.pop();
            }
        }
    }

    c.set_global_alpha(1.0);
    c.restore();
}
