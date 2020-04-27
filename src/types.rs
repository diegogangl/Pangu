use noise::{NoiseFn, Perlin, Point2, Seedable};


/// Macro to scale a point
///
/// This macro can also apply Domain Warping
/// on the X and Y axis.
///
/// # Arguments
///
/// * `var` - The Point3 variable.
/// * `fac` - The scaling factor
/// * `warp` - Domain warping value
macro_rules! scale {
    ($var:ident, $fac:expr) => {
        [$var[0] * $fac, $var[1] * $fac]
    };

    ($var:ident, $fac:expr, $warp:expr) => {
        [$var[0] * $fac + $warp, $var[1] * $fac + $warp]
    };
}


/// Trait for terrain types
///
/// This is the common interface for terrain types
pub trait TerrainType {

    /// Get height at a point in the terrain
    ///
    /// # Arguments
    ///
    /// * `point`: Coordinates in the terrain
    fn height_at(&self, point: Point2<f64>) -> f64;

    /// Set seed and initialize noise functions
    ///
    /// # Arguments
    ///
    /// * `seed` - Initial seed to use (every octave has a different one)
    fn set_seed(&mut self, seed: u32);
}


/// Smooth Hills terrain
///
/// Generates smooth, rolling valleys
#[derive(Clone)]
pub struct SmoothHills {

    // General scale (first octave)
    pub difference: f64,

    // Flat area between hills
    pub flat: f64,

    // Noise on hills
    pub detail: f64,

    // Amount of domain warping to apply
    pub twist: f64,

    /// Perlin noises for the octaves
    pub perlin: Vec<Perlin>,
}


impl TerrainType for SmoothHills {

    fn set_seed(&mut self, seed: u32) {
        for i in 0..4 {
            self.perlin.push(Perlin::new().set_seed(seed + i));
        }

    }

    fn height_at(&self, point: Point2<f64>) -> f64 {

        //---------------------------------------------------------------------
        // DOMAIN WARPING
        //---------------------------------------------------------------------

        let domain_scale = 1.5;

        let mut current_point = scale!(point, domain_scale);
        let mut warp = self.perlin[0].get(current_point);

        current_point = scale!(current_point, domain_scale);
        warp += self.perlin[1].get(current_point) * 0.2;

        current_point = scale!(current_point, domain_scale);
        warp += self.perlin[2].get(current_point) * 0.1;

        warp *= self.twist;


        //---------------------------------------------------------------------
        // FRACTAL NOISE
        //---------------------------------------------------------------------

        // Basic shape of the terrain
        current_point = scale!(current_point, 0.2, warp);
        let mut result = {
            let signal = self.perlin[0].get(current_point);

            signal.abs().powf(self.flat) * self.difference
        };

        let persistences = [
            self.detail * 0.5,
            self.detail * 0.25,
            self.detail * 0.1,
            self.detail * 0.05,
        ];

        // Octave 1
        current_point = scale!(current_point, 2.5, warp * result);
        result += {
            let signal = self.perlin[1].get(current_point) * persistences[0];
            signal.powi(2).abs()
        };

        // Octave 2
        current_point = scale!(current_point, 2.0, warp * result);
        result += {
            let signal = self.perlin[2].get(current_point) * persistences[1];
            signal.powi(2)
        };

        // Octave 3
        current_point = scale!(current_point, 2.0, warp * result);
        result += {
            let signal = self.perlin[2].get(current_point) * persistences[2];
            signal.powi(2)
        };

        // Octave 4
        current_point = scale!(current_point, 2.0, warp * result);
        result += {
            let signal = self.perlin[3].get(current_point) * persistences[3];
            signal.powi(2)
        };

        result
    }
}


impl Default for SmoothHills {
    fn default() -> Self {
        SmoothHills {
            difference: 0.0,
            flat: 0.0,
            detail: 4.0,
            twist: 1.0,
            perlin: Vec::with_capacity(4),
        }
    }
}

