use super::math;
use super::curve::Curve;
use rand::distributions::{Distribution, Uniform};


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


impl Default for Terraces {
    fn default() -> Self {
        Terraces {
            enabled: false,
            invert: false,
            curve: Curve::new(),
        }
    }
}


impl Terraces {

    /// Return a Terrace modifier with points from list
    ///
    /// # Arguments
    ///
    /// * `points` - A list of control points. Range: [0..100]
    /// * `height` - The terrain's height
    pub fn from_list(points: Vec<f64>, height: f64) -> Self {
        debug!("Adding control points for terrace");

        let mut curve = Curve::new();

        points.iter().for_each(|p| {
           let point = math::percent_to_value(*p, height);
           curve.add_point(point);
        });

        Terraces {
            enabled: true,
            invert: false,
            curve: curve,
        }
    }


    /// Calculate the terrace effect
    ///
    /// # Arguments
    /// * `value - A height value from the terrain
    pub fn run(&self, value: f64) -> f64 {

        // Get indices of the nearest two points
        let indexes = self.curve.points_near(value);

        // If some control points are missing get the output value
        // of the nearest control point and return. This can
        // happen when value < lowest_point or value > highest_point
        if indexes.0 == indexes.1 {
            return self.curve.point(indexes.1);
        }

        // Get values and calculate alpha parameter for lerping
        let mut input_0 = self.curve.point(indexes.0);
        let mut input_1 = self.curve.point(indexes.1);
        let mut alpha = (value - input_0) / (input_1 - input_0);

        if self.invert {
            alpha = 1.0 - alpha;
            std::mem::swap(&mut input_0, &mut input_1);
        }

        math::lerp(input_1, input_0, alpha.powi(2))
    }
}


/// Style of smoothing effect to apply
#[derive(Clone, Debug)]
pub enum SmoothStyle {
    RADIAL,
    LINEAR
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

    /// Number of rows in terrain
    pub rows: f64,

    /// Number of columns in terrain
    pub columns: f64,
}


impl Default for Smooth {
    fn default() -> Self {
        Smooth {
            enabled: false,
            style: SmoothStyle::RADIAL,
            radial_fac: 0.0,
            radial_size: (0.0, 0.0),
            linear_fac: (0.0, 0.0),
            linear_start: (0.0, 0.0),
            linear_invert: (false, false),
            rows: 64.0,
            columns: 64.0,
        }
    }
}


impl Smooth {

    /// Run the smooth modifier effect
    ///
    /// # Arguments
    ///
    /// * `x` - X index of the current height point
    /// * `y` - Y index of the current height point
    pub fn run(&self, x: u32, y:u32) -> f64 {
        match self.style {
            SmoothStyle::LINEAR => self.linear(x, y),
            SmoothStyle::RADIAL => self.radial(x, y),
        }
    }


    /// Create a circular smooth effect
    ///
    /// Only works in square terrains for now
    ///
    /// # Arguments
    ///
    /// * `x` - X index of the current height point
    /// * `y` - Y index of the current height point
    fn radial(&self, x: u32, y: u32) -> f64 {
        let center_x = self.columns / 2.0;
        let center_y = self.rows / 2.0;

        let max_dist = math::distance(center_x, center_y,
                                      self.columns - self.radial_size.0,
                                      self.rows - self.radial_size.1);

        let dist = math::distance(center_x, center_y, x as f64, y as f64);
        let normalized = (dist / max_dist).min(1.0);

        // Normalized with a power of <1 creates pointy terrains
        math::lerp(0.0, 1.0, normalized.powf(self.radial_fac - normalized))
    }


    /// Create a smooth effect on one or two axis
    ///
    /// # Arguments
    ///
    /// * `x` - X index of the current height point
    /// * `y` - Y index of the current height point
    fn linear(&self, x: u32, y: u32) -> f64 {
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
}


/// Thermal Erosion modifier
///
/// Simulates terrain breaking loose and falling down
/// a slope and piling at the bottom
#[derive(Clone, Debug)]
pub struct ThermalErosion {
    /// Enable the modifier
    pub enabled: bool,

