extern crate noise;

use noise::{NoiseFn, Perlin, Point3, Seedable};

/// Noise function that outputs custom fractal noise
///
/// Landfractal is based on fBm but allows for tweaking
/// each octave independently.
#[derive(Clone, Debug, Default)]
pub struct LandFractal {
    z_scale: f64,
    seed: u32,
    sources: Vec<Perlin>,
}


impl LandFractal {
    pub const DEFAULT_Z_SCALE: f64 = 15.0;

    pub fn new(seed: u32) -> Self {
        LandFractal { seed: seed,
                      z_scale: Self::DEFAULT_Z_SCALE,
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
        mask_point = scale!(point, mask_control);
        let mask = self.sources[0].get(mask_point);


        //------------------------------------------------------------------------------------------
        // DOMAIN WARPING
        //------------------------------------------------------------------------------------------

        let domain_base = 1.5;
        domain_point = scale!(point, domain_base);
        let mut domain = self.sources[0].get(domain_point);

        domain_point = scale!(point, domain_base);
        domain += self.sources[1].get(domain_point) * 0.5;

        domain_point = scale!(point, domain_base);
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
        point = scale!(point, octave1_scale, domain);

        let octave1 = self.sources[1].get(point) * octave1_persistence;
        result += octave1;


        //------------------------------------------------------------------------------------------
        // Larger details

        let octave2_scale = 2.0;
        let octave2_persistence = 0.4;
        point = scale!(point, octave2_scale, domain);

        let octave2 = self.sources[2].get(point) * octave2_persistence;
        result += octave2;


        //------------------------------------------------------------------------------------------
        // Larger details

        let octave3_scale = 2.0;
        let octave3_persistence = 0.25;
        point = scale!(point, octave3_scale, domain);

        let octave3 = self.sources[3].get(point) * octave3_persistence;
        result += octave3;


        //------------------------------------------------------------------------------------------
        // Larger details

        let octave4_scale = 2.0;
        let octave4_persistence = 0.1;
        point = scale!(point, octave4_scale, domain);

        let octave4 = self.sources[4].get(point) * octave4_persistence;
        result += octave4;

        //------------------------------------------------------------------------------------------
        // Larger details
        let octave5_scale = 2.0;
        let octave5_persistence = 0.01;
        point = scale!(point, octave5_scale, domain);

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
        original_point = scale!(original_point, blend1_scale, domain);

        let blend1 = self.sources[1].get(original_point) * blend1_persistence;
        blend += blend1;

        result = mask.mul_add(result - blend, blend);

        result * self.z_scale
    }
}
