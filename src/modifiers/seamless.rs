use super::Modifier;
use super::super::map::Map2D;
use super::super::math;

use pyo3::prelude::*;
use pyo3::types::PyDict;


/// Make terrain tileable
///
/// Takes the edges and interpolates them to make the terrain
/// tileable on X and Y. Also transitions from the edge towards
/// the center fading the change, so it looks more natural.
#[derive(Clone, Debug)]
pub struct Seamless {

    /// Percentage of column/rows to use to fade
    /// the seamless transition
    pub fade: f64,

}


impl Seamless {
    pub fn new(params: &PyDict) -> PyResult<Self> {
        Ok(Seamless {
            fade: get!(params, "fade"),
        })
    }
}


impl Modifier for Seamless {
    fn run(&mut self, hmap: &mut Map2D<f64>) {
        let height = hmap.height();
        let width = hmap.width();

        let min_fade = {
            let total = (width - 1).min(height - 1) as f64;
            math::percent_to_value(self.fade, total / 2.0)
        };

        let fac_step = 0.99 / (min_fade.max(2.0) - 1.0);
        let mut factor = 0.99;

        // Set borders to an interpolation of both opposing sides
        for i in 0..=1 {
            for x in 0..height {
                let far_side = height - (i + 1);
                let val = math::lerp(hmap[i][x], hmap[far_side][x], 0.5);

                hmap[i][x] = val;
                hmap[far_side][x] = val;
            }

            for y in 0..width {
                let far_side = width - (i + 1);
                let val = math::lerp(hmap[y][i], hmap[y][far_side], 0.5);

                hmap[y][i] = val;
                hmap[y][far_side] = val;
            }
        }


        // Fade the change from the borders towards the center
        for i in 1..=min_fade as usize {
            factor -= fac_step;

            for x in 0..height {
                hmap[i + 1][x] = math::lerp(hmap[i][x],
                                            hmap[i + 1][x],
                                            factor);

                let far_side = height - (i + 1);
                hmap[far_side][x] = math::lerp(hmap[height - i][x],
                                               hmap[far_side][x],
                                               factor);
            }

            for y in 0..width {
                hmap[y][i + 1] = math::lerp(hmap[y][i],
                                            hmap[y][i + 1],
                                            factor);

                let far_side = width - (i + 1);
                hmap[y][far_side] = math::lerp(hmap[y][width - i],
                                               hmap[y][far_side],
                                               factor);
            }
        }
    }
}
