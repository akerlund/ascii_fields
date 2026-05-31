//! Mode name -> factory function.

use crate::animation::Animation;
use crate::animations::*;

pub struct ModeInfo {
  pub name: &'static str,
  pub description: &'static str,
  pub factory: fn() -> Box<dyn Animation>,
}

pub static MODES: &[ModeInfo] = &[
  ModeInfo { name: "wave-plane",    description: "cyclic grayscale wave plane",                             factory: || Box::new(plane::WavePlane) },
  ModeInfo { name: "flower-sphere", description: "dark flower-like ASCII sphere",                          factory: || Box::new(codex::FlowerSphere) },
  ModeInfo { name: "areas",         description: "dark oval sphere with floating ASCII regions",           factory: || Box::new(areas::Areas) },
  ModeInfo { name: "waves",         description: "top-down surf waves hitting a beach",                    factory: || Box::new(waves::Waves) },
  ModeInfo { name: "flares",        description: "solar flare-like arcs and tendrils",                     factory: || Box::new(flares::Flares) },
  ModeInfo { name: "galaxy",        description: "rotating spiral galaxy",                                 factory: || Box::new(galaxy::Galaxy) },
  ModeInfo { name: "black-hole",    description: "lensed black hole accretion disk",                       factory: || Box::new(black_hole::BlackHole) },
  ModeInfo { name: "night-sky",     description: "twinkling stars, milky way, shooting stars",             factory: || Box::new(night_sky::NightSky) },
  ModeInfo { name: "life",          description: "Conway's Game of Life with fading trails",               factory: || Box::new(life::Life::default()) },
  ModeInfo { name: "drops",         description: "raindrops rippling across a pond",                       factory: || Box::new(drops::Drops::default()) },
  ModeInfo { name: "orbitals",      description: "hydrogen electron probability clouds",                   factory: || Box::new(orbitals::Orbitals::default()) },
  ModeInfo { name: "qfield",        description: "quantum vacuum foam with particle pairs",                factory: || Box::new(qfield::QField::default()) },
  ModeInfo { name: "double-slit",   description: "double-slit interference build-up",                      factory: || Box::new(double_slit::DoubleSlit::default()) },
  ModeInfo { name: "wave-well",     description: "quantum wave packet in a potential well",                factory: || Box::new(wave_well::WaveWell) },
  ModeInfo { name: "mandelbrot",    description: "endless zoom into the Mandelbrot set",                   factory: || Box::new(mandelbrot::Mandelbrot) },
  ModeInfo { name: "julia",         description: "morphing, zooming Julia set",                            factory: || Box::new(julia::Julia) },
  ModeInfo { name: "burning-ship",  description: "endless zoom into the Burning Ship fractal",             factory: || Box::new(burning_ship::BurningShip) },
  ModeInfo { name: "newton",        description: "Newton's-method fractal with rotating roots",            factory: || Box::new(newton::Newton) },
  ModeInfo { name: "sierpinski",    description: "zooming Sierpinski triangle fractal",                    factory: || Box::new(sierpinski::Sierpinski) },
  ModeInfo { name: "rain",          description: "falling Matrix-style character rain",                    factory: || Box::new(rain::Rain::default()) },
  ModeInfo { name: "fire",          description: "rising campfire flames",                                 factory: || Box::new(fire::Fire) },
  ModeInfo { name: "plasma",        description: "classic sinusoidal plasma",                              factory: || Box::new(plasma::Plasma) },
  ModeInfo { name: "lightning",     description: "branching lightning bolts in a storm",                   factory: || Box::new(lightning::Lightning::default()) },
  ModeInfo { name: "clouds",        description: "drifting fractal-noise clouds",                          factory: || Box::new(clouds::Clouds) },
  ModeInfo { name: "starfield",     description: "flying through a 3D starfield",                          factory: || Box::new(starfield::Starfield::default()) },
  ModeInfo { name: "aurora",        description: "northern lights curtains",                               factory: || Box::new(aurora::Aurora) },
  ModeInfo { name: "whirlpool",     description: "swirling vortex / maelstrom",                            factory: || Box::new(whirlpool::Whirlpool) },
  ModeInfo { name: "hypercube",     description: "rotating 4D tesseract wireframe",                        factory: || Box::new(hypercube::Hypercube::default()) },
  ModeInfo { name: "tunnel",        description: "flight through a winding tunnel",                        factory: || Box::new(tunnel::Tunnel) },
  ModeInfo { name: "lightspeed",    description: "jump to lightspeed star streaks",                        factory: || Box::new(lightspeed::Lightspeed::default()) },
  ModeInfo { name: "feynman",       description: "animated Feynman diagrams",                              factory: || Box::new(feynman::Feynman::default()) },
  ModeInfo { name: "magnetic",      description: "bar-magnet dipole field lines",                          factory: || Box::new(magnetic::Magnetic) },
  ModeInfo { name: "gwaves",        description: "black holes merging, gravitational waves",               factory: || Box::new(gwaves::GravitationalWaves::default()) },
  ModeInfo { name: "doppler",       description: "exoplanet radial-velocity Doppler shift",                factory: || Box::new(doppler::Doppler::default()) },
  ModeInfo { name: "storm",         description: "Jupiter Great Red Spot vortex",                          factory: || Box::new(storm::Storm) },
  ModeInfo { name: "lava",          description: "boiling lava with bursting bubbles",                     factory: || Box::new(lava::Lava) },
  ModeInfo { name: "vax_lamp",      description: "wax lamp blobs stretching and merging",                  factory: || Box::new(vax_lamp::VaxLamp) },
  ModeInfo { name: "circuit",       description: "circuit board traces with data pulses",                  factory: || Box::new(circuit::Circuit) },
  ModeInfo { name: "network",       description: "network topology with moving packets",                   factory: || Box::new(network::Network) },
  ModeInfo { name: "cpu",           description: "CPU pipeline, registers, ALU, cache and data pulses",    factory: || Box::new(cpu::Cpu) },
  ModeInfo { name: "mach",          description: "sonic boom / Mach cone from a moving source",            factory: || Box::new(mach::Mach) },
  ModeInfo { name: "longitudinal",  description: "longitudinal compression wave",                          factory: || Box::new(longitudinal::Longitudinal) },
  ModeInfo { name: "dna",           description: "rotating DNA double helix",                              factory: || Box::new(dna::Dna) },
  ModeInfo { name: "molecule",      description: "rotating 3D molecular cage",                             factory: || Box::new(molecule::Molecule::default()) },
  ModeInfo { name: "lensing",       description: "gravitational lensing: stars bent by a moving mass",     factory: || Box::new(lensing::Lensing) },
  ModeInfo { name: "rd",            description: "Gray-Scott reaction-diffusion (spots, stripes, mazes)",  factory: || Box::new(rd::Rd::default()) },
  ModeInfo { name: "chladni",       description: "Chladni plate nodal patterns (cymatics)",                factory: || Box::new(chladni::Chladni) },
  ModeInfo { name: "curl",          description: "particles drifting through a curl-noise flow field",     factory: || Box::new(curl::Curl::default()) },
  ModeInfo { name: "dla",           description: "diffusion-limited aggregation: a growing fractal frost", factory: || Box::new(dla::Dla::default()) },
  ModeInfo { name: "karman",        description: "Karman vortex street behind a circular obstacle",        factory: || Box::new(karman::Karman::default()) },
  ModeInfo { name: "phyllotaxis",   description: "golden-angle sunflower spiral",                          factory: || Box::new(phyllotaxis::Phyllotaxis) },
  ModeInfo { name: "lorenz",        description: "the Lorenz strange attractor",                           factory: || Box::new(lorenz::Lorenz::default()) },
  ModeInfo { name: "drum",          description: "vibrational eigenmodes of a circular drum (Bessel)",     factory: || Box::new(drum::Drum::default()) },
];

pub fn create(name: &str) -> Option<Box<dyn Animation>> {
  MODES.iter().find(|m| m.name == name).map(|m| (m.factory)())
}
