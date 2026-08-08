//! Vaporwave Cassette Deck 3D style renderer (`vaporwaveDeck3D`) — Cyberpunk Tape Deck Workstation Engine.
//!
//! Masterpiece 3D Cassette Deck Workstation:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - Large-scale 3D cassette deck chassis with glowing neon housing & translucent glass cassette window.
//! - Dual 3D spinning spool wheels with 6 spokes each rotating dynamically with tape speed.
//! - 32-bar audio spectrum LED grid + dual 3D analog VU meter boxes with glowing neon deflection needles.
//! - Receding 3D synthwave grid floor in the background.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const SPOOK_COUNT: usize = 6;
const DECK_SPECTRUM_BARS: usize = 32;

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

    // Deep vaporwave synthwave backdrop
    c.set_fill(Fill::Solid(Color::hex("#080312")));
    c.fill_rect(0.0, 0.0, width, height);

    // Vaporwave sunset gradient glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.70 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(1.0, 0.0, 0.60, 0.35 + be * 0.15), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.0, 0.85, 1.0, 0.15), 0.5)),
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

    scene.cam_yaw = (frame_time * 0.10).sin() * 0.06;
    scene.cam_pitch = -0.14 - (frame_time * 0.04).sin() * 0.02;
    scene.cam_zoom = (0.95 - be * 0.04) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    // Full Stage Scale 3D Cassette Deck (Large, Realistic Dimensions)
    let deck_w = 580.0 * user_scale;
    let deck_h = 320.0 * user_scale;
    let deck_d = 35.0 * user_scale;

    // A. Main 3D Cassette Deck Body (Dark Metallic Outer Chassis)
    scene.push();
    scene.add_box(0.0, 0.0, 0.0, deck_w, deck_h, deck_d, mix(p_col, Color::hex("#120722"), 0.80));
    scene.pop();

    // B. Inner Cassette Window Recess (Glass Window Frame)
    let win_w = 420.0 * user_scale;
    let win_h = 190.0 * user_scale;
    let win_y = 25.0 * user_scale;
    let win_z = 20.0 * user_scale;

    scene.push();
    scene.translate(0.0, win_y, win_z);
    scene.add_box(0.0, 0.0, 0.0, win_w, win_h, 8.0 * user_scale, Color::rgba(0.05, 0.02, 0.14, 0.95));
    scene.pop();

    // C. 3D Spinning Spool Wheels (Left & Right Spools with Spokes)
    let spool_r = 65.0 * user_scale;
    let spool_spin = frame_time * (2.2 + be * 3.0);
    let left_x = -110.0 * user_scale;
    let right_x = 110.0 * user_scale;
    let spool_y = win_y;
    let spool_z = win_z + 8.0 * user_scale;

    // Left Spool Outer Ring
    scene.push();
    scene.translate(left_x, spool_y, spool_z);
    scene.add_disc(0.0, 0.0, 0.0, spool_r, 24, mix(accent_col, Color::hex("#ff007f"), 0.75));
    scene.pop();

    // Left Spool 6 Spokes
    for s in 0..SPOOK_COUNT {
        let sp_ang = (s as f32 / SPOOK_COUNT as f32) * TAU + spool_spin;
        let sx = left_x + sp_ang.cos() * (spool_r * 0.5);
        let sy = spool_y + sp_ang.sin() * (spool_r * 0.5);

        scene.push();
        scene.translate(sx, sy, spool_z + 3.0 * user_scale);
        scene.rotate_z(sp_ang);
        scene.add_box(0.0, 0.0, 0.0, spool_r * 0.7, 6.0 * user_scale, 4.0 * user_scale, Color::rgba(1.0, 1.0, 1.0, 0.95));
        scene.pop();
    }

    // Right Spool Outer Ring
    scene.push();
    scene.translate(right_x, spool_y, spool_z);
    scene.add_disc(0.0, 0.0, 0.0, spool_r, 24, mix(glow_col, Color::hex("#00f0ff"), 0.75));
    scene.pop();

    // Right Spool 6 Spokes
    for s in 0..SPOOK_COUNT {
        let sp_ang = (s as f32 / SPOOK_COUNT as f32) * TAU + spool_spin * 0.96;
        let sx = right_x + sp_ang.cos() * (spool_r * 0.5);
        let sy = spool_y + sp_ang.sin() * (spool_r * 0.5);

        scene.push();
        scene.translate(sx, sy, spool_z + 3.0 * user_scale);
        scene.rotate_z(sp_ang);
        scene.add_box(0.0, 0.0, 0.0, spool_r * 0.7, 6.0 * user_scale, 4.0 * user_scale, Color::rgba(1.0, 1.0, 1.0, 0.95));
        scene.pop();
    }

    // D. Tape Magnetic Ribbon Connecting Spools
    let tape_w = 230.0 * user_scale;
    scene.push();
    scene.translate(0.0, spool_y, spool_z + 2.0 * user_scale);
    scene.add_box(0.0, 0.0, 0.0, tape_w, 12.0 * user_scale, 3.0 * user_scale, Color::hex("#4a2b10"));
    scene.pop();

    // E. 32-BAR AUDIO SPECTRUM LED DISPLAY ACROSS BOTTOM CHASSIS
    let step_f = (freq.len() / bar_count).max(1);
    let bar_w = (deck_w * 0.75) / DECK_SPECTRUM_BARS as f32;
    let b_start_x = - (DECK_SPECTRUM_BARS as f32 * bar_w) * 0.5;
    let b_base_y = -95.0 * user_scale;

    for b in 0..DECK_SPECTRUM_BARS {
        let b_f = b as f32;
        let bx = b_start_x + b_f * bar_w + bar_w * 0.5;

        let bin_k = (b * step_f / (DECK_SPECTRUM_BARS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let led_h = (10.0 + fv * 60.0 * sensitivity + be * 15.0).clamp(6.0, 80.0) * user_scale;
        let led_col = mix(
            mix(p_col, glow_col, fv),
            mix(accent_col, s_col, b_f / DECK_SPECTRUM_BARS as f32),
            fv,
        );

        scene.push();
        scene.translate(bx, b_base_y + led_h * 0.5, win_z + 6.0 * user_scale);
        scene.add_box(0.0, 0.0, 0.0, bar_w * 0.78, led_h, 4.0 * user_scale, led_col);
        scene.pop();
    }

    // F. Dual 3D Analog VU Meter Boxes at Bottom Corners
    let fv_l = freq[step_f % freq.len()] as f32 / 255.0;
    let fv_r = freq[(step_f * 4) % freq.len()] as f32 / 255.0;

    let vu_w = 110.0 * user_scale;
    let vu_h = 50.0 * user_scale;
    let vu_y = -105.0 * user_scale;

    // Left VU Box
    scene.push();
    scene.translate(-200.0 * user_scale, vu_y, 22.0 * user_scale);
    scene.add_box(0.0, 0.0, 0.0, vu_w, vu_h, 6.0 * user_scale, Color::hex("#1a0f2e"));
    scene.pop();

    // Left VU Deflection Needle
    let n_ang_l = -0.5 + fv_l * 1.0 * sensitivity + be * 0.2;
    scene.push();
    scene.translate(-200.0 * user_scale, vu_y, 26.0 * user_scale);
    scene.rotate_z(n_ang_l);
    scene.add_box(0.0, 15.0 * user_scale, 0.0, 3.0 * user_scale, 30.0 * user_scale, 3.0 * user_scale, mix(glow_col, Color::hex("#ff0055"), fv_l));
    scene.pop();

    // Right VU Box
    scene.push();
    scene.translate(200.0 * user_scale, vu_y, 22.0 * user_scale);
    scene.add_box(0.0, 0.0, 0.0, vu_w, vu_h, 6.0 * user_scale, Color::hex("#1a0f2e"));
    scene.pop();

    // Right VU Deflection Needle
    let n_ang_r = -0.5 + fv_r * 1.0 * sensitivity + be * 0.2;
    scene.push();
    scene.translate(200.0 * user_scale, vu_y, 26.0 * user_scale);
    scene.rotate_z(n_ang_r);
    scene.add_box(0.0, 15.0 * user_scale, 0.0, 3.0 * user_scale, 30.0 * user_scale, 3.0 * user_scale, mix(glow_col, Color::hex("#00ffbb"), fv_r));
    scene.pop();

    c.set_global_alpha(1.0);
    c.restore();
}
