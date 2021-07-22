use super::Modifier;
use super::super::map::Map2D;
use super::super::curve::Curve;

use pyo3::prelude::*;
use pyo3::types::PyDict;

extern crate rayon;
use rayon::prelude::*;

/// Remap modifier
///
/// Remap the height of the terrain according to a curve function
#[derive(Clone, Debug)]
pub struct Remap {
    /// Control points
    pub curve: Curve,
}


impl Modifier for Remap {
    fn run(&mut self, hmap: &mut Map2D<f64>) {
        hmap.contents.par_iter_mut().for_each(|value| {
            *value = self.curve.interpolate(*value);
        });
    }

}


impl Remap {

    pub fn new(params: &PyDict) -> PyResult<Self> {
        let points: Vec<(f64, f64)> = get!(params, "points");

        debug!("Adding control points for remap");

        let mut curve = Curve::new();

        points.iter().for_each(|p| {
           curve.add_point(p.0, p.1);
        });

        Ok(Remap {
            curve: curve,
        })
    }
}

