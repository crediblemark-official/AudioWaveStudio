//! Dual Wave Horizon style renderer (`dualWaveHorizon`) — 3D Twin Wave Canyon & Cyber Sun Engine.
//!
//! Re-imagined 3D Horizon Design:
//! - Twin 3D perspective audio wave walls receding into a central horizon vanishing point.
//! - Left wall driven by low/mid frequencies; Right wall driven by mid/high frequencies.
//! - Glowing Retro Cyber Sun & energy halo pulse at the central horizon vanishing point.
//! - Synthwave perspective floor grid converging into the horizon sun.
//! - Dynamic particles drifting through the 3D horizon space.

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const SEGS_3D: usize = 36;
const PARTICLE_COUNT: usize = 30;

/// Project 3D world coords to 2D screen coords
fn project_horizon(wx: f32, wy: f32, wz: f32, fov: f32, screen_cx: f32, screen_cy: f32) -> (f32, f32) {
    let rz = wz + 4.0; // Push depth back
    if rz <= 0.05 { return (screen_cx, screen_cy); }
    let sx = screen_cx + (wx / rz) * fov;
    let sy = screen_cy - (wy / rz) * fov;
    (sx, sy)
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col   = theme_primary(theme);
    let s_col   = theme_secondary(theme);
    let acc_col = theme_accent(theme);
    let glow    = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width  * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;
    let bar_count    = ctx.config.reactivity.bar_count.clamp(16, 64);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq      = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let step_f = (freq.len() / bar_count).max(1);

    let screen_cx = width  * 0.5 + pos_offset_x;
    let screen_cy = height * 0.52 + pos_offset_y;
    let fov       = (width * 0.58 + be * 20.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. BACKGROUND — Deep Midnight Synthwave Sky
    // -------------------------------------------------------------------------
    let _bg = Fill::linear_gradient(
        screen_cx, 0.0, screen_cx, height,
        &[
            (0.00, Color::hex("#02010a")),
            (0.50, mix(Color::hex("#0c0326"), p_col.with_alpha(1.0), 0.14)),
            (0.75, mix(Color::hex("#150433"), acc_col.with_alpha(1.0), 0.10)),
            (1.00, Color::hex("#010006")),
        ],
    );
//     c.set_fill(bg);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 2. CYBER SUN & HORIZON VANISHING POINT GLOW
    // -------------------------------------------------------------------------
    let sun_r = (32.0 + be * 24.0 + bs * 10.0) * user_scale;

    // Outer Sun Glow
    let sun_glow = Fill::radial_gradient(
        screen_cx, screen_cy, 0.0,
        screen_cx, screen_cy, sun_r * 3.5,
        &[
            (0.00, Color::rgba(1.0, 0.30, 0.70, 0.60 + be * 0.20)),
            (0.35, Color::rgba(0.0, 0.80, 1.0, 0.25 + be * 0.10)),
            (0.70, mix(glow, Color::rgba(0.5, 0.1, 0.8, 0.08), 0.5)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(sun_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // Sun Disc Body
    let sun_fill = Fill::linear_gradient(
        screen_cx, screen_cy - sun_r,
        screen_cx, screen_cy + sun_r,
        &[
            (0.00, Color::rgba(1.0, 0.95, 0.30, 0.98)), // Yellow top
            (0.50, Color::rgba(1.0, 0.30, 0.60, 0.95)), // Pink mid
            (1.00, Color::rgba(0.80, 0.10, 0.40, 0.90)), // Magenta bottom
        ],
    );
    c.set_fill(sun_fill);
    c.set_shadow(Color::rgba(1.0, 0.30, 0.60, 0.90), 22.0 * user_scale);
    c.fill_circle(screen_cx, screen_cy, sun_r);

    // Sun Horizontal Cut Lines (Retro Sun Grid)
    for cut in 1..=4 {
        let cut_f = cut as f32 / 5.0;
        let cut_y = screen_cy + (cut_f - 0.2) * sun_r * 1.6;
        let cut_h = (1.5 + cut_f * 2.5) * user_scale;
        c.set_fill(Fill::Solid(Color::hex("#02010a")));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_rect(screen_cx - sun_r * 1.1, cut_y, sun_r * 2.2, cut_h);
    }

    // -------------------------------------------------------------------------
    // 3. PERSPECTIVE FLOOR GRID (Converging to Cyber Sun)
    // -------------------------------------------------------------------------
    let floor_grid_lines = 16usize;
    let grid_max_z = 8.0f32;

    // Longitudinal perspective grid lines
    for i in 0..=floor_grid_lines {
        let t = i as f32 / floor_grid_lines as f32;
        let wx = (t - 0.5) * 12.0 * user_scale;

        let p_near = project_horizon(wx, -0.8 * user_scale, 0.2, fov, screen_cx, screen_cy);
        let p_far  = project_horizon(wx * 0.1, -0.8 * user_scale, grid_max_z, fov, screen_cx, screen_cy);

        let line_col = mix(Color::rgba(0.0, 0.85, 1.0, 0.6), acc_col, t).with_alpha(0.35 + be * 0.10);
        c.set_stroke(Fill::Solid(line_col));
        c.set_line_width((1.0 + be * 0.5) * user_scale);
        c.set_shadow(line_col, 4.0 * user_scale);
        c.stroke_line(p_near.0, p_near.1, p_far.0, p_far.1);
    }

    // Latitudinal horizontal grid lines
    let lat_grid_count = 10usize;
    for j in 0..=lat_grid_count {
        let t = j as f32 / lat_grid_count as f32;
        let wz = 0.2 + t * t * (grid_max_z - 0.2); // Exponential depth spacing

        let p_left  = project_horizon(-6.0 * user_scale * (1.0 - t * 0.8), -0.8 * user_scale, wz, fov, screen_cx, screen_cy);
        let p_right = project_horizon( 6.0 * user_scale * (1.0 - t * 0.8), -0.8 * user_scale, wz, fov, screen_cx, screen_cy);

        let alpha = (0.35 - t * 0.22).max(0.05);
        let line_col = mix(Color::rgba(0.0, 0.75, 1.0, 0.8), s_col, t).with_alpha(alpha + be * 0.05);
        c.set_stroke(Fill::Solid(line_col));
        c.set_line_width((0.8 + (1.0 - t) * 0.6) * user_scale);
        c.set_shadow(line_col, 3.0 * user_scale);
        c.stroke_line(p_left.0, p_left.1, p_right.0, p_right.1);
    }

    // -------------------------------------------------------------------------
    // 4. DUAL 3D WAVE CANYON WALLS (Left & Right Channels)
    // -------------------------------------------------------------------------
    let max_wave_h = 2.2 * user_scale;

    for side in [-1.0f32, 1.0f32] {
        let is_left = side < 0.0;

        let mut wall_base_pts: Vec<(f32, f32)> = Vec::with_capacity(SEGS_3D + 1);
        let mut wall_top_pts: Vec<(f32, f32)>  = Vec::with_capacity(SEGS_3D + 1);
        let mut wall_spikes: Vec<((f32, f32), (f32, f32), Color)> = Vec::with_capacity(SEGS_3D);

        for i in 0..=SEGS_3D {
            let t = i as f32 / SEGS_3D as f32; // 0=near, 1=far horizon
            let wz = 0.2 + t * (grid_max_z - 0.2);

            // Channel frequency bin index
            let bin_idx = if is_left {
                (i * step_f / 2).min(freq.len().saturating_sub(1))
            } else {
                ((SEGS_3D - i) * step_f / 2 + freq.len() / 2).min(freq.len().saturating_sub(1))
            };
            let fv = freq[bin_idx] as f32 / 255.0;

            // Wave height calculation
            let wave_decay = 1.0 - t * 0.55; // Taper toward horizon
            let osc = (frame_time * 2.0 + t * PI * 4.0 + side * 1.5).sin() * 0.15;
            let h = (fv * max_wave_h * sensitivity + be * max_wave_h * 0.15 + osc)
                .clamp(0.1 * user_scale, max_wave_h) * wave_decay;

            // Canyon wall X position (curves slightly inward near horizon)
            let wx = side * (1.8 + (1.0 - t) * 1.4) * user_scale;

            let p_base = project_horizon(wx, -0.8 * user_scale, wz, fov, screen_cx, screen_cy);
            let p_top  = project_horizon(wx, -0.8 * user_scale + h, wz, fov, screen_cx, screen_cy);

            wall_base_pts.push(p_base);
            wall_top_pts.push(p_top);

            // Vertical 3D Equalizer Pillar Line along the wall
            if i < SEGS_3D {
                let col = if is_left {
                    mix(
                        Color::rgba(0.0, 0.90, 1.0, 0.9),  // Cyan
                        Color::rgba(0.9, 0.20, 0.85, 0.9), // Pink
                        fv,
                    )
                } else {
                    mix(
                        Color::rgba(1.0, 0.30, 0.50, 0.9), // Coral
                        Color::rgba(1.0, 0.90, 0.20, 0.9), // Yellow
                        fv,
                    )
                };
                wall_spikes.push((p_base, p_top, col));
            }
        }

        // --- Render Wall Quads & Ribbons ---
        // A. Wall Translucent Fill
        for i in 0..SEGS_3D {
            let t = i as f32 / SEGS_3D as f32;
            let alpha = (0.45 - t * 0.30).max(0.06);

            let quad = vec![
                wall_base_pts[i],
                wall_top_pts[i],
                wall_top_pts[i + 1],
                wall_base_pts[i + 1],
            ];

            let fill_col = if is_left {
                mix(p_col, Color::rgba(0.0, 0.70, 1.0, alpha), 0.5)
            } else {
                mix(acc_col, Color::rgba(1.0, 0.20, 0.60, alpha), 0.5)
            };

            c.set_fill(Fill::Solid(fill_col.with_alpha(alpha)));
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.fill_polygon(&quad);
        }

        // B. Vertical Equalizer Pillars on Wall
        for ((base_p, top_p, spike_col), i) in wall_spikes.iter().zip(0..) {
            let t = i as f32 / SEGS_3D as f32;
            let alpha = (0.90 - t * 0.45).clamp(0.20, 0.95);
            let lw = (1.5 + (1.0 - t) * 2.0 + be * 1.0) * user_scale;

            c.set_stroke(Fill::Solid(spike_col.with_alpha(alpha)));
            c.set_line_width(lw);
            c.set_shadow(*spike_col, (6.0 * (1.0 - t)) * user_scale);
            c.stroke_line(base_p.0, base_p.1, top_p.0, top_p.1);

            // Pillar Top Cap Glow Dot
            c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, alpha)));
            c.set_shadow(*spike_col, 8.0 * user_scale);
            c.fill_circle(top_p.0, top_p.1, (2.2 * (1.0 - t * 0.5)) * user_scale);
        }

        // C. Glowing Wave Crest Line along Top of Wall
        let crest_col = if is_left {
            Color::rgba(0.0, 0.95, 1.0, 0.95)
        } else {
            Color::rgba(1.0, 0.35, 0.85, 0.95)
        };

        c.set_stroke(Fill::Solid(crest_col.with_alpha(0.30)));
        c.set_line_width(12.0 * user_scale);
        c.set_shadow(crest_col, 16.0 * user_scale);
        c.stroke_polyline(&wall_top_pts);

        c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.98)));
        c.set_line_width(1.8 * user_scale);
        c.set_shadow(crest_col, 8.0 * user_scale);
        c.stroke_polyline(&wall_top_pts);
    }

    // -------------------------------------------------------------------------
    // 5. DRIFTING HORIZON PARTICLES
    // -------------------------------------------------------------------------
    for p in 0..PARTICLE_COUNT {
        let pf = p as f32;
        let speed = 0.20 + (p % 5) as f32 * 0.08;
        let t_z = (frame_time * speed + pf * 0.19) % 1.0;
        let wz = 0.2 + t_z * (grid_max_z - 0.2);

        let angle = (pf * 0.618034 * PI * 2.0) + frame_time * 0.2;
        let r_dist = (0.5 + (p % 4) as f32 * 0.6) * user_scale;
        let wx = angle.cos() * r_dist;
        let wy = -0.5 * user_scale + (angle.sin() * 0.6 + 0.6) * max_wave_h;

        let pt = project_horizon(wx, wy, wz, fov, screen_cx, screen_cy);
        let alpha = (1.0 - t_z) * (0.4 + (p % 3) as f32 * 0.2);

        let p_col_curr = mix(Color::rgba(0.0, 0.9, 1.0, alpha), Color::rgba(1.0, 0.3, 0.8, alpha), (p % 2) as f32);
        c.set_fill(Fill::Solid(p_col_curr));
        c.set_shadow(p_col_curr, 6.0 * user_scale);
        c.fill_circle(pt.0, pt.1, (1.5 + (p % 3) as f32 * 1.0) * user_scale * (1.0 - t_z * 0.5));
    }

    // -------------------------------------------------------------------------
    // 6. BEAT FLASH
    // -------------------------------------------------------------------------
    if bs > 0.65 {
        let fa = (bs - 0.65) * 0.25;
        c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, fa)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
//         c.fill_rect(0.0, 0.0, width, height);
    }

    let _ = (s_col, PI);

    c.set_global_alpha(1.0);
    c.restore();
}
