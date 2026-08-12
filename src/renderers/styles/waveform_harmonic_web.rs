//! Waveform Harmonic Web style renderer (`waveformHarmonicWeb`).
//!
//! Interlaced String Harmonic Web:
//! - Optical fiber neon strings cross-connecting waveform sample points at phase offsets,
//!   forming an intricate 3D string art geometric mesh.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

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

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let pcm = ctx.time_data;

    let cx = width  * 0.5 + pos_offset_x;
    let cy = height * 0.50 + pos_offset_y;

    let span_w = width * 0.88 * user_scale;
    let start_x = cx - span_w * 0.5;
    let max_amp = height * 0.36 * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#02010a")));
//     c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, width * 0.65,
        &[
            (0.00, mix(glow, Color::rgba(0.0, 0.9, 1.0, 0.3), 0.5).with_alpha(0.22 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
    c.fill_rect(0.0, 0.0, width, height);

    let node_count = 60usize;
    let step_p = (pcm.len() / node_count).max(1);
    let mut nodes: Vec<(f32, f32)> = Vec::with_capacity(node_count);

    for i in 0..node_count {
        let t = i as f32 / (node_count - 1) as f32;
        let x = start_x + t * span_w;

        let sample_idx = (i * step_p).min(pcm.len().saturating_sub(1));
        let val = (pcm[sample_idx] as f32 / 128.0 - 1.0) * sensitivity;
        let y = cy + val * max_amp;

        nodes.push((x, y));
    }

    // Interlace String Mesh Connections (Connect node i with node i+offset)
    let offsets = [3, 7, 13, 21];
    for &offset in &offsets {
        for i in 0..node_count.saturating_sub(offset) {
            let (x1, y1) = nodes[i];
            let (x2, y2) = nodes[i + offset];

            let t = i as f32 / node_count as f32;
            let string_col = mix(
                mix(p_col, acc_col, t),
                mix(glow, Color::rgba(0.0, 0.9, 1.0, 0.8), offset as f32 / 21.0),
                0.5,
            );

            let alpha = (0.45 - (offset as f32 / 30.0)).clamp(0.08, 0.45);
            c.set_stroke(Fill::Solid(string_col.with_alpha(alpha)));
            c.set_line_width(0.9 * user_scale);
            c.set_shadow(string_col, 4.0 * user_scale);
            c.stroke_line(x1, y1, x2, y2);
        }
    }

    // Core Waveform Line
    if nodes.len() >= 2 {
        let core_col = mix(acc_col, Color::rgba(1.0, 1.0, 1.0, 0.95), 0.4);
        c.set_stroke(Fill::Solid(core_col));
        c.set_line_width(2.2 * user_scale);
        c.set_shadow(core_col, 10.0 * user_scale);
        c.stroke_polyline(&nodes);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
