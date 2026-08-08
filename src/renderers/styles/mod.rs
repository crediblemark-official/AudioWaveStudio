//! Dedicated submodules for each of the Visualizer Styles.

pub mod acoustic_cymascope;
pub mod api_3d;
pub mod audio_prism_3d;
pub mod aurora_wave;
pub mod cassette_tape;
pub mod chrono_reactor;
pub mod circular_bars;
pub mod cyber_black_hole;
pub mod cyber_horizon;
pub mod cyber_ring_3d;
pub mod cyber_tunnel_3d;
pub mod cylinder_matrix_3d;
pub mod dj_controller;
pub mod dual_portal_bridge;
pub mod equalizer;
pub mod holographic_vinyl;
pub mod hologram_stage;
pub mod laser_wall;
pub mod matrix_rain;
pub mod mercury_fluid;
pub mod minimal;
pub mod nebula_cloud_3d;
pub mod neon_biohazard;
pub mod neon_city_3d;
pub mod neon_lotus;
pub mod neon_metropolis_3d;
pub mod orbit_spike;
pub mod oscilloscope;
pub mod particle_wave_3d;
pub mod pulse_rings;
pub mod quantum_eye;
pub mod quantum_ribbon;
pub mod radial;
pub mod radial_ripple_3d;
pub mod retro_radio;
pub mod smooth_spectrum;
pub mod solar_flare_crown;
pub mod speaker_3d;
pub mod speaker_explosion;
pub mod speaker_splatter;
pub mod speaker_trio;
pub mod spectrum;
pub mod spiral_galaxy;
pub mod supernova_burst;
pub mod synthwave_highway_3d;
pub mod three_d;
pub mod turntable;
pub mod vaporwave_deck_3d;
pub mod vinyl_record;
pub mod vu_meter;
pub mod warp_drive;
pub mod waterfall_3d;
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
    VisualizerStyle::CyberHorizon => cyber_horizon::render(c, ctx),
    VisualizerStyle::CyberBlackHole => cyber_black_hole::render(c, ctx),
    VisualizerStyle::SupernovaBurst => supernova_burst::render(c, ctx),
    VisualizerStyle::QuantumEye => quantum_eye::render(c, ctx),
    VisualizerStyle::ChronoReactor => chrono_reactor::render(c, ctx),
    VisualizerStyle::SolarFlareCrown => solar_flare_crown::render(c, ctx),
    VisualizerStyle::WarpDrive => warp_drive::render(c, ctx),
    VisualizerStyle::NeonBiohazard => neon_biohazard::render(c, ctx),
    VisualizerStyle::NeonLotus => neon_lotus::render(c, ctx),
    VisualizerStyle::HolographicVinyl => holographic_vinyl::render(c, ctx),
    VisualizerStyle::AcousticCymascope => acoustic_cymascope::render(c, ctx),
    VisualizerStyle::SynthwaveHighway3D => synthwave_highway_3d::render(c, ctx),
    VisualizerStyle::MercuryFluid => mercury_fluid::render(c, ctx),
    VisualizerStyle::NeonMetropolis3D => neon_metropolis_3d::render(c, ctx),
    VisualizerStyle::MatrixRain => matrix_rain::render(c, ctx),
    VisualizerStyle::QuantumRibbon => quantum_ribbon::render(c, ctx),
    VisualizerStyle::AudioPrism3D => audio_prism_3d::render(c, ctx),
    VisualizerStyle::VaporwaveDeck3D => vaporwave_deck_3d::render(c, ctx),
    VisualizerStyle::NebulaCloud3D => nebula_cloud_3d::render(c, ctx),
    VisualizerStyle::CyberTunnel3D => cyber_tunnel_3d::render(c, ctx),
    VisualizerStyle::LaserWall => laser_wall::render(c, ctx),
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
    VisualizerStyle::RetroRadio => retro_radio::render(c, ctx),
    VisualizerStyle::DjController => dj_controller::render(c, ctx),
    VisualizerStyle::SpeakerExplosion => speaker_explosion::render(c, ctx),
    VisualizerStyle::OrbitSpike => orbit_spike::render(c, ctx),
    VisualizerStyle::CyberRing3D => cyber_ring_3d::render(c, ctx),
    VisualizerStyle::HologramStage => hologram_stage::render(c, ctx),
    VisualizerStyle::DualPortalBridge => dual_portal_bridge::render(c, ctx),
    VisualizerStyle::ParticleWave3D => particle_wave_3d::render(c, ctx),
    VisualizerStyle::CylinderMatrix3D => cylinder_matrix_3d::render(c, ctx),
  }
}
