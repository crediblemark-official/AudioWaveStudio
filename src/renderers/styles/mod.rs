//! Dedicated submodules for each of the Visualizer Styles.

pub mod api_3d;
pub mod aurora_wave;
pub mod circular_bars;
pub mod equalizer;
pub mod flame_fire;
pub mod minimal;
pub mod neon_city_3d;
pub mod oscilloscope;
pub mod pulse_rings;
pub mod radial;
pub mod smooth_spectrum;
pub mod speaker_3d;
pub mod radial_ripple_3d;
pub mod waterfall_3d;
pub mod cassette_tape;
pub mod vinyl_record;
pub mod turntable;
pub mod speaker_splatter;
pub mod speaker_trio;
pub mod spectrum;
pub mod spiral_galaxy;
pub mod three_d;
pub mod vu_meter;
pub mod waveform_fill;
pub mod woofer;

use crate::config::VisualizerStyle;
use crate::gpu2d::GpuCanvas;
use crate::renderers::RenderContext;

pub fn render_style(style: &VisualizerStyle, c: &mut GpuCanvas, ctx: &mut RenderContext) {
  match style {
    VisualizerStyle::Spectrum => spectrum::render(c, ctx),
    VisualizerStyle::Radial => radial::render(c, ctx),
    VisualizerStyle::Oscilloscope => oscilloscope::render(c, ctx),
    VisualizerStyle::Equalizer => equalizer::render(c, ctx),
    VisualizerStyle::Minimal => minimal::render(c, ctx),
    VisualizerStyle::WaveformFill => waveform_fill::render(c, ctx),
    VisualizerStyle::CircularBars => circular_bars::render(c, ctx),
    VisualizerStyle::SmoothSpectrum => smooth_spectrum::render(c, ctx),
    VisualizerStyle::PulseRings => pulse_rings::render(c, ctx),
    VisualizerStyle::VuMeter => vu_meter::render(c, ctx),
    VisualizerStyle::AuroraWave => aurora_wave::render(c, ctx),
    VisualizerStyle::FlameFire => flame_fire::render(c, ctx),
    VisualizerStyle::SpiralGalaxy => spiral_galaxy::render(c, ctx),
    VisualizerStyle::ThreeD => three_d::render(c, ctx),
    VisualizerStyle::Api3D => api_3d::render(c, ctx),
    VisualizerStyle::NeonCity3D => neon_city_3d::render(c, ctx),
    VisualizerStyle::Speaker3D => speaker_3d::render(c, ctx),
    VisualizerStyle::SpeakerTrio => speaker_trio::render(c, ctx),
    VisualizerStyle::SpeakerSplatter => speaker_splatter::render(c, ctx),
    VisualizerStyle::RadialRipple3D => radial_ripple_3d::render(c, ctx),
    VisualizerStyle::Waterfall3D => waterfall_3d::render(c, ctx),
    VisualizerStyle::CassetteTape => cassette_tape::render(c, ctx),
    VisualizerStyle::VinylRecord => vinyl_record::render(c, ctx),
    VisualizerStyle::Turntable => turntable::render(c, ctx),
  }
}