    /// Talus angle in radians. Soil at the top of a slope whose
    /// inclination is higher than this value will be broken
    /// and moved to its lowest neighbor
    pub talus: f64,

    /// Number of times to run the algorithm on the terrain
    pub iterations: u8,
}


impl Default for ThermalErosion {
    fn default() -> Self {
        ThermalErosion {
            enabled: false,
            talus: 0.02,
            iterations: 1,
        }
    }
}


impl ThermalErosion {

    /// Run the thermal erosion simulation
    ///
    /// # Arguments
    ///
    /// * `verts` - Reference to the vertices vector
    pub fn run(&self, hmap: &mut Vec<f64>) {
        let size = (hmap.len() as f64).sqrt() as u32;

        for _ in 0..self.iterations {
            for x in 0..size {
                for y in 0..size {

                   // Maximum slope found
                   let mut slope_max = 0.0;

                   // Index of the lowest neighbor (1D)
                   let mut lowest_index = 0;

                   // Current height
                   let center_idx = math::index_1d(x, y, size);
                   let center = hmap[center_idx];

                   // Rotated Von Neuhmann neighbors
                   let nw = if x > 0 && y < size - 1 {
                       Some(math::index_1d(x - 1, y + 1, size))
                   } else {
                       None
                   };

                   let ne = if x < size - 1 && y < size - 1 {
                       Some(math::index_1d(x + 1, y + 1, size))
                   } else {
                       None
                   };

                   let sw = if x > 0 && y > 0 {
                       Some(math::index_1d(x - 1, y - 1, size))
                   } else {
                       None
                   };

                   let se = if x < size - 1 && y > 0 {
                       Some(math::index_1d(x + 1, y - 1, size))
                   } else {
                       None
                   };

                   // Find lowest neighbor
                   [nw, sw, se, ne].iter().for_each(|index|{
                        match index {
                            Some(i) => {
                                 let diff = center - hmap[*i];

                                 if diff > slope_max {
                                    slope_max = diff;
                                    lowest_index = *i;
                                 }
                             },

                            _ => ()
                        }
                    });


                    // Move soil
                    if slope_max > self.talus {

                       // According to the algorithm this should be 2.0,
                       // but it causes oscillations. This value works
                       // correctly for some reason, who am I to judge?
                       let magic_number = 4.0;

                       // Remove from current
                       hmap[center_idx] -= slope_max / magic_number;

                       // Add to neighbor
                       hmap[lowest_index] += slope_max / magic_number;
                    }
                }
            }
        }
    }
}


// TODO: Convert flux into a type (struct)
// TODO: Add methods to get out flow safely
// TODO: Convert velocity into a type (struct)


#[derive(Clone, Debug)]
pub struct WaterErosion {

    /// Enable the modifier
    pub enabled: bool,

    /// Number of times to run the algorithm on the terrain
    pub iterations: u8,

    pub evaporation: f64,
    pub rain_rate: f64,
    pub soil_capacity: f64,

    size:  u32,
    water: Vec<f64>,
    sediment: Vec<f64>,
    flux: Vec<[f64; 4]>,
    velocity: Vec<[f64; 2]>,
}


impl WaterErosion {

    fn rain(&mut self) {

        let mut rng = rand::thread_rng();
        let dist = Uniform::from(1..self.size - 2);

        for _ in 0..100 {
            let x = dist.sample(&mut rng);
            let y = dist.sample(&mut rng);
            let i = math::index_1d(x as u32, y as u32, self.size);
            self.water[i] += self.rain_rate;


            {
                let i = math::index_1d(x - 1 as u32, y - 1 as u32, self.size);
                self.water[i] += self.rain_rate;
            }
            {
                let i = math::index_1d(x as u32, y - 1 as u32, self.size);
                self.water[i] += self.rain_rate;
            }

            {
                let i = math::index_1d(x - 1 as u32, y + 1 as u32, self.size);
                self.water[i] += self.rain_rate;
            }

            {
                let i = math::index_1d(x + 1 as u32, y as u32, self.size);
                self.water[i] += self.rain_rate;
            }

            {
                let i = math::index_1d(x as u32, y + 1 as u32, self.size);
                self.water[i] += self.rain_rate;
            }

            {
                let i = math::index_1d(x + 1 as u32, y + 1 as u32, self.size);
                self.water[i] += self.rain_rate;
            }

            {
                let i = math::index_1d(x - 1 as u32, y + 1 as u32, self.size);
                self.water[i] += self.rain_rate;
            }

        }
    }


