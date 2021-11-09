use super::super::map::Map2D;
use super::super::math;
use super::Modifier;

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


        Ok(Invert { factor })
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

        let columns = hmap.width();

        // TODO: WIP for masks

        let mut mask = Map2D::with_size(hmap.width(), hmap.height(), 0.0);
        mask.contents
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, value)| {
                let x = i / columns;

                let fac = ((columns as f64 + x as f64) / columns as f64) - 1.0;


                if fac > 0.0 {
                    *value = fac;
                } else {
                    *value = 0.0;
                };
            });

        hmap.contents
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, value)| {
                let x = i / columns;
                let y = i % columns;

                let fac = math::lerp(*value * self.factor, *value, mask[x][y]);
                let rec = math::lerp(fac + recover, fac, mask[x][y]);
                *value = rec;
                // *value += recover;
            });
    }
}
