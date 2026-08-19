//! Radial Aperture Iris style renderer (`radialApertureIris`) — Photorealistic Camera Lens Engine.
//!
//! Masterpiece Photorealistic Camera Aperture Iris:
//! - 12 Overlapping curved titanium aperture blades with metallic specular sheen & bevel highlights.
//! - Anodized aluminum camera lens barrel housing with laser-etched f-stop markings (f/1.4..f/16).
//! - Multi-coated optical glass lens element with antireflective cyan/magenta optical coating sheen.
//! - Mechanical audio-reactive iris opening & closing (opens wide with bass energy, snaps on beats).
//! - Optical lens bloom, camera shutter light leaks, & floating lens flare dust particles.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const APERTURE_BLADES: usize = 12;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.10, 0.05);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let rot = frame_time * 0.04;
    let step = ((freq.len() as f32) / APERTURE_BLADES as f32).floor().max(1.0) as usize;

    // Curated Camera Lens Metallic Palette (theme-dominant optical coating)
    let lens_cyan = mix(Color::rgba(0.0, 0.92, 1.0, 1.0), s.glow, 0.75);
    let lens_magenta = mix(Color::rgba(1.0, 0.15, 0.85, 1.0), s.s_col, 0.75);
    let metal_dark = Color::rgba(0.08, 0.09, 0.12, 0.98);
    let metal_light = Color::rgba(0.40, 0.44, 0.52, 0.95);

    // -------------------------------------------------------------------------
    // 1. ANODIZED ALUMINUM CAMERA LENS BARREL & F-STOP APERTURE RING
    // -------------------------------------------------------------------------
    let barrel_r = s.base_r * 1.55;
    let barrel_fill = Fill::radial_gradient(
        s.cx - barrel_r * 0.20,
        s.cy - barrel_r * 0.20,
        0.0,
        s.cx,
        s.cy,
        barrel_r,
        &[
            (0.0, Color::rgba(0.22, 0.24, 0.30, 0.98)),
            (0.70, metal_dark),
            (1.0, Color::rgba(0.04, 0.05, 0.07, 0.98)),
        ],
    );
    c.set_fill(barrel_fill);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.80), 24.0 * s.user_scale);
    c.fill_circle(s.cx, s.cy, barrel_r);

    // Metallic Lens Bevel Highlight
    c.set_stroke(Fill::Solid(lens_cyan.with_alpha(0.60)));
    c.set_line_width(2.0 * s.user_scale);
    c.stroke_circle(s.cx, s.cy, barrel_r);

    // 120 Micro-Ticks & Degree Markers around Lens Barrel
    let micro_ticks = 120usize;
    for t_i in 0..micro_ticks {
        let a = (t_i as f32 / micro_ticks as f32) * TAU + rot * 0.2;
        let is_major = t_i % 10 == 0;
        let t_len = if is_major { 7.0 * s.user_scale } else { 3.5 * s.user_scale };
        let (s_a, c_a) = a.sin_cos();

        let x0 = s.cx + c_a * (barrel_r - 2.0 * s.user_scale);
        let y0 = s.cy + s_a * (barrel_r - 2.0 * s.user_scale);
        let x1 = s.cx + c_a * (barrel_r - 2.0 * s.user_scale - t_len);
        let y1 = s.cy + s_a * (barrel_r - 2.0 * s.user_scale - t_len);

        let t_col = if is_major { Color::WHITE } else { metal_light.with_alpha(0.60) };
        c.set_stroke(Fill::Solid(t_col));
        c.set_line_width(1.2 * s.user_scale);
        c.stroke_line(x0, y0, x1, y1);
    }

    // Laser-Etched F-Stop Text Markings
    let f_stops = ["f/1.4", "f/2", "f/2.8", "f/4", "f/5.6", "f/8", "f/11", "f/16"];
    for (f_i, f_text) in f_stops.iter().enumerate() {
        let fa = (f_i as f32 / f_stops.len() as f32) * TAU + rot * 0.2;
        let tx = s.cx + fa.cos() * (barrel_r - 12.0 * s.user_scale);
        let ty = s.cy + fa.sin() * (barrel_r - 12.0 * s.user_scale);

        c.draw_text(
            f_text,
            tx,
            ty,
            (9.0 * s.user_scale).clamp(6.0, 14.0),
            "monospace",
            700.0,
            false,
            TextAlign::Center,
            Fill::Solid(Color::rgba(0.90, 0.95, 1.0, 0.85)),
            1.0,
            &Default::default(),
        );
    }

    // -------------------------------------------------------------------------
    // 2. MULTI-COATED OPTICAL GLASS LENS (BEHIND APERTURE BLADES)
    // -------------------------------------------------------------------------
    let glass_r = s.base_r * 1.35;
    let glass_grad = Fill::radial_gradient(
        s.cx - glass_r * 0.25,
        s.cy - glass_r * 0.25,
        0.0,
        s.cx,
        s.cy,
        glass_r,
        &[
            (0.0, mix(Color::WHITE, lens_cyan, 0.40).with_alpha(0.95)),
            (0.40, mix(lens_cyan, lens_magenta, 0.50).with_alpha(0.85)),
            (0.80, mix(s.p_col, Color::hex("#020810"), 0.80)),
            (1.0, Color::rgba(0.02, 0.04, 0.08, 0.98)),
        ],
    );
    c.set_fill(glass_grad);
    c.fill_circle(s.cx, s.cy, glass_r);

    // -------------------------------------------------------------------------
    // 3. 12 OVERLAPPING CURVED TITANIUM APERTURE BLADES
    // -------------------------------------------------------------------------
    // Mechanical iris movement: opens wide on bass, snaps tight on Quiet
    let open_ratio = (0.18 + s.be * 0.50).clamp(0.12, 0.72);
    let open_r = s.base_r * open_ratio;
    let outer_r = barrel_r * 0.94;
    let mid_r = (outer_r + open_r) * 0.50 + s.base_r * 0.08;

    let slot_angle = TAU / APERTURE_BLADES as f32;

    for b in 0..APERTURE_BLADES {
        let bf = b as f32;
        let a0 = (bf / APERTURE_BLADES as f32) * TAU + rot;
        let a1 = ((bf + 1.0) / APERTURE_BLADES as f32) * TAU + rot;
        let tip_a = a0 + slot_angle * 0.65;

        let audio_v = radial_common::swept_bin(freq, step, b, APERTURE_BLADES, &s) * s.sensitivity;
        let local_open = open_r * (1.0 + (bf * 1.5).sin() * 0.10 + audio_v * 0.12);

        let blade_pts = build_curved_blade_pts(
            s.cx, s.cy, a0, a1, outer_r, mid_r, local_open, tip_a, 8,
        );

        // Dark Titanium Metal Blade Fill
        let blade_col = mix(metal_dark, mix(s.p_col, lens_cyan, bf / APERTURE_BLADES as f32), 0.35);
        c.set_fill(Fill::Solid(blade_col));
        c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.75), 10.0 * s.user_scale);
        c.fill_polygon(&blade_pts);

        // Metallic Bevel Edge Highlight on Blade Overlap Seam
        let (tip_cx, tip_cy) = (s.cx + tip_a.cos() * local_open, s.cy + tip_a.sin() * local_open);
        let (mid_cx, mid_cy) = (s.cx + a1.cos() * mid_r, s.cy + a1.sin() * mid_r);

        c.set_stroke(Fill::Solid(mix(metal_light, Color::WHITE, 0.60)));
        c.set_line_width(1.8 * s.user_scale);
        c.set_shadow(lens_cyan, 6.0 * s.user_scale);
        c.stroke_line(tip_cx, tip_cy, mid_cx, mid_cy);

        // Blade Pivot Hardware Pin
        let pin_x = s.cx + a0.cos() * (outer_r * 0.96);
        let pin_y = s.cy + a0.sin() * (outer_r * 0.96);
        c.set_fill(Fill::Solid(mix(metal_light, Color::WHITE, 0.75)));
        c.fill_circle(pin_x, pin_y, 2.2 * s.user_scale);
    }

    // -------------------------------------------------------------------------
    // 4. OPTICAL LENS BLOOM & CAMERA SHUTTER LIGHT LEAKS
    // -------------------------------------------------------------------------
    let glow_r = open_r * 1.8;
    let optical_bloom = Fill::radial_gradient(
        s.cx,
        s.cy,
        0.0,
        s.cx,
        s.cy,
        glow_r,
        &[
            (0.0, mix(Color::WHITE, lens_cyan, 0.25).with_alpha(0.92)),
            (0.50, s.glow.with_alpha(0.45 + s.be * 0.30)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(optical_bloom);
    c.fill_circle(s.cx, s.cy, glow_r);

    // -------------------------------------------------------------------------
    // 5. FLOATING OPTICAL LENS FLARE DUST PARTICLES
    // -------------------------------------------------------------------------
    let mote_count = (18.0 + s.be * 22.0).clamp(10.0, 44.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.35 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 27.0).sin() * TAU;
        let m_dist = open_r * 0.5 + m_t * (barrel_r * 0.85);

        let mx = s.cx + m_angle.cos() * m_dist;
        let my = s.cy + m_angle.sin() * m_dist;

        let m_sz = (2.2 * (1.0 - m_t) + 1.2 + s.bs * 1.8).clamp(1.0, 5.0) * s.user_scale;
        let m_col = mix(lens_cyan, Color::WHITE, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(lens_cyan, 6.0 * s.user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    radial_common::finish(c, ctx, &s);
}

fn build_curved_blade_pts(
    cx: f32,
    cy: f32,
    a0: f32,
    a1: f32,
    outer_r: f32,
    mid_r: f32,
    open_r: f32,
    tip_a: f32,
    segments: usize,
) -> Vec<(f32, f32)> {
    let mut pts: Vec<(f32, f32)> = Vec::with_capacity(segments * 2 + 4);

    for k in 0..=segments {
        let a = a0 + (a1 - a0) * (k as f32 / segments as f32);
        pts.push((cx + a.cos() * outer_r, cy + a.sin() * outer_r));
    }

    let right_mid_a = a1 * 0.72 + tip_a * 0.28;
    for k in 0..=segments {
        let t = k as f32 / segments as f32;
        let a = if t < 0.5 {
            a1 + (right_mid_a - a1) * (t * 2.0)
        } else {
            right_mid_a + (tip_a - right_mid_a) * ((t - 0.5) * 2.0)
        };
        let r = if t < 0.5 {
            outer_r + (mid_r - outer_r) * (t * 2.0)
        } else {
            mid_r + (open_r - mid_r) * ((t - 0.5) * 2.0)
        };
        pts.push((cx + a.cos() * r, cy + a.sin() * r));
    }

    let left_mid_a = a0 * 0.72 + tip_a * 0.28;
    for k in (0..=segments).rev() {
        let t = (segments - k) as f32 / segments as f32;
        let a = if t < 0.5 {
            tip_a + (left_mid_a - tip_a) * (t * 2.0)
        } else {
            left_mid_a + (a0 - left_mid_a) * ((t - 0.5) * 2.0)
        };
        let r = if t < 0.5 {
            open_r + (mid_r - open_r) * (t * 2.0)
        } else {
            mid_r + (outer_r - mid_r) * ((t - 0.5) * 2.0)
        };
        pts.push((cx + a.cos() * r, cy + a.sin() * r));
    }

    pts
}
