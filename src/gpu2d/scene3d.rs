//! Scene3D — a small retained-mode 3D geometry builder for the native wgpu
//! engine (`GpuRenderer`).
//!
//! Styles push triangles/quads/boxes/rings (CPU side, once per frame) through
//! a Canvas2D-style transform stack; `GpuRenderer` uploads the finished
//! triangles and draws them in a depth-tested pass right after the 2D scene
//! (see `GpuRenderer::render_into` / `record_scene_3d`).
//!
//! World conventions (right-handed, y-up, +z toward the viewer):
//!   - The origin is the centre of the frame.
//!   - The default camera sits at z = +h/2 looking at the origin with a 90°
//!     vertical FOV, so the z = 0 plane projects 1:1 onto frame pixels
//!     (see `crate::renderers::three_d_engine::view_proj`). Negative z
//!     recedes into the screen and gets smaller via perspective — that is what
//!     makes the "3D" read as real depth instead of the old fake transforms.
//!
//! Per-frame budget: the whole scene is rebuilt and re-uploaded every frame
//! (the same strategy as GpuCanvas), so no retained GPU state is needed.

use std::f32::consts::TAU;

use glam::{Mat4, Vec3};

use super::Color;

/// 3D vertex: world-space position, per-vertex normal and RGBA color.
/// The field layout must match `RawVertex3` in `renderer.rs` and the WGSL
/// vertex input in `three_d_shader.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct V3 {
  pub position: [f32; 3],
  pub normal: [f32; 3],
  pub color: [f32; 4],
}

pub struct Scene3D {
  verts: Vec<V3>,
  idx: Vec<u32>,
  stack: Vec<Mat4>,
  cur: Mat4,
  /// Camera tuning applied by `GpuRenderer` when it builds the view-projection:
  ///   - `cam_yaw`: rotation of the view around the world Y axis (radians),
  ///   - `cam_pitch`: rotation around the world X axis (radians),
  ///   - `cam_zoom`: 1.0 = default distance (h/2); <1 pulls the camera in for
  ///     a stronger perspective, >1 pushes it back.
  pub cam_yaw: f32,
  pub cam_pitch: f32,
  pub cam_zoom: f32,
  pub target_x: f32,
  pub target_y: f32,
}

impl Default for Scene3D {
  fn default() -> Self {
    Self::new()
  }
}

impl Scene3D {
  pub fn new() -> Scene3D {
    Scene3D {
      verts: Vec::new(),
      idx: Vec::new(),
      stack: Vec::new(),
      cur: Mat4::IDENTITY,
      cam_yaw: 0.0,
      cam_pitch: 0.0,
      cam_zoom: 1.0,
      target_x: 0.0,
      target_y: 0.0,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.idx.is_empty()
  }

  pub fn verts(&self) -> &[V3] {
    &self.verts
  }

  pub fn idx(&self) -> &[u32] {
    &self.idx
  }

  pub fn clear(&mut self) {
    self.verts.clear();
    self.idx.clear();
    self.stack.clear();
    self.cur = Mat4::IDENTITY;
    self.cam_yaw = 0.0;
    self.cam_pitch = 0.0;
    self.cam_zoom = 1.0;
  }

  // --- transform stack (last call is applied to the vertex FIRST, like Canvas2D) ---

  pub fn push(&mut self) {
    self.stack.push(self.cur);
  }

  pub fn pop(&mut self) {
    if let Some(m) = self.stack.pop() {
      self.cur = m;
    }
  }

  pub fn translate(&mut self, x: f32, y: f32, z: f32) {
    self.cur = self.cur * Mat4::from_translation(Vec3::new(x, y, z));
  }

  pub fn rotate_x(&mut self, a: f32) {
    self.cur = self.cur * Mat4::from_rotation_x(a);
  }

  pub fn rotate_y(&mut self, a: f32) {
    self.cur = self.cur * Mat4::from_rotation_y(a);
  }

  pub fn rotate_z(&mut self, a: f32) {
    self.cur = self.cur * Mat4::from_rotation_z(a);
  }

  pub fn scale(&mut self, sx: f32, sy: f32, sz: f32) {
    self.cur = self.cur * Mat4::from_scale(Vec3::new(sx, sy, sz));
  }

  fn push_vertex(&mut self, pos: Vec3, n: Vec3, c: Color) -> u32 {
    self.verts.push(V3 {
      position: self.cur.transform_point3(pos).to_array(),
      normal: self.cur.transform_vector3(n).normalize_or_zero().to_array(),
      color: [c.r, c.g, c.b, c.a],
    });
    (self.verts.len() - 1) as u32
  }

  fn push_tri(&mut self, a: Vec3, b: Vec3, c: Vec3, n: Vec3, col: Color) {
    let n = if n.length_squared() < 1e-9 { Vec3::Z } else { n.normalize() };
    let i0 = self.push_vertex(a, n, col);
    let i1 = self.push_vertex(b, n, col);
    let i2 = self.push_vertex(c, n, col);
    self.idx.extend_from_slice(&[i0, i1, i2]);
  }

  /// A two-triangle quad. The surface normal is derived from the winding
  /// order (a,b,c,d must trace the quad boundary consistently).
  fn push_quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, col: Color) {
    let n = (b - a).cross(d - a);
    self.push_tri(a, b, c, n, col);
    self.push_tri(a, c, d, n, col);
  }

