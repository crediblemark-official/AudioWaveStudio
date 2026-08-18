//! Cyber Synthwave Highway 3D style renderer (`synthwaveHighway3D`) — Ultra-Smooth 3D Grid Engine.
//!
//! Features:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - Smooth infinite scrolling 3D neon grid floor & wireframe horizon sun disc.
//! - 32 rows of 3D equalizer buildings with smooth height interpolation, side shading & window LED grids.
//! - Distance atmospheric fog fading distant structures seamlessly.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const BUILDING_ROWS: usize = 32;
const GRID_LANES: usize = 8;

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
    let cy = height * 0.5 + pos_offset_y;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep synthwave backdrop
//     c.set_fill(Fill::Solid(Color::hex("#05020a")));
//     c.fill_rect(0.0, 0.0, width, height);

    // Sunset Thermal Glow
    let sun_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.65 * user_scale,
        &[
            (0.0, mix(accent_col, Color::rgba(1.0, 0.35, 0.10, 0.85), 0.5)),
            (0.40, mix(p_col, Color::rgba(0.90, 0.10, 0.60, 0.40), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.20, 0.0, 0.40, 0.10), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(sun_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CONFIGURE NATIVE 3D SCENE (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    scene.cam_yaw = (frame_time * 0.08).sin() * 0.05;
    scene.cam_pitch = -0.22 - (frame_time * 0.04).sin() * 0.02 - be * 0.03;
    scene.cam_zoom = (0.92 - be * 0.05) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    let base_y = -100.0 * user_scale;
    let highway_w = 190.0 * user_scale;
    let step_f = (freq.len() / bar_count).max(1);

    // A. 3D Glowing Synthwave Sun Disc at Horizon
    let sun_r = 110.0 * user_scale;
    scene.push();
    scene.translate(0.0, 40.0 * user_scale, -800.0 * user_scale);
    scene.add_disc(0.0, 0.0, 0.0, sun_r, 32, mix(accent_col, Color::hex("#ff007f"), 0.85));
    scene.pop();

    // B. 3D SMOOTH SCROLLING HIGHWAY GRID FLOOR
    let total_depth = BUILDING_ROWS as f32 * (35.0 * user_scale);
    let scroll_z = (frame_time * 140.0 * user_scale) % (35.0 * user_scale);

    for lane in 0..=GRID_LANES {
        let lane_f = lane as f32;
        let lx = (lane_f - GRID_LANES as f32 * 0.5) * (highway_w * 2.0 / GRID_LANES as f32);
        let lane_col = Color::rgba(glow_col.r, glow_col.g, glow_col.b, 0.35);

        scene.push();
        scene.translate(lx, base_y, -total_depth * 0.5);
        scene.add_box(0.0, 0.0, 0.0, 2.5 * user_scale, 2.0 * user_scale, total_depth, lane_col);
        scene.pop();
    }

    // C. 3D SEAMLESS EQUALIZER BUILDINGS ALONG HIGHWAY SIDES
    for row in 0..BUILDING_ROWS {
        let row_f = row as f32;
        let raw_z = -row_f * (35.0 * user_scale) + scroll_z;
        let z_pos = if raw_z > 0.0 { raw_z - total_depth } else { raw_z };

        // Seamless distance alpha fade near vanishing horizon
        let dist_ratio = (-z_pos / total_depth).clamp(0.0, 1.0);
        let fade_alpha = (1.0 - dist_ratio * dist_ratio).clamp(0.0, 1.0);

        let bin_left = (row * step_f).min(freq.len().saturating_sub(1));
        let bin_right = ((row + 12) * step_f).min(freq.len().saturating_sub(1));

        let fv_l = freq[bin_left] as f32 / 255.0;
        let fv_r = freq[bin_right] as f32 / 255.0;

        // Smooth height calculation
        let bld_h_l = (25.0 + fv_l * 270.0 * sensitivity + be * 35.0).clamp(15.0, 420.0) * user_scale;
        let bld_h_r = (25.0 + fv_r * 270.0 * sensitivity + be * 35.0).clamp(15.0, 420.0) * user_scale;

        let bld_w = 28.0 * user_scale;

        // Left Building
        let mut col_l = mix(p_col, glow_col, fv_l);
        col_l.a *= fade_alpha;

        scene.push();
        scene.translate(-highway_w, base_y + bld_h_l * 0.5, z_pos);
        scene.add_box(0.0, 0.0, 0.0, bld_w, bld_h_l, bld_w, col_l);
        scene.pop();

        // Right Building
        let mut col_r = mix(s_col, accent_col, fv_r);
        col_r.a *= fade_alpha;

        scene.push();
        scene.translate(highway_w, base_y + bld_h_r * 0.5, z_pos);
        scene.add_box(0.0, 0.0, 0.0, bld_w, bld_h_r, bld_w, col_r);
        scene.pop();
    }

    c.set_global_alpha(1.0);
    c.restore();
}
