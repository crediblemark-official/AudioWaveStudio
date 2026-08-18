//! Speaker Explosion style renderer (`speakerExplosion`).
//!
//! Audio-reactive speaker cone, pulsing woofer ring, flying 3D paint splatters,
//! and radiant audio ray spikes with glowing tip motes (no blunt/flat caps).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p = theme_primary(theme);
    let s = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let bass_mult    = ctx.config.reactivity.bass_multiplier;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;
    let bar_count    = ctx.config.reactivity.bar_count.clamp(16, 96);

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let center_x = width * 0.5 + pos_offset_x;
    let center_y = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_size = reference_size * 0.35 * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. BACKGROUND GLOWING SHOCKWAVE AURA
    // -------------------------------------------------------------------------
    let shock_r = base_size * (1.2 + be * 0.45);
    let bg_glow = Fill::radial_gradient(
        center_x, center_y, 0.0,
        center_x, center_y, shock_r,
        &[
            (0.00, mix(glow, Color::WHITE, 0.3).with_alpha(0.35 + be * 0.25)),
            (0.40, p.with_alpha(0.20 + be * 0.15)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);

    // -------------------------------------------------------------------------
    // 2. SPEAKER WOOFER RING & CONE
    // -------------------------------------------------------------------------
    let woofer_r = base_size * (0.85 + be * 0.12);

    // Cabinet Rim
    c.set_fill(Fill::Solid(Color::hex("#121218")));
    c.set_stroke(Fill::Solid(mix(p, Color::WHITE, 0.3)));
    c.set_line_width((4.0 + bs * 3.0) * user_scale);
    c.set_shadow(glow, (20.0 + bs * 15.0) * user_scale);
    c.fill_circle(center_x, center_y, woofer_r * 1.15);

    // Speaker Cone Disc
    c.set_fill(Fill::Solid(Color::hex("#050508")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(center_x, center_y, woofer_r);

    // Center Dust Cap (Pulsing)
    let cap_r = woofer_r * (0.35 + be * 0.15);
    c.set_fill(Fill::Solid(mix(p, glow, (be * 0.5).min(1.0))));
    c.set_shadow(glow, (12.0 + be * 10.0) * user_scale);
    c.fill_circle(center_x, center_y, cap_r);

    // -------------------------------------------------------------------------
    // 3. RADIANT AUDIO RAY SPIKES (Tapered with glowing tip motes)
    // -------------------------------------------------------------------------
    let max_spike_len = base_size * 1.30;
    let step_f = (freq.len() / bar_count.max(1)).max(1);

    for i in 0..bar_count {
        let angle = (i as f32 / bar_count as f32) * TAU + frame_time * 0.05;

        let k = (i * step_f).min(freq.len().saturating_sub(1));
        let raw_v = freq[k] as f32 / 255.0;
        let spike_len = (raw_v * sensitivity * max_spike_len).clamp(8.0, (max_spike_len * 1.5).max(8.0));

        let x1 = center_x + angle.cos() * (woofer_r * 0.92);
        let y1 = center_y + angle.sin() * (woofer_r * 0.92);
        let x2 = center_x + angle.cos() * (woofer_r * 0.92 + spike_len);
        let y2 = center_y + angle.sin() * (woofer_r * 0.92 + spike_len);

        let ray_col = mix(p, s, i as f32 / bar_count as f32);

        c.set_stroke(Fill::Solid(ray_col.with_alpha(0.88)));
        c.set_line_width((2.0 + raw_v * 3.5).clamp(1.5, 8.0));
        c.set_shadow(ray_col, (8.0 + bs * 6.0) * user_scale);
        c.stroke_line(x1, y1, x2, y2);

        // Glowing rounded tip mote at spike endpoint (eliminates flat "ujung buntung")
        let tip_r = (2.2 + raw_v * 2.5) * user_scale;
        c.set_fill(Fill::Solid(mix(ray_col, Color::WHITE, 0.7)));
        c.set_shadow(glow, (10.0 + bs * 8.0) * user_scale);
        c.fill_circle(x2, y2, tip_r);
    }

    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 4. FLOATING 3D EXPLOSIVE PARTICLES & PAINT DROPLETS
    // -------------------------------------------------------------------------
    let num_splatters = 42usize;
    for i in 0..num_splatters {
        let p_t = ((frame_time * 0.5 + i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let dist = woofer_r * 1.05 + p_t * (height * 0.35);
        let p_angle = (i as f32 * 1.618) * TAU; // Golden ratio angle dispersion

        let px = center_x + p_angle.cos() * dist;
        let py = center_y + p_angle.sin() * dist;

        let p_col = mix(accent, Color::WHITE, p_t);
        let p_size = (1.8 + (1.0 - p_t) * 3.5) * user_scale;
        let alpha = (1.0 - p_t).powf(1.5);

        c.set_fill(Fill::Solid(p_col.with_alpha(alpha)));
        c.set_shadow(p_col, 6.0 * user_scale);
        c.fill_circle(px, py, p_size);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
