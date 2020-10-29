use super::Modifier;
use super::super::map::Map2D;
use super::super::math;

use noise::{NoiseFn, Perlin, Seedable};

use pyo3::prelude::*;
use pyo3::types::PyDict;


/// Island modifier
///
/// Applies a mask to flatten some areas of the terrain,
/// generating an island in the middle.
#[derive(Clone, Debug)]
pub struct Island {
    /// Slope for the island
    pub slope: f64,

    /// Total elevation for the island
    pub elevation: f64,

    /// Noise size
    pub breakup: f64,

    /// Roughness of the noise
    pub roughness: f64,

    /// Intensity of the noise breaking the circle shape
    pub noise_intensity: f64,

    /// Perlin noises
    perlin: Vec<Perlin>,

    /// Maximum distance. Used to normalize the radial gradient.
    max_dist: f64,

    /// Center of the radial gradient
    center: [f64; 2],

    /// The island mask
    mask: Map2D<f64>,
}


impl Modifier for Island {
    fn run(&mut self, hmap: &mut Map2D<f64>) {

        // Counting on mask and hmap having the same number of
        // values, otherwise it's crash time!
        for (x, y) in hmap.iter_indices() {

            let mut noise = 0.0;
            let mut amplitude = 1.1 - (self.breakup / 100.0) ;
            let mut current_point = [x as f64 * self.breakup / 100.0,
                                     y as f64 * self.breakup / 100.0];

            for i in 0..5 {
                noise += self.perlin[i].get(current_point) * amplitude;
                current_point = [current_point[0] * 4.0,
                                 current_point[1] * 4.0];

                amplitude /= self.roughness;
            }

            // Generate radial gradient
            let radial = {
                let dist = math::distance(self.center, [x as f64, y as f64]);
                let normalized = (dist.sqrt() / self.max_dist.sqrt()).min(1.0);
                math::lerp(0.0, 1.0, normalized.powf(3.0))
            };

            // Second, smaller radial gradient
            let radial2 = {
                let max_dist2 = self.max_dist / 1.5;
                let dist = math::distance(self.center, [x as f64, y as f64]);
                let normalized = (dist.sqrt() / max_dist2.sqrt()).min(1.0);
                math::lerp(0.0, 1.0, normalized.powf(3.0))
            };

            // Adjust resulting noise
            noise = math::lerp(0.0, 1.0, noise);
            noise *= self.noise_intensity;

            let mut mask = math::clamp(radial - noise, 0.0, 1.0);
            mask *= self.elevation;
            mask = mask.powf(self.slope);

            self.mask[x][y] = mask * radial2;

            // Substract noise from radial and apply mask
            hmap[x][y] *= self.mask[x][y];
        }
    }


}


impl Island {

    pub fn new(params: &PyDict) -> PyResult<Self> {
        let rows: f64 = get!(params, "rows");
        let columns: f64 = get!(params, "columns");
        let seed: u32 = get!(params, "seed");

        // Initialize needed values
        let center_x = rows / 2.0;
        let center_y = columns / 2.0;
        let mut size: f64 = get!(params, "size");
        size = math::remap([0.0, 100.0], size, [columns / 2.5, 1.0]);
        let max_dist = math::distance([center_x, center_y],
                                      [columns - size, rows - size]);

        // Initialize noise generators
        let mut perlin: Vec<Perlin> = Vec::with_capacity(6);

        for i in 0..5 {
            perlin.push(Perlin::new().set_seed(seed + i));
        }

        // Convert parameters
        let mut elevation: f64 = get!(params, "elevation");
        elevation = math::remap([0.0, 100.0], elevation, [0.0, 1.0]);

        let mut breakup: f64 = get!(params, "breakup");
        breakup = math::remap([0.0, 100.0], breakup, [0.0, 10.0]);

        let mut roughness: f64 = get!(params, "roughness");
        roughness = math::remap([0.0, 100.0], roughness, [6.0, 3.0]);

        let mut noise_intensity: f64 = get!(params, "noise_intensity");
        noise_intensity = math::remap([0.0, 100.0], noise_intensity, [0.0, 1.5]);

        let mut slope: f64 = get!(params, "slope");
        slope = math::remap([0.0, 100.0], slope, [0.2, 2.0]);


        Ok(Island {
            mask: Map2D::with_size(rows as usize, columns as usize, 0.0),
            center: [center_x, center_y],
            max_dist: max_dist,
            perlin: perlin,
            slope: slope,
            elevation: elevation,
            breakup: breakup,
            roughness: roughness,
            noise_intensity: noise_intensity
        })
    }
}

