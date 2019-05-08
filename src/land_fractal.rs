extern crate noise;

use noise::{NoiseFn, Perlin, Point3, Seedable};

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
    ///
    /// Returns a vector of Perlin noise functions with
    /// different seeds
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
    ///
    /// Returns the scaled point
    fn scale_point(&self, point: Point3<f64>) -> Point3<f64> {
        [point[0] * Self::LACUNARITY,
         point[1] * Self::LACUNARITY,
         point[2] * Self::LACUNARITY]
    }


    /// Set the Z multiplier
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


/// Get noise value for 2D coordinates
impl NoiseFn<Point3<f64>> for LandFractal {
    fn get(&self, mut point: Point3<f64>) -> f64 {
        let mut result;
        let mut domain_point;
        let mut original_point = point;
        let mask_point;


        //------------------------------------------------------------------------------------------
        // BLEND MASK
        //------------------------------------------------------------------------------------------
        let mask_control = 1.1;
        mask_point = [point[0] * mask_control, point[1] * mask_control,
                        point[2] * mask_control];
        let mask = self.sources[0].get(mask_point);




        //------------------------------------------------------------------------------------------
        // DOMAIN WARPING
        //------------------------------------------------------------------------------------------

        let domain_base = 1.5;
        domain_point = [point[0] * domain_base, point[1] * domain_base,
                        point[2] * domain_base];

        let mut domain = self.sources[0].get(domain_point);

        domain_point = [domain_point[0] * domain_base, domain_point[1] * domain_base,
                        domain_point[2] * domain_base];

        domain += self.sources[1].get(domain_point) * 0.5;
        domain_point = [domain_point[0] * domain_base, domain_point[1] * domain_base,
                        domain_point[2] * domain_base];

        domain += self.sources[2].get(domain_point) * 0.25;
        domain *= 0.10;


        //------------------------------------------------------------------------------------------
        // BASE FRACTAL NOISE
        //------------------------------------------------------------------------------------------

        //------------------------------------------------------------------------------------------
        // Basic shape of the terrain
        let base_control = 1.5;
        result = self.sources[0].get(point) * base_control;

        //------------------------------------------------------------------------------------------
        // Basic features of the terrain
        let octave1_scale = 1.4;
        let octave1_persistence = 0.9;
        point = [point[0] * octave1_scale + domain, point[1] * octave1_scale + domain,
                 point[2] * octave1_scale];

        let octave1 = self.sources[1].get(point) * octave1_persistence;
        result += octave1;


        //------------------------------------------------------------------------------------------
        // Larger details

        let octave2_scale = 2.0;
        let octave2_persistence = 0.4;
        point = [point[0] * octave2_scale + domain, point[1] * octave2_scale + domain,
                 point[2] * octave2_scale];

        let octave2 = self.sources[2].get(point) * octave2_persistence;
        result += octave2;


        //------------------------------------------------------------------------------------------
        // Larger details

        let octave3_scale = 2.0;
        let octave3_persistence = 0.25;
        point = [point[0] * octave3_scale + domain, point[1] * octave3_scale + domain,
                 point[2] * octave3_scale];

        let octave3 = self.sources[3].get(point) * octave3_persistence;
        result += octave3;


        //------------------------------------------------------------------------------------------
        // Larger details

        let octave4_scale = 2.0;
        let octave4_persistence = 0.1;
        point = [point[0] * octave4_scale + domain, point[1] * octave4_scale + domain,
                 point[2] * octave4_scale];

        let octave4 = self.sources[4].get(point) * octave4_persistence;
        result += octave4;

        //------------------------------------------------------------------------------------------
        // Larger details
        let octave5_scale = 2.0;
        let octave5_persistence = 0.01;
        point = [point[0] * octave5_scale + domain, point[1] * octave5_scale + domain,
                 point[2] * octave5_scale];

        let octave5 = self.sources[5].get(point) * octave5_persistence;
        result += octave5;


        //------------------------------------------------------------------------------------------
        // BLEND NOISE
        //------------------------------------------------------------------------------------------

        //------------------------------------------------------------------------------------------
        // Basic shape of the terrain
        let base_control = 0.1;
        let mut blend;
        blend = self.sources[3].get(original_point) * base_control;


        //------------------------------------------------------------------------------------------
        // Basic features of the terrain
        let blend1_scale = 2.0;
        let blend1_persistence = 0.2;
        original_point = [original_point[0] * blend1_scale + domain,
                          original_point[1] * blend1_scale + domain,
                          original_point[2] * blend1_scale];

        let blend1 = self.sources[1].get(original_point) * blend1_persistence;
        blend += blend1;

        result = mask.mul_add(result - blend, blend);


        (result / self.scale) * self.z_scale

    }
}
