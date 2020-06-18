use super::Modifier;
use super::super::map::{Map2D, neighbors};
use super::super::math;

use noise::{NoiseFn, Perlin, Point2, Seedable};

use pyo3::prelude::*;
use pyo3::types::PyDict;

use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;


/// Island modifier
///
/// Applies a mask to flatten some areas of the terrain,
/// generating an island in the middle.
#[derive(Clone, Debug)]
pub struct Island {
    /// Enable the modifier
    pub enabled: bool,

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
    fn is_enabled(&self) -> bool { false }

    fn run(&mut self, hmap: &mut Map2D<f64>) {

        // Counting on mask and hmap having the same number of
        // values, otherwise it's crash time!
        for (x, y) in hmap.iter_indices() {

            let mut noise = 0.0;
            let mut current_point = [x as f64 * 0.015, y as f64 * 0.015];
            let mut amplitude = 1.0;

            for i in 0..5 {
                noise += self.perlin[i].get(current_point) * amplitude;
                current_point = [current_point[0] * 3.0,
                                 current_point[1] * 3.0];

                amplitude /= 5.0;
            }

            // Adjust resulting noise
            noise = math::bright_contrast(noise, 2.0, 1.0);

            // Generate radial gradient
            let radial = {
                let dist = math::distance(self.center, [x as f64, y as f64]);
                let normalized = (dist.sqrt() / self.max_dist.sqrt()).min(1.0);

                math::lerp(0.0, 1.0, normalized.powi(5))
            };

            // Substract noise from radial and apply mask
            self.mask[x][y] = math::clamp(radial - noise, 0.0, 1.0);
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
        let max_dist = math::distance([center_x, center_y],
                                      [columns - 20.0, rows - 20.0]);

        // Initialize noise generators
        let mut perlin: Vec<Perlin> = Vec::with_capacity(6);

        for i in 0..5 {
            perlin.push(Perlin::new().set_seed(seed + i));
        }

        Ok(Island {
            enabled: get!(params, "enabled"),
            mask: Map2D::with_size(rows as usize, columns as usize, 0.0),
            center: [center_x, center_y],
            max_dist: max_dist,
            perlin: perlin,
        })
    }
}

