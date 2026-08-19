//! Holographic Vinyl style renderer (`holographicVinyl`) — Hyper-Realistic Iridescent Turntable Engine.
//!
//! Masterpiece 33⅓ RPM Holographic Vinyl LP Turntable:
//! - Hyper-realistic spinning holographic vinyl record with 38 micro-groove song tracks.
//! - 4-Blade Anisotropic Prismatic Rainbow Specular Sheen sweeping across the record.
//! - Audio-reactive holographic wave rings rippling along record tracks with music frequencies.
//! - Metallic stroboscopic turntable platter rim with rotating neon LED indicators.
//! - Iridescent holographic paper record label ("HOLOGRAPHIC LP • 33⅓ RPM STEREO").
//! - Metallic S-curved tonearm with arm pivot base, counterweight, tracking drift, headshell, & glowing laser stylus tip.
//! - Atmospheric cosmic holographic dust motes drifting in space.
//! - Full UI Theme colors and settings integration (Scale, Position X & Y, Sensitivity, Bar Count).

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

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
    let rot = ctx.rotation_angle;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;
    let reference_size = width.min(height);
    let base_disc_r = reference_size * 0.30 * user_scale;
    let disc_r = base_disc_r * (1.0 + be * 0.025);

    // Curated Prismatic Holographic Colors
    let holo_cyan = mix(glow_col, Color::rgba(0.0, 0.92, 1.0, 1.0), 0.70);
    let holo_magenta = mix(accent_col, Color::rgba(1.0, 0.15, 0.85, 1.0), 0.70);
    let holo_yellow = Color::rgba(1.0, 0.90, 0.15, 1.0);
    let vinyl_dark = Color::rgba(0.06, 0.05, 0.10, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC PRISMATIC BACKDROP & RADIAL HOLOGRAPHIC GLOW
    // -------------------------------------------------------------------------
    let bg_haze = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        disc_r * 2.2,
        &[
            (0.0, mix(holo_cyan, holo_magenta, 0.5).with_alpha(0.22 + be * 0.16)),
            (0.40, p_col.with_alpha(0.12)),
            (0.75, s_col.with_alpha(0.05)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_haze);

    // B. Orbiting Floating Holographic Stardust Motes
    let mote_count = 36usize;
    for m_i in 0..mote_count {
        let m_t = m_i as f32 / mote_count as f32;
        let m_speed = 0.25 + (m_i % 4) as f32 * 0.12;
        let m_angle = m_t * TAU + frame_time * m_speed;
        let m_dist = disc_r * 1.08 + (m_i as f32 * 19.0).sin().abs() * (height * 0.18) + be * 12.0;

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.2 + (m_i % 3) as f32 * 1.2 + bs * 1.8).clamp(1.5, 6.0);
        let m_col = mix(holo_cyan, holo_magenta, (m_i as f32 * 0.3 + frame_time).sin() * 0.5 + 0.5)
            .with_alpha(0.50 + (m_i as f32 * 0.7).cos().abs() * 0.40);

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(m_col, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz * user_scale);
    }
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 2. TURNTABLE PLATTER & ROTATING STROBOSCOPIC NEON RIM
    // -------------------------------------------------------------------------
    let platter_r = disc_r + 6.0 * user_scale;
    let platter_grad = Fill::radial_gradient(
        cx - platter_r * 0.20,
        cy - platter_r * 0.20,
        0.0,
        cx,
        cy,
        platter_r,
        &[
            (0.0, Color::rgba(0.20, 0.22, 0.28, 0.98)),
            (0.85, Color::rgba(0.10, 0.11, 0.15, 0.98)),
            (1.0, Color::rgba(0.04, 0.05, 0.08, 0.98)),
        ],
    );
    c.set_fill(platter_grad);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.75), 24.0 * user_scale);
    c.fill_circle(cx, cy, platter_r);

    // Metallic Platter Bevel Rim Highlight
    c.set_stroke(Fill::Solid(holo_cyan.with_alpha(0.50)));
    c.set_line_width(1.8 * user_scale);
    c.stroke_circle(cx, cy, platter_r);

    // Rotating Neon Stroboscopic Dots on Platter Rim
    let strobe_dots = 48usize;
    for d_i in 0..strobe_dots {
        let da = (d_i as f32 / strobe_dots as f32) * TAU + rot * 1.5;
        let dx = cx + da.cos() * (platter_r - 3.0 * user_scale);
        let dy = cy + da.sin() * (platter_r - 3.0 * user_scale);
        let dot_col = if d_i % 2 == 0 {
            mix(holo_cyan, Color::WHITE, 0.70)
        } else {
            holo_magenta.with_alpha(0.60)
        };
        c.set_fill(Fill::Solid(dot_col));
        c.fill_circle(dx, dy, 1.8 * user_scale);
    }

    // -------------------------------------------------------------------------
    // 3. SPINNING POLISHED HOLOGRAPHIC VINYL DISC & MICRO-GROOVES
    // -------------------------------------------------------------------------
    c.save();
    c.translate(cx, cy);
    c.rotate(rot * 1.5); // Vinyl disc continuous rotation
    c.translate(-cx, -cy);

    // Dark Holographic Base Vinyl Disc
    let vinyl_grad = Fill::radial_gradient(
        cx - disc_r * 0.25,
        cy - disc_r * 0.25,
        0.0,
        cx,
        cy,
        disc_r,
        &[
            (0.0, Color::rgba(0.14, 0.12, 0.20, 0.98)),
            (0.35, vinyl_dark),
            (0.85, Color::rgba(0.03, 0.02, 0.06, 0.98)),
            (1.0, Color::rgba(0.08, 0.06, 0.12, 0.98)),
        ],
    );
    c.set_fill(vinyl_grad);
    c.fill_circle(cx, cy, disc_r);

    // Outer lead-in groove rim
    c.set_stroke(Fill::Solid(holo_cyan.with_alpha(0.70)));
    c.set_line_width(1.5 * user_scale);
    c.stroke_circle(cx, cy, disc_r * 0.97);

    // 38 Micro-Groove Sound Tracks with Audio-Reactive Ripples
    let label_r = disc_r * 0.35;
    let groove_start_r = label_r + disc_r * 0.08;
    let groove_end_r = disc_r * 0.94;
    let total_grooves = 38usize;

    let step_f = (freq.len() / bar_count).max(1);

    for g_i in 0..total_grooves {
        let g_t = g_i as f32 / total_grooves as f32;
        let bin_k = (g_i * step_f / (total_grooves / bar_count.max(1)).max(1)).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let gr = groove_start_r + g_t * (groove_end_r - groove_start_r) + fv * 3.0 * sensitivity;

        // Skip song track gap lines to form distinct track bands
        if g_i % 9 != 8 {
            let groove_col = mix(
                Color::rgba(0.20, 0.18, 0.30, 0.35),
                mix(holo_cyan, holo_magenta, g_t),
                fv * 0.60,
            );
            c.set_stroke(Fill::Solid(groove_col));
            c.set_line_width((0.8 + fv * 0.8) * user_scale);
            c.stroke_circle(cx, cy, gr);
        }
    }

    // Dead-wax ungrooved inner run-out ring
    c.set_stroke(Fill::Solid(holo_magenta.with_alpha(0.45)));
    c.set_line_width(1.2 * user_scale);
    c.stroke_circle(cx, cy, label_r + disc_r * 0.03);

    // -------------------------------------------------------------------------
    // 4. QUAD BUTTERFLY HOLOGRAPHIC ANISOTROPIC RAINBOW LIGHT SHEEN
    // -------------------------------------------------------------------------
    // Real holographic vinyl diffracts light into 4 sweeping rainbow wedges
    for k_flare in 0..4 {
        let sheen_offset = k_flare as f32 * (TAU / 4.0);
        let w_angle = rot * 1.2 + sheen_offset;
        let mut sheen_pts = vec![(cx, cy)];

        let wedge_steps = 18usize;
        for k in 0..=wedge_steps {
            let a = w_angle - 0.32 + (k as f32 / wedge_steps as f32) * 0.64;
            let wx = cx + a.cos() * disc_r;
            let wy = cy + a.sin() * disc_r;
            sheen_pts.push((wx, wy));
        }
        sheen_pts.push((cx, cy));

        // Rainbow Specular Gradient
        let alpha_m = 0.22 + be * 0.12;
        let sheen_grad = Fill::radial_gradient(
            cx,
            cy,
            label_r,
            cx,
            cy,
            disc_r,
            &[
                (0.0, Color::rgba(1.0, 1.0, 1.0, 0.0)),
                (0.30, holo_cyan.with_alpha(alpha_m)),
                (0.55, holo_magenta.with_alpha(alpha_m)),
                (0.75, holo_yellow.with_alpha(alpha_m * 0.85)),
                (0.90, toxic_green_hue(glow_col).with_alpha(alpha_m * 0.70)),
                (1.0, Color::rgba(1.0, 1.0, 1.0, 0.0)),
            ],
        );

        c.set_fill(sheen_grad);
        c.fill_polygon(&sheen_pts);
    }

    // -------------------------------------------------------------------------
    // 5. IRIDESCENT HOLOGRAPHIC PAPER RECORD LABEL
    // -------------------------------------------------------------------------
    let label_grad = Fill::linear_gradient(
        cx - label_r,
        cy - label_r,
        cx + label_r,
        cy + label_r,
        &[
            (0.0, mix(holo_cyan, Color::WHITE, 0.40)),
            (0.40, mix(holo_magenta, Color::WHITE, 0.50)),
            (0.80, mix(p_col, glow_col, 0.5)),
            (1.0, mix(s_col, Color::hex("#080412"), 0.40)),
        ],
    );
    c.set_fill(label_grad);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.60), 10.0 * user_scale);
    c.fill_circle(cx, cy, label_r);

    // Outer paper label border ring
    c.set_stroke(Fill::Solid(Color::WHITE.with_alpha(0.85)));
    c.set_line_width(2.0 * user_scale);
    c.stroke_circle(cx, cy, label_r);

    // Color racing band on label
    let stripe_h = label_r * 0.35;
    c.set_fill(Fill::Solid(s_col.with_alpha(0.85)));
    c.fill_rect(cx - label_r * 0.90, cy - label_r * 0.55, label_r * 1.80, stripe_h);

    // Label Header Text
    c.draw_text(
        "HOLOGRAPHIC LP  •  33⅓ RPM",
        cx,
        cy - label_r * 0.28,
        (label_r * 0.15).clamp(8.0, 13.0),
        "monospace",
        700.0,
        false,
        TextAlign::Center,
        Fill::Solid(Color::WHITE),
        1.0,
        &Default::default(),
    );

    c.draw_text(
        "SIDE A",
        cx - label_r * 0.55,
        cy + label_r * 0.15,
        (label_r * 0.16).clamp(8.0, 12.0),
        "sans-serif",
        800.0,
        false,
        TextAlign::Center,
        Fill::Solid(Color::rgba(0.10, 0.08, 0.15, 0.95)),
        1.0,
        &Default::default(),
    );

    if !ctx.config.text.cassette_label.trim().is_empty() {
        let track_title = ctx.config.text.cassette_label.to_uppercase();
        c.draw_text(
            &track_title,
            cx,
            cy + label_r * 0.55,
            (label_r * 0.15).clamp(8.0, 13.0),
            "monospace",
            700.0,
            false,
            TextAlign::Center,
            Fill::Solid(Color::rgba(0.08, 0.06, 0.12, 0.95)),
            1.0,
            &Default::default(),
        );
    }

    // Spindle center hole & metallic spindle ring
    let spindle_r = (label_r * 0.16).clamp(5.0, 14.0);
    let spindle_ring_r = spindle_r * 1.8;

    c.set_stroke(Fill::Solid(Color::rgba(0.85, 0.90, 1.0, 0.90)));
    c.set_line_width(1.5 * user_scale);
    c.stroke_circle(cx, cy, spindle_ring_r);

    c.set_fill(Fill::Solid(Color::rgba(0.03, 0.02, 0.05, 0.98)));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, spindle_r);

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(1.2 * user_scale);
    c.stroke_circle(cx, cy, spindle_r);

    // Center image inside vinyl label (spins with record)
    draw_radial_center_image(c, ctx, cx, cy, label_r * 0.80);

    c.restore();

    // -------------------------------------------------------------------------
    // 6. METALLIC S-CURVED TONEARM & GLOWING LASER STYLUS NEEDLE
    // -------------------------------------------------------------------------
    let arm_base_x = cx + disc_r * 1.15;
    let arm_base_y = cy - disc_r * 1.10;

    // Arm pivot base disc
    c.set_fill(Fill::Solid(Color::rgba(0.18, 0.20, 0.26, 0.98)));
    c.set_stroke(Fill::Solid(holo_cyan.with_alpha(0.85)));
    c.set_line_width(1.8 * user_scale);
    c.fill_circle(arm_base_x, arm_base_y, 16.0 * user_scale);
    c.stroke_circle(arm_base_x, arm_base_y, 16.0 * user_scale);

    // Audio bass reactivity & mechanical playback wobble
    let wobble_speed = rot * 3.5;
  let wobble_dx = (wobble_speed.sin() * 2.5 + be * 6.0 + bs * 4.0).clamp(-8.0, 8.0);
  let wobble_dy = ((wobble_speed * 1.4).cos() * 2.0 + be * 4.0 - bs * 3.0).clamp(-6.0, 6.0);

    // Smooth tracking drift across vinyl micro-grooves
    let track_drift = (rot * 0.015) % 0.38; // Slowly sweeps inward over time
    let needle_r = disc_r * (0.92 - track_drift);
    let needle_angle = -std::f32::consts::FRAC_PI_4 - 0.22;

    let needle_target_x = cx + needle_angle.cos() * needle_r + wobble_dx;
    let needle_target_y = cy + needle_angle.sin() * needle_r + wobble_dy;

    // S-curved metallic tonearm tube
    let mid_arm_x = arm_base_x - (arm_base_x - needle_target_x) * 0.48 + wobble_dx * 0.4;
    let mid_arm_y = arm_base_y + (needle_target_y - arm_base_y) * 0.52 + wobble_dy * 0.4;

    // Arm shadow
    c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, 0.55)));
    c.set_line_width(4.5 * user_scale);
    c.stroke_polyline(&[
        (arm_base_x + 3.0, arm_base_y + 4.0),
        (mid_arm_x + 3.0, mid_arm_y + 4.0),
        (needle_target_x + 3.0, needle_target_y + 4.0),
    ]);

    // Metallic silver tube
    c.set_stroke(Fill::Solid(Color::rgba(0.88, 0.92, 0.98, 0.95)));
    c.set_line_width(3.5 * user_scale);
    c.stroke_polyline(&[
        (arm_base_x, arm_base_y),
        (mid_arm_x, mid_arm_y),
        (needle_target_x, needle_target_y),
    ]);

    // Pickup Cartridge Headshell
    c.set_fill(Fill::Solid(Color::rgba(0.10, 0.12, 0.16, 0.98)));
    c.fill_rounded_rect(
        needle_target_x - 8.0 * user_scale,
        needle_target_y - 6.0 * user_scale,
        16.0 * user_scale,
        12.0 * user_scale,
        3.0 * user_scale,
    );

    // Glowing Laser Stylus Tip (Glows white-hot & pulses on beat hits!)
    let needle_col = mix(holo_magenta, Color::WHITE, bs);
    c.set_fill(Fill::Solid(needle_col));
    c.set_shadow(needle_col, (12.0 + bs * 10.0) * user_scale);
    c.fill_circle(needle_target_x, needle_target_y, (3.5 + bs * 2.0) * user_scale);

    c.set_global_alpha(1.0);
    c.restore();
}

fn toxic_green_hue(col: Color) -> Color {
    mix(col, Color::rgba(0.20, 1.0, 0.40, 1.0), 0.60)
}

