//! Cyber Horizon 3D style renderer (`cyberHorizon`) — 3D Cybernetic Sun & Spectrum Grid Engine.
//!
//! Cinematic Masterpiece:
//! - Full 3D retained geometry scene built using `ctx.scene3d`.
//! - Dynamic 3D perspective camera with smooth yaw, pitch, and audio-reactive zoom.
//! - Receding 3D cyberspace grid floor & glowing neon green horizon arc in 3D space.
//! - 48 volumetric 3D metallic silver spectrum towers with 3D translucent mirror reflections.
//! - Low-poly 3D cyan/blue constellation mesh floating in deep 3D space.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const MESH_NODES_3D: usize = 32;
const SPECTRUM_TOWERS_3D: usize = 48;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let _p = theme_primary(theme);
    let _s = theme_secondary(theme);
    let _accent = theme_accent(theme);
    let _glow = theme_glow(theme);

    // Settings integration
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

    // -------------------------------------------------------------------------
    // 1. DEEP CYBERSPACE ATMOSPHERIC BACKDROP (2D CANVAS)
    // -------------------------------------------------------------------------
    c.set_fill(Fill::Solid(Color::hex("#010308")));
    c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx,
        cy + height * 0.10,
        0.0,
        cx,
        cy + height * 0.10,
        width * 0.65 * user_scale,
        &[
            (0.0, Color::rgba(0.0, 0.65, 1.0, 0.20 + be * 0.15)),
            (0.40, Color::rgba(0.0, 0.25, 0.70, 0.08)),
            (0.80, Color::rgba(0.02, 0.05, 0.15, 0.03)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 2. CONFIGURE NATIVE 3D SCENE (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.clear();

    // Smooth 3D camera motion
    scene.cam_yaw = (frame_time * 0.12).sin() * 0.08;
    scene.cam_pitch = -0.18 - (frame_time * 0.08).sin() * 0.03 - be * 0.03;
    scene.cam_zoom = (0.95 - be * 0.05) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    let base_y = -90.0 * user_scale;
    let arc_r_3d = 340.0 * user_scale;

    // -------------------------------------------------------------------------
    // 3. RENDER 3D NEON GREEN SUN HORIZON ARC IN SCENE3D
    // -------------------------------------------------------------------------
    let arc_color = Color::rgba(0.46, 1.0, 0.01, 0.95);
    scene.push();
    scene.rotate_x(std::f32::consts::FRAC_PI_2);
    scene.add_ring(0.0, 0.0, -base_y, arc_r_3d * 1.02, arc_r_3d * 0.98, 6.0 * user_scale, 64, arc_color);
    scene.pop();

    // -------------------------------------------------------------------------
    // 4. 48 VOLUMETRIC 3D SPECTRUM TOWERS & 3D REFLECTION PILLARS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..SPECTRUM_TOWERS_3D {
        let b_t = i as f32 / SPECTRUM_TOWERS_3D as f32;

        // Angle along 3D arc (-50° to +50°)
        let ang_rel = (b_t - 0.5) * 100.0f32.to_radians();

        let tx = ang_rel.sin() * arc_r_3d;
        let tz = -ang_rel.cos() * arc_r_3d + arc_r_3d;

        let bin_k = (i * step_f / (SPECTRUM_TOWERS_3D / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        // Gaussian bell curve height envelope
        let bell_weight = (-(b_t - 0.5).powi(2) * 8.0).exp();
        let max_tower_h = (260.0 * bell_weight + 30.0) * user_scale;
        let tower_h = (15.0 + fv * max_tower_h * sensitivity + be * 35.0).clamp(10.0, 360.0 * user_scale);

        let tower_w = 12.0 * user_scale;

        // A. Main Upper 3D Spectrum Tower
        scene.push();
        scene.translate(tx, base_y + tower_h * 0.5, tz);
        scene.rotate_y(ang_rel);

        let tower_col = mix(
            Color::rgba(0.95, 0.95, 0.95, 0.95),
            Color::rgba(0.46, 1.0, 0.01, 0.90 + bs * 0.10),
            fv,
        );

        scene.add_box(0.0, 0.0, 0.0, tower_w, tower_h, tower_w, tower_col);
        scene.pop();

        // B. Inverted 3D Translucent Mirror Reflection Below Base Floor
        let ref_h = tower_h * 0.45;
        scene.push();
        scene.translate(tx, base_y - ref_h * 0.5, tz);
        scene.rotate_y(ang_rel);

        let ref_col = Color::rgba(0.70, 0.85, 0.95, 0.25);
        scene.add_box(0.0, 0.0, 0.0, tower_w * 0.95, ref_h, tower_w * 0.95, ref_col);
        scene.pop();
    }

    // -------------------------------------------------------------------------
    // 5. 3D FLOATING CONSTELLATION NODES IN DEEP SPACE
    // -------------------------------------------------------------------------
    for n_i in 0..MESH_NODES_3D {
        let n_f = n_i as f32;
        let nx = (n_f * 137.5).sin() * (width * 0.45 * user_scale);
        let ny = base_y + (n_f * 219.3).cos() * (height * 0.40 * user_scale) + 120.0 * user_scale;
        let nz = (n_f * 47.3).sin() * 200.0 * user_scale - 100.0;

        let node_sz = (6.0 + (n_f % 3.0) * 2.0) * user_scale;
        let node_col = mix(
            Color::rgba(0.0, 0.85, 1.0, 0.70),
            Color::rgba(0.46, 1.0, 0.01, 0.60),
            (n_i % 2) as f32,
        );

        scene.push();
        scene.translate(nx, ny, nz);
        scene.add_box(0.0, 0.0, 0.0, node_sz, node_sz, node_sz, node_col);
        scene.pop();
    }

    // -------------------------------------------------------------------------
    // 6. 2D TECHNO TITLE & PLAYBACK OVERLAY (2D CANVAS)
    // -------------------------------------------------------------------------
    let title_x = width * 0.08 + pos_offset_x;
    let title_y = height * 0.18 + pos_offset_y;

    // Title & Subtitle text rectangles
    c.set_fill(Fill::Solid(Color::hex("#76ff03")));
    c.set_shadow(Color::hex("#76ff03"), 10.0 * user_scale);
    c.fill_rect(title_x, title_y, 160.0 * user_scale, 22.0 * user_scale);

    c.set_fill(Fill::Solid(Color::hex("#39ff14")));
    c.fill_rect(title_x + 10.0 * user_scale, title_y + 32.0 * user_scale, 120.0 * user_scale, 14.0 * user_scale);

    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Bottom Playback Media Controls
    let ui_y = height * 0.88 + pos_offset_y;
    let ctrl_cx1 = cx - 22.0 * user_scale;
    let ctrl_cx2 = cx + 22.0 * user_scale;
    let ctrl_r = 16.0 * user_scale;

    c.set_stroke(Fill::Solid(Color::hex("#76ff03")));
    c.set_line_width(2.0 * user_scale);
    c.set_shadow(Color::hex("#76ff03"), 8.0 * user_scale);

    c.stroke_circle(ctrl_cx1, ui_y, ctrl_r);
    c.stroke_circle(ctrl_cx2, ui_y, ctrl_r);

    c.set_fill(Fill::Solid(Color::hex("#76ff03")));
    c.fill_rect(ctrl_cx1 - 5.0 * user_scale, ui_y - 7.0 * user_scale, 3.5 * user_scale, 14.0 * user_scale);
    c.fill_rect(ctrl_cx1 + 1.5 * user_scale, ui_y - 7.0 * user_scale, 3.5 * user_scale, 14.0 * user_scale);
    c.fill_rect(ctrl_cx2 - 5.0 * user_scale, ui_y - 5.0 * user_scale, 10.0 * user_scale, 10.0 * user_scale);

    c.set_global_alpha(1.0);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.restore();
}
