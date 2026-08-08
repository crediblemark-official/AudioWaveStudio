//! Regression checks for the native 3D pass: world +y must render on top
//! (a Y-flip in clip space is required because wgpu maps clip +y to the top of
//! the texture while glam's `vulkan::perspective` emits y-down NDC), +x must
//! stay on the right (no horizontal mirror), and the depth test must keep the
//! nearer box visible regardless of draw order.

use audiowave_studio_lib::gpu2d::{Color, GpuCanvas, GpuRenderer, Scene3D};

const W: u32 = 128;
const H: u32 = 128;

fn chan(buf: &[u8], x: u32, y: u32, c: usize) -> f32 {
    buf[(y * W + x) as usize * 4 + c] as f32 / 255.0
}

fn avg(buf: &[u8], x: u32, y: u32) -> f32 {
    let i = (y * W + x) as usize * 4;
    (buf[i] as f32 + buf[i + 1] as f32 + buf[i + 2] as f32) / 3.0 / 255.0
}

#[test]
fn three_d_orientation_is_upright() {
    let mut r = pollster::block_on(GpuRenderer::new(W, H)).expect("renderer");
    let canvas = GpuCanvas::new(W, H);
    let mut scene = Scene3D::new();
    // Box centred at world (0, +32, -20), world y spans 0..64.
    scene.add_box(0.0, 32.0, -20.0, 64.0, 64.0, 20.0, Color::WHITE);
    let mesh = canvas.finish_with(scene);
    r.render_into(&mesh, 0);
    let buf = r.readback(0);

    let upper = avg(&buf, W / 2, H / 4);
    let lower = avg(&buf, W / 2, 3 * H / 4);
    eprintln!("upper={upper:.3} lower={lower:.3}");
    assert!(upper > 0.3, "box at +y must appear in the TOP half (upper={upper:.3})");
    assert!(lower < 0.3, "box must NOT appear in the BOTTOM half (lower={lower:.3})");
}

#[test]
fn three_d_left_right_not_mirrored() {
    let mut r = pollster::block_on(GpuRenderer::new(W, H)).expect("renderer");
    let canvas = GpuCanvas::new(W, H);
    let mut scene = Scene3D::new();
    // Bright red box on the right (+x). Its front face lights mostly red.
    scene.add_box(32.0, 0.0, -20.0, 64.0, 64.0, 20.0, Color::rgba(1.0, 0.0, 0.0, 1.0));
    let mesh = canvas.finish_with(scene);
    r.render_into(&mesh, 0);
    let buf = r.readback(0);

    let right_r = chan(&buf, 3 * W / 4, H / 2, 0);
    let right_g = chan(&buf, 3 * W / 4, H / 2, 1);
    let left_r = chan(&buf, W / 4, H / 2, 0);
    eprintln!("right_r={right_r:.3} right_g={right_g:.3} left_r={left_r:.3}");
    assert!(right_r > 0.4, "box at +x must appear on the RIGHT (r={right_r:.3})");
    assert!(right_g < 0.3, "right sample must be red, not white (g={right_g:.3})");
    assert!(left_r < 0.3, "box must NOT appear on the LEFT (r={left_r:.3})");
}

#[test]
fn three_d_depth_occlusion() {
    let mut r = pollster::block_on(GpuRenderer::new(W, H)).expect("renderer");
    let canvas = GpuCanvas::new(W, H);
    let mut scene = Scene3D::new();
    // Front box added FIRST, back box added SECOND. Painter's order would let
    // the back (blue) box overpaint the front (white) one; the depth test must
    // keep the nearer white box visible.
    scene.add_box(0.0, 0.0, -20.0, 40.0, 40.0, 40.0, Color::WHITE);
    scene.add_box(0.0, 0.0, -100.0, 40.0, 40.0, 40.0, Color::rgba(0.0, 0.0, 1.0, 1.0));
    let mesh = canvas.finish_with(scene);
    r.render_into(&mesh, 0);
    let buf = r.readback(0);

    let centre = avg(&buf, W / 2, H / 2);
    eprintln!("centre={centre:.3}");
    // The white front box renders at ~0.86 avg; a bare blue back box averages
    // ~0.31 (only its blue channel lights). 0.6 cleanly separates the two.
    assert!(centre > 0.6, "nearer white box must occlude the blue back box (avg={centre:.3})");
}
