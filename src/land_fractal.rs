extern crate noise;

use noise::{NoiseFn, Perlin, Point2, Seedable};

/// Noise function that outputs custom fractal noise
///
/// Landfractal is based on fBm but allows for tweaking
/// each octave independently.
#[derive(Clone, Debug, Default)]
pub struct LandFractal {
    z_scale: f64,
    scale: f64,
    seed: u32,
    sources: Vec<Perlin>,
}


impl LandFractal {
    pub const DEFAULT_SEED: u32 = 0;
    pub const DEFAULT_Z_SCALE: f64 = 15.0;

    const OCTAVES: usize = 6;
    const LACUNARITY: f64 = std::f64::consts::PI * 2.0 / 3.0;
    const PERSISTENCE: f64 = 0.5;

    pub fn new() -> Self {
        LandFractal { seed: Self::DEFAULT_SEED,
                      scale: 2.0 - Self::PERSISTENCE.powi(Self::OCTAVES as i32 - 1),
                      z_scale: Self::DEFAULT_Z_SCALE,
                      sources: Self::build_sources(Self::DEFAULT_SEED) }
    }


    /// Setup the Perlin noise functions for the octaves
    ///
    /// # Arguments
    ///
    /// * `seed` - The base seed for the noises
    fn build_sources(seed: u32) -> Vec<Perlin> {
        let mut sources = Vec::with_capacity(Self::OCTAVES);

        for x in 0..Self::OCTAVES {
            sources.push(Perlin::new().set_seed(seed + x as u32));
        }

        sources
    }


    /// Scale the coordinates for the next octave
    ///
    /// Each octave in the fractal increases its frequency
    /// by multiplying its coordinates by the lacunarity value.
    /// This results in smaller, more detailed noise for each
    /// octave.
    ///
    /// # Arguments
    ///
    /// * `point` - Coordinates to scale
    fn scale_point(&self, point: Point2<f64>) -> Point2<f64> {
        [point[0] * Self::LACUNARITY, point[1] * Self::LACUNARITY]
    }


    pub fn set_z_scale(self, z_scale: f64) -> Self {
        LandFractal { z_scale, ..self }
    }
}


impl Seedable for LandFractal {
    fn set_seed(self, seed: u32) -> Self {
        if self.seed == seed {
            return self;
        }

        LandFractal { seed,
                      sources: Self::build_sources(seed),
                      ..self }
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}


impl NoiseFn<Point2<f64>> for LandFractal {
    fn get(&self, mut point: Point2<f64>) -> f64 {
        let mut result;

        // Octave 0 - The basic shape of the terrain
        let base = self.sources[0].get(point);

        result = base * 1.5;

        // Octave 1 - Large details
        point = self.scale_point(point);
        let octave1 = self.sources[1].get(point) * Self::PERSISTENCE;

        result += octave1;

        // Octave 2 - Large details breakup
        point = self.scale_point(point);
        let mut octave2 = self.sources[2].get(point);
        octave2 *= Self::PERSISTENCE.powi(2);

        result += octave2;

        // Octave 3 - Medium details
        point = self.scale_point(point);
        let mut octave3 = self.sources[3].get(point);
        octave3 *= Self::PERSISTENCE.powi(4);

        result += octave3;

        // Octave 4 - Smaller details
        point = self.scale_point(point);
        let mut octave4 = self.sources[4].get(point);
        octave4 *= Self::PERSISTENCE.powi(6);

        result += octave4;

        // Octave 5 - Fine details
        point = self.scale_point(point);
        let mut octave5 = self.sources[5].get(point);
        octave5 *= Self::PERSISTENCE.powi(7);

        result += octave5;

        (result / self.scale) * self.z_scale
    }
}
