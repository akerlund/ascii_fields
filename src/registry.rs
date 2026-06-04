//! Mode name -> factory function.

use crate::animation::Animation;
use crate::animations::*;

pub struct ModeInfo {
  pub name: &'static str,
  pub description: &'static str,
  pub factory: fn() -> Box<dyn Animation>,
}

pub static MODES: &[ModeInfo] = &[
  ModeInfo {
    name: "wave-plane",
    description: "Cyclic grayscale wave plane",
    factory: || Box::new(plane::WavePlane),
  },
  ModeInfo {
    name: "flower-sphere",
    description: "Dark flower-like ASCII sphere",
    factory: || Box::new(codex::FlowerSphere),
  },
  ModeInfo {
    name: "areas",
    description: "Dark oval sphere with floating ASCII regions",
    factory: || Box::new(areas::Areas),
  },
  ModeInfo {
    name: "waves",
    description: "Top-down surf waves hitting a beach",
    factory: || Box::new(waves::Waves),
  },
  ModeInfo {
    name: "flares",
    description: "Solar flare-like arcs and tendrils",
    factory: || Box::new(flares::Flares),
  },
  ModeInfo { name: "galaxy", description: "Rotating spiral galaxy", factory: || Box::new(galaxy::Galaxy) },
  ModeInfo {
    name: "black-hole",
    description: "Lensed black hole accretion disk",
    factory: || Box::new(black_hole::BlackHole),
  },
  ModeInfo {
    name: "night-sky",
    description: "Twinkling stars, milky way, shooting stars",
    factory: || Box::new(night_sky::NightSky),
  },
  ModeInfo {
    name: "life",
    description: "Conway's Game of Life with fading trails",
    factory: || Box::new(life::Life::default()),
  },
  ModeInfo {
    name: "drops",
    description: "Raindrops rippling across a pond",
    factory: || Box::new(drops::Drops::default()),
  },
  ModeInfo {
    name: "caustics",
    description: "Underwater light caustics rippling over a surface",
    factory: || Box::new(terrain::Caustics::default()),
  },
  ModeInfo {
    name: "orbitals",
    description: "Hydrogen electron probability clouds",
    factory: || Box::new(orbitals::Orbitals::default()),
  },
  ModeInfo {
    name: "qfield",
    description: "Quantum vacuum foam with particle pairs",
    factory: || Box::new(qfield::QField::default()),
  },
  ModeInfo {
    name: "double-slit",
    description: "Double-slit interference build-up",
    factory: || Box::new(double_slit::DoubleSlit::default()),
  },
  ModeInfo {
    name: "wave-well",
    description: "Quantum wave packet in a potential well",
    factory: || Box::new(wave_well::WaveWell),
  },
  ModeInfo {
    name: "mandelbrot",
    description: "Endless zoom into the Mandelbrot set",
    factory: || Box::new(mandelbrot::Mandelbrot),
  },
  ModeInfo { name: "julia", description: "Morphing, zooming Julia set", factory: || Box::new(julia::Julia) },
  ModeInfo {
    name: "burning-ship",
    description: "Endless zoom into the Burning Ship fractal",
    factory: || Box::new(burning_ship::BurningShip),
  },
  ModeInfo {
    name: "newton",
    description: "Newton's-method fractal with rotating roots",
    factory: || Box::new(newton::Newton),
  },
  ModeInfo {
    name: "sierpinski",
    description: "Zooming Sierpinski triangle fractal",
    factory: || Box::new(sierpinski::Sierpinski),
  },
  ModeInfo {
    name: "penrose",
    description: "Aperiodic Penrose-like interference tiling",
    factory: || Box::new(patterns::Penrose::default()),
  },
  ModeInfo {
    name: "rain",
    description: "Falling Matrix-style character rain",
    factory: || Box::new(rain::Rain::default()),
  },
  ModeInfo { name: "fire", description: "Rising campfire flames", factory: || Box::new(fire::Fire) },
  ModeInfo { name: "plasma", description: "Classic sinusoidal plasma", factory: || Box::new(plasma::Plasma) },
  ModeInfo {
    name: "lightning",
    description: "Branching lightning bolts in a storm",
    factory: || Box::new(lightning::Lightning::default()),
  },
  ModeInfo {
    name: "clouds",
    description: "Drifting fractal-noise clouds",
    factory: || Box::new(clouds::Clouds),
  },
  ModeInfo {
    name: "starfield",
    description: "Flying through a 3D starfield",
    factory: || Box::new(starfield::Starfield::default()),
  },
  ModeInfo { name: "aurora", description: "Northern lights curtains", factory: || Box::new(aurora::Aurora) },
  ModeInfo {
    name: "pulsar",
    description: "Rotating neutron-star beam sweep",
    factory: || Box::new(space::Pulsar::default()),
  },
  ModeInfo {
    name: "supernova",
    description: "Overlapping stellar shock shells, new seed each burst",
    factory: || Box::new(space::Supernova::default()),
  },
  ModeInfo {
    name: "solar-wind",
    description: "Charged particles flowing around a magnetosphere",
    factory: || Box::new(space::SolarWind::default()),
  },
  ModeInfo {
    name: "cosmic-web",
    description: "Large-scale filamentary structure of the universe",
    factory: || Box::new(cosmic_web::CosmicWeb),
  },
  ModeInfo {
    name: "soap-bubbles",
    description: "Iridescent soap bubbles drifting upward",
    factory: || Box::new(soap_bubbles::SoapBubbles::default()),
  },
  ModeInfo {
    name: "whirlpool",
    description: "Swirling vortex / maelstrom",
    factory: || Box::new(whirlpool::Whirlpool),
  },
  ModeInfo {
    name: "hypercube",
    description: "Rotating 4D tesseract wireframe",
    factory: || Box::new(hypercube::Hypercube::default()),
  },
  ModeInfo {
    name: "tunnel",
    description: "Flight through a winding tunnel",
    factory: || Box::new(tunnel::Tunnel),
  },
  ModeInfo {
    name: "lightspeed",
    description: "Jump to lightspeed star streaks",
    factory: || Box::new(lightspeed::Lightspeed::default()),
  },
  ModeInfo {
    name: "feynman",
    description: "Animated Feynman diagrams",
    factory: || Box::new(feynman::Feynman::default()),
  },
  ModeInfo {
    name: "magnetic",
    description: "Bar-magnet dipole field lines",
    factory: || Box::new(magnetic::Magnetic),
  },
  ModeInfo {
    name: "ferrofluid",
    description: "Magnetic fluid spikes around moving field sources",
    factory: || Box::new(physics::Ferrofluid::default()),
  },
  ModeInfo {
    name: "gwaves",
    description: "Black holes merging, gravitational waves",
    factory: || Box::new(gwaves::GravitationalWaves::default()),
  },
  ModeInfo {
    name: "doppler",
    description: "Exoplanet radial-velocity Doppler shift",
    factory: || Box::new(doppler::Doppler::default()),
  },
  ModeInfo {
    name: "storm",
    description: "Jupiter Great Red Spot vortex",
    factory: || Box::new(storm::Storm),
  },
  ModeInfo {
    name: "lava",
    description: "Boiling lava with bursting bubbles",
    factory: || Box::new(lava::Lava),
  },
  ModeInfo {
    name: "vax_lamp",
    description: "Wax lamp blobs stretching and merging",
    factory: || Box::new(vax_lamp::VaxLamp),
  },
  ModeInfo {
    name: "circuit",
    description: "Circuit board traces with data pulses",
    factory: || Box::new(circuit::Circuit),
  },
  ModeInfo {
    name: "network",
    description: "Network topology with moving packets",
    factory: || Box::new(network::Network),
  },
  ModeInfo {
    name: "cpu",
    description: "CPU pipeline, registers, ALU, cache and data pulses",
    factory: || Box::new(cpu::Cpu),
  },
  ModeInfo {
    name: "mach",
    description: "Sonic boom / Mach cone from a moving source",
    factory: || Box::new(mach::Mach),
  },
  ModeInfo {
    name: "schlieren",
    description: "Heat-haze and shockwave density gradients",
    factory: || Box::new(physics::Schlieren::default()),
  },
  ModeInfo {
    name: "seismograph",
    description: "Earthquake wavefronts through layered ground",
    factory: || Box::new(terrain::Seismograph::default()),
  },
  ModeInfo {
    name: "convection",
    description: "Rayleigh-Benard-like heat convection rolls",
    factory: || Box::new(physics::Convection::default()),
  },
  ModeInfo {
    name: "reconnection",
    description: "Magnetic field lines snapping and reconnecting",
    factory: || Box::new(physics::Reconnection::default()),
  },
  ModeInfo {
    name: "longitudinal",
    description: "Longitudinal compression wave",
    factory: || Box::new(longitudinal::Longitudinal),
  },
  ModeInfo { name: "dna", description: "Rotating DNA double helix", factory: || Box::new(dna::Dna) },
  ModeInfo {
    name: "molecule",
    description: "Rotating 3D molecular cage",
    factory: || Box::new(molecule::Molecule::default()),
  },
  ModeInfo {
    name: "lensing",
    description: "Gravitational lensing: stars bent by a moving mass",
    factory: || Box::new(lensing::Lensing),
  },
  ModeInfo {
    name: "rd",
    description: "Gray-Scott reaction-diffusion (spots, stripes, mazes)",
    factory: || Box::new(rd::Rd::default()),
  },
  ModeInfo {
    name: "chladni",
    description: "Chladni plate nodal patterns (cymatics)",
    factory: || Box::new(chladni::Chladni),
  },
  ModeInfo {
    name: "curl",
    description: "Particles drifting through a curl-noise flow field",
    factory: || Box::new(curl::Curl::default()),
  },
  ModeInfo {
    name: "dla",
    description: "Diffusion-limited aggregation: a growing fractal frost",
    factory: || Box::new(dla::Dla::default()),
  },
  ModeInfo {
    name: "karman",
    description: "Karman vortex street behind a circular obstacle",
    factory: || Box::new(karman::Karman::default()),
  },
  ModeInfo {
    name: "topography",
    description: "Animated contour map with rivers and flow lines",
    factory: || Box::new(terrain::Topography::default()),
  },
  ModeInfo {
    name: "dunes",
    description: "Wind-driven sand ripples migrating over dunes",
    factory: || Box::new(terrain::Dunes::default()),
  },
  ModeInfo {
    name: "phyllotaxis",
    description: "Golden-angle sunflower spiral",
    factory: || Box::new(phyllotaxis::Phyllotaxis),
  },
  ModeInfo {
    name: "quasicrystal",
    description: "Fivefold wave interference quasicrystal",
    factory: || Box::new(patterns::Quasicrystal::default()),
  },
  ModeInfo {
    name: "moire",
    description: "Rotating line-grid moire interference",
    factory: || Box::new(patterns::Moire::default()),
  },
  ModeInfo {
    name: "voronoi",
    description: "Moving Voronoi cell boundaries",
    factory: || Box::new(patterns::Voronoi::default()),
  },
  ModeInfo {
    name: "reaction-rings",
    description: "Belousov-Zhabotinsky-style chemical wave rings",
    factory: || Box::new(patterns::ReactionRings::default()),
  },
  ModeInfo {
    name: "nbody",
    description: "Gravitational bodies orbiting through a shared field",
    factory: || Box::new(space::NBody::default()),
  },
  ModeInfo {
    name: "strange",
    description: "Morphing De Jong strange attractor",
    factory: || Box::new(patterns::Strange::default()),
  },
  ModeInfo {
    name: "lorenz",
    description: "The Lorenz strange attractor",
    factory: || Box::new(lorenz::Lorenz::default()),
  },
  ModeInfo {
    name: "drum",
    description: "Vibrational eigenmodes of a circular drum (Bessel)",
    factory: || Box::new(drum::Drum::default()),
  },
];

pub fn info(name: &str) -> Option<&'static ModeInfo> {
  MODES.iter().find(|m| m.name == name)
}

pub fn supports_charset(name: &str) -> bool {
  !matches!(name, "circuit" | "cpu" | "dna" | "doppler" | "feynman" | "nbody" | "network" | "rain")
}
