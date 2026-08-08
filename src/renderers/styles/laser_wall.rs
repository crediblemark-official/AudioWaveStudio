//! Acoustic Laser Equalizer Wall style renderer (`laserWall`) — Concert Stage Engine.
//!
//! Masterpiece 3D Concert Stage:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - 48 3D volumetric LED wall equalizer pillars spanning the full stage width.
//! - 24 glowing 3D cross-beam lasers shooting diagonally into the dark concert sky.
//! - Translucent 3D mirror floor reflections below the stage floor.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const LED_PILLARS_3D: usize = 48;
const CROSS_LASERS: usize = 24;

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

    let cx = width * 0.5;
    let cy = height * 0.5;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep midnight concert stage backdrop
    c.set_fill(Fill::Solid(Color::hex("#010207")));
    c.fill_rect(0.0, 0.0, width, height);

    // Concert stage laser thermal aura
    let stage_glow = Fill::radial_gradient(
        cx,
        cy + height * 0.15,
        0.0,
        cx,
        cy + height * 0.15,
        width * 0.70,
        &[
            (0.0, mix(glow_col, Color::rgba(1.0, 0.10, 0.50, 0.30), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.0, 0.85, 1.0, 0.12), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(stage_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CONFIGURE NATIVE 3D CONCERT STAGE SCENE (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    scene.cam_yaw = (frame_time * 0.12).sin() * 0.08;
    scene.cam_pitch = -0.16 - (frame_time * 0.06).sin() * 0.03 - be * 0.03;
    scene.cam_zoom = (0.95 - be * 0.05) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    let base_y = -110.0 * user_scale;
    let step_f = (freq.len() / bar_count).max(1);
    let stage_w = 640.0 * user_scale;
    let col_w = (stage_w / LED_PILLARS_3D as f32) * 0.82;

    // -------------------------------------------------------------------------
    // 2. 48 LED WALL EQUALIZER PILLARS ON AN ARCED LEANING STAGE WALL
    // -------------------------------------------------------------------------
    let arc_depth = stage_w * 0.18;
    let max_tilt = 0.20;

    for i in 0..LED_PILLARS_3D {
        let i_f = i as f32;
        let t_p = (i_f - LED_PILLARS_3D as f32 * 0.5 + 0.5) / LED_PILLARS_3D as f32; // -0.5..0.5
        let x_pos = t_p * stage_w;

        // Recess the edges backward -> a concave arena wall, not a flat row
        let z_pos = -(t_p * t_p) * arc_depth * 4.0;

        let bin_k = (i * step_f / (LED_PILLARS_3D / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let pillar_h = (20.0 + fv * 300.0 * sensitivity + be * 45.0).clamp(15.0, 420.0) * user_scale;

        let pillar_col = mix(
            mix(p_col, glow_col, fv),
            mix(accent_col, s_col, (i % 4) as f32 / 4.0),
            fv,
        );

        // Lean the pillar slightly toward the stage center (inward V-wall)
        let tilt = t_p * max_tilt;

        // Main 3D LED Box
        scene.push();
        scene.translate(x_pos, base_y + pillar_h * 0.5, z_pos);
        scene.rotate_z(tilt);
        scene.add_box(0.0, 0.0, 0.0, col_w, pillar_h, col_w, pillar_col);
        scene.pop();

        // 3D Translucent Mirror Reflection Below Stage (mirrors the lean)
        let ref_h = pillar_h * 0.40;
        let ref_col = Color::rgba(pillar_col.r, pillar_col.g, pillar_col.b, 0.22);
        scene.push();
        scene.translate(x_pos, base_y - ref_h * 0.5, z_pos);
        scene.rotate_z(tilt);
        scene.add_box(0.0, 0.0, 0.0, col_w * 0.95, ref_h, col_w * 0.95, ref_col);
        scene.pop();

        // White-hot cap light on strong bars
        if fv > 0.7 || bs > 0.4 {
            scene.push();
            scene.translate(x_pos, base_y + pillar_h + 2.0 * user_scale, z_pos);
            scene.rotate_z(tilt);
            scene.add_box(0.0, 0.0, 0.0, col_w * 0.9, 6.0 * user_scale, col_w * 0.9, Color::WHITE.with_alpha(0.9));
            scene.pop();
        }
    }

    // -------------------------------------------------------------------------
    // 3. 24 CROSS-BEAM 3D LASER RAYS SHOOTING INTO THE SKY
    // -------------------------------------------------------------------------
    for l_i in 0..CROSS_LASERS {
        let l_f = l_i as f32;
        let l_x = (l_f - CROSS_LASERS as f32 * 0.5 + 0.5) * (stage_w / CROSS_LASERS as f32);

        let bin_k = (l_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let laser_h = (280.0 + fv * 160.0 * sensitivity + be * 60.0) * user_scale;
        let tilt_ang = ((l_f * 0.35 + frame_time * 1.2).sin() * 0.40).clamp(-0.6, 0.6);

        let laser_col = mix(
            mix(accent_col, Color::rgba(1.0, 0.10, 0.60, 0.90 + bs * 0.10), 0.6),
            glow_col,
            (l_i % 2) as f32,
        );

        scene.push();
        scene.translate(l_x, base_y + laser_h * 0.5, -20.0 * user_scale);
        scene.rotate_z(tilt_ang);
        scene.add_box(0.0, 0.0, 0.0, 4.0 * user_scale, laser_h, 4.0 * user_scale, laser_col);
        scene.pop();
    }

    c.set_global_alpha(1.0);
    c.restore();
}
