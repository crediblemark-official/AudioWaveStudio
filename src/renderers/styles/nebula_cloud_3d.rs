//! Cosmic Nebula Particle Cloud 3D style renderer (`nebulaCloud3D`) — Volumetric Space Engine.
//!
//! Masterpiece 3D Cosmic Cloud:
//! - Glowing 3D central star core surrounded by 120 swirling 3D starlight particle discs.
//! - Audio-reactive double-helix spiral orbits & bass shockwave compression.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const NEBULA_PARTICLES: usize = 120;

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

    // Deep cosmic space backdrop
    c.set_fill(Fill::Solid(Color::hex("#010207")));
    c.fill_rect(0.0, 0.0, width, height);

    // Volumetric nebula core glow
    let nebula_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.65 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.35 + be * 0.15), 0.5)),
            (0.40, mix(p_col, Color::rgba(0.80, 0.10, 0.60, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.20, 0.0, 0.40, 0.05), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(nebula_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CONFIGURE NATIVE 3D SCENE (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    scene.cam_yaw = frame_time * 0.20;
    scene.cam_pitch = -0.18 + (frame_time * 0.10).sin() * 0.08;
    scene.cam_zoom = (0.92 - be * 0.04) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    // Central Glowing Star Core
    let core_r = (24.0 + be * 16.0 * sensitivity) * user_scale;
    scene.push();
    scene.add_disc(0.0, 0.0, 0.0, core_r, 32, mix(glow_col, Color::rgba(1.0, 1.0, 1.0, 0.95), 0.7));
    scene.pop();

    // -------------------------------------------------------------------------
    // 2. 120 SWIRLING 3D STARLIGHT PARTICLE DISCS IN DOUBLE HELIX ORBITS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for p_i in 0..NEBULA_PARTICLES {
        let p_f = p_i as f32;
        let bin_k = (p_i * step_f / (NEBULA_PARTICLES / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        // Double-helix orbital radius & angle
        let orbit_r = (40.0 + p_f * 2.2 + fv * 80.0 * sensitivity + be * 35.0) * user_scale;
        let angle = p_f * 0.22 + frame_time * (0.30 + (p_i % 3) as f32 * 0.10);

        let px = angle.cos() * orbit_r;
        let py = (angle * 2.0 + p_f * 0.5).sin() * (45.0 * user_scale);
        let pz = angle.sin() * orbit_r;

        let particle_r = (5.0 + (p_f % 5.0) * 2.0 + fv * 6.0) * user_scale;

        let particle_col = mix(
            mix(p_col, glow_col, (p_i % 3) as f32 / 3.0),
            mix(accent_col, s_col, (p_i % 2) as f32),
            p_f / NEBULA_PARTICLES as f32,
        );

        scene.push();
        scene.translate(px, py, pz);
        scene.add_disc(0.0, 0.0, 0.0, particle_r, 16, particle_col);
        scene.pop();
    }

    c.set_global_alpha(1.0);
    c.restore();
}
