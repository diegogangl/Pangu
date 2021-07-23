use super::Modifier;
use super::super::map::Map2D;
use super::super::math;
use super::super::curve::Curve;

use pyo3::prelude::*;
use pyo3::types::PyDict;

extern crate rayon;
use rayon::prelude::*;

/// Terraces modifier
///
/// Creates a terraces-like effect by flattening certain areas
/// of the terrain. These areas are defined by their height,
/// according to the control points in the curve.
#[derive(Clone, Debug)]
pub struct Terraces {
    /// Invert Slope
    pub invert: bool,

    /// Control points for the terraces
    pub curve: Curve,

    /// Slope for terraces
    pub slopes: Vec<i32>,
}


impl Modifier for Terraces {
    fn run(&mut self, hmap: &mut Map2D<f64>) {
        hmap.contents.par_iter_mut().enumerate().for_each(|(i, value)|{
            // Get indices of the nearest two points
            let indexes = self.curve.points_near(*value);

            // If some control points are missing get the output value
            // of the nearest control point and return. This can
            // happen when value < lowest_point or value > highest_point
            if indexes.0 == indexes.1 {
                *value = self.curve.point(indexes.1).input;
            } else {
                // Get values and calculate alpha parameter for lerping
                let mut input_0 = self.curve.point(indexes.0).input;
                let mut input_1 = self.curve.point(indexes.1).input;
                let mut alpha = (*value - input_0) / (input_1 - input_0);
                let slope = self.slopes[indexes.1];

                if self.invert {
                    alpha = 1.0 - alpha;
                    std::mem::swap(&mut input_0, &mut input_1);
                }

                *value = math::lerp(input_1, input_0, alpha.powi(slope));
            }

        })
    }
}


impl Terraces {

    pub fn new(params: &PyDict) -> PyResult<Self> {
        let height: f64 = get!(params, "height");
        let points: Vec<f64> = get!(params, "points");
        let slopes: Vec<i32> = get!(params, "slopes");

        debug!("Adding control points for terrace");

        let mut curve = Curve::new();

        points.iter().for_each(|p| {
           let point = math::percent_to_value(*p, height);
           curve.add_point(point, point);
        });

        Ok(Terraces {
            invert: get!(params, "invert"),
            curve: curve,
            slopes: slopes,
        })
    }
}

