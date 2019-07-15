use super::math;
use super::curve::Curve;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;


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


/// Velocity field for water erosion
///
/// Represents current velocity on a single cell
#[derive(Clone, Debug)]
struct Velocity {
    pub u: f64,
    pub v: f64,
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity {
            u: 0.0,
            v: 0.0,
        }
    }
}

impl Velocity {

    /// Get magnitude of current velocity
    pub fn magnitude(&self) -> f64 {
        (self.u.powi(2) + self.v.powi(2)).sqrt()
    }
}


/// Outflow velocity (flux) for water erosion
///
/// Represents outflow on a single cell
#[derive(Clone, Debug)]
struct Outflow {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

impl Default for Outflow {
    fn default() -> Self {
        Outflow {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        }
    }
}

impl Outflow {

    /// Calculate total outflow for this cell
    pub fn total(&self) -> f64 {
        self.left + self.right + self.top + self.bottom
    }
}


/// Water Erosion modifier
///
/// Simulates erosion caused by water (rain and rivers). This
/// is based on the "Fast Hydraulic Erosion Simulation and
/// Visualization on GPU" paper by Xing mei, et all.
///
/// This method is based on the virtual pipes model, with
/// a field for the velocity of the running water.
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
    sediment_tmp: Vec<f64>,
    flux: Vec<Outflow>,
    velocity: Vec<Velocity>,
}


impl WaterErosion {

