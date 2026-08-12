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
pub mod dual_wave_horizon;
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
pub mod radial_common;
pub mod radial_ripple_3d;
pub mod radial_spike_blade;
pub mod radial_plasma_aura;
pub mod radial_cyber_rune;
pub mod radial_neon_orbiter;
pub mod radial_starlight_halo;
pub mod radial_vortex_spiral;
pub mod radial_bio_shuriken;
pub mod radial_hex_core;
pub mod radial_sonic_mandala;
pub mod radial_laser_curtain;
pub mod radial_solar_flare_burst;
pub mod radial_aperture_iris;
pub mod radial_radar_sweep;
pub mod radial_gear_mechanism;
pub mod radial_fireworks_burst;
pub mod radial_kaleidoscope;
pub mod radial_orrery;
pub mod radial_clockwork;
pub mod radial_geodesic_web;
pub mod pulsing_pill_ring;
pub mod pulsing_liquid_aura;
pub mod pulsing_dual_ring;
pub mod pulsing_shockwave;
pub mod pulsing_neon_arcade;
pub mod pulsing_laser_web;
pub mod pulsing_cosmic_dust;
pub mod pulsing_cyber_shield;
pub mod pulsing_sunburst_corona;
pub mod pulsing_barcode_pill;
pub mod saturn_halo;
pub mod star_hexagon;
pub mod quantum_cloud;
pub mod hyperdrive_tunnel;
pub mod nebula_ring;
pub mod tactical_hud;
pub mod crystal_prism;
pub mod synthwave_sun;
pub mod biomorphic_bloom;
pub mod infinity_loop;
pub mod liquid_tri_lobe_aura;
pub mod liquid_ferrofluid_spikes;
pub mod liquid_molten_mercury;
pub mod liquid_concentric_drop;
pub mod liquid_jellyfish_tentacles;
pub mod liquid_oil_slick;
pub mod liquid_vortex_swirl;
pub mod liquid_metaball_lava;
pub mod liquid_toxic_slime;
pub mod liquid_cymascope_water;
pub mod liquid_bioluminescent_plasma;
pub mod liquid_plasma_blob_3d;
pub mod liquid_chromatic_viscosity;
pub mod liquid_hydro_electric_arcs;
pub mod liquid_bioluminescent_plankton;
pub mod liquid_radioactive_isotope;
pub mod liquid_neon_cyber_goo;
pub mod liquid_molten_gold_stream;
pub mod liquid_magma_crust_core;
pub mod liquid_quantum_fluid;
pub mod glass_box_quantum_plasma;
pub mod glass_box_neon_spectrum;
pub mod glass_box_cyber_grid;
pub mod glass_box_bioluminescent_jellyfish;
pub mod glass_box_molten_lava;
pub mod glass_box_laser_matrix;
pub mod glass_box_liquid_chrome;
pub mod glass_box_cosmic_nebula;
pub mod glass_box_hologram_core;
pub mod glass_box_matrix_rain;
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
pub mod waveform_seismograph;
pub mod waveform_dual_tube;
pub mod waveform_voxel_terrain;
pub mod waveform_sine_comb;
pub mod waveform_harmonic_web;
pub mod waveform_stepped_arcade;
pub mod waveform_barcode_pulse;
pub mod waveform_curtain_beams;
pub mod waveform_oscillating_rings;
pub mod waveform_topographic_ribbon;
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
    VisualizerStyle::DualWaveHorizon => dual_wave_horizon::render(c, ctx),
    VisualizerStyle::WaveformSeismograph => waveform_seismograph::render(c, ctx),
    VisualizerStyle::WaveformDualTube => waveform_dual_tube::render(c, ctx),
    VisualizerStyle::WaveformVoxelTerrain => waveform_voxel_terrain::render(c, ctx),
    VisualizerStyle::WaveformSineComb => waveform_sine_comb::render(c, ctx),
    VisualizerStyle::WaveformHarmonicWeb => waveform_harmonic_web::render(c, ctx),
    VisualizerStyle::WaveformSteppedArcade => waveform_stepped_arcade::render(c, ctx),
    VisualizerStyle::WaveformBarcodePulse => waveform_barcode_pulse::render(c, ctx),
    VisualizerStyle::WaveformCurtainBeams => waveform_curtain_beams::render(c, ctx),
    VisualizerStyle::WaveformOscillatingRings => waveform_oscillating_rings::render(c, ctx),
    VisualizerStyle::WaveformTopographicRibbon => waveform_topographic_ribbon::render(c, ctx),
    VisualizerStyle::RadialSpikeBlade => radial_spike_blade::render(c, ctx),
    VisualizerStyle::RadialPlasmaAura => radial_plasma_aura::render(c, ctx),
    VisualizerStyle::RadialCyberRune => radial_cyber_rune::render(c, ctx),
    VisualizerStyle::RadialNeonOrbiter => radial_neon_orbiter::render(c, ctx),
    VisualizerStyle::RadialStarlightHalo => radial_starlight_halo::render(c, ctx),
    VisualizerStyle::RadialVortexSpiral => radial_vortex_spiral::render(c, ctx),
    VisualizerStyle::RadialBioShuriken => radial_bio_shuriken::render(c, ctx),
    VisualizerStyle::RadialHexCore => radial_hex_core::render(c, ctx),
    VisualizerStyle::RadialSonicMandala => radial_sonic_mandala::render(c, ctx),
    VisualizerStyle::RadialLaserCurtain => radial_laser_curtain::render(c, ctx),
    VisualizerStyle::RadialSolarFlareBurst => radial_solar_flare_burst::render(c, ctx),
    VisualizerStyle::RadialApertureIris => radial_aperture_iris::render(c, ctx),
    VisualizerStyle::RadialRadarSweep => radial_radar_sweep::render(c, ctx),
    VisualizerStyle::RadialGearMechanism => radial_gear_mechanism::render(c, ctx),
    VisualizerStyle::RadialFireworksBurst => radial_fireworks_burst::render(c, ctx),
    VisualizerStyle::RadialKaleidoscope => radial_kaleidoscope::render(c, ctx),
    VisualizerStyle::RadialOrrery => radial_orrery::render(c, ctx),
    VisualizerStyle::RadialClockwork => radial_clockwork::render(c, ctx),
    VisualizerStyle::RadialGeodesicWeb => radial_geodesic_web::render(c, ctx),
    VisualizerStyle::PulsingPillRing => pulsing_pill_ring::render(c, ctx),
    VisualizerStyle::PulsingLiquidAura => pulsing_liquid_aura::render(c, ctx),
    VisualizerStyle::PulsingDualRing => pulsing_dual_ring::render(c, ctx),
    VisualizerStyle::PulsingShockwave => pulsing_shockwave::render(c, ctx),
    VisualizerStyle::PulsingNeonArcade => pulsing_neon_arcade::render(c, ctx),
    VisualizerStyle::PulsingLaserWeb => pulsing_laser_web::render(c, ctx),
    VisualizerStyle::PulsingCosmicDust => pulsing_cosmic_dust::render(c, ctx),
    VisualizerStyle::PulsingCyberShield => pulsing_cyber_shield::render(c, ctx),
    VisualizerStyle::PulsingSunburstCorona => pulsing_sunburst_corona::render(c, ctx),
    VisualizerStyle::PulsingBarcodePill => pulsing_barcode_pill::render(c, ctx),
    VisualizerStyle::SaturnHalo => saturn_halo::render(c, ctx),
    VisualizerStyle::StarHexagon => star_hexagon::render(c, ctx),
    VisualizerStyle::QuantumCloud => quantum_cloud::render(c, ctx),
    VisualizerStyle::HyperdriveTunnel => hyperdrive_tunnel::render(c, ctx),
    VisualizerStyle::NebulaRing => nebula_ring::render(c, ctx),
    VisualizerStyle::TacticalHud => tactical_hud::render(c, ctx),
    VisualizerStyle::CrystalPrism => crystal_prism::render(c, ctx),
    VisualizerStyle::SynthwaveSun => synthwave_sun::render(c, ctx),
    VisualizerStyle::BiomorphicBloom => biomorphic_bloom::render(c, ctx),
    VisualizerStyle::InfinityLoop => infinity_loop::render(c, ctx),
    VisualizerStyle::LiquidTriLobeAura => liquid_tri_lobe_aura::render(c, ctx),
    VisualizerStyle::LiquidFerrofluidSpikes => liquid_ferrofluid_spikes::render(c, ctx),
    VisualizerStyle::LiquidMoltenMercury => liquid_molten_mercury::render(c, ctx),
    VisualizerStyle::LiquidConcentricDrop => liquid_concentric_drop::render(c, ctx),
    VisualizerStyle::LiquidJellyfishTentacles => liquid_jellyfish_tentacles::render(c, ctx),
    VisualizerStyle::LiquidOilSlick => liquid_oil_slick::render(c, ctx),
    VisualizerStyle::LiquidVortexSwirl => liquid_vortex_swirl::render(c, ctx),
    VisualizerStyle::LiquidMetaballLava => liquid_metaball_lava::render(c, ctx),
    VisualizerStyle::LiquidToxicSlime => liquid_toxic_slime::render(c, ctx),
    VisualizerStyle::LiquidCymascopeWater => liquid_cymascope_water::render(c, ctx),
    VisualizerStyle::LiquidBioluminescentPlasma => liquid_bioluminescent_plasma::render(c, ctx),
    VisualizerStyle::LiquidPlasmaBlob3D => liquid_plasma_blob_3d::render(c, ctx),
    VisualizerStyle::LiquidChromaticViscosity => liquid_chromatic_viscosity::render(c, ctx),
    VisualizerStyle::LiquidHydroElectricArcs => liquid_hydro_electric_arcs::render(c, ctx),
    VisualizerStyle::LiquidBioluminescentPlankton => liquid_bioluminescent_plankton::render(c, ctx),
    VisualizerStyle::LiquidRadioactiveIsotope => liquid_radioactive_isotope::render(c, ctx),
    VisualizerStyle::LiquidNeonCyberGoo => liquid_neon_cyber_goo::render(c, ctx),
    VisualizerStyle::LiquidMoltenGoldStream => liquid_molten_gold_stream::render(c, ctx),
    VisualizerStyle::LiquidMagmaCrustCore => liquid_magma_crust_core::render(c, ctx),
    VisualizerStyle::LiquidQuantumFluid => liquid_quantum_fluid::render(c, ctx),
    VisualizerStyle::GlassBoxQuantumPlasma => glass_box_quantum_plasma::render(c, ctx),
    VisualizerStyle::GlassBoxNeonSpectrum => glass_box_neon_spectrum::render(c, ctx),
    VisualizerStyle::GlassBoxCyberGrid => glass_box_cyber_grid::render(c, ctx),
    VisualizerStyle::GlassBoxBioluminescentJellyfish => glass_box_bioluminescent_jellyfish::render(c, ctx),
    VisualizerStyle::GlassBoxMoltenLava => glass_box_molten_lava::render(c, ctx),
    VisualizerStyle::GlassBoxLaserMatrix => glass_box_laser_matrix::render(c, ctx),
    VisualizerStyle::GlassBoxLiquidChrome => glass_box_liquid_chrome::render(c, ctx),
    VisualizerStyle::GlassBoxCosmicNebula => glass_box_cosmic_nebula::render(c, ctx),
    VisualizerStyle::GlassBoxHologramCore => glass_box_hologram_core::render(c, ctx),
    VisualizerStyle::GlassBoxMatrixRain => glass_box_matrix_rain::render(c, ctx),
  }
}
