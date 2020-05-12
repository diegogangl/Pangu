use super::Modifier;
use super::super::map::Map2D;

use pyo3::prelude::*;
use pyo3::types::PyDict;


/// Pixelate modifier
///
/// Pixelate the terrain, causing it to look "blocky" (like Minecraft)
pub struct Pixelate {

    /// Enable the modifier
    pub enabled: bool,

    // Size of the blocks. This is the divider for the stride, so
    // bigger sizes mean smaller blocks.
    pub block: usize,
}


impl Pixelate {
    pub fn new(params: &PyDict) -> PyResult<Self> {
        Ok(Pixelate {
            enabled: true,
            block: get!(params, "block"),
        })
    }
}


impl Modifier for Pixelate {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn run(&mut self, hmap: &mut Map2D<f64>) {
        let width = hmap.width();
        let height = hmap.height();
        let block = self.block;

        for x in (0..width).step_by(block) {
            for y in (0..height).step_by(block) {

                // Regular coordinates
                if hmap.is_inside(x + block, y + block) {
                    let center_x = x + (block / 2);
                    let center_y = y + (block / 2);

                    let z = hmap[center_x][center_y];

                    for i in 0..block {
                        for j in 0..block {
                            hmap[x + i][y + j] = z;
                        }
                    }

                // Boundary condition
                } else {

                    let remaining_x = (width - x).min(block);
                    let remaining_y = (height - y).min(block);

                    let center_x = x + (remaining_x / 2);
                    let center_y = y + (remaining_y / 2);

                    let z = hmap[center_x][center_y];

                    for i in 0..remaining_x {
                        for j in 0..remaining_y {
                            hmap[x + i][y + j] = z;
                        }
                    }
                }
            }
        }
    }
}

