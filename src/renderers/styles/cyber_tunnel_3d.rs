//! Geometric Cyber Tunnel Matrix 3D style renderer (`cyberTunnel3D`) — 3D Hollow Tunnel Engine.
//!
//! Masterpiece 3D Hollow Tunnel:
//! - 16 hollow 3D tunnel rings receding into deep perspective space (center is 100% open for camera view).
//! - 4 perimeter wall panels (Top, Bottom, Left, Right) pulsing with audio frequencies.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const TUNNEL_RINGS: usize = 20;

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

    // Deep cyberspace backdrop
//     c.set_fill(Fill::Solid(Color::hex("#010207")));
//     c.fill_rect(0.0, 0.0, width, height);

    // Tunnel vanishing point ambient glow
    let tunnel_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.60 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.35), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.70, 0.0, 0.90, 0.15), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(tunnel_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CONFIGURE NATIVE 3D SCENE (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    scene.cam_yaw = (frame_time * 0.12).sin() * 0.08;
    scene.cam_pitch = (frame_time * 0.08).cos() * 0.06;
    scene.cam_zoom = 0.95 / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    let step_f = (freq.len() / bar_count).max(1);
    let ring_depth = 12.0 * user_scale;
    let wall_thick = 16.0 * user_scale;

    // Render 20 Hollow 3D Tunnel Perimeter Rings
    for r in 0..TUNNEL_RINGS {
        let r_f = r as f32;
        let z_pos = -r_f * (45.0 * user_scale) + (frame_time * 120.0 % (45.0 * user_scale));

        let bin_k = (r * step_f / (TUNNEL_RINGS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let tunnel_sz = (240.0 + fv * 90.0 * sensitivity + be * 40.0) * user_scale;
        let half_sz = tunnel_sz * 0.5;

        let ring_col = mix(
            mix(p_col, glow_col, fv),
            mix(accent_col, s_col, r_f / TUNNEL_RINGS as f32),
            r_f / TUNNEL_RINGS as f32,
        );

        // A. Top Wall Panel
        scene.push();
        scene.translate(0.0, half_sz, z_pos);
        scene.add_box(0.0, 0.0, 0.0, tunnel_sz, wall_thick, ring_depth, ring_col);
        scene.pop();

        // B. Bottom Wall Panel
        scene.push();
        scene.translate(0.0, -half_sz, z_pos);
        scene.add_box(0.0, 0.0, 0.0, tunnel_sz, wall_thick, ring_depth, ring_col);
        scene.pop();

        // C. Left Wall Panel
        scene.push();
        scene.translate(-half_sz, 0.0, z_pos);
        scene.add_box(0.0, 0.0, 0.0, wall_thick, tunnel_sz, ring_depth, ring_col);
        scene.pop();

        // D. Right Wall Panel
        scene.push();
        scene.translate(half_sz, 0.0, z_pos);
        scene.add_box(0.0, 0.0, 0.0, wall_thick, tunnel_sz, ring_depth, ring_col);
        scene.pop();
    }

    c.set_global_alpha(1.0);
    c.restore();
}
