//! Native 3D GPU Engine module (built on the lightweight `glam` math crate).
//!
//! Replaced the OpenGL-based `three-d` / `three-d-asset` crates: this app's
//! visualizer renders headless (no window/surface — frames are rasterized into
//! an off-screen texture and read back as RGBA for the live preview and FFmpeg
//! export), so a window/swapchain-oriented GL renderer was the wrong fit.
//!
//! Camera model: the default eye sits at z = +h/2 looking at the target with a
//! 90° vertical FOV. `cam_zoom` scales the eye distance; `cam_yaw`/`cam_pitch`
//! swing the eye around the target; `target_x`/`target_y` pan the camera target
//! in 3D world space without distorting perspective angles.

use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec3};

/// Vertical field of view. 90° keeps the z = 0 plane pixel-exact for the
/// default camera distance (h/2): half-height = dist * tan(fov/2) = h/2.
pub const FOV_Y: f32 = FRAC_PI_2;

/// Build the view-projection matrix for a canvas of `width`×`height` with the
/// given per-style camera tuning and target panning.
pub fn view_proj(
  width: u32,
  height: u32,
  cam_yaw: f32,
  cam_pitch: f32,
  cam_zoom: f32,
  target_x: f32,
  target_y: f32,
) -> Mat4 {
  let (w, h) = (width as f32, height.max(1) as f32);
  let dist = (h * 0.5 * cam_zoom).max(1.0);
  let target = Vec3::new(target_x, target_y, 0.0);
  let offset = (Mat4::from_rotation_y(cam_yaw) * Mat4::from_rotation_x(cam_pitch) * Vec3::new(0.0, 0.0, dist).extend(1.0))
    .truncate();
  let eye = target + offset;
  let view = glam::camera::rh::view::look_at_mat4(eye, target, Vec3::Y);
  let proj = glam::camera::rh::proj::vulkan::perspective(FOV_Y, w / h, dist * 0.0001, dist * 20.0);
  let flip_y = Mat4::from_scale(Vec3::new(1.0, -1.0, 1.0));
  flip_y * proj * view
}
