//! GpuRenderer — wgpu device/queue that rasterizes a GpuCanvas mesh to RGBA.

use super::scene::{Mesh, Vertex};
use super::scene3d::Scene3D;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;

const THREE_D_SHADER_SRC: &str = include_str!("three_d_shader.wgsl");
/// Depth-buffer format used by the native 3D scene pass.
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const TEXTURE_LAYERS: u32 = 24;
/// Atlas array-layer size. 2048 keeps a >1080p background photo near 1:1 for
/// 1080p/1440p exports (a 1024 cap visibly softened photos vs the TS preview,
/// which draws the full-resolution image via the browser's high-quality
/// drawImage). Costs ~384 MB GPU memory (24 layers × 2048² × 4).
pub const LAYER_SIZE: u32 = 2048;
#[allow(dead_code)]
pub const GLYPH_LAYER: u32 = 0;
/// Persistent layer used for the custom background image (above per-frame text layers 0..19).
pub const IMAGE_LAYER: u32 = 20;
/// Per-frame layer used for the film-grain noise tile.
pub const NOISE_LAYER: u32 = 21;
/// Persistent layer used for the radial center image.
pub const RADIAL_CENTER_IMAGE_LAYER: u32 = 22;

const SHADER_SRC: &str = include_str!("shader.wgsl");
const POST_FX_SRC: &str = include_str!("postfx.wgsl");

/// Render target for a scene pass (self.texture or the post-pass texture
/// when compositing over a post-processed background).
#[derive(Clone, Copy, Debug)]
enum SceneTarget {
  Frame,
  Post,
}

/// Parameters for a post-processing pass (screen effects that sample the frame).
#[derive(Clone, Copy, Debug)]
pub struct PostFx {
  /// Effect id: 1 = glitch, 2 = chromatic, 3 = zoom, 4 = invert,
  /// 5 = bars, 6 = shockwave, 7 = pixelate, 8 = tilt, 9 = heat haze,
  /// 10 = hue shift.
  pub mode: u32,
  pub intensity: f32,
  /// Seconds since export start.
  pub time: f32,
  /// 0..=1 energy of the current beat, decays between beats.
  pub beat: f32,
  /// Render frames per second (glitch color-bar timing).
  pub fps: f32,
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct RawFxParams {
  mode: u32,
  intensity: f32,
  time: f32,
  beat: f32,
  width: f32,
  height: f32,
  fps: f32,
  pad: [f32; 3],
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct RawVertex {
  position: [f32; 2],
  color: [f32; 4],
  uv: [f32; 2],
  tex_id: f32,
}

impl From<&Vertex> for RawVertex {
  fn from(v: &Vertex) -> Self {
    RawVertex {
      position: v.position,
      color: v.color,
      uv: v.uv,
      tex_id: v.tex_id,
    }
  }
}

fn vertex_layout() -> Vec<wgpu::VertexAttribute> {
  vec![
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 },
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 8, shader_location: 1 },
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2 },
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32, offset: 32, shader_location: 3 },
  ]
}

// --- Native 3D scene (Scene3D) vertex/uniform types ---

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct RawVertex3 {
  position: [f32; 3],
  normal: [f32; 3],
  color: [f32; 4],
}

impl From<&super::scene3d::V3> for RawVertex3 {
  fn from(v: &super::scene3d::V3) -> Self {
    RawVertex3 { position: v.position, normal: v.normal, color: v.color }
  }
}

fn vertex_layout_3d() -> Vec<wgpu::VertexAttribute> {
  vec![
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 24, shader_location: 2 },
  ]
}

/// Uniform block for the 3D pipeline, matching `three_d_shader.wgsl`:
/// `view_proj` (mat4), `light_dir`, `light_col`, `ambient` (vec4 each).
/// glam::Mat4 is column-major, so a `[f32; 16]` copies straight through.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct RawCamUniform {
  view_proj: [f32; 16],
  light_dir: [f32; 4],
  light_col: [f32; 4],
  ambient: [f32; 4],
}

impl RawCamUniform {
  fn new(view_proj: Mat4) -> RawCamUniform {
    // Light from the upper-left-front, biased toward the viewer (+z) so the
    // faces that face the camera get lit.
    let light_dir = glam::Vec3::new(-0.4, 0.7, 0.8).normalize();
    RawCamUniform {
      view_proj: view_proj.to_cols_array(),
      light_dir: [light_dir.x, light_dir.y, light_dir.z, 0.0],
      light_col: [0.72, 0.72, 0.85, 1.0],
      ambient: [0.4, 0.4, 0.52, 0.55],
    }
  }
}