    fn evaporate(&mut self) {

        for i in 0..self.water.len() {
            let w = self.water[i] * (1.0 - self.evaporation);
            self.water[i] = w.max(0.0);
        }
    }


    fn flow(&mut self, heights: &mut Vec<f64>) {

        // Outflux computation settings
        ////////////////////////////////////////////////////////////
        let a = 0.00005;        // Cross-sectional area of the pipe
        let gravity = 9.81;

        let flux_factor = a * gravity;


        // Outflow Flux Computation with boundary conditions
        ////////////////////////////////////////////////////////////
        for x in 0..self.size {
            for y in 0..self.size {
                let center_idx = math::index_1d(x, y, self.size);
                let center = heights[center_idx] + self.water[center_idx];

                let l_flux = if x > 0 {
                    let idx = math::index_1d(x - 1, y, self.size);
                    let dh = center - (heights[idx] + self.water[idx]);
                    let result = self.flux[center_idx][0] + flux_factor * dh;
                    result.max(0.0)
                } else {
                    0.0
                };


                let r_flux = if x < self.size - 1 {
                    let idx = math::index_1d(x + 1, y, self.size);
                    let dh = center - (heights[idx] + self.water[idx]);
                    let result = self.flux[center_idx][1] + flux_factor * dh;
                    result.max(0.0)
                } else {
                    0.0
                };

                let b_flux = if y > 0 {
                    let idx = math::index_1d(x, y - 1, self.size);
                    let dh = center - (heights[idx] + self.water[idx]);
                    let result = self.flux[center_idx][2] + flux_factor * dh;
                    result.max(0.0)
                } else {
                    0.0
                };

                let t_flux = if y < self.size - 1 {
                    let idx = math::index_1d(x, y + 1, self.size);
                    let dh = center - (heights[idx] + self.water[idx]);
                    let result = self.flux[center_idx][3] + flux_factor * dh;
                    result.max(0.0)
                } else {
                    0.0
                };

                let total_flux = l_flux + r_flux + b_flux + t_flux;
                let k = (self.water[center_idx] / total_flux).min(1.0);

                self.flux[center_idx] = [l_flux * k, r_flux * k, b_flux * k, t_flux * k];
            }
        }


        // Update water surface and velocity field
        ////////////////////////////////////////////////////////////
        for x in 0..self.size {
            for y in 0..self.size {
                let i = math::index_1d(x, y, self.size);
                let in_flow = {
                    let mut flow = 0.0;

                    // R FLUX
                    if x > 0 {
                        let i = math::index_1d(x - 1, y, self.size);
                        flow += self.flux[i][1];
                    }

                    // L FLUX
                    if x < self.size - 1 {
                        let i = math::index_1d(x + 1, y, self.size);
                        flow += self.flux[i][0];
                    }

                    // T FLUX
                    if y > 0 {
                        let i = math::index_1d(x, y - 1, self.size);
                        flow += self.flux[i][3];
                    }

                    // B FLUX
                    if y < self.size - 1 {
                        let i = math::index_1d(x, y + 1, self.size);
                        flow += self.flux[i][2];
                    }

                    flow
                };

                let out_flow: f64 = self.flux[i].iter().sum();
                let delta_v = in_flow - out_flow;
                let old_water = self.water[i];
                self.water[i] += delta_v;
                self.water[i] = self.water[i].max(0.0);
                let mean_water = (old_water + self.water[i]) / 2.0;

                if mean_water == 0.0 {
                    self.velocity[i] = [0.0, 0.0];
                } else {
                    // R FLUX
                    let r_out = if x > 0 {
                        let i = math::index_1d(x - 1, y, self.size);
                        self.flux[i][1]
                    } else {
                        0.0
                    };

                    // L FLUX
                    let l_out = if x < self.size - 1 {
                        let i = math::index_1d(x + 1, y, self.size);
                        self.flux[i][0]
                    } else {
                        0.0
                    };

                    // T FLUX
                    let t_out = if y > 0 {
                        let i = math::index_1d(x, y - 1, self.size);
                        self.flux[i][3]
                    } else {
                        0.0
                    };

                    // B FLUX
                    let b_out = if y < self.size - 1 {
                        let i = math::index_1d(x, y + 1, self.size);
                        self.flux[i][2]
                    } else {
                        0.0
                    };

                    let u = {
                        ((r_out - self.flux[i][0] - l_out + self.flux[i][1]) / mean_water) / 2.0
                    };

                    let v = {
                        ((t_out - self.flux[i][2] - b_out + self.flux[i][3]) / mean_water) / 2.0
                    };

                    self.velocity[i] = [u, v];
                }
            }
        }
    }


