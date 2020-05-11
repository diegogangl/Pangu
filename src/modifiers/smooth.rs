use super::Modifier;
use super::super::map::Map2D;
use super::super::math;

use pyo3::prelude::*;
use pyo3::types::PyDict;


/// Style of smoothing effect to apply
#[derive(Clone, Debug)]
pub enum SmoothStyle {
    RADIAL,
    LINEAR,
    EDGES
}


/// Smooth modifier
///
/// Smooths out the terrain in a circle on linearly
/// along an axis. This struct also has the rows and
/// columns of the terrain as f64 to avoid too many
/// type casts.
#[derive(Clone, Debug)]
pub struct Smooth {

    /// Enable the modifier
    pub enabled: bool,

    /// Smoothing style
    pub style: SmoothStyle,

    /// Slope of the radial effect
    pub radial_fac: f64,

    /// Size (radius) of the radial effect
    pub radial_size: (f64, f64),

    /// Slope of the linear effect on X/Y
    pub linear_fac: (f64, f64),

    /// Starting coordinate for linear on X/Y
    pub linear_start: (f64, f64),

    /// Invert the linear effect on X/Y
    pub linear_invert: (bool, bool),

    /// Apply effect on X axis for edges style
    pub edges_use_x: bool,

    /// Apply effect on Y axis for edges style
    pub edges_use_y: bool,

    /// Number of rows in terrain
    pub rows: f64,

    /// Number of columns in terrain
    pub columns: f64,

}


impl Modifier for Smooth {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn run(&mut self, hmap: &mut Map2D<f64>) {

        match self.style {
            SmoothStyle::LINEAR => for (x, y) in hmap.iter_indices() {
                                        hmap[x][y] *= self.linear(x, y);
                                   },

            SmoothStyle::RADIAL => for (x, y) in hmap.iter_indices() {
                                        hmap[x][y] *= self.radial(x, y);
                                   },

            SmoothStyle::EDGES => for (x, y) in hmap.iter_indices() {
                                        hmap[x][y] *=  self.from_edges(x, y);
                                   },
        }

    }
}


impl Smooth {
    pub fn new(params: &PyDict) -> PyResult<Self> {
        Ok(Smooth {
            enabled: get!(params, "enabled"),
            radial_fac: get!(params, "radial_fac"),
            radial_size: get!(params, "radial_size"),
            linear_fac: get!(params, "linear_fac"),
            linear_start: get!(params, "linear_start"),
            linear_invert: get!(params, "linear_invert"),
            edges_use_x: get!(params, "edges_use_x"),
            edges_use_y: get!(params, "edges_use_y"),
            columns: get!(params, "columns"),
            rows: get!(params, "rows"),
            style: match get!(params, "style") {
                "RADIAL" => SmoothStyle::RADIAL,
                "LINEAR" => SmoothStyle::LINEAR,
                "EDGES" => SmoothStyle::EDGES,
                _ => SmoothStyle::RADIAL,
            }
        })
    }


    /// Create a circular smooth effect
    ///
    /// Only works in square terrains for now
    ///
    /// # Arguments
    ///
    /// * `x` - X index of the current height point
    /// * `y` - Y index of the current height point
    fn radial(&self, x: usize, y: usize) -> f64 {
        let center_x = self.columns / 2.0;
        let center_y = self.rows / 2.0;

        let max_dist = math::distance([center_x, center_y],
                                      [self.columns - self.radial_size.0,
                                      self.rows - self.radial_size.1]);

        let dist = math::distance([center_x, center_y], [x as f64, y as f64]);
        let normalized = (dist.sqrt() / max_dist.sqrt()).min(1.0);

        // Normalized with a power of <1 creates pointy terrains
        math::lerp(0.0, 1.0, normalized.powf(self.radial_fac - normalized))
    }


    /// Create a smooth effect on one or two axis
    ///
    /// # Arguments
    ///
    /// * `x` - X index of the current height point
    /// * `y` - Y index of the current height point
    fn linear(&self, x: usize, y: usize) -> f64 {
        let mut multiplier = 1.0;

        if self.linear_fac.0 > 0.0 {
            let fac = if self.linear_invert.0 {
               (self.columns - (x as f64 + self.linear_start.0)) / self.columns
            } else {
               ((self.columns + (x as f64 - self.linear_start.0)) / self.columns) - 1.0
            };

            if fac > 0.0 {
                multiplier = fac.powf(self.linear_fac.0);
            } else {
                multiplier = 0.0;
            };
        };

        if self.linear_fac.1 > 0.0 {
            let fac = if self.linear_invert.1 {
               (self.rows - (y as f64 + self.linear_start.1)) / self.rows
            } else {
               ((self.rows + (y as f64 - self.linear_start.1)) / self.rows) - 1.0
            };

            if fac > 0.0 {
                multiplier *= fac.powf(self.linear_fac.1);
            } else {
                multiplier *= 0.0;
            };
        };

        multiplier
    }


    /// Create a smooth effect from the edges inwards
    ///
    /// # Arguments
    ///
    /// * `x` - X index of the current height point
    /// * `y` - Y index of the current height point
    fn from_edges(&self, x: usize, y: usize) -> f64 {

        let x_multiplier = if self.edges_use_x {
            let x_float = x as f64;
            let center_x = self.columns / 2.0;

            let abs_x = if x_float < center_x {
                x_float / center_x
            } else {
                (center_x - (x_float - center_x)) / center_x
            };

            math::cos_interp(0.0, 1.0, abs_x)
        } else {
            1.0
        };


        let y_multiplier = if self.edges_use_y {
            let y_float = y as f64;
            let center_y = self.rows / 2.0;

            let abs_y = if y_float < center_y {
                y_float / center_y
            } else {
                (center_y - (y_float - center_y)) / center_y
            };

            math::cos_interp(0.0, 1.0, abs_y)
        } else {
            1.0
        };


        x_multiplier * y_multiplier
    }
}
