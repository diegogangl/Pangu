use super::Modifier;
use super::super::map::Map2D;

use pyo3::prelude::*;
use pyo3::types::PyDict;


/// Empty modifier
///
/// A modifier that does nothing. Only here for the _ case in the match arm.
pub struct Empty {}

impl Empty {
    pub fn new(params: &PyDict) -> PyResult<Self> {
        error!("Adding Empty modifier! Params are: {:?}", params);
        Ok(Empty {})
    }
}

impl Modifier for Empty {
    fn run(&mut self, _hmap: &mut Map2D<f64>) {}
}