  // --- primitives (local space, transformed by the active matrix) ---

  /// A quad from 4 local-space corners (winding order a,b,c,d).
  pub fn quad(&mut self, a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3], color: Color) {
    self.push_quad(
      Vec3::from_array(a),
      Vec3::from_array(b),
      Vec3::from_array(c),
      Vec3::from_array(d),
      color,
    );
  }

  /// Axis-aligned box centred at (cx,cy,cz) with full extents (sx,sy,sz).
  pub fn add_box(&mut self, cx: f32, cy: f32, cz: f32, sx: f32, sy: f32, sz: f32, color: Color) {
    let (x0, x1) = (cx - sx * 0.5, cx + sx * 0.5);
    let (y0, y1) = (cy - sy * 0.5, cy + sy * 0.5);
    let (z0, z1) = (cz - sz * 0.5, cz + sz * 0.5);
    // +z
    self.push_quad(v(x0, y0, z1), v(x1, y0, z1), v(x1, y1, z1), v(x0, y1, z1), color);
    // -z
    self.push_quad(v(x0, y0, z0), v(x0, y1, z0), v(x1, y1, z0), v(x1, y0, z0), color);
    // +y
    self.push_quad(v(x0, y1, z0), v(x0, y1, z1), v(x1, y1, z1), v(x1, y1, z0), color);
    // -y
    self.push_quad(v(x0, y0, z0), v(x1, y0, z0), v(x1, y0, z1), v(x0, y0, z1), color);
    // +x
    self.push_quad(v(x1, y0, z0), v(x1, y1, z0), v(x1, y1, z1), v(x1, y0, z1), color);
    // -x
    self.push_quad(v(x0, y0, z0), v(x0, y0, z1), v(x0, y1, z1), v(x0, y1, z0), color);
  }

  /// Flat disc in the XY plane (facing +z / -z), double-sided.
  pub fn add_disc(&mut self, cx: f32, cy: f32, cz: f32, r: f32, segments: u32, color: Color) {
    let c = Vec3::new(cx, cy, cz);
    let seg = segments.max(3);
    for i in 0..seg {
      let a0 = i as f32 / seg as f32 * TAU;
      let a1 = (i + 1) as f32 / seg as f32 * TAU;
      let p0 = c + Vec3::new(a0.cos() * r, a0.sin() * r, 0.0);
      let p1 = c + Vec3::new(a1.cos() * r, a1.sin() * r, 0.0);
      self.push_tri(c, p0, p1, Vec3::Z, color);
      self.push_tri(c, p1, p0, -Vec3::Z, color);
    }
  }

  /// Flat disc in the XZ plane (facing +y / -y, e.g. horizontal tabletop/deck), double-sided.
  pub fn add_disc_xz(&mut self, cx: f32, cy: f32, cz: f32, r: f32, segments: u32, color: Color) {
    let c = Vec3::new(cx, cy, cz);
    let seg = segments.max(3);
    for i in 0..seg {
      let a0 = i as f32 / seg as f32 * TAU;
      let a1 = (i + 1) as f32 / seg as f32 * TAU;
      let p0 = c + Vec3::new(a0.cos() * r, 0.0, a0.sin() * r);
      let p1 = c + Vec3::new(a1.cos() * r, 0.0, a1.sin() * r);
      self.push_tri(c, p0, p1, Vec3::Y, color);
      self.push_tri(c, p1, p0, -Vec3::Y, color);
    }
  }

  /// Volumetric 3D cylinder along Y axis with top & bottom caps parallel to XZ plane.
  pub fn add_cylinder_y(
    &mut self,
    cx: f32,
    cy: f32,
    cz: f32,
    r: f32,
    height: f32,
    segments: u32,
    color: Color,
  ) {
    let seg = segments.max(3);
    let h_half = height * 0.5;
    let yt = cy + h_half;
    let yb = cy - h_half;
    let ct = Vec3::new(cx, yt, cz);
    let cb = Vec3::new(cx, yb, cz);

    for i in 0..seg {
      let a0 = i as f32 / seg as f32 * TAU;
      let a1 = (i + 1) as f32 / seg as f32 * TAU;
      let (cos0, sin0) = (a0.cos(), a0.sin());
      let (cos1, sin1) = (a1.cos(), a1.sin());

      let p0_top = Vec3::new(cx + cos0 * r, yt, cz + sin0 * r);
      let p1_top = Vec3::new(cx + cos1 * r, yt, cz + sin1 * r);
      let p0_bot = Vec3::new(cx + cos0 * r, yb, cz + sin0 * r);
      let p1_bot = Vec3::new(cx + cos1 * r, yb, cz + sin1 * r);

      // Top cap (+Y)
      self.push_tri(ct, p0_top, p1_top, Vec3::Y, color);
      // Bottom cap (-Y)
      self.push_tri(cb, p1_bot, p0_bot, -Vec3::Y, color);
      // Side wall quad
      self.push_quad(p0_bot, p1_bot, p1_top, p0_top, color);
    }
  }

  /// Solid extruded annulus ("coin") centred at (cx,cy,cz), flat in the XY
  /// plane with thickness `depth` along z. Real edges (outer/inner walls) make
  /// it read as 3D when the scene is rotated.
  pub fn add_ring(
    &mut self,
    cx: f32,
    cy: f32,
    cz: f32,
    r_outer: f32,
    r_inner: f32,
    depth: f32,
    segments: u32,
    color: Color,
  ) {
    self.add_band(cx, cy, cz, r_inner, r_outer, &vec![r_outer; segments as usize], depth, color);
  }

  /// Annulus whose outer radius varies per segment (`radii`, length = segments)
  /// — a flat, wavy band with real thickness along z.
  pub fn add_band(
    &mut self,
    cx: f32,
    cy: f32,
    cz: f32,
    r_inner: f32,
    _r_outer: f32,
    radii: &[f32],
    depth: f32,
    color: Color,
  ) {
    let segs = radii.len().max(3);
    let zt = cz + depth * 0.5;
    let zb = cz - depth * 0.5;
    let z = |z: f32| Vec3::new(0.0, 0.0, z);
    let c = Vec3::new(cx, cy, cz);
    for i in 0..segs {
      let j = (i + 1) % segs;
      let a0 = i as f32 / segs as f32 * TAU;
      let a1 = j as f32 / segs as f32 * TAU;
      let r0 = radii[i].max(0.0);
      let r1 = radii[j].max(0.0);
      let (co0, si0) = a0.sin_cos();
      let (co1, si1) = a1.sin_cos();
      let o0 = Vec3::new(co0 * r0, si0 * r0, 0.0);
      let o1 = Vec3::new(co1 * r1, si1 * r1, 0.0);
      let n0 = Vec3::new(co0 * r_inner, si0 * r_inner, 0.0);
      let n1 = Vec3::new(co1 * r_inner, si1 * r_inner, 0.0);
      // Top annulus (+z) and bottom annulus (-z).
      self.push_quad(c + n0 + z(zt), c + o0 + z(zt), c + o1 + z(zt), c + n1 + z(zt), color);
      self.push_quad(c + n0 + z(zb), c + n1 + z(zb), c + o1 + z(zb), c + o0 + z(zb), color);
      // Outer wall (normal points outward) and inner wall.
      self.push_quad(c + o0 + z(zb), c + o1 + z(zb), c + o1 + z(zt), c + o0 + z(zt), color);
      self.push_quad(c + n1 + z(zb), c + n0 + z(zb), c + n0 + z(zt), c + n1 + z(zt), color);
    }
  }
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
  Vec3::new(x, y, z)
}
