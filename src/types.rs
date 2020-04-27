use super::math;
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


/// Macro to casue ridgedness (sharpness) on heights
///
/// This macro creates an expresion to assign a value.
///
/// # Arguments
///
/// * `self` - A procedural instance
/// * `signal` - The signal from the noise function
/// * `level` - Level at which the ridgedness should be activated
macro_rules! ridge {
    ($signal:ident, $factor:ident) => {
        math::lerp(1.0 - $signal.abs(), $signal, $factor)
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


/// Basic Terrain Type
///
/// A simple generator using fractal sum
///
#[derive(Clone)]
pub struct Basic {

    /// Perlin noises for the octaves
    pub perlin: Vec<Perlin>,

    /// Large roughness
    pub breakup: f64,

    /// Detail roughness
    pub roughness: f64,
}


impl TerrainType for Basic {

    fn set_seed(&mut self, seed: u32) {
        for i in 0..6 {
            self.perlin.push(Perlin::new().set_seed(seed + i));
        };
    }

    fn height_at(&self, point: Point2<f64>) -> f64 {
        let mut result = 0.0;
        let mut current_point = point;
        let mut amplitude = 1.0;

        for i in 0..6 {
            result += self.perlin[i].get(current_point) * amplitude;
            current_point = scale!(current_point, self.breakup);
            amplitude /= self.roughness;
        }

        result
    }

}




impl Default for Basic {
    fn default() -> Self {
        Basic {
            perlin: Vec::with_capacity(7),
            breakup: 2.0,
            roughness: 1.5,
        }
    }
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


/// Mountainous terrain
///
/// Generates ridged, rough terrains
#[derive(Clone)]
pub struct Mountains {

    // Ridgedness (spikey-ness) of the mountains
    pub ridgedness: f64,

    // Sharpness of the medium terrain features
    pub sharpness: f64,

    // Number of mountains (scale at the 3rd octave)
    pub breakup: f64,

    // Terrain roughness (persistence of higher octaves)
    pub roughness: f64,

    // Amount of domain warping to apply
    pub twist: f64,

    /// Perlin noises for the octaves
    pub perlin: Vec<Perlin>,
}


impl TerrainType for Mountains {

    fn set_seed(&mut self, seed: u32) {
        for i in 0..6 {
            self.perlin.push(Perlin::new().set_seed(seed + i));
        }
    }

    fn height_at(&self, point: Point2<f64>) -> f64 {
        //---------------------------------------------------------------------
        // DOMAIN WARPING
        //---------------------------------------------------------------------

        let domain_scale = 1.5;

        let mut current_point = scale!(point, domain_scale);
        let mut domain = self.perlin[0].get(current_point);

        current_point = scale!(current_point, domain_scale);
        domain += self.perlin[1].get(current_point) * 0.5;

        current_point = scale!(current_point, domain_scale);
        domain += self.perlin[2].get(current_point) * 0.25;

        domain *= self.twist;


        //---------------------------------------------------------------------
        // FRACTAL NOISE
        //---------------------------------------------------------------------

        // Aliases for settings
        let ridgedness = self.ridgedness;
        let sharpness = self.sharpness;
        let breakup = 0.5 + self.breakup;
        let roughness = self.roughness;

        // Amplitude to multiply each octave
        let mut amp = 1.0;

        // Simple macro to increase amplitude after each octave
        macro_rules! increase_amp {
            ($amp:ident, $result:ident) => {
                $amp *= 0.5 * $result.min(0.01).max(1.0)
            };
        }

        // Octave 0
        current_point = scale!(current_point, 0.2, domain);
        let mut result = {
            let signal = self.perlin[0].get(current_point);
            ridge!(signal, ridgedness) * 0.75
        };

        increase_amp!(amp, result);


        // Octave 1
        current_point = scale!(current_point, breakup, domain * amp);
        result += {
            let signal = self.perlin[1].get(current_point);
            ridge!(signal, sharpness) * amp * 0.5
        };

        increase_amp!(amp, result);


        // Octave 2
        current_point = scale!(current_point, 2.0, domain * amp);
        result += {
            let signal = self.perlin[2].get(current_point);
            ridge!(signal, sharpness) * amp * 0.25
        };

        increase_amp!(amp, result);


        // Octave 3
        current_point = scale!(current_point, 2.0, domain * amp);
        result += {
            let signal = self.perlin[3].get(current_point);
            (1.0 - signal.abs()) * amp * (roughness / 2.0)
        };

        increase_amp!(amp, result);


        // Octave 4
        current_point = scale!(current_point, 2.0, domain * amp);
        result += {
            let signal = self.perlin[4].get(current_point);
            signal * amp * result * roughness
        };

        increase_amp!(amp, result);


        // Octave 5
        current_point = scale!(current_point, 2.0);
        result += {
            let signal = self.perlin[5].get(current_point);
            signal * amp * result * roughness
        };

        increase_amp!(amp, result);


        // Octave 6
        current_point = scale!(current_point, 3.0, domain);
        result += {
            let signal = self.perlin[2].get(current_point);
            signal * amp * roughness / (result.min(0.001).max(1.0))
        };

        result


    }
}


impl Default for Mountains {
    fn default() -> Self {
        Mountains {
            ridgedness: 0.0,
            sharpness: 0.0,
            breakup: 0.0,
            roughness: 0.0,
            twist: 0.0,
            perlin: Vec::with_capacity(4),
        }
    }
}