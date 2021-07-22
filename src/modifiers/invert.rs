use super::Modifier;
use super::super::map::Map2D;
use super::super::math;

use pyo3::prelude::*;
use pyo3::types::PyDict;

extern crate rayon;
use rayon::prelude::*;


/// Invert modifier
///
/// Inverts the terrain
#[derive(Clone, Debug)]
pub struct Invert {
    /// Intensity of the inversion
    pub factor: f64,
}

impl Invert {
    pub fn new(params: &PyDict) -> PyResult<Self> {
        let mut factor: f64 = get!(params, "factor");
        factor = math::remap([0.0, 100.0], factor, [1.0, -1.0]);

        Ok(Invert {
            factor: factor,
        })
    }
}


impl Modifier for Invert {
    fn run(&mut self, hmap: &mut Map2D<f64>) {

        // Recover height from the factor pushing the map down
        let recover = if self.factor < 0.0 {
            hmap.max * self.factor.abs()
        } else {
            0.0
        };

        hmap.contents.par_iter_mut().for_each(|value| {
            *value *= self.factor;
            *value += recover;
        });
    }
}
