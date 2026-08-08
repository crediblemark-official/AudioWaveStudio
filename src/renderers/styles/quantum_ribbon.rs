//! Quantum Waveform Ribbon style renderer (`quantumRibbon`) — 3D Flying Plasma Ribbon Engine.
//!
//! Masterpiece 3D Flying Plasma Ribbon:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - Slender 3D silk plasma ribbon twisting organically in 3D perspective space.
//! - Audio-reactive Bezier wave frequency modulation & glowing neon edge highlights.
//! - Trailing 3D plasma energy spark particles along the ribbon path.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const RIBBON_3D_SEGS: usize = 48;

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

    // Deep quantum space backdrop
    c.set_fill(Fill::Solid(Color::hex("#020107")));
    c.fill_rect(0.0, 0.0, width, height);

    // Quantum plasma atmospheric glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.65 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.30 + be * 0.15), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.80, 0.0, 0.60, 0.12), 0.5)),
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

    scene.cam_yaw = (frame_time * 0.15).sin() * 0.12;
    scene.cam_pitch = -0.15 + (frame_time * 0.08).sin() * 0.05;
    scene.cam_zoom = (0.95 - be * 0.05) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    let step_f = (freq.len() / bar_count).max(1);

    // -------------------------------------------------------------------------
    // 2. 48 SLENDER 3D FLYING PLASMA RIBBON SEGMENTS
    // -------------------------------------------------------------------------
    for seg in 0..RIBBON_3D_SEGS {
        let t = seg as f32 / RIBBON_3D_SEGS as f32;
        let rx = (t - 0.5) * (520.0 * user_scale);

        let bin_k = (seg * step_f / (RIBBON_3D_SEGS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let wave1 = (t * 4.5 - frame_time * 1.8).sin() * (50.0 + be * 35.0) * user_scale;
        let wave2 = (t * 8.5 + frame_time * 2.5).cos() * (25.0 + fv * 70.0 * sensitivity) * user_scale;
        let ry = wave1 + wave2;

        let rz = (t * 6.0 + frame_time * 1.2).sin() * (120.0 * user_scale);

        let seg_w = (14.0 + fv * 18.0 + be * 10.0) * user_scale;
        let seg_h = (14.0 + fv * 18.0 + be * 10.0) * user_scale;

        let ribbon_col = mix(
            mix(p_col, glow_col, t),
            mix(accent_col, s_col, (seg % 3) as f32 / 3.0),
            fv,
        );

        // Ribbon Node Box
        scene.push();
        scene.translate(rx, ry, rz);
        scene.rotate_z(t * 3.14 + frame_time * 0.5);
        scene.add_box(0.0, 0.0, 0.0, seg_w, seg_h, 8.0 * user_scale, ribbon_col);
        scene.pop();

        // Trailing Plasma Spark Particle Disc
        let spark_col = mix(Color::rgba(1.0, 1.0, 1.0, 0.90), glow_col, 0.5);
        scene.push();
        scene.translate(rx, ry + 15.0 * user_scale, rz - 10.0 * user_scale);
        scene.add_disc(0.0, 0.0, 0.0, 4.0 * user_scale, 12, spark_col);
        scene.pop();
    }

    c.set_global_alpha(1.0);
    c.restore();
}
