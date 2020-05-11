use super::Modifier;
use super::super::map::Map2D;
use super::super::math;
use super::super::curve::Curve;

use pyo3::prelude::*;
use pyo3::types::PyDict;


/// Terraces modifier
///
/// Creates a terraces-like effect by flattening certain areas
/// of the terrain. These areas are defined by their height,
/// according to the control points in the curve.
#[derive(Clone, Debug)]
pub struct Terraces {
    /// Enable the modifier
    pub enabled: bool,

    /// Invert Slope
    pub invert: bool,

    /// Control points for the terraces
    pub curve: Curve,
}


impl Modifier for Terraces {
    fn is_enabled(&self) -> bool { false }

    fn run(&mut self, hmap: &mut Map2D<f64>) {
        for (x, y) in hmap.iter_indices() {
            let z = hmap[x][y];

            // Get indices of the nearest two points
            let indexes = self.curve.points_near(z);

            // If some control points are missing get the output value
            // of the nearest control point and return. This can
            // happen when value < lowest_point or value > highest_point
            if indexes.0 == indexes.1 {
                hmap[x][y] = self.curve.point(indexes.1);
            } else {
                // Get values and calculate alpha parameter for lerping
                let mut input_0 = self.curve.point(indexes.0);
                let mut input_1 = self.curve.point(indexes.1);
                let mut alpha = (z - input_0) / (input_1 - input_0);

                if self.invert {
                    alpha = 1.0 - alpha;
                    std::mem::swap(&mut input_0, &mut input_1);
                }

                hmap[x][y] = math::lerp(input_1, input_0, alpha.powi(2));
            }

        }
    }
}


impl Terraces {

    pub fn new(params: &PyDict) -> PyResult<Self> {
        let height: f64 = get!(params, "height");
        let points: Vec<f64> = get!(params, "points");

        debug!("Adding control points for terrace");

        let mut curve = Curve::new();

        points.iter().for_each(|p| {
           let point = math::percent_to_value(*p, height);
           curve.add_point(point);
        });

        Ok(Terraces {
            enabled: get!(params, "enabled"),
            invert: get!(params, "invert"),
            curve: curve,
        })
    }
}

