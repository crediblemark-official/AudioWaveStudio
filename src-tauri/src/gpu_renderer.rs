use crate::renderer::RenderConfig;
const SPECTRUM_BINS: usize = 128;
const WAVEFORM_LEN: usize = 1024;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    resolution: [f32; 4],
    params: [f32; 4],
    params2: [f32; 4],
    primary_color: [f32; 4],
    secondary_color: [f32; 4],
    accent_color: [f32; 4],
    bg_color: [f32; 4],
}

pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    staging_buffer: wgpu::Buffer,
    config_buffer: wgpu::Buffer,
    spectrum_buffer: wgpu::Buffer,
    waveform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
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
            label: Some("Visualizer WGSL"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Frame"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&Default::default());

        let frame_size = (width as usize * height as usize * 4) as u64;
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback"),
            size: frame_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let config_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let spectrum_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Spectrum"),
            size: (SPECTRUM_BINS * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let waveform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Waveform"),
            size: (WAVEFORM_LEN * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Visualizer Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("BG"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: config_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: spectrum_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: waveform_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            device,
            queue,
            pipeline,
            texture,
            texture_view,
            staging_buffer,
            config_buffer,
            spectrum_buffer,
            waveform_buffer,
            bind_group,
            width,
            height,
        })
    }

    pub fn render_frame(
        &self,
        config: &RenderConfig,
        spectrum: &[f32],
        waveform: &[f32],
        bass_energy: f32,
        time: f32,
    ) -> Vec<u8> {
        let uniforms = Uniforms {
            resolution: [self.width as f32, self.height as f32, 0.0, 0.0],
            params: [
                style_to_int(&config.style) as f32,
                config.bar_count.min(SPECTRUM_BINS) as f32,
                config.sensitivity,
                config.bass_multiplier,
            ],
            params2: [
                bass_energy,
                time,
                waveform.len().min(WAVEFORM_LEN) as f32,
                0.0,
            ],
            primary_color: color_f32(config.primary_color),
            secondary_color: color_f32(config.secondary_color),
            accent_color: color_f32(config.accent_color),
            bg_color: color_f32(config.bg_color),
        };

        self.queue
            .write_buffer(&self.config_buffer, 0, bytemuck::bytes_of(&uniforms));

        let mut spec_data = [0.0f32; SPECTRUM_BINS];
        let n = spectrum.len().min(SPECTRUM_BINS);
        spec_data[..n].copy_from_slice(&spectrum[..n]);
        self.queue
            .write_buffer(&self.spectrum_buffer, 0, bytemuck::cast_slice(&spec_data));

        let mut wave_data = [0.0f32; WAVEFORM_LEN];
        let n = waveform.len().min(WAVEFORM_LEN);
        wave_data[..n].copy_from_slice(&waveform[..n]);
        self.queue
            .write_buffer(&self.waveform_buffer, 0, bytemuck::cast_slice(&wave_data));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Enc"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Vis Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: uniforms.bg_color[0] as f64,
                            g: uniforms.bg_color[1] as f64,
                            b: uniforms.bg_color[2] as f64,
                            a: uniforms.bg_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.width * 4),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = self.staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().ok();
        let data = slice.get_mapped_range().to_vec();
        self.staging_buffer.unmap();
        data
    }
}

fn style_to_int(style: &str) -> u32 {
    match style {
        "radial" => 1,
        "oscilloscope" => 2,
        "equalizer" => 3,
        "minimal" => 4,
        _ => 0,
    }
}

