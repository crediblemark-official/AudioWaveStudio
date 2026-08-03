//! Debug probe for the text glow path (run manually):
//! cargo test --test glow_probe -- --ignored --nocapture
//!
//! Requires the `/tmp/awcmp-stress/inputs/frame_015.bin` fixture (a real audio
//! frame written by hand during debugging — no script regenerates it) and a
//! Vulkan-capable GPU, so it is `#[ignore]`d by default like `compare_export`.
use audiowave_studio_lib::config::{VisualizerConfig, VisualizerStyle};
use audiowave_studio_lib::gpu2d::{GpuCanvas, GpuRenderer};
use audiowave_studio_lib::renderers::{draw_frame, RenderState};
use std::fs;

#[test]
#[ignore = "manual probe: needs /tmp/awcmp-stress/inputs/frame_015.bin fixture + GPU"]
fn glow_probe() {
  let cfg_path = "../scripts/compare-config-stress.json";
  let cfg_json = fs::read_to_string(cfg_path).unwrap();
  let mut config: VisualizerConfig = serde_json::from_str(&cfg_json).unwrap();
  config.style = VisualizerStyle::Spectrum;

  let bin = fs::read("/tmp/awcmp-stress/inputs/frame_015.bin").unwrap();
  let freq = bin[0..512].to_vec();
  let time = bin[512..1536].to_vec();

  let mut rstate = RenderState::new(config.reactivity.bar_count, 0xC0FFEE);
  let mut canvas = GpuCanvas::new(480, 270);
  draw_frame(&mut canvas, &mut rstate, &config, &freq, &time, 0.5, true);
  let mesh = canvas.finish();

  eprintln!("== mesh diagnostics ==");
  eprintln!("atlases: {}", mesh.atlases.len());
  for a in &mesh.atlases {
    eprintln!("  layer={} {}x{}", a.layer, a.width, a.height);
  }
  let textured: Vec<&audiowave_studio_lib::gpu2d::Vertex> =
    mesh.verts.iter().filter(|v| v.tex_id > 0.5).collect();
  eprintln!("textured verts: {}", textured.len());
  if let Some(maxa) = textured.iter().map(|v| v.color[3]).fold(None, |m: Option<f32>, a| Some(m.map_or(a, |x: f32| x.max(a)))) {
    eprintln!("max textured alpha: {maxa}");
  }
  let mut min_alpha = 1.0f32;
  for v in &textured {
    min_alpha = min_alpha.min(v.color[3]);
  }
  eprintln!("min textured alpha: {min_alpha}");
  // unique tex_ids
  let mut layers: Vec<f32> = textured.iter().map(|v| v.tex_id).collect();
  layers.sort_by(|a, b| a.partial_cmp(b).unwrap());
  layers.dedup();
  eprintln!("textured layers used: {:?}", layers);

  // Render and sample pixels across the text band.
  let mut gpu = pollster::block_on(GpuRenderer::new(480, 270)).expect("gpu");
  let rgba = gpu.render(&mesh);
  let lum = |x: usize, y: usize| -> u32 {
    let o = (y * 480 + x) * 4;
    ((0.299 * rgba[o] as f32 + 0.587 * rgba[o + 1] as f32 + 0.114 * rgba[o + 2] as f32)) as u32
  };
  eprintln!("== row 30 luminance ==");
  let mut line = String::new();
  for x in (0..480).step_by(6) {
    line.push_str(&format!("{x}:{} ", lum(x, 30)));
  }
  eprintln!("{line}");
  // gap points specifically
  eprintln!(
    "gap x=21:{} x=33:{} x=78:{} x=108:{}",
    lum(21, 30),
    lum(33, 30),
    lum(78, 30),
    lum(108, 30)
  );
}
