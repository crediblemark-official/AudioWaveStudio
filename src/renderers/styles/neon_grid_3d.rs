//! Neon Grid 3D style renderer (`neonGrid3D`) — Tron Audio Terrain Engine.
//!
//! Manual perspective projection for full visual control:
//! - A 2D frequency grid projected in 3D perspective (Tron / synthwave aesthetic).
//! - Each column of the grid represents a frequency band; height = amplitude.
//! - Grid lines drawn as polylines with neon glow, brighter where audio peaks.
//! - Cross-lines (depth rows) connect columns at each Z slice.
//! - Floor grid reflection mirrors the terrain below the baseline.
//! - Slow camera yaw + beat-driven zoom pulse.
//! - Depth-based color fade (near = bright, far = dim).

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const COLS: usize = 48;  // frequency columns (X axis)
const ROWS: usize = 20;  // depth slices    (Z axis)

/// Project a 3D world point to 2D screen coords.
/// Camera looks down the -Z axis from slightly above.
fn project(wx: f32, wy: f32, wz: f32, cam_yaw: f32, fov: f32, screen_cx: f32, screen_cy: f32) -> (f32, f32) {
    // Rotate around Y axis (yaw)
    let rx = wx * cam_yaw.cos() + wz * cam_yaw.sin();
    let rz = -wx * cam_yaw.sin() + wz * cam_yaw.cos() + 5.0; // push away from camera

    if rz <= 0.01 { return (screen_cx, screen_cy); }

    let sx = screen_cx + (rx / rz) * fov;
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
    let bar_count    = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq      = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let screen_cx = width  * 0.5 + pos_offset_x;
    let screen_cy = height * 0.54 + pos_offset_y;

    let step_f = (freq.len() / bar_count).max(1);

    // Camera parameters
    let cam_yaw  = (frame_time * 0.12).sin() * 0.18; // slow sweep
    let fov      = (width * 0.70 + be * 30.0) * user_scale; // beat zoom pulse

    // World grid dimensions
    let grid_w = 2.8f32;  // world units wide
    let grid_d = 2.0f32;  // world units deep
    let max_amp = 1.0f32 * user_scale; // world units tall (max bar height)

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. BACKGROUND — deep synthwave sky gradient
    // -------------------------------------------------------------------------
    let sky_grad = Fill::linear_gradient(
        screen_cx, 0.0, screen_cx, screen_cy,
        &[
            (0.00, Color::hex("#020109")),
            (0.55, mix(Color::hex("#08031a"), p_col.with_alpha(1.0), 0.06)),
            (1.00, mix(Color::hex("#0d0520"), acc_col.with_alpha(1.0), 0.08)),
        ],
    );
    c.set_fill(sky_grad);
    c.fill_rect(0.0, 0.0, width, screen_cy + height * 0.05);

    // Ground below horizon
    let ground_grad = Fill::linear_gradient(
        screen_cx, screen_cy, screen_cx, height,
        &[
            (0.00, mix(Color::hex("#050215"), p_col.with_alpha(1.0), 0.05)),
            (1.00, Color::hex("#020109")),
        ],
    );
    c.set_fill(ground_grad);
    c.fill_rect(0.0, screen_cy, width, height - screen_cy);

    // Horizon glow
    let hglow = Fill::radial_gradient(
        screen_cx, screen_cy, 0.0,
        screen_cx, screen_cy, width * 0.65,
        &[
            (0.00, mix(p_col, acc_col, 0.5).with_alpha(0.30 + be * 0.15)),
            (0.40, mix(glow, s_col, 0.4).with_alpha(0.12 + be * 0.06)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(hglow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 2. BUILD HEIGHT FIELD
    //    heights[row][col] — row 0 = nearest, row ROWS-1 = farthest
    // -------------------------------------------------------------------------
    let n_cols = COLS.min(bar_count);
    let mut heights = vec![vec![0.0f32; n_cols]; ROWS];

    for col in 0..n_cols {
        let bin_k = (col * step_f / ((n_cols / bar_count.max(1)).max(1)))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;
        let h = fv * max_amp * sensitivity + be * max_amp * 0.08;

        for row in 0..ROWS {
            // Heights taper with distance (far rows have lower peaks)
            let row_scale = 1.0 - row as f32 / ROWS as f32 * 0.45;
            // Each row gets a small time-offset wave for "terrain ripple"
            let ripple_phase = frame_time * 1.4 - row as f32 * 0.18;
            let ripple = (ripple_phase + col as f32 * 0.22).sin() * 0.04 * row_scale;
            heights[row][col] = (h * row_scale + ripple).max(0.0);
        }
    }

    // -------------------------------------------------------------------------
    // 3. PROJECT ALL GRID POINTS TO SCREEN
    // -------------------------------------------------------------------------
    let mut pts = vec![vec![(0.0f32, 0.0f32); n_cols]; ROWS];

    for row in 0..ROWS {
        for col in 0..n_cols {
            let t_col = col as f32 / (n_cols - 1).max(1) as f32;
            let t_row = row as f32 / (ROWS  - 1).max(1) as f32;

            let wx = (t_col - 0.5) * grid_w;
            let wy = heights[row][col];
            let wz = t_row * grid_d; // 0=near, grid_d=far

            pts[row][col] = project(wx, wy, wz, cam_yaw, fov, screen_cx, screen_cy);
        }
    }

    // Floor points (wy = 0) — used for reflections and grid floor
    let mut floor_pts = vec![vec![(0.0f32, 0.0f32); n_cols]; ROWS];
    for row in 0..ROWS {
        for col in 0..n_cols {
            let t_col = col as f32 / (n_cols - 1).max(1) as f32;
            let t_row = row as f32 / (ROWS  - 1).max(1) as f32;
            let wx = (t_col - 0.5) * grid_w;
            let wz = t_row * grid_d;
            floor_pts[row][col] = project(wx, 0.0, wz, cam_yaw, fov, screen_cx, screen_cy);
        }
    }

    // -------------------------------------------------------------------------
    // 4. DRAW TERRAIN SURFACE — filled quads back-to-front
    // -------------------------------------------------------------------------
    // Draw back rows first for correct painter's algorithm
    for row in (0..ROWS - 1).rev() {
        let depth_t = row as f32 / (ROWS - 1) as f32; // 0=near, 1=far

        for col in 0..n_cols.saturating_sub(1) {
            let t_col  = (col as f32 + 0.5) / n_cols as f32;
            let fv_mid = heights[row][col].max(heights[row][col + 1]) / max_amp;

            // Color: position-based + height-based blend
            let surface_col = mix(
                mix(p_col, acc_col, t_col),
                mix(acc_col, glow, fv_mid),
                fv_mid * 0.7 + depth_t * 0.1,
            );
            // Fade with depth
            let alpha = (0.82 - depth_t * 0.45).clamp(0.15, 0.85);

            // Quad: near-left, near-right, far-right, far-left
            let p0 = pts[row    ][col    ];
            let p1 = pts[row    ][col + 1];
            let p2 = pts[row + 1][col + 1];
            let p3 = pts[row + 1][col    ];

            // Skip degenerate quads (behind camera)
            if [p0, p1, p2, p3].iter().any(|&(x, y)| x.is_nan() || y.is_nan()) { continue; }

            let quad_poly = vec![p0, p1, p2, p3];
            c.set_fill(Fill::Solid(surface_col.with_alpha(alpha)));
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.fill_polygon(&quad_poly);
        }
    }

    // -------------------------------------------------------------------------
    // 5. DRAW GRID LINES — neon wireframe on top of surfaces
    // -------------------------------------------------------------------------

    // Longitudinal lines (along Z/depth axis) — one per column
    for col in 0..n_cols {
        let t_col = col as f32 / (n_cols - 1).max(1) as f32;
        let fv_max = heights.iter().map(|row| row[col]).fold(0.0f32, f32::max);
        let line_col = mix(
            mix(p_col, acc_col, t_col),
            mix(glow, Color::rgba(1.0, 1.0, 1.0, 1.0), fv_max),
            fv_max * 0.55,
        );
        let lw = (0.8 + fv_max * 2.5 + be * 1.0) * user_scale;

        let col_pts: Vec<(f32, f32)> = (0..ROWS).map(|r| pts[r][col]).collect();

        c.set_stroke(Fill::Solid(line_col.with_alpha(0.85)));
        c.set_line_width(lw);
        c.set_shadow(line_col, (6.0 + fv_max * 10.0) * user_scale);
        c.stroke_polyline(&col_pts);
    }

    // Latitudinal lines (along X axis) — one per row (depth slice)
    for row in 0..ROWS {
        let depth_t = row as f32 / (ROWS - 1) as f32;
        let fv_row_max = heights[row].iter().cloned().fold(0.0f32, f32::max);
        let row_col = mix(
            mix(s_col, p_col, depth_t),
            glow,
            fv_row_max * 0.4,
        );
        let alpha = (0.65 - depth_t * 0.38).clamp(0.12, 0.70);
        let lw    = (0.5 + fv_row_max * 1.5) * user_scale;

        let row_pts: Vec<(f32, f32)> = (0..n_cols).map(|c| pts[row][c]).collect();

        c.set_stroke(Fill::Solid(row_col.with_alpha(alpha)));
        c.set_line_width(lw);
        c.set_shadow(row_col, (3.0 + fv_row_max * 6.0) * user_scale);
        c.stroke_polyline(&row_pts);
    }

    // -------------------------------------------------------------------------
    // 6. FLOOR GRID — flat neon grid below the terrain
    // -------------------------------------------------------------------------
    // Longitudinal floor lines
    for col in (0..n_cols).step_by(3) {
        let t_col   = col as f32 / (n_cols - 1).max(1) as f32;
        let fl_col  = mix(p_col, acc_col, t_col).with_alpha(0.18 + be * 0.06);
        let fl_pts: Vec<(f32, f32)> = (0..ROWS).map(|r| floor_pts[r][col]).collect();
        c.set_stroke(Fill::Solid(fl_col));
        c.set_line_width(0.6 * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&fl_pts);
    }
    // Latitudinal floor lines
    for row in (0..ROWS).step_by(2) {
        let depth_t = row as f32 / (ROWS - 1) as f32;
        let fl_col  = mix(acc_col, glow, depth_t).with_alpha((0.22 - depth_t * 0.12).max(0.04));
        let fl_pts: Vec<(f32, f32)> = (0..n_cols).map(|c| floor_pts[row][c]).collect();
        c.set_stroke(Fill::Solid(fl_col));
        c.set_line_width(0.6 * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&fl_pts);
    }

    // -------------------------------------------------------------------------
    // 7. BEAT FLASH — full-screen white flash on hard beat
    // -------------------------------------------------------------------------
    if bs > 0.55 {
        let flash_a = (bs - 0.55) * 0.35;
        c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, flash_a)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_rect(0.0, 0.0, width, height);
    }

    let _ = (s_col, PI);

    c.set_global_alpha(1.0);
    c.restore();
}
