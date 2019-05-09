extern crate noise;
extern crate test;

use noise::{NoiseFn, Perlin, Point3, Seedable};


/// Noise function that outputs custom fractal noise
///
/// The noise function generate 6 octaves of Fbm noise,
/// blended with another 2 octaves of smoother terrain.
/// Coordinates are also warped by another small Fbm noise.
#[derive(Clone, Debug, Default)]
pub struct LandFractal {

    /// Multiplier for the terrain height
    z_scale: f64,

    /// Roughness of the terrain (last octaves)
    roughness: f64,

    /// Perlin noises for the octaves
    sources: Vec<Perlin>,
}


impl LandFractal {
    const DEFAULT_Z_SCALE: f64 = 15.0;
    const DEFAULT_ROUGHNESS: f64 = 0.5;

    pub fn new(seed: u32) -> Self {
        LandFractal { z_scale: Self::DEFAULT_Z_SCALE,
                      roughness: Self::DEFAULT_ROUGHNESS,
                      sources: Self::build_sources(seed) }
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
        let mut sources = Vec::with_capacity(6);

        for i in 0..6 {
            sources.push(Perlin::new().set_seed(seed + i));
        }

        sources
    }

    setter!(set_z_scale, z_scale, f64);
    setter!(set_roughness, roughness, f64);
}


/// Macro to scale a point
///
/// This macro can also apply Domain Warping
/// on the X and Y axis.
///
/// # Arguments
///
/// * `var` - The Point3 variable.
/// * `fac - The scaling factor
/// * `warp - Domain warping value
macro_rules! scale {
    ($var:ident, $fac:expr) => {
        [$var[0] * $fac, $var[1] * $fac, $var[2] * $fac]
    };

    ($var:ident, $fac:expr, $warp:expr) => {
        [$var[0] * $fac + $warp,
         $var[1] * $fac + $warp,
         $var[2] * $fac]
    };

}


impl NoiseFn<Point3<f64>> for LandFractal {

    /// Get noise value
    ///
    /// # Arguments
    /// * `point` - The coordinates in 3D space for the noise
    fn get(&self, point: Point3<f64>) -> f64 {
        let mut result;
        let mut domain;
        let mut blend;
        let mut current_point;


        //------------------------------------------------------------------------------------------
        // BLEND MASK
        //------------------------------------------------------------------------------------------
        let mask_control = 1.1;
        current_point = scale!(point, mask_control);

        let mask = self.sources[0].get(current_point);


        //------------------------------------------------------------------------------------------
        // DOMAIN WARPING
        //------------------------------------------------------------------------------------------

        let domain_scale = 1.5;

        current_point = scale!(point, domain_scale);
        domain = self.sources[1].get(current_point);

        current_point = scale!(current_point, domain_scale);
        domain += self.sources[1].get(current_point) * 0.5;

        current_point = scale!(current_point, domain_scale);
        domain += self.sources[2].get(current_point) * 0.25;

        domain *= 0.10;


        //------------------------------------------------------------------------------------------
        // BASE FRACTAL NOISE
        //------------------------------------------------------------------------------------------

        //------------------------------------------------------------------------------------------
        // Basic shape of the terrain

        let base_control = 1.5;
        result = self.sources[0].get(point) * base_control;


        //------------------------------------------------------------------------------------------
        // Large features of the terrain

        let octave1_scale = 1.4;
        let octave1_persistence = 0.9;

        current_point = scale!(point, octave1_scale, domain);
        result += self.sources[1].get(current_point) * octave1_persistence;


        //------------------------------------------------------------------------------------------
        // Larger details

        let octave2_scale = 2.0;
        let octave2_persistence = 0.4;

        current_point = scale!(current_point, octave2_scale, domain);
        result += self.sources[2].get(current_point) * octave2_persistence;


        //------------------------------------------------------------------------------------------
        // Medium details

        let octave3_scale = 2.0;
        let octave3_persistence = 0.25;

        current_point = scale!(current_point, octave3_scale, domain);
        result += self.sources[3].get(current_point) * octave3_persistence;


        //------------------------------------------------------------------------------------------
        // Small details

        let octave4_scale = 2.0;

        current_point = scale!(current_point, octave4_scale, domain);
        result += self.sources[4].get(current_point) * self.roughness / 5.0;


        //------------------------------------------------------------------------------------------
        // Fine details
        let octave5_scale = 2.0;

        current_point = scale!(current_point, octave5_scale, domain);
        result += self.sources[5].get(current_point) * self.roughness / 10.0;


        //------------------------------------------------------------------------------------------
        // BLEND NOISE
        //------------------------------------------------------------------------------------------

        //------------------------------------------------------------------------------------------
        // Basic shape of the terrain

        let base_control = 0.1;
        blend = self.sources[3].get(point) * base_control;


        //------------------------------------------------------------------------------------------
        // Extra-details

        let blend1_scale = 2.0;
        let blend1_persistence = 0.2;

        current_point = scale!(point, blend1_scale, domain);
        blend += self.sources[1].get(current_point) * blend1_persistence;


        result = mask.mul_add(result - blend, blend);
        result * self.z_scale
    }
}


mod benches {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;

    #[bench]
    fn get(b: &mut Bencher) {
        let noise_fn = LandFractal::new(0);
        b.iter(|| noise_fn.get([0.0, 0.0, 0.0]));
    }
}
