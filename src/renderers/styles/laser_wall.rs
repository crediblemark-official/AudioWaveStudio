//! Acoustic Laser Equalizer Wall style renderer (`laserWall`) — Concert Stage Laser Engine.
//!
//! Masterpiece Concert Stage Laser Show:
//! - 48 3D volumetric LED wall equalizer pillars spanning the stage arena.
//! - 24 high-power concert laser beams shooting across the arena with white-hot intense laser cores & wide volumetric neon bloom!
//! - Audio-reactive laser fans, rhythmic angle scanning & base lens flare emitters.
//! - Translucent 3D mirror floor reflections below the stage floor.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const LED_PILLARS_3D: usize = 48;
const SCAN_LASERS: usize = 24;

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

    // Deep midnight concert arena backdrop
    c.set_fill(Fill::Solid(Color::hex("#010207")));
    c.fill_rect(0.0, 0.0, width, height);

    // Concert stage laser thermal haze
    let stage_glow = Fill::radial_gradient(
        cx,
        cy + height * 0.15,
        0.0,
        cx,
        cy + height * 0.15,
        width * 0.70 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(1.0, 0.10, 0.60, 0.35 + be * 0.15), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.0, 0.85, 1.0, 0.15), 0.5)),
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
        let t_p = (i_f - LED_PILLARS_3D as f32 * 0.5 + 0.5) / LED_PILLARS_3D as f32;
        let x_pos = t_p * stage_w;
        let z_pos = -(t_p * t_p) * arc_depth * 4.0;

        let bin_k = (i * step_f / (LED_PILLARS_3D / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let pillar_h = (20.0 + fv * 280.0 * sensitivity + be * 45.0).clamp(15.0, 420.0) * user_scale;

        let pillar_col = mix(
            mix(p_col, glow_col, fv),
            mix(accent_col, s_col, (i % 4) as f32 / 4.0),
            fv,
        );

        let tilt = t_p * max_tilt;

        // Main 3D LED Box
        scene.push();
        scene.translate(x_pos, base_y + pillar_h * 0.5, z_pos);
        scene.rotate_z(tilt);
        scene.add_box(0.0, 0.0, 0.0, col_w, pillar_h, col_w, pillar_col);
        scene.pop();

        // 3D Translucent Floor Reflection
        let ref_h = pillar_h * 0.40;
        let ref_col = Color::rgba(pillar_col.r, pillar_col.g, pillar_col.b, 0.22);
        scene.push();
        scene.translate(x_pos, base_y - ref_h * 0.5, z_pos);
        scene.rotate_z(tilt);
        scene.add_box(0.0, 0.0, 0.0, col_w * 0.95, ref_h, col_w * 0.95, ref_col);
        scene.pop();
    }

    // -------------------------------------------------------------------------
    // 3. 24 REAL VOLUMETRIC HIGH-POWER CONCERT LASER BEAMS & SCANNING FANS
    // -------------------------------------------------------------------------
    let laser_start_y = cy + height * 0.15;
    let step_laser_x = width * 0.90 / SCAN_LASERS as f32;

    for l_i in 0..SCAN_LASERS {
        let l_f = l_i as f32;
        let emitter_x = (cx - width * 0.45) + l_f * step_laser_x + step_laser_x * 0.5;

        let bin_k = (l_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        // Dynamic laser fan scanning angle
        let sweep_speed = 0.8 + (l_i % 5) as f32 * 0.15;
        let scan_angle = ((l_f * 0.30 + frame_time * sweep_speed).sin() * 0.55).clamp(-0.8, 0.8);

        let laser_length = (height * 0.95 + fv * 200.0 * sensitivity + be * 80.0) * user_scale;
        let end_x = emitter_x + scan_angle.sin() * laser_length;
        let end_y = laser_start_y - scan_angle.cos() * laser_length;

        let neon_color = mix(
            mix(accent_col, Color::rgba(1.0, 0.0, 0.55, 0.85), (l_i % 3) as f32 / 3.0),
            mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 0.85), (l_i % 2) as f32),
            fv,
        );

        // A. Base Laser Emitter Flare (Concert Projector Lens)
        let flare = Fill::radial_gradient(
            emitter_x,
            laser_start_y,
            0.0,
            emitter_x,
            laser_start_y,
            (14.0 + fv * 10.0 + be * 8.0) * user_scale,
            &[
                (0.0, Color::rgba(1.0, 1.0, 1.0, 0.95)),
                (0.40, neon_color),
                (1.0, Color::TRANSPARENT),
            ],
        );
        c.set_fill(flare);
        c.fill_circle(emitter_x, laser_start_y, (14.0 + fv * 10.0 + be * 8.0) * user_scale);

        // B. Outer Volumetric Laser Neon Halo (Wide Soft Glow)
        c.set_line_width((10.0 + fv * 8.0 + bs * 4.0) * user_scale);
        c.set_stroke(Fill::Solid(Color::rgba(neon_color.r, neon_color.g, neon_color.b, 0.25 + fv * 0.20)));
        c.set_shadow(neon_color, (16.0 + fv * 12.0) * user_scale);
        c.stroke_line(emitter_x, laser_start_y, end_x, end_y);

        // C. Intense White-Hot Core Laser Beam (Real High-Power Laser Line)
        c.set_line_width((2.2 + fv * 2.0) * user_scale);
        c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.95)));
        c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.90), 6.0 * user_scale);
        c.stroke_line(emitter_x, laser_start_y, end_x, end_y);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