    fn erosion(&mut self, heights: &mut Vec<f64>) {
        let kc = 1.0;                  // sediment capacity constant
        let ks = 0.0001 * 12.0 * 10.0;  // dissolving constant
        let kd = 0.0001 * 12.0 * 10.0;  // deposition constant

        let up = [0.0, 0.0, 1.0];

        for x in 0..self.size {
            for y in 0..self.size {
                let i = math::index_1d(x, y, self.size);

                // local velocity
                let u_v = self.velocity[i][0];
                let v_v = self.velocity[i][1];

                let normal = {
                   let right = if x < self.size - 1 {
                       let i = math::index_1d(x + 1, y, self.size);
                       heights[i]
                   } else {
                       let i = math::index_1d(self.size - 1, y, self.size);
                       heights[i]
                   };

                   let left = if x > 0 {
                       let i = math::index_1d(x - 1, y, self.size);
                       heights[i]
                   } else {
                       let i = math::index_1d(0, y, self.size);
                       heights[i]
                   };

                   let top = if y < self.size - 1 {
                       let i = math::index_1d(x, y + 1, self.size);
                       heights[i]
                   } else {
                       let i = math::index_1d(x, self.size - 1, self.size);
                       heights[i]
                   };

                   let bottom = if y > 0 {
                       let i = math::index_1d(x, y - 1, self.size);
                       heights[i]
                   } else {
                       let i = math::index_1d(x, 0, self.size);
                       heights[i]
                   };

                   math::normalize(&[right - left, top - bottom , 2.0])
                };

                let cosa = math::dot(&normal, &up);
                let sin_alpha = cosa.acos().sin().max(0.5);

                // local sediment capacity of the flow
                let capacity = kc * (u_v * u_v + v_v * v_v).sqrt()
                                     * sin_alpha
                                     * (self.water[i].min(0.01) / 0.01);

                if capacity > self.sediment[i] {
                    let d = ks * (capacity - self.sediment[i]);
                    heights[i] -= d;
                    self.sediment[i] += d;
                }
                // deposit onto ground
                else {
                    let d = kd * (self.sediment[i] - capacity);
                    heights[i] += d;
                    self.sediment[i] -= d;
                }
            }
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        WaterErosion {
            enabled: false,
            iterations: 20,
            evaporation: 0.00005,
            rain_rate: 1.0 / 16.0,
            soil_capacity: 0.1,
            water: vec![0.0; capacity],
            sediment: vec![0.0; capacity],
            sediment_tmp: vec![0.0; capacity],
            flux: vec![[0.0, 0.0, 0.0, 0.0]; capacity],
            velocity: vec![[0.0, 0.0]; capacity],
            size: (capacity as f64).sqrt() as u32,
        }

    }


    pub fn run(&mut self, heights: &mut Vec<f64>) {
        for _ in 0..self.iterations {
            self.rain();
            self.flow(heights);
            self.erosion(heights);
            self.evaporate();

        }
   }
}
