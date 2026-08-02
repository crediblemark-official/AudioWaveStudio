//! GpuRenderer — wgpu device/queue that rasterizes a GpuCanvas mesh to RGBA.

use super::scene::{Mesh, Vertex};
use bytemuck::{Pod, Zeroable};

pub const TEXTURE_LAYERS: u32 = 24;
pub const LAYER_SIZE: u32 = 1024;
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

/// Parameters for a post-processing pass (screen effects that sample the frame).
#[derive(Clone, Copy, Debug)]
pub struct PostFx {
  /// Effect id: 1 = glitch, 2 = chromatic, 3 = zoom, 4 = invert,
  /// 5 = bars, 6 = shockwave, 7 = pixelate, 8 = tilt, 9 = heat haze.
  pub mode: u32,
  pub intensity: f32,
  /// Seconds since export start.
  pub time: f32,
  /// 0..=1 energy of the current beat, decays between beats.
  pub beat: f32,
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
  pipeline: wgpu::RenderPipeline,
  width: u32,
  height: u32,
  // Post-processing pass (screen effects that need frame sampling).
  post_texture: wgpu::Texture,
  post_texture_view: wgpu::TextureView,
  post_pipeline: wgpu::RenderPipeline,
  post_params_buf: wgpu::Buffer,
  post_bind_group: wgpu::BindGroup,
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
      ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("BG"),
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&atlas_view) },
        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
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
              src_factor: wgpu::BlendFactor::SrcAlpha,
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
      pipeline,
      width,
      height,
      post_texture,
      post_texture_view,
      post_pipeline,
      post_params_buf,
      post_bind_group,
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
  /// Returns the scaled layer-space dimensions so callers can compute UVs.
  #[allow(dead_code)]
  pub fn upload_image_layer(&self, layer: u32, rgba: &[u8], w: u32, h: u32) -> Option<(u32, u32)> {
    if layer >= TEXTURE_LAYERS || w == 0 || h == 0 || layer == GLYPH_LAYER {
      return None;
    }
    let scale = (LAYER_SIZE as f32 / w.max(h) as f32).min(1.0);
    let tw = ((w as f32 * scale) as u32).max(1).min(LAYER_SIZE);
    let th = ((h as f32 * scale) as u32).max(1).min(LAYER_SIZE);
    let src = wgpu::TextureDescriptor {
      label: Some("tmp"),
      size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    };
    let tmp = self.device.create_texture(&src);
    self.queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: &tmp,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      rgba,
      wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h) },
      wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    let tmp_view = tmp.create_view(&Default::default());
    let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("img"),
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });
    let bgl = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("scBGL"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
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
      ],
    });
    let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("scBG"),
      layout: &bgl,
      entries: &[
        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tmp_view) },
        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
      ],
    });
    let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("scPL"),
      bind_group_layouts: &[&bgl],
      push_constant_ranges: &[],
    });
    let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("scale"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
          label: Some("scale shader"),
          source: wgpu::ShaderSource::Wgsl(SCALE_SHADER.into()),
        }),
        entry_point: Some("vs_main"),
        buffers: &[],
        compilation_options: Default::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
          label: Some("scale fs"),
          source: wgpu::ShaderSource::Wgsl(SCALE_SHADER.into()),
        }),
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: wgpu::TextureFormat::Rgba8Unorm,
          blend: None,
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: Default::default(),
      }),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: None,
    });

    let tmp_target = self.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("scaled"),
      size: wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let target_view = tmp_target.create_view(&Default::default());

    let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("scale enc"),
    });
    {
      let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("scale pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &target_view,
          resolve_target: None,
          ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
        })],
        depth_stencil_attachment: None,
        ..Default::default()
      });
      pass.set_pipeline(&pipeline);
      pass.set_bind_group(0, &bg, &[]);
      pass.draw(0..3, 0..1);
    }
    let mut tmp_buf = Vec::new();
    tmp_buf.resize((tw as usize) * (th as usize) * 4, 0);
    // copy target -> staging for readback
    let tmp_staging = self.device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("sc staging"),
      size: Self::staging_size(tw, th),
      usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    enc.copy_texture_to_buffer(
      wgpu::ImageCopyTexture {
        texture: &tmp_target,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      wgpu::ImageCopyBuffer {
        buffer: &tmp_staging,
        layout: wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(Self::row_bytes(tw)),
          rows_per_image: Some(th),
        },
      },
      wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
    );
    self.queue.submit(std::iter::once(enc.finish()));
    let slice = tmp_staging.slice(..);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |r| {
      let _ = tx.send(r);
    });
    self.device.poll(wgpu::Maintain::Wait);
    rx.recv().ok();
    {
      let data = slice.get_mapped_range();
      tmp_buf.copy_from_slice(&Self::deinterleave_rows(&data, tw, th));
    }
    tmp_staging.unmap();
    self.upload_layer(layer, &tmp_buf, tw, th);
    Some((tw, th))
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

  /// Record the scene pass: upload atlases, build geometry and draw the frame
  /// into `self.texture`. Caller submits the encoder after adding copies/post.
  fn record_scene(&mut self, enc: &mut wgpu::CommandEncoder, mesh: &Mesh) {
    for atlas in &mesh.atlases {
      self.upload_layer(atlas.layer, &atlas.rgba, atlas.width, atlas.height);
    }
    self.ensure_geometry(mesh);

    let clear = mesh.clear;
    let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("gpu2d pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &self.texture_view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color {
            r: clear.r as f64,
            g: clear.g as f64,
            b: clear.b as f64,
            a: clear.a as f64,
          }),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      ..Default::default()
    });

    if !mesh.idx.is_empty() {
      pass.set_pipeline(&self.pipeline);
      pass.set_bind_group(0, &self.bind_group, &[]);
      pass.set_vertex_buffer(0, self.vert_buf.as_ref().unwrap().slice(..));
      pass.set_index_buffer(self.idx_buf.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint32);
      pass.draw_indexed(0..mesh.idx.len() as u32, 0, 0..1);
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
    self.copy_to_staging(&mut enc, slot, &self.texture);
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
    };
    self
      .queue
      .write_buffer(&self.post_params_buf, 0, bytemuck::bytes_of(&params));

    let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("frame fx enc"),
    });
    self.record_scene(&mut enc, mesh);

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

#[allow(dead_code)]
const SCALE_SHADER: &str = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0)
    );
    var o: VsOut;
    o.pos = vec4(p[vi], 0.0, 1.0);
    o.uv = p[vi] * 0.5 + 0.5;
    o.uv.y = 1.0 - o.uv.y;
    return o;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.uv);
}
"#;