    /// Rain step
    ///
    /// New water is added every step. Rain drops fall down
    /// in a random distribution with a ceratin amount of water.
    fn rain(&mut self, seed: u8) {

        let rand_seed = [seed; 32];
        let mut rng = StdRng::from_seed(rand_seed);
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


    /// Evaporation step
    ///
    /// Some amount of water is evaporated into the air
    /// every step due to air temperature.
    fn evaporate(&mut self) {
        let evaporation = 1.0 - self.evaporation;

        self.water.iter_mut().for_each(|w| {
            *w *= evaporation;
            *w = w.max(0.0);
        });
    }


    /// Flow and velocity fields update step
    ///
    /// The ouflow flux is calculated every step. Then, the water and
    /// velocity maps are updated.
    ///
    /// # Outflow calculation
    ///
    /// Every cell exchanges water with its four neighbors through
    /// virtual pipes. The flow map mantains the outflow flux (flow velocity)
    /// for each cell. At each step this flux is accelerated by the
    /// height difference of the cells (ground + water).
    /// If the outflow flux is higher than the water of the cell, it gets
    /// scaled down by the K factor to avoid negative water height.
    ///
    /// No water can flow out of the grid, so in boundary conditions the
    /// outflow is zero.
    ///
    /// # Water map update
    ///
    /// The new water height at the current cell is calculated by collecting
    /// the inflow flux and sending out the outflow flux (inflow - outflow).
    ///
    /// # Velocity update
    ///
    /// The velocity field is also updated from the outflow flux as the
    /// average amount of water that passes through the cell per unit of time.
    ///
    /// # Arguments
    ///
    /// * `heights` - The heightmap
    fn flow(&mut self, heights: &mut Vec<f64>) {

        // Cross-sectional area of the pipe
        let pipe_area = 0.005;
        let gravity = 9.81;

        let flux_factor = pipe_area * gravity;


        // Calculate outflow flux
        for x in 0..self.size {
            for y in 0..self.size {
                let center_idx = math::index_1d(x, y, self.size);
                let center = heights[center_idx] + self.water[center_idx];

                let l_flux = if x > 0 {
                    let idx = math::index_1d(x - 1, y, self.size);
                    let dh = center - (heights[idx] + self.water[idx]);
                    let result = self.flux[center_idx].left + flux_factor * dh;
                    result.max(0.0)
                } else {
                    0.0
                };


                let r_flux = if x < self.size - 1 {
                    let idx = math::index_1d(x + 1, y, self.size);
                    let dh = center - (heights[idx] + self.water[idx]);
                    let result = self.flux[center_idx].right + flux_factor * dh;
                    result.max(0.0)
                } else {
                    0.0
                };

                let b_flux = if y > 0 {
                    let idx = math::index_1d(x, y - 1, self.size);
                    let dh = center - (heights[idx] + self.water[idx]);
                    let result = self.flux[center_idx].bottom + flux_factor * dh;
                    result.max(0.0)
                } else {
                    0.0
                };

                let t_flux = if y < self.size - 1 {
                    let idx = math::index_1d(x, y + 1, self.size);
                    let dh = center - (heights[idx] + self.water[idx]);
                    let result = self.flux[center_idx].top + flux_factor * dh;
                    result.max(0.0)
                } else {
                    0.0
                };

                let total_flux = l_flux + r_flux + b_flux + t_flux;
                let k = (self.water[center_idx] / total_flux).min(1.0);

                self.flux[center_idx].left = l_flux * k;
                self.flux[center_idx].right = r_flux * k;
                self.flux[center_idx].top = t_flux * k;
                self.flux[center_idx].bottom = b_flux * k;
            }
        }


        // Update water surface
        for x in 0..self.size {
            for y in 0..self.size {
                let i = math::index_1d(x, y, self.size);
                let in_flow = {
                    let mut flow = 0.0;

                    // Right in flux
                    if x > 0 {
                        let i = math::index_1d(x - 1, y, self.size);
                        flow += self.flux[i].right;
                    }

                    // Left in flux
                    if x < self.size - 1 {
                        let i = math::index_1d(x + 1, y, self.size);
                        flow += self.flux[i].left;
                    }

                    // Top in flux
                    if y > 0 {
                        let i = math::index_1d(x, y - 1, self.size);
                        flow += self.flux[i].top;
                    }

                    // Bottom in flux
                    if y < self.size - 1 {
                        let i = math::index_1d(x, y + 1, self.size);
                        flow += self.flux[i].bottom;
                    }

                    flow
                };

                let delta_v = in_flow - self.flux[i].total();
                let old_water = self.water[i];
                self.water[i] += delta_v;
                self.water[i] = self.water[i].max(0.0);
                let mean_water = (old_water + self.water[i]) / 2.0;

                // Update velocity field
                if mean_water == 0.0 {
                    self.velocity[i].u = 0.0;
                    self.velocity[i].v = 0.0;
                } else {
                    let r_in = if x > 0 {
                        let i = math::index_1d(x - 1, y, self.size);
                        self.flux[i].right
                    } else {
                        0.0
                    };

                    let l_in = if x < self.size - 1 {
                        let i = math::index_1d(x + 1, y, self.size);
                        self.flux[i].left
                    } else {
                        0.0
                    };

                    let t_in = if y > 0 {
                        let i = math::index_1d(x, y - 1, self.size);
                        self.flux[i].top
                    } else {
                        0.0
                    };

                    let b_in = if y < self.size - 1 {
                        let i = math::index_1d(x, y + 1, self.size);
                        self.flux[i].bottom
                    } else {
                        0.0
                    };

                    self.velocity[i].u = {
                        ((r_in - self.flux[i].left - l_in + self.flux[i].right) / mean_water) / 2.0
                    };

                    self.velocity[i].v = {
                        ((t_in - self.flux[i].bottom - b_in + self.flux[i].top) / mean_water) / 2.0
                    };
                }
            }
        }
    }

    /// Sediment transport step
    ///
    /// Every step the suspended sediment is transported with
    /// the local velocity. The destination cell is calculated
    /// from the velocity using the current cell as the origin.
    fn sediment_transport(&mut self) {
        for x in 0..self.size {
            for y in 0..self.size {
                let i = math::index_1d(x, y, self.size);

                // Where flow comes from
                let src_x = x as f64 - self.velocity[i].u;
                let src_y = y as f64 - self.velocity[i].v;

                let mut x0 = {
                    let val = src_x.floor();
                    if val > 0.0 { val as u32 } else { 0 }
                };

                let mut y0 = {
                    let val = src_y.floor();
                    if val > 0.0 { val as u32 } else { 0 }
                };

                let mut x1 = x0 + 1;
                let mut y1 = y0 + 1;

                // Calculate interpolation factors
                let fx = src_x - x0 as f64;
                let fy = src_y - y0 as f64;

                // Clamp to grid borders
                x0 = math::clamp(x0, 0, self.size - 1);
                x1 = math::clamp(x1, 0, self.size - 1);
                y0 = math::clamp(y0, 0, self.size - 1);
                y1 = math::clamp(y1, 0, self.size - 1);

                self.sediment_tmp[i] = {
                  let index_1 = math::index_1d(x0, y0, self.size);
                  let index_2 = math::index_1d(x1, y0, self.size);
                  let lerp_1 = math::lerp(self.sediment[index_1], self.sediment[index_2], fx);

                  let index_3 = math::index_1d(x0, y1, self.size);
                  let index_4 = math::index_1d(x1, y1, self.size);
                  let lerp_2 = math::lerp(self.sediment[index_3], self.sediment[index_4], fx);

                  math::lerp(lerp_1, lerp_2, fy)
                };

            }
        }

        // Write temp values to sediment map
        (0..self.sediment.len()).for_each(|i| {
            self.sediment[i] = self.sediment_tmp[i];
        });

    }


    /// Erosion step
    ///
    /// Every step some sediment is eroded from the ground (heights)
    /// and transported by the water. At the same time, some
    /// sediment will be deposited on the ground.
    ///
    /// Whether the sediment is eroded or deposited and how much
    /// is controlled by the local capacity of the cell. This is
    /// calculated from these factors:
    ///
    /// - The local tilt angle (normal of the cell)
    /// - Local velocity
    /// - Capacity constant
    /// - Dissolving constant
    /// - Deposition constant
    ///
    /// # Arguments
    ///
    /// * `heights` - The heightmap
    fn erosion(&mut self, heights: &mut Vec<f64>) {
        let kc = 1.0;                  // sediment capacity constant
        let ks = 0.0001 * 12.0 * 10.0;  // dissolving constant
        let kd = 0.0001 * 12.0 * 10.0;  // deposition constant

        let up = [0.0, 0.0, 1.0];

        for x in 0..self.size {
            for y in 0..self.size {
                let i = math::index_1d(x, y, self.size);

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
                let capacity = kc * self.velocity[i].magnitude()
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


    /// Constructor
    ///
    /// # Arguments
    ///
    /// * `capacity` - How much memory to allocate for each map
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
            flux: vec![Outflow::default(); capacity],
            velocity: vec![Velocity::default(); capacity],
            size: (capacity as f64).sqrt() as u32,
        }

    }


    /// Run the Water Erosion simulation
    ///
    /// # Arguments
    ///
    /// * `heights` - The heightmap
    pub fn run(&mut self, heights: &mut Vec<f64>) {

        debug!("Starting Water Erosion Simulation");
        debug!("Iterations: {:?}", self.iterations);
        debug!("Rain Rate: {:?}", self.rain_rate);
        debug!("Evaporation: {:?}", self.evaporation);
        debug!("Soil Capacity: {:?}", self.soil_capacity);

        for time in 0..self.iterations {
            self.rain(time as u8);
            self.flow(heights);
            self.erosion(heights);
            self.sediment_transport();
            self.evaporate();
        }
   }
}
