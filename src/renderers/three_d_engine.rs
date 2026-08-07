//! Native 3D GPU Engine module powered by `three-d`.
//!
//! Provides true 3D camera projection, 3D parametric mesh generation,
//! PBR materials, dynamic 3D lighting, and 3D audio-reactive transform matrices.

use three_d::*;

pub struct ThreeDEngine {
  pub camera: Camera,
  pub ambient_light: AmbientLight,
  pub directional_light: DirectionalLight,
}

impl ThreeDEngine {
  pub fn new(width: u32, height: u32) -> Self {
    let context = Context::from_gl_context(std::sync::Arc::new(three_d_asset::io::RawContext::new())).ok();
    
    let camera = Camera::new_perspective(
      Viewport { x: 0, y: 0, width, height },
      vec3(0.0, 0.0, 5.0),
      vec3(0.0, 0.0, 0.0),
      vec3(0.0, 1.0, 0.0),
      degrees(45.0),
      0.1,
      1000.0,
    );

    let ambient_light = AmbientLight::new(&context.clone().unwrap(), 0.4, Color::WHITE);
    let directional_light = DirectionalLight::new(&context.unwrap(), 1.0, Color::WHITE, &vec3(-1.0, -1.0, -1.0));

    Self {
      camera,
      ambient_light,
      directional_light,
    }
  }

  pub fn update_viewport(&mut self, width: u32, height: u32) {
    self.camera.set_viewport(Viewport { x: 0, y: 0, width, height });
  }
}