pub struct GpuRenderer {
  device: wgpu::Device,
  queue: wgpu::Queue,
  texture: wgpu::Texture,
  texture_view: wgpu::TextureView,
  /// Ping-pong readback buffers so frame N can be read back while the GPU
  /// renders frame N+1 into the other staging buffer.
  staging: [wgpu::Buffer; 2],
  /// Persistent vertex/index buffers, grown on demand, rewritten each frame
  /// with `queue.write_buffer` to avoid per-frame GPU allocations.
  vert_buf: Option<wgpu::Buffer>,
  vert_cap: usize,
  idx_buf: Option<wgpu::Buffer>,
  idx_cap: usize,
  #[allow(dead_code)]
  atlas_texture: wgpu::Texture,
  #[allow(dead_code)]
  atlas_view: wgpu::TextureView,
  bind_group: wgpu::BindGroup,
  bind_group_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  /// Dedicated native-resolution 2D textures for the custom background image
  /// and the radial-center image. Freed from the fixed-size atlas, so large
  /// photos keep full detail instead of being capped at LAYER_SIZE (the old
  /// cap visibly softened exports vs the TS preview's high-quality drawImage).
  bg_image_tex: wgpu::Texture,
  bg_image_view: wgpu::TextureView,
  radial_image_tex: wgpu::Texture,
  radial_image_view: wgpu::TextureView,
  /// Hard cap for background image textures (~8K, covers virtually all photos);
  /// larger sources are area-averaged down to this.
  max_img_dim: u32,
  pipeline: wgpu::RenderPipeline,
  /// Additive blend pipeline for glow composite mode.
  additive_pipeline: wgpu::RenderPipeline,
  /// Canvas2D `globalCompositeOperation = 'screen'` blend pipeline
  /// (premultiplied src * (1 - dst) + dst).
  screen_pipeline: wgpu::RenderPipeline,
  width: u32,
  height: u32,
  // Post-processing pass (screen effects that need frame sampling).
  post_texture: wgpu::Texture,
  post_texture_view: wgpu::TextureView,
  post_pipeline: wgpu::RenderPipeline,
  post_params_buf: wgpu::Buffer,
  post_bind_group: wgpu::BindGroup,
  // Native 3D scene pass (Scene3D), depth-tested after the 2D scene.
  depth_view: wgpu::TextureView,
  vert_buf_3d: Option<wgpu::Buffer>,
  vert_cap_3d: usize,
  idx_buf_3d: Option<wgpu::Buffer>,
  idx_cap_3d: usize,
  cam_buf: wgpu::Buffer,
  cam_bind_group: wgpu::BindGroup,
  pipeline_3d: wgpu::RenderPipeline,
}

