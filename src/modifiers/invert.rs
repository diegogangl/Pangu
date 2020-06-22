use super::Modifier;
use super::super::map::Map2D;

use pyo3::prelude::*;
use pyo3::types::PyDict;


/// Invert modifier
///
/// Inverts the terrain
#[derive(Clone, Debug)]
pub struct Invert {
}

impl Invert {
    pub fn new(params: &PyDict) -> PyResult<Self> {
        Ok(Invert {
        })
    }
}


impl Modifier for Invert {
    fn run(&mut self, hmap: &mut Map2D<f64>) {
        for i in 0..hmap.contents.len() {
            hmap.contents[i] *= -1.0;
            hmap.contents[i] += hmap.max;

        }
    }
}