fn color_f32(c: [u8; 4]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

const SHADER_SRC: &str = r#"
struct Uniforms {
    resolution: vec4<f32>,
    params: vec4<f32>,
    params2: vec4<f32>,
    primary_color: vec4<f32>,
    secondary_color: vec4<f32>,
    accent_color: vec4<f32>,
    bg_color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var<storage, read> spectrum: array<f32, 128>;
@group(0) @binding(2) var<storage, read> waveform: array<f32, 1024>;

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

fn boost(raw: f32, sens: f32, bass_m: f32, idx: u32) -> f32 {
    if (raw <= 0.0001) { return 0.0; }
    let db = log(raw) / log(10.0);
    let norm = clamp((db * 20.0 + 100.0) / 70.0, 0.0, 1.0);
    var bm: f32 = 1.0;
    if (idx < 8u) { bm = 1.0 + bass_m * 0.6; }
    else if (idx < 24u) { bm = 1.0 + bass_m * 0.3; }
    return min(norm * sens * bm * 2.5, 1.0);
}

fn lerp_c(c1: vec4<f32>, c2: vec4<f32>, t: f32) -> vec4<f32> {
    return vec4(mix(c1.rgb, c2.rgb, t), 1.0);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let px = in.uv * u.resolution.xy;
    let w = u.resolution.x;
    let h = u.resolution.y;
    let s = u32(u.params.x);
    switch s {
        case 0u: { return fs_spectrum(px, w, h); }
        case 1u: { return fs_radial(px, w, h); }
        case 2u: { return fs_scope(px, w, h); }
        case 3u: { return fs_eq(px, w, h); }
        case 4u: { return fs_minimal(px, w, h); }
        default: { return fs_spectrum(px, w, h); }
    }
}

fn fs_spectrum(px: vec2<f32>, w: f32, h: f32) -> vec4<f32> {
    let bc = max(u32(u.params.y), 1u);
    let tw = w * 0.85;
    let gap = 4.0;
    let bw = max((tw - gap * f32(bc - 1u)) / f32(bc), 2.0);
    let sx = (w - tw) * 0.5;
    let cy = h * 0.55;
    let mh = h * 0.45;

    if (px.x < sx || px.x > sx + tw || px.y < cy - mh || px.y > cy) {
        return u.bg_color;
    }
    let rx = px.x - sx;
    let bi = min(u32(rx / (bw + gap)), bc - 1u);
    let bx = sx + f32(bi) * (bw + gap);
    if (px.x < bx || px.x > bx + bw) { return u.bg_color; }

    let val = boost(spectrum[bi], u.params.z, u.params.w, bi);
    let bh = val * mh;
    if (px.y >= cy - bh) { return lerp_c(u.secondary_color, u.primary_color, val); }
    return u.bg_color;
}

fn fs_radial(px: vec2<f32>, w: f32, h: f32) -> vec4<f32> {
    let bc = min(max(u32(u.params.y), 1u), 96u);
    let cx = w * 0.5;
    let cy = h * 0.48;
    let br = min(w, h) * 0.18 + u.params2.x * u.params.w * 30.0;
    let ms = min(w, h) * 0.3;
    let dx = px.x - cx;
    let dy = px.y - cy;
    let d = sqrt(dx * dx + dy * dy);
    if (d <= br) { return u.primary_color; }

    var ang = atan2(dy, dx);
    if (ang < 0.0) { ang += 6.2831853; }

    for (var i = 0u; i < bc; i++) {
        let val = boost(spectrum[i], u.params.z, u.params.w, i);
        let sh = val * ms;
        let sa = f32(i) / f32(bc) * 6.2831853;
        var ad = abs(ang - sa);
        if (ad > 3.1415926) { ad = 6.2831853 - ad; }
        let aw = 6.2831853 / f32(bc) * 0.4;
        if (ad < aw && d > br && d < br + sh) {
            return lerp_c(u.primary_color, u.accent_color, val);
        }
    }
    return u.bg_color;
}

fn fs_scope(px: vec2<f32>, w: f32, h: f32) -> vec4<f32> {
    let wl = max(u32(u.params2.z), 2u);
    let cy = h * 0.52;
    let ma = h * 0.35 * u.params.z;
    let fx = px.x / w * f32(wl - 1u);
    let i0 = min(u32(fx), wl - 1u);
    let i1 = min(i0 + 1u, wl - 1u);
    let frac = fx - f32(i0);
    let y0 = cy + waveform[i0] * ma;
    let y1 = cy + waveform[i1] * ma;
    let ty = mix(y0, y1, frac);
    let d = abs(px.y - ty);
    if (d < 2.0) { return u.primary_color; }
    if (d < 6.0) { return vec4(u.primary_color.rgb, 1.0 - (d - 2.0) / 4.0); }
    return u.bg_color;
}

fn fs_eq(px: vec2<f32>, w: f32, h: f32) -> vec4<f32> {
    let bc = min(max(u32(u.params.y), 1u), 48u);
    let rows = 18u;
    let aw = w * 0.8;
    let bku = aw / f32(bc) - 4.0;
    let bkh = (h * 0.35) / f32(rows) - 3.0;
    let sx = (w - aw) * 0.5;
    let sy = h * 0.6;

    if (px.x < sx || px.x > sx + aw || px.y > sy || px.y < sy - f32(rows) * (bkh + 3.0)) {
        return u.bg_color;
    }
    let col = min(u32((px.x - sx) / (bku + 4.0)), bc - 1u);
    let val = boost(spectrum[col], u.params.z, 1.0, col);
    let ar = u32(val * f32(rows));
    let row = u32((sy - px.y) / (bkh + 3.0));
    if (row >= rows) { return u.bg_color; }

    let bx = sx + f32(col) * (bku + 4.0);
    let by = sy - f32(row) * (bkh + 3.0);
    if (px.x < bx || px.x > bx + bku || px.y < by || px.y > by + bkh) { return u.bg_color; }

    if (row < ar) {
        if (row > u32(f32(rows) * 0.8)) { return u.accent_color; }
        if (row > u32(f32(rows) * 0.5)) { return u.primary_color; }
        return u.secondary_color;
    }
    return vec4(0.157, 0.157, 0.196, 0.2);
}

fn fs_minimal(px: vec2<f32>, w: f32, h: f32) -> vec4<f32> {
    let bc = min(max(u32(u.params.y), 1u), 64u);
    let aw = w * 0.7;
    let bw = aw / f32(bc) - 3.0;
    let sx = (w - aw) * 0.5;
    let cy = h * 0.55;

    if (px.x < sx || px.x > sx + aw) { return u.bg_color; }
    let idx = min(u32((px.x - sx) / (bw + 3.0)), bc - 1u);
    let val = boost(spectrum[idx], u.params.z, 1.0, idx);
    let bh = max(val * h * 0.35, 4.0);
    let bx = sx + f32(idx) * (bw + 3.0);
    let by = cy - bh * 0.5;
    if (px.x >= bx && px.x <= bx + bw && px.y >= by && px.y <= by + bh) {
        return u.primary_color;
    }
    return u.bg_color;
}
"#;
