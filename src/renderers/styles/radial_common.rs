//! Shared setup/teardown helpers for the radial style family.
//!
//! Every radial style repeats the same ~35 lines of theme/scale/center
//! boilerplate plus the same glowing-ring + black-disc + center-image ending.
//! This module centralises that so each style file only contains its unique
//! visual loop.

use std::f32::consts::{PI, TAU};

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::draw_radial_center_image;
use crate::renderers::{
    bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

/// Pre-computed shared values for a radial style.
pub struct RadialSetup {
    pub cx: f32,
    pub cy: f32,
    pub base_r: f32,
    pub inner_r: f32,
    pub be: f32,
    pub bs: f32,
    pub user_scale: f32,
    pub sensitivity: f32,
    pub bass_mult: f32,
    /// Continuously-advancing sweep angle (radians) used so the beat/prominence
    /// emphasis rotates evenly around the ring instead of stacking at one angle.
    pub sweep: f32,
    pub p_col: Color,
    /// Secondary theme colour (reserved for styles that layer a secondary tone).
    #[allow(dead_code)]
    pub s_col: Color,
    pub accent: Color,
    pub glow: Color,
}

/// Reads theme + reactivity + scale from `ctx`, computes the radial centre and
/// the inner core radius, opens the canvas transform and clears the shadow.
///
/// - `base_r_mult`: multiplier for the base radius (`110.0` / `115.0` / ...).
/// - `inner_be`: bass-energy coefficient for the inner core radius.
/// - `inner_bs`: beat-strength coefficient for the inner core radius.
pub fn setup(
    c: &mut GpuCanvas,
    ctx: &RenderContext,
    base_r_mult: f32,
    inner_be: f32,
    inner_bs: f32,
) -> RadialSetup {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let s_col = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let bass_mult = ctx.config.reactivity.bass_multiplier;
    let user_scale = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = ctx.config.position_y * height * 0.5;

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r = base_r_mult * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * inner_be + bs * inner_bs);

    // Sweep angle is re-randomised on every detected beat via a hash of the
    // beat counter, so the audio/bass emphasis scatters to a fresh pseudo-random
    // position on each beat instead of marching around the ring in order.
    let sweep = sweep_angle(ctx.beat_count);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    RadialSetup {
        cx,
        cy,
        base_r,
        inner_r,
        be,
        bs,
        user_scale,
        sensitivity,
        bass_mult,
        sweep,
        p_col,
        s_col,
        accent,
        glow,
    }
}

/// Deterministic pseudo-random sweep angle (radians in 0..TAU) derived from a
/// hash of the beat counter. Consecutive beats therefore land on scattered,
/// non-sequential positions around the ring.
pub fn sweep_angle(beat_count: u64) -> f32 {
    let mut x = beat_count.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 29;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 32;
    (x as u32 as f32) / (u32::MAX as f32) * TAU
}

/// Sampled frequency-bin value for `slot` of `slots` around the ring, with the
/// bin index **rotated by the sweep** so the spectral shape scatters to a new
/// pseudo-random alignment on each beat instead of freezing at one angle.
pub fn swept_bin(freq: &[u8], step: usize, slot: usize, slots: usize, s: &RadialSetup) -> f32 {
    let off = ((s.sweep / TAU) * slots as f32) as usize % slots.max(1);
    let idx = (slot + off) % slots.max(1);
    bin_value(freq, step, idx)
}

/// Interleaved opposing bin mapping: even indices map to top half, odd indices map to opposite bottom half,
/// ensuring full 360° opposing balance without consecutive clustering.
pub fn opposing_interleaved_bin(freq: &[u8], step: usize, slot: usize, slots: usize, beat_count: u64) -> f32 {
    let sweep = sweep_angle(beat_count);
    let off = ((sweep / TAU) * slots as f32) as usize % slots.max(1);
    let mapped = if slot % 2 == 0 {
        slot / 2
    } else {
        slots.saturating_sub(1) - (slot - 1) / 2
    };
    let idx = (mapped + off) % slots.max(1);
    bin_value(freq, step, idx)
}

/// Fully randomized / prime-scattered bin mapping: scatters frequency bins randomly across 360° using prime permutation.
pub fn full_random_scattered_bin(freq: &[u8], step: usize, slot: usize, slots: usize, beat_count: u64) -> f32 {
    let sweep = sweep_angle(beat_count);
    let off = ((sweep / TAU) * slots as f32) as usize % slots.max(1);
    let mapped = (slot * 17) % slots.max(1);
    let idx = (mapped + off) % slots.max(1);
    bin_value(freq, step, idx)
}

/// Spatially smoothed frequency bin mapping for liquid surfaces:
/// Performs a 3-tap weighted moving average over adjacent slots around a 360° ring
/// so that liquid boundaries form silky smooth continuous organic curves without sharp zig-zag spikes.
pub fn smooth_ring_bin(freq: &[u8], step: usize, slot: usize, slots: usize) -> f32 {
    if slots == 0 || freq.is_empty() {
        return 0.0;
    }
    let prev = if slot == 0 { slots - 1 } else { slot - 1 };
    let next = (slot + 1) % slots;

    let v_prev = bin_value(freq, step, prev);
    let v_curr = bin_value(freq, step, slot);
    let v_next = bin_value(freq, step, next);

    v_prev * 0.25 + v_curr * 0.50 + v_next * 0.25
}

/// Beat prominence bump (0..1) centred on the sweep angle: on each beat the
/// strong emphasis appears at a fresh pseudo-random angle (see
/// [`sweep_angle`]), so beats light up scattered sectors instead of marching
/// around in order. `be`/`bs` keep the uniform pumping for everyone.
pub fn beat_bump(s: &RadialSetup, angle: f32) -> f32 {
    beat_bump_at(s.sweep, s.bs, angle)
}

/// Beat prominence bump from raw sweep/beat values (for styles that do not go
/// through [`RadialSetup`], e.g. the standalone `radial` crown).
pub fn beat_bump_at(sweep: f32, bs: f32, angle: f32) -> f32 {
    let mut d = (angle - sweep).rem_euclid(TAU);
    if d > PI {
        d = TAU - d;
    }
    let width = 0.5f32;
    let falloff = ((d / width).powi(2)).exp().min(1.0);
    (1.0 - falloff) * bs.min(1.5)
}

/// Draws the shared glowing centre ring, black disc and the user's radial
/// centre image, then restores the transform opened by [`setup`].
pub fn finish(c: &mut GpuCanvas, ctx: &RenderContext, s: &RadialSetup) {
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(s.glow));
    c.set_line_width((3.0 + s.be * 2.0) * s.user_scale);
    c.set_shadow(s.glow, (16.0 + s.bs * 12.0) * s.user_scale);
    c.stroke_circle(s.cx, s.cy, s.inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(s.cx, s.cy, s.inner_r * 0.96);

    draw_radial_center_image(c, ctx, s.cx, s.cy, s.inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