impl GpuRenderer {
  pub async fn new(width: u32, height: u32) -> Result<Self, String> {
    let instance = wgpu::Instance::default();
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
      })
      .await
      .ok_or("No GPU adapter found")?;

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: Some("AudioWave GPU Export"),
          ..Default::default()
        },
        None,
      )
      .await
      .map_err(|e| format!("GPU device error: {}", e))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("gpu2d"),
      source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
    });

    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Frame"),
      size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });
    let texture_view = texture.create_view(&Default::default());

    let staging_a = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Readback A"),
      size: Self::staging_size(width, height),
      usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let staging_b = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Readback B"),
      size: Self::staging_size(width, height),
      usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Atlas Array"),
      size: wgpu::Extent3d {
        width: LAYER_SIZE,
        height: LAYER_SIZE,
        depth_or_array_layers: TEXTURE_LAYERS,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    let atlas_view = atlas_texture.create_view(&Default::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("Linear"),
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });

    // Dummy 1x1 textures so the bind group always has valid views; replaced on
    // the first real `upload_background_image`.
    let dummy_tex = |device: &wgpu::Device, label: &str| {
      let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
      });
      let view = tex.create_view(&Default::default());
      (tex, view)
    };
    let (bg_image_tex, bg_image_view) = dummy_tex(&device, "Background Image (dummy)");
    let (radial_image_tex, radial_image_view) = dummy_tex(&device, "Radial Center Image (dummy)");

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("BGL"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2Array,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 3,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
      ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("BG"),
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&atlas_view) },
        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&bg_image_view) },
        wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&radial_image_view) },
      ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("PL"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("gpu2d pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        buffers: &[wgpu::VertexBufferLayout {
          array_stride: std::mem::size_of::<RawVertex>() as u64,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes: &vertex_layout(),
        }],
        compilation_options: Default::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: wgpu::TextureFormat::Rgba8Unorm,
          blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
              src_factor: wgpu::BlendFactor::SrcAlpha,
              dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
              operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
              src_factor: wgpu::BlendFactor::One,
              dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
              operation: wgpu::BlendOperation::Add,
            },
          }),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: Default::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        ..Default::default()
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: None,
    });

    // Additive blend pipeline for screen/glow composite mode.
    let additive_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("gpu2d additive pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        buffers: &[wgpu::VertexBufferLayout {
          array_stride: std::mem::size_of::<RawVertex>() as u64,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes: &vertex_layout(),
        }],
        compilation_options: Default::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: wgpu::TextureFormat::Rgba8Unorm,
          blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
              src_factor: wgpu::BlendFactor::SrcAlpha,
              dst_factor: wgpu::BlendFactor::One,
              operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
              src_factor: wgpu::BlendFactor::One,
              dst_factor: wgpu::BlendFactor::One,
              operation: wgpu::BlendOperation::Add,
            },
          }),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: Default::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        ..Default::default()
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: None,
    });

    // Canvas2D 'screen' blend pipeline. Vertex colors for Screen batches are
    // premultiplied on the CPU at batch flush, so `src * (1 - dst) + dst`
    // matches the compositing spec formula for an opaque backdrop:
    //   Co = αs·Cs·(1 − Cb) + Cb,  αo = αs + αb·(1 − αs) = 1
    let screen_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("gpu2d screen pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        buffers: &[wgpu::VertexBufferLayout {
          array_stride: std::mem::size_of::<RawVertex>() as u64,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes: &vertex_layout(),
        }],
        compilation_options: Default::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: wgpu::TextureFormat::Rgba8Unorm,
          blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
              src_factor: wgpu::BlendFactor::OneMinusDst,
              dst_factor: wgpu::BlendFactor::One,
              operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
              src_factor: wgpu::BlendFactor::One,
              dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
              operation: wgpu::BlendOperation::Add,
            },
          }),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: Default::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        ..Default::default()
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: None,
    });

    // --- Post-processing pass (frame-sampling screen effects) ---
    let post_texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Post Frame"),
      size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let post_texture_view = post_texture.create_view(&Default::default());

    let post_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("Post Sampler"),
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });

    let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("postfx"),
      source: wgpu::ShaderSource::Wgsl(POST_FX_SRC.into()),
    });
    let post_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("PostBGL"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ],
    });
    let post_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("PostPL"),
      bind_group_layouts: &[&post_bgl],
      push_constant_ranges: &[],
    });
    let post_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("postfx pipeline"),
      layout: Some(&post_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &post_shader,
        entry_point: Some("vs_main"),
        buffers: &[],
        compilation_options: Default::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &post_shader,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: wgpu::TextureFormat::Rgba8Unorm,
          blend: None,
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: Default::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        ..Default::default()
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: None,
    });

    let post_params_buf = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Post Params"),
      size: std::mem::size_of::<RawFxParams>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let post_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("PostBG"),
      layout: &post_bgl,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::Sampler(&post_sampler),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(&texture_view),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: post_params_buf.as_entire_binding(),
        },
      ],
    });

    // --- Native 3D scene pass (Scene3D) ---
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("3D Depth"),
      size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: DEPTH_FORMAT,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&Default::default());

    let shader_3d = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("three_d"),
      source: wgpu::ShaderSource::Wgsl(THREE_D_SHADER_SRC.into()),
    });
    let cam_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("3D BGL"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    });
    let pipeline_layout_3d = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("3D PL"),
      bind_group_layouts: &[&cam_bind_group_layout],
      push_constant_ranges: &[],
    });
    let pipeline_3d = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("three_d pipeline"),
      layout: Some(&pipeline_layout_3d),
      vertex: wgpu::VertexState {
        module: &shader_3d,
        entry_point: Some("vs_main"),
        buffers: &[wgpu::VertexBufferLayout {
          array_stride: std::mem::size_of::<RawVertex3>() as u64,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes: &vertex_layout_3d(),
        }],
        compilation_options: Default::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader_3d,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: wgpu::TextureFormat::Rgba8Unorm,
          blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
              src_factor: wgpu::BlendFactor::SrcAlpha,
              dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
              operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
              src_factor: wgpu::BlendFactor::One,
              dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
              operation: wgpu::BlendOperation::Add,
            },
          }),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: Default::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        // No culling: the styles build every face explicitly and want the
        // inside of hollow shapes (rings, open boxes) visible too.
        cull_mode: None,
        ..Default::default()
      },
      depth_stencil: Some(wgpu::DepthStencilState {
        format: DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      }),
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: None,
    });
    let cam_buf = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("3D Cam"),
      size: std::mem::size_of::<RawCamUniform>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let cam_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("3D BG"),
      layout: &cam_bind_group_layout,
      entries: &[wgpu::BindGroupEntry { binding: 0, resource: cam_buf.as_entire_binding() }],
    });

    let max_img_dim = device.limits().max_texture_dimension_2d.min(8192);

    Ok(Self {
      device,
      queue,
      texture,
      texture_view,
      staging: [staging_a, staging_b],
      vert_buf: None,
      vert_cap: 0,
      idx_buf: None,
      idx_cap: 0,
      atlas_texture,
      atlas_view,
      bind_group,
      bind_group_layout,
      sampler,
      bg_image_tex,
      bg_image_view,
      radial_image_tex,
      radial_image_view,
      max_img_dim,
      pipeline,
      additive_pipeline,
      screen_pipeline,
      width,
      height,
      post_texture,
      post_texture_view,
      post_pipeline,
      post_params_buf,
      post_bind_group,
      depth_view,
      vert_buf_3d: None,
      vert_cap_3d: 0,
      idx_buf_3d: None,
      idx_cap_3d: 0,
      cam_buf,
      cam_bind_group,
      pipeline_3d,
    })
  }

  #[allow(dead_code)]
  pub fn width(&self) -> u32 {
    self.width
  }

  #[allow(dead_code)]
  pub fn height(&self) -> u32 {
    self.height
  }

  /// Upload RGBA data into one layer of the atlas array (size LAYER_SIZE).
  #[allow(dead_code)]
  pub fn upload_layer(&self, layer: u32, rgba: &[u8], w: u32, h: u32) {
    if layer >= TEXTURE_LAYERS || rgba.len() < (w as usize) * (h as usize) * 4 {
      return;
    }
    self.queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: &self.atlas_texture,
        mip_level: 0,
        origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
        aspect: wgpu::TextureAspect::All,
      },
      rgba,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(w * 4),
        rows_per_image: Some(h),
      },
      wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
  }

  /// Upload an image into a layer, scaled to fit LAYER_SIZE while preserving aspect.
  /// Downscaling uses a CPU area-average (box) resample — a single bilinear
  /// sample would alias and soften large photos, unlike the browser's
  /// high-quality drawImage used by the TS preview. Returns the scaled
  /// layer-space dimensions so callers can compute UVs.
  #[allow(dead_code)]
  pub fn upload_image_layer(&self, layer: u32, rgba: &[u8], w: u32, h: u32) -> Option<(u32, u32)> {
    if layer >= TEXTURE_LAYERS || w == 0 || h == 0 || layer == GLYPH_LAYER {
      return None;
    }
    if w <= LAYER_SIZE && h <= LAYER_SIZE {
      self.upload_layer(layer, rgba, w, h);
      return Some((w, h));
    }
    let scale = LAYER_SIZE as f32 / w.max(h) as f32;
    let tw = ((w as f32 * scale) as u32).max(1).min(LAYER_SIZE);
    let th = ((h as f32 * scale) as u32).max(1).min(LAYER_SIZE);
    let resized = Self::area_average_resize(rgba, w, h, tw, th);
    self.upload_layer(layer, &resized, tw, th);
    Some((tw, th))
  }

  /// Upload a background image (custom background or radial-center) into a
  /// dedicated native-resolution 2D texture — NOT the fixed-size atlas — so
  /// large photos keep their full detail, matching the TS preview's
  /// high-quality `drawImage`. Rebuilds the bind group so the new texture view
  /// is sampled. Returns the uploaded dimensions for cover-fit UV mapping.
  pub fn upload_background_image(&mut self, layer: u32, rgba: &[u8], w: u32, h: u32) -> Option<(u32, u32)> {
    if (layer != IMAGE_LAYER && layer != RADIAL_CENTER_IMAGE_LAYER) || w == 0 || h == 0 {
      return None;
    }
    let max_dim = self.max_img_dim;
    let (tw, th, data) = if w > max_dim || h > max_dim {
      let scale = max_dim as f32 / w.max(h) as f32;
      let tw = ((w as f32 * scale) as u32).max(1).min(max_dim);
      let th = ((h as f32 * scale) as u32).max(1).min(max_dim);
      (tw, th, Self::area_average_resize(rgba, w, h, tw, th))
    } else {
      (w, h, rgba.to_vec())
    };
    let tex = self.device.create_texture(&wgpu::TextureDescriptor {
      label: Some(if layer == IMAGE_LAYER { "Background Image" } else { "Radial Center Image" }),
      size: wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    let view = tex.create_view(&Default::default());
    self.queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: &tex,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      &data,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(tw * 4),
        rows_per_image: Some(th),
      },
      wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
    );
    if layer == IMAGE_LAYER {
      self.bg_image_tex = tex;
      self.bg_image_view = view;
    } else {
      self.radial_image_tex = tex;
      self.radial_image_view = view;
    }
    self.rebuild_bind_group();
    Some((tw, th))
  }

  /// Rebuild the scene bind group against the current background-image texture
  /// views (textures are replaced whenever `upload_background_image` is called).
  fn rebuild_bind_group(&mut self) {
    let new_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("BG"),
      layout: &self.bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&self.atlas_view) },
        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&self.bg_image_view) },
        wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&self.radial_image_view) },
      ],
    });
    self.bind_group = new_bg;
  }

  /// Separable area-average (box) resample. Downscale only; 1:1 maps to an
  /// exact copy. Produces smooth, alias-free output comparable to the
  /// browser's high-quality image downscale.
  pub fn area_average_resize(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let mut tmp = vec![0u8; (dw as usize) * (sh as usize) * 4];
    for y in 0..sh {
      let row = (y as usize) * (sw as usize) * 4;
      let out = (y as usize) * (dw as usize) * 4;
      for x in 0..dw {
        let sx0 = ((x as u64) * (sw as u64) / (dw as u64)) as usize;
        let sx1 = (((x + 1) as u64) * (sw as u64)).div_ceil(dw as u64) as usize;
        let n = (sx1 - sx0).max(1);
        let mut acc = [0u32; 4];
        for sx in sx0..sx1 {
          let o = row + sx * 4;
          for c in 0..4 {
            acc[c] += src[o + c] as u32;
          }
        }
        for c in 0..4 {
          tmp[out + (x as usize) * 4 + c] = (acc[c] / n as u32) as u8;
        }
      }
    }
    let mut dst = vec![0u8; (dw as usize) * (dh as usize) * 4];
    for x in 0..dw {
      for y in 0..dh {
        let sy0 = ((y as u64) * (sh as u64) / (dh as u64)) as usize;
        let sy1 = (((y + 1) as u64) * (sh as u64)).div_ceil(dh as u64) as usize;
        let n = (sy1 - sy0).max(1);
        let mut acc = [0u32; 4];
        for sy in sy0..sy1 {
          let o = ((sy as usize) * (dw as usize) + x as usize) * 4;
          for c in 0..4 {
            acc[c] += tmp[o + c] as u32;
          }
        }
        for c in 0..4 {
          dst[((y as usize) * (dw as usize) + x as usize) * 4 + c] = (acc[c] / n as u32) as u8;
        }
      }
    }
    dst
  }

  /// Grow the persistent vertex/index buffers to fit `mesh` and upload the
  /// current geometry via `queue.write_buffer` (command-ordered, so it cannot
  /// race with an in-flight render pass).
  fn ensure_geometry(&mut self, mesh: &Mesh) {
    let vneed = mesh.verts.len();
    if self.vert_cap < vneed {
      self.vert_buf = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("verts"),
        size: (vneed.max(1) * std::mem::size_of::<RawVertex>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
      self.vert_cap = vneed;
    }
    if vneed > 0 {
      let verts: Vec<RawVertex> = mesh.verts.iter().map(RawVertex::from).collect();
      self
        .queue
        .write_buffer(self.vert_buf.as_ref().unwrap(), 0, bytemuck::cast_slice(&verts));
    }

    let ineed = mesh.idx.len();
    if self.idx_cap < ineed {
      self.idx_buf = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("idx"),
        size: (ineed.max(1) * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
      self.idx_cap = ineed;
    }
    if ineed > 0 {
      self
        .queue
        .write_buffer(self.idx_buf.as_ref().unwrap(), 0, bytemuck::cast_slice(&mesh.idx));
    }
  }

  /// Persistent vertex/index buffers for the 3D scene, grown on demand and
  /// rewritten each frame via `queue.write_buffer` (same strategy as
  /// `ensure_geometry`).
  fn ensure_geometry_3d(&mut self, scene: &Scene3D) {
    let vneed = scene.verts().len();
    if self.vert_cap_3d < vneed {
      self.vert_buf_3d = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("verts3d"),
        size: (vneed.max(1) * std::mem::size_of::<RawVertex3>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
      self.vert_cap_3d = vneed;
    }
    if vneed > 0 {
      let verts: Vec<RawVertex3> = scene.verts().iter().map(RawVertex3::from).collect();
      self
        .queue
        .write_buffer(self.vert_buf_3d.as_ref().unwrap(), 0, bytemuck::cast_slice(&verts));
    }

    let ineed = scene.idx().len();
    if self.idx_cap_3d < ineed {
      self.idx_buf_3d = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("idx3d"),
        size: (ineed.max(1) * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
      self.idx_cap_3d = ineed;
    }
    if ineed > 0 {
      self
        .queue
        .write_buffer(self.idx_buf_3d.as_ref().unwrap(), 0, bytemuck::cast_slice(scene.idx()));
    }
  }

  /// Record the native 3D pass into `target`: uploads the scene geometry +
  /// camera uniform, clears depth to 1.0 and draws the triangles on top of the
  /// current frame contents (the caller keeps the colour attachment with
  /// `LoadOp::Load`, so the 2D background/style underneath is preserved).
  fn record_scene_3d(&mut self, enc: &mut wgpu::CommandEncoder, scene: &Scene3D, target: SceneTarget) {
    if scene.is_empty() {
      return;
    }
    self.ensure_geometry_3d(scene);

    let view_proj = crate::renderers::three_d_engine::view_proj(
      self.width,
      self.height,
      scene.cam_yaw,
      scene.cam_pitch,
      scene.cam_zoom,
      scene.target_x,
      scene.target_y,
    );
    let uniforms = RawCamUniform::new(view_proj);
    self
      .queue
      .write_buffer(&self.cam_buf, 0, bytemuck::bytes_of(&uniforms));

    let view = match target {
      SceneTarget::Frame => &self.texture_view,
      SceneTarget::Post => &self.post_texture_view,
    };
    let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("three_d pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
      })],
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: &self.depth_view,
        depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
        stencil_ops: None,
      }),
      ..Default::default()
    });

    pass.set_pipeline(&self.pipeline_3d);
    pass.set_bind_group(0, &self.cam_bind_group, &[]);
    pass.set_vertex_buffer(0, self.vert_buf_3d.as_ref().unwrap().slice(..));
    pass.set_index_buffer(self.idx_buf_3d.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint32);
    pass.draw_indexed(0..scene.idx().len() as u32, 0, 0..1);
  }

  /// Record the scene pass: upload atlases, build geometry and draw the frame
  /// into `self.texture`. Caller submits the encoder after adding copies/post.
  fn record_scene(&mut self, enc: &mut wgpu::CommandEncoder, mesh: &Mesh) {
    let clear = mesh.clear;
    self.record_scene_op(
      enc,
      mesh,
      SceneTarget::Frame,
      wgpu::LoadOp::Clear(wgpu::Color {
        r: clear.r as f64,
        g: clear.g as f64,
        b: clear.b as f64,
        a: clear.a as f64,
      }),
    );
  }

  /// Like `record_scene` but composites over the CURRENT target contents
  /// (LoadOp::Load) instead of clearing — used to draw the foreground mesh on
  /// top of a post-processed background.
  fn record_scene_over(&mut self, enc: &mut wgpu::CommandEncoder, mesh: &Mesh, target: SceneTarget) {
    self.record_scene_op(enc, mesh, target, wgpu::LoadOp::Load);
  }

  fn record_scene_op(
    &mut self,
    enc: &mut wgpu::CommandEncoder,
    mesh: &Mesh,
    target: SceneTarget,
    load: wgpu::LoadOp<wgpu::Color>,
  ) {
    for atlas in &mesh.atlases {
      self.upload_layer(atlas.layer, &atlas.rgba, atlas.width, atlas.height);
    }
    self.ensure_geometry(mesh);

    let view = match target {
      SceneTarget::Frame => &self.texture_view,
      SceneTarget::Post => &self.post_texture_view,
    };

    let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("gpu2d pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
        resolve_target: None,
        ops: wgpu::Operations {
          load,
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      ..Default::default()
    });

    if !mesh.idx.is_empty() {
      pass.set_bind_group(0, &self.bind_group, &[]);
      pass.set_vertex_buffer(0, self.vert_buf.as_ref().unwrap().slice(..));
      pass.set_index_buffer(self.idx_buf.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint32);

      if mesh.batches.is_empty() {
        // Fallback: no batches recorded, draw all with normal pipeline.
        pass.set_pipeline(&self.pipeline);
        pass.draw_indexed(0..mesh.idx.len() as u32, 0, 0..1);
      } else {
        use super::scene::BlendMode;
        let mut current_blend = None;
        for batch in &mesh.batches {
          if current_blend != Some(batch.blend) {
            match batch.blend {
              BlendMode::Normal => pass.set_pipeline(&self.pipeline),
              BlendMode::Additive => pass.set_pipeline(&self.additive_pipeline),
              BlendMode::Screen => pass.set_pipeline(&self.screen_pipeline),
            }
            current_blend = Some(batch.blend);
          }
          pass.draw_indexed(batch.idx_start..(batch.idx_start + batch.idx_count), 0, 0..1);
        }
      }
    }
  }

  fn copy_to_staging(&self, enc: &mut wgpu::CommandEncoder, slot: usize, tex: &wgpu::Texture) {
    enc.copy_texture_to_buffer(
      wgpu::ImageCopyTexture {
        texture: tex,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      wgpu::ImageCopyBuffer {
        buffer: &self.staging[slot],
        layout: wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(Self::row_bytes(self.width)),
          rows_per_image: Some(self.height),
        },
      },
      wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
    );
  }

  /// Upload atlases, build the frame geometry and submit a render pass that
  /// copies the result into `staging[slot]`. Never blocks; the matching
  /// `readback` call waits only for that slot.
  pub fn render_into(&mut self, mesh: &Mesh, slot: usize) {
    let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("frame enc"),
    });
    self.record_scene(&mut enc, mesh);
    self.record_scene_3d(&mut enc, &mesh.scene3d, SceneTarget::Frame);
    self.copy_to_staging(&mut enc, slot, &self.texture);
    self.queue.submit(std::iter::once(enc.finish()));
  }

  /// Two-pass render for `backgroundOnly` screen effects: the background mesh
  /// is drawn into `self.texture`, run through the post-processing pipeline
  /// into `post_texture`, then the foreground mesh is composited OVER the
  /// post-processed background (LoadOp::Load — no clear) and the combined
  /// result is copied into `staging[slot]`. Mirrors canvasRenderer, which
  /// applies frame-sampling effects to the background BEFORE drawing the
  /// visualizer style on top.
  pub fn render_bg_fx_then_over(&mut self, bg_mesh: &Mesh, fg_mesh: &Mesh, fx: &PostFx, slot: usize) {
    let params = RawFxParams {
      mode: fx.mode,
      intensity: fx.intensity,
      time: fx.time,
      beat: fx.beat,
      width: self.width as f32,
      height: self.height as f32,
      pad: [0.0, 0.0, 0.0],
      fps: fx.fps,
    };
    self
      .queue
      .write_buffer(&self.post_params_buf, 0, bytemuck::bytes_of(&params));

    let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("frame bg fx enc"),
    });
    self.record_scene(&mut enc, bg_mesh);

    {
      let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("postfx bg pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &self.post_texture_view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        ..Default::default()
      });
      pass.set_pipeline(&self.post_pipeline);
      pass.set_bind_group(0, &self.post_bind_group, &[]);
      pass.draw(0..3, 0..1);
    }

    // Submit the background + post-fx FIRST, then the foreground in a second
    // submission: a later `write_buffer` into the shared geometry buffers in
    // the same submit can alias the background pass's reads on some drivers
    // (observed on Intel ANV), leaving the post-fx output sampling stale
    // pixels.
    self.queue.submit(std::iter::once(enc.finish()));

    let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("frame fg over enc"),
    });
    // Composite the foreground (style/particles/notes/text) over the fx'd
    // background without clearing, then draw the native 3D scene on top with a
    // real depth buffer.
    self.record_scene_over(&mut enc, fg_mesh, SceneTarget::Post);
    self.record_scene_3d(&mut enc, &fg_mesh.scene3d, SceneTarget::Post);
    self.copy_to_staging(&mut enc, slot, &self.post_texture);
    self.queue.submit(std::iter::once(enc.finish()));
  }

  /// Like `render_into`, but run the frame through the post-processing
  /// pipeline first (screen effects that sample the frame) and copy the
  /// post-pass result into `staging[slot]`.
  pub fn render_into_fx(&mut self, mesh: &Mesh, fx: &PostFx, slot: usize) {
    let params = RawFxParams {
      mode: fx.mode,
      intensity: fx.intensity,
      time: fx.time,
      beat: fx.beat,
      width: self.width as f32,
      height: self.height as f32,
      pad: [0.0, 0.0, 0.0],
      fps: fx.fps,
    };
    self
      .queue
      .write_buffer(&self.post_params_buf, 0, bytemuck::bytes_of(&params));

    let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("frame fx enc"),
    });
    self.record_scene(&mut enc, mesh);
    self.record_scene_3d(&mut enc, &mesh.scene3d, SceneTarget::Frame);

    {
      let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("postfx pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &self.post_texture_view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        ..Default::default()
      });
      pass.set_pipeline(&self.post_pipeline);
      pass.set_bind_group(0, &self.post_bind_group, &[]);
      pass.draw(0..3, 0..1);
    }

    self.copy_to_staging(&mut enc, slot, &self.post_texture);
    self.queue.submit(std::iter::once(enc.finish()));
  }

  /// Wait for the copy into `staging[slot]` to complete (spinning on a
  /// non-blocking poll so other slots' work keeps running) and return the
  /// deinterleaved RGBA frame.
  pub fn readback(&self, slot: usize) -> Vec<u8> {
    let slice = self.staging[slot].slice(..);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |r| {
      let _ = tx.send(r);
    });
    loop {
      if rx.recv_timeout(std::time::Duration::from_millis(1)).is_ok() {
        break;
      }
      self.device.poll(wgpu::Maintain::Poll);
    }
    let data = {
      let mapped = slice.get_mapped_range();
      Self::deinterleave_rows(&mapped, self.width, self.height)
    };
    self.staging[slot].unmap();
    data
  }

  pub fn render(&mut self, mesh: &Mesh) -> Vec<u8> {
    self.render_into(mesh, 0);
    self.readback(0)
  }

  pub fn jpeg(&mut self, mesh: &Mesh) -> Result<Vec<u8>, String> {
    let rgba = self.render(mesh);
    self.rgba_to_jpeg(&rgba)
  }

  /// Row stride padded to `COPY_BYTES_PER_ROW_ALIGNMENT` (256) for wgpu copies.
  fn row_bytes(width: u32) -> u32 {
    let bytes = width * 4;
    (bytes + 255) & !255
  }

  fn staging_size(width: u32, height: u32) -> u64 {
    Self::row_bytes(width) as u64 * height as u64
  }

  /// Strip row padding left by the aligned readback copy.
  fn deinterleave_rows(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize * 4;
    let row = Self::row_bytes(width) as usize;
    let mut out = Vec::with_capacity(w * height as usize);
    for r in 0..height as usize {
      let start = r * row;
      out.extend_from_slice(&data[start..start + w]);
    }
    out
  }

  pub fn rgba_to_jpeg(&self, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
    for px in rgba.chunks_exact(4) {
      rgb.push(px[0]);
      rgb.push(px[1]);
      rgb.push(px[2]);
    }
    let mut out: Vec<u8> = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 95);
    enc.encode(
      &rgb,
      self.width,
      self.height,
      image::ExtendedColorType::Rgb8,
    )
    .map_err(|e| format!("JPEG encode failed: {}", e))?;
    Ok(out)
  }
}

