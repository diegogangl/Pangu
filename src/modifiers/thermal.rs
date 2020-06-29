use super::Modifier;
use super::super::map::{Map2D, neighbors};

use pyo3::prelude::*;
use pyo3::types::PyDict;


/// Thermal Erosion modifier
///
/// Simulates terrain breaking loose and falling down
/// a slope and piling at the bottom
#[derive(Clone, Debug)]
pub struct ThermalErosion {
    /// Talus angle in radians. Soil at the top of a slope whose
    /// inclination is higher than this value will be broken
    /// and moved to its lowest neighbor
    pub talus: f64,

    /// Number of times to run the algorithm on the terrain
    pub iterations: u8,
}

impl ThermalErosion {
    pub fn new(params: &PyDict) -> PyResult<Self> {
        Ok(ThermalErosion {
            talus: get!(params, "talus"),
            iterations: get!(params, "iterations"),
        })
    }
}

impl Modifier for ThermalErosion {
    fn run(&mut self, hmap: &mut Map2D<f64>) {
        for _ in 0..self.iterations {
            for (x, y) in hmap.iter_indices() {
                // Maximum slope found
                let mut slope_max = 0.0;

                // Index of the lowest neighbor
                let mut lowest = (0, 0);

                // Current height
                let center = hmap[x][y];

                neighbors::VON_NEUMANN.iter().for_each(|target| {
                    let neighbor = hmap.safe_find((x, y), *target);

                    if let Some((x1, y1)) = neighbor {
                        let diff = center - hmap[x1][y1];

                        if diff > slope_max {
                            slope_max = diff;
                            lowest = (x1, y1);
                        }
                    };
                });

                // Move soil
                if slope_max > self.talus {

                    // According to the algorithm this should be 2.0,
                    // but it causes oscillations. This value works
                    // correctly for some reason, who am I to judge?
                    let magic_number = 4.0;

                    // Remove from current
                    hmap[x][y] -= slope_max / magic_number;

                    // Add to neighbor
                    hmap[lowest.0][lowest.1] += slope_max / magic_number;
                }
            }
        }
    }

}
