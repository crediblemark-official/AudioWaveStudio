//! Background image scaling parity tests.
//!
//! `upload_image_layer` downscales images larger than `LAYER_SIZE` (1024) with
//! a CPU area-average resample instead of a single bilinear sample, so the
//! exported background image keeps as much detail as the TS preview's
//! high-quality `drawImage`. These tests pin the resampler's behavior.

use audiowave_studio_lib::gpu2d::GpuRenderer;

fn px(rgba: &[u8], w: usize, x: usize, y: usize) -> [u8; 4] {
  let o = (y * w + x) * 4;
  [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]]
}

#[test]
fn area_average_resize_is_identity_at_1_to_1() {
  let src: Vec<u8> = (0..4 * 4 * 4).map(|i| (i * 7) as u8).collect();
  let out = GpuRenderer::area_average_resize(&src, 4, 4, 4, 4);
  assert_eq!(out, src, "1:1 resize must be an exact copy");
}

#[test]
fn area_average_resize_averages_checkerboard() {
  // 4x4 black/white checkerboard -> 2x2 must be mid-gray (not black/white
  // like a nearest/bilinear sample would give).
  let mut src = vec![0u8; 4 * 4 * 4];
  for y in 0..4 {
    for x in 0..4 {
      let v = if (x + y) % 2 == 0 { 255u8 } else { 0u8 };
      let o = (y * 4 + x) * 4;
      src[o] = v;
      src[o + 1] = v;
      src[o + 2] = v;
      src[o + 3] = 255;
    }
  }
  let out = GpuRenderer::area_average_resize(&src, 4, 4, 2, 2);
  for y in 0..2 {
    for x in 0..2 {
      let p = px(&out, 2, x, y);
      assert!(
        (p[0] as i32 - 127).abs() <= 1,
        "checkerboard cell must average to ~127, got {}",
        p[0]
      );
      assert_eq!(p[3], 255, "alpha must be preserved");
    }
  }
}

#[test]
fn area_average_resize_preserves_sharp_edge() {
  // 32x32 left=white right=black -> 16x16 keeps a clean vertical split.
  let mut src = vec![0u8; 32 * 32 * 4];
  for y in 0..32 {
    for x in 0..32 {
      let v = if x < 16 { 255u8 } else { 0u8 };
      let o = (y * 32 + x) * 4;
      src[o] = v;
      src[o + 1] = v;
      src[o + 2] = v;
      src[o + 3] = 255;
    }
  }
  let out = GpuRenderer::area_average_resize(&src, 32, 32, 16, 16);
  for y in 0..16 {
    for x in 0..16 {
      let p = px(&out, 16, x, y);
      let expect = if x < 8 { 255u8 } else { 0u8 };
      assert_eq!(p[0], expect, "sharp edge must survive downscale");
    }
  }
}
