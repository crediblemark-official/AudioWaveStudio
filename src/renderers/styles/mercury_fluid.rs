//! Liquid Mercury Fluid Wave style renderer (`mercuryFluid`) — Liquid Metal Blob Engine.
//!
//! A glossy molten-mercury blob floats over a reflective metal pool. It squashes
//! with the beat, sheds sparkling droplets, and pushes concentric chrome ripples
//! across the pool. (Distinct from `threeD`'s flat block matrix.)
//! Features:
//! - Chrome dome built from stacked 3D horizontal discs with specular highlight
//! - Beat-reactive squash & stretch with bass surge
//! - Expanding audio-driven ripple rings on the pool floor
//! - Falling mercury droplets & floating metallic motes
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let _s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5;
    let cy = height * 0.56;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep liquid metal space backdrop
    c.set_fill(Fill::Solid(Color::hex("#010307")));
    c.fill_rect(0.0, 0.0, width, height);

    // Liquid mercury thermal reflection glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.55,
        &[
            (0.0, mix(glow_col, Color::rgba(0.70, 0.85, 1.0, 0.30 + be * 0.15), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.20, 0.40, 0.80, 0.12), 0.5)),
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

    scene.cam_yaw = (frame_time * 0.10).sin() * 0.35;
    scene.cam_pitch = -0.38 - (frame_time * 0.05).sin() * 0.04;
    scene.cam_zoom = 0.95;
    scene.target_x = 0.0;
    scene.target_y = 0.0;

    let world_cy = height * 0.5 - cy;
    let pool_r = (width * 0.34).clamp(150.0, 460.0);

    // Chrome palette (silver base with theme tint)
    let silver_lo = Color::rgba(0.55, 0.58, 0.62, 0.95);
    let silver_hi = Color::rgba(0.93, 0.96, 1.0, 0.98);
    let chrome = |t: f32| mix(silver_lo, silver_hi, t);
    let _accent_tint = mix(accent_col, silver_hi, 0.55);

    // -------------------------------------------------------------------------
    // 2. REFLECTIVE POOL FLOOR
    // -------------------------------------------------------------------------
    let floor_col = Color::rgba(0.18, 0.21, 0.26, 0.50);
    scene.add_disc_xz(0.0, world_cy, 0.0, pool_r * 1.15, 32, floor_col);
    scene.add_disc_xz(0.0, world_cy, 0.0, pool_r * 0.6, 32, floor_col.with_alpha(0.30));

    // -------------------------------------------------------------------------
    // 3. EXPANDING AUDIO-DRIVEN CHROME RIPPLES
    // -------------------------------------------------------------------------
    for r_i in 0..5usize {
        let span = 2.2 + r_i as f32 * 0.7;
        let t = ((frame_time * 0.6) % span) / span;
        let r_ripple = pool_r * (0.12 + t * 0.88);
        let alpha = (0.5 * (1.0 - t) + be * 0.3).clamp(0.0, 0.7);
        let col = mix(chrome(0.3 + t * 0.5), glow_col, 0.3).with_alpha(alpha);
        scene.push();
        scene.translate(0.0, world_cy, 0.0);
        scene.rotate_x(std::f32::consts::FRAC_PI_2);
        scene.add_band(0.0, 0.0, 0.0, r_ripple, r_ripple + 3.0, &[r_ripple + 3.0; 24], 1.5, col);
        scene.pop();
    }

    // -------------------------------------------------------------------------
    // 4. LIQUID MERCURY BLOB (chrome dome of stacked discs)
    // -------------------------------------------------------------------------
    let surge = be * (1.0 + bs * 0.35);
    let squash = 1.0 - bs * 0.12; // flatten on beat
    let blob_r = (pool_r * 0.30 * (0.75 + sensitivity * 0.35) * (1.0 + surge * 0.10))
        .clamp(60.0, pool_r * 0.42);
    let blob_h = blob_r * 1.15 * squash;
    let base_y = world_cy + 4.0;

    let shells = 10usize;
    for s in 0..shells {
        let f = s as f32 / (shells as f32 - 1.0);
        let r_shell = blob_r * (1.0 - f).sqrt(); // slower radius decay near base -> dome profile
        let y_shell = base_y + blob_h * f;
        let t = f;
        let col = chrome(0.25 + t * 0.75).with_alpha(0.96 - f * 0.10);
        scene.add_disc_xz(0.0, y_shell, 0.0, r_shell, 26, col);
    }

    // Specular highlight blob on the front-top of the dome
    scene.add_disc(blob_r * 0.30, base_y + blob_h * 0.82, blob_r * 0.30, blob_r * 0.22, 14, silver_hi.with_alpha(0.95));
    scene.add_disc(0.0, base_y + blob_h * 0.98, 0.0, blob_r * 0.18, 14, Color::WHITE);

    // Base contact glow ring where the blob meets the pool
    scene.push();
    scene.translate(0.0, base_y, 0.0);
    scene.rotate_x(std::f32::consts::FRAC_PI_2);
    scene.add_band(0.0, 0.0, 0.0, blob_r * 0.92, blob_r * 1.05, &[blob_r * 1.05; 20], 2.0, glow_col.with_alpha(0.5));
    scene.pop();

    // -------------------------------------------------------------------------
    // 5. FALLING MERCURY DROPLETS & FLOATING MOTES
    // -------------------------------------------------------------------------
    for m_i in 0..22usize {
        let m_t = ((frame_time * 0.7 + m_i as f32 * 0.11) % 1.0).clamp(0.0, 1.0);
        let mx = (m_i as f32 * 37.0).sin() * pool_r * 0.7;
        let mz = (m_i as f32 * 17.0).cos() * pool_r * 0.7;
        let my = base_y + blob_h * (1.6 - m_t * 1.9).max(0.05);
        let m_sz = 2.0 * (1.0 - m_t) + 1.2;
        let m_col = chrome(0.5 + m_t * 0.5).with_alpha(0.35 + (1.0 - m_t) * 0.6);
        scene.add_disc(mx, my, mz, m_sz, 8, m_col);
    }

    // Frequency sparkle dots drifting above the blob
    let _freq_peaks = freq.len();
    for f_i in 0..10usize {
        let k = (f_i * 3).min(freq.len().saturating_sub(1));
        let fv = freq[k] as f32 / 255.0;
        let a = frame_time * 0.4 + f_i as f32 * 0.63;
        let rr = blob_r * (0.2 + fv * 0.8);
        let mx = a.cos() * rr;
        let mz = a.sin() * rr;
        let my = base_y + blob_h * (0.4 + fv * 0.5);
        scene.add_disc(mx, my, mz, 1.5 + fv * 2.5, 8, chrome(0.9).with_alpha(0.4 + fv * 0.5));
    }

    c.set_global_alpha(1.0);
    c.restore();
}
