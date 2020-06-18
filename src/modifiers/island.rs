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
#[derive(Clone, Debug)]
pub struct Island {
    /// Enable the modifier
    pub enabled: bool,

    /// Invert Slope
    pub invert: bool,
}

impl Modifier for Island {
    fn is_enabled(&self) -> bool { false }

    fn run(&mut self, hmap: &mut Map2D<f64>) {
        let rows = hmap.height();
        let columns = hmap.width();
        let center_x = (rows / 2) as f64;
        let center_y = (columns / 2) as f64;

        let mut mask = Map2D::with_size(rows, columns, 0.0);
        let max_dist = math::distance([center_x, center_y], [(columns - 20) as f64, (rows - 20) as f64]);

        let mut perlin: Vec<Perlin> = Vec::with_capacity(6);

        for i in 0..5 {
            perlin.push(Perlin::new().set_seed(0 + i));
        }


        for (x, y) in mask.iter_indices() {

            let mut result = 0.0;
            let mut current_point = [x as f64 * 0.015, y as f64 * 0.015];
            let mut amplitude = 1.0;

            for i in 0..5 {
                result += perlin[i].get(current_point) * amplitude;
                current_point = [current_point[0] * 3.0, current_point[1] * 3.0];
                amplitude /= 5.0;
            }

            result = math::bright_contrast(result, 2.0, 1.0);
            let dist = math::distance([center_x, center_y],
                                      [x as f64, y as f64]);
            let normalized = (dist.sqrt() / max_dist.sqrt()).min(1.0);
            mask[x][y] = math::lerp(0.0, 1.0, normalized.powi(5));
            mask[x][y] = math::clamp(mask[x][y] - result, 0.0, 1.0);
        }

        for (x, y) in hmap.iter_indices() {
            hmap[x][y] *= mask[x][y];
        }
    }
}


impl Island {

    pub fn new(params: &PyDict) -> PyResult<Self> {
        Ok(Island {
            enabled: get!(params, "enabled"),
            invert: get!(params, "invert"),
        })
    }
}

