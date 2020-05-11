use super::Modifier;
use super::super::map::{Map2D, neighbors};
use super::super::math;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;


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


/// A source of water
#[derive(Clone, Copy, Debug)]
pub struct Spring {

    /// X coordinate
    pub x: u32,

    /// Y coordinate
    pub y: u32,

    /// Radius (size) of the spring
    pub radius: u32,

    /// Amount of water added per-iteration
    pub amount: f64,
}


impl Default for Spring {
    fn default() -> Self {
        Spring {
            x: 1,
            y: 1,
            radius: 2,
            amount: 1.0,
        }
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

    /// Rate of evaporation
    pub evaporation: f64,

    /// Amount of water on each rain drop
    pub rain_rate: f64,

    /// Global constant for how much soil water can hold
    pub soil_capacity: f64,

    /// Springs (constant sources of water)
    pub springs: Vec<Spring>,

    /// Size of the maps
    size:  u32,

    /// Water map
    water: Map2D<f64>,

    /// Sediment map and temporary sediment map
    sediment: Map2D<f64>,
    sediment_tmp: Map2D<f64>,

    /// Outflow map
    flux: Map2D<Outflow>,

    /// Velocity field map
    velocity: Map2D<Velocity>,
}



impl Modifier for WaterErosion {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn run(&mut self, hmap: &mut Map2D<f64>) {
        debug!("Starting Water Erosion Simulation");
        debug!("Iterations: {:?}", self.iterations);
        debug!("Rain Rate: {:?}", self.rain_rate);
        debug!("Evaporation: {:?}", self.evaporation);
        debug!("Soil Capacity: {:?}", self.soil_capacity);

        for time in 0..self.iterations {
            self.rain(time as u8);
            self.add_springs_water();
            self.flow(hmap);
            self.erosion(hmap);
            self.sediment_transport();
            self.evaporate();
        }
    }
}



impl WaterErosion {
    pub fn new(params: &PyDict) -> PyResult<Self> {
        let rows: usize = get!(params, "rows");
        let columns: usize = get!(params, "columns");
        let springs: Vec<&PyDict> = get!(params, "springs");
        let capacity: usize = rows * columns;

        let mut water = WaterErosion {
            enabled: get!(params, "enabled"),
            iterations: get!(params, "iterations"),
            evaporation: get!(params, "evaporation"),
            rain_rate: get!(params, "rain_rate"),
            soil_capacity: get!(params, "soil_capacity"),
            water: Map2D::with_size(columns, rows, 0.0),
            sediment: Map2D::with_size(columns, rows, 0.0),
            sediment_tmp: Map2D::with_size(columns, rows, 0.0),
            flux: Map2D::with_size(columns, rows, Outflow::default()),
            velocity: Map2D::with_size(columns, rows, Velocity::default()),
            size: (capacity as f64).sqrt() as u32,
            springs: vec![Spring::default(); 0],
        };

        let _ = water.add_springs(springs);
        Ok(water)
    }


    /// Add Springs from the springs list
    ///
    /// # Arguments
    ///
    /// * `springs` - Vector of dictionaries with settings
    fn add_springs(&mut self, springs: Vec<&PyDict>) -> PyResult<()> {
        for spring in springs {
            self.springs.push(Spring {
                x: get!(spring, "x"),
                y: get!(spring, "y"),
                radius: get!(spring, "radius"),
                amount: get!(spring, "amount"),
            })
        };

        Ok(())
    }


    /// Rain step
    ///
    /// New water is added every step. Rain drops fall down
    /// in a random distribution with a ceratin amount of water.
    fn rain(&mut self, seed: u8) {

        let rand_seed = [seed; 32];
        let mut rng = StdRng::from_seed(rand_seed);
        let dist = Uniform::from(1..self.size - 2);

        for _ in 0..100 {
            let x = dist.sample(&mut rng) as usize;
            let y = dist.sample(&mut rng) as usize;
            self.water[x][y] += self.rain_rate;

            neighbors::MOORE.iter().for_each(|dir|{
                let (x1, y1) = self.water.find((x, y), *dir);
                self.water[x1][y1] += self.rain_rate;
            });
        }
    }


    /// Add water on an area around a coordinate
    ///
    /// # Arguments
    ///
    /// * `spring` - Source of water for this drop
    fn drop_water(&mut self, spring: Spring) {
        for x1 in 0..spring.radius * 2 {
            for y1 in 0..spring.radius * 2 {
                let x = if x1 < spring.radius {
                    spring.x.checked_sub(x1)
                } else {
                    spring.x.checked_add(x1)
                };

                let y = if y1 < spring.radius {
                    spring.y.checked_sub(y1)
                } else {
                    spring.y.checked_add(y1)
                };

                if let (Some(x), Some(y)) = (x, y) {
                    if x < self.size && y < self.size  {
                        let dist = math::distance([spring.x, spring.y], [x, y]) as f64;
                        let rad2 = (spring.radius as f64).powi(2);

                        if dist < rad2 {
                            let amount = spring.amount * 0.002 * (rad2 - dist);
                            self.water[x as usize][y as usize] += amount;
                        }
                    }
                }
            }
        }
    }


    /// Add water from springs
    fn add_springs_water(&mut self) {
       self.springs.clone()
                   .iter()
                   .for_each(|s| self.drop_water(*s));
    }


    /// Evaporation step
    ///
    /// Some amount of water is evaporated into the air
    /// every step due to air temperature.
    fn evaporate(&mut self) {
        let evaporation = 1.0 - self.evaporation;

        self.water.iter_indices().for_each(|(x, y)| {
            self.water[x][y] = (self.water[x][y] * evaporation).max(0.0);
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
    fn flow(&mut self, heights: &mut Map2D<f64>) {

        // Flux factor = Pipe area * Gravity
        //
        //  Pipe area = 0.005
        /// (Cross-sectional area of the pipe. Lowering this
        /// makes the simulation more subtle)
        ///
        /// Gravity = 9.81
        const FLUX_FACTOR: f64 = 0.04905;


        // Calculate outflow flux
        for x in 0..self.size {
            for y in 0..self.size {
                let xu = x as usize;
                let yu = y as usize;

                let center = heights[xu][yu] + self.water[xu][yu];

                let l_flux = if x > 0 {
                    let dh = center - (heights[xu-1][yu] + self.water[xu-1][yu]);
                    let result = self.flux[xu][yu].left + FLUX_FACTOR * dh;
                    result.max(0.0)
                } else {
                    0.0
                };


                let r_flux = if x < self.size - 1 {
                    let dh = center - (heights[xu+1][yu] + self.water[xu+1][yu]);
                    let result = self.flux[xu][yu].right + FLUX_FACTOR * dh;
                    result.max(0.0)
                } else {
                    0.0
                };

                let b_flux = if y > 0 {
                    let dh = center - (heights[xu][yu-1] + self.water[xu][yu-1]);
                    let result = self.flux[xu][yu].bottom + FLUX_FACTOR * dh;
                    result.max(0.0)
                } else {
                    0.0
                };

                let t_flux = if y < self.size - 1 {
                    let dh = center - (heights[xu][yu+1] + self.water[xu][yu+1]);
                    let result = self.flux[xu][yu].top + FLUX_FACTOR * dh;
                    result.max(0.0)
                } else {
                    0.0
                };

                let total_flux = l_flux + r_flux + b_flux + t_flux;
                let k = (self.water[xu][yu] / total_flux).min(1.0);

                self.flux[xu][yu].left = l_flux * k;
                self.flux[xu][yu].right = r_flux * k;
                self.flux[xu][yu].top = t_flux * k;
                self.flux[xu][yu].bottom = b_flux * k;
            }
        }


        // Update water surface
        for x in 0..self.size {
            for y in 0..self.size {
                let xu = x as usize;
                let yu = y as usize;

                let in_flow = {
                    let mut flow = 0.0;


                    // Right in flux
                    if x > 0 {
                        flow += self.flux[xu - 1][yu].right;
                    }

                    // Left in flux
                    if x < self.size - 1 {
                        flow += self.flux[xu + 1][yu].left;
                    }

                    // Top in flux
                    if y > 0 {
                        flow += self.flux[xu][yu - 1].top;
                    }

                    // Bottom in flux
                    if y < self.size - 1 {
                        flow += self.flux[xu][yu + 1].bottom;
                    }

                    flow
                };

                let delta_v = in_flow - self.flux[xu][yu].total();
                let old_water = self.water[xu][yu];
                self.water[xu][yu] += delta_v;
                self.water[xu][yu] = self.water[xu][yu].max(0.0);
                let mean_water = (old_water + self.water[xu][yu]) / 2.0;

                // Update velocity field
                if mean_water == 0.0 {
                    self.velocity[xu][yu].u = 0.0;
                    self.velocity[xu][yu].v = 0.0;
                } else {
                    let r_in = if x > 0 {
                        self.flux[xu - 1][yu].right
                    } else {
                        0.0
                    };

                    let l_in = if x < self.size - 1 {
                        self.flux[xu + 1][yu].left
                    } else {
                        0.0
                    };

                    let t_in = if y > 0 {
                        self.flux[xu][yu - 1].top
                    } else {
                        0.0
                    };

                    let b_in = if y < self.size - 1 {
                        self.flux[xu][yu + 1].bottom
                    } else {
                        0.0
                    };

                    self.velocity[xu][yu].u = {
                        ((r_in - self.flux[xu][yu].left - l_in + self.flux[xu][yu].right) / mean_water) / 2.0
                    };

                    self.velocity[xu][yu].v = {
                        ((t_in - self.flux[xu][yu].bottom - b_in + self.flux[xu][yu].top) / mean_water) / 2.0
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
        for (x, y) in self.sediment.iter_indices() {

                // Where flow comes from
                let src_x = x as f64 - self.velocity[x][y].u;
                let src_y = y as f64 - self.velocity[x][y].v;

                let mut x0 = {
                    let val = src_x.floor();
                    if val > 0.0 { val as usize } else { 0 }
                };

                let mut y0 = {
                    let val = src_y.floor();
                    if val > 0.0 { val as usize } else { 0 }
                };

                let mut x1 = x0 + 1;
                let mut y1 = y0 + 1;

                // Calculate interpolation factors
                let fx = src_x - x0 as f64;
                let fy = src_y - y0 as f64;

                // Clamp to grid borders
                let size = (self.size as usize) - 1;
                x0 = math::clamp(x0, 0, size);
                x1 = math::clamp(x1, 0, size);
                y0 = math::clamp(y0, 0, size);
                y1 = math::clamp(y1, 0, size);

                self.sediment_tmp[x][y] = {
                  let lerp_1 = math::lerp(self.sediment[x0][y0],
                                          self.sediment[x1][y0], fx);

                  let lerp_2 = math::lerp(self.sediment[x0][y1],
                                          self.sediment[x1][y1], fx);

                  math::lerp(lerp_1, lerp_2, fy)
                };
        }

        // Write temp values to sediment map
        for (x, y) in self.sediment.iter_indices() {
            self.sediment[x][y] = self.sediment_tmp[x][y];
        }
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
    fn erosion(&mut self, heights: &mut Map2D<f64>) {

        /// Dissolving constant
        const KS: f64 = 0.01;

        /// Deposition constant
        const KD: f64 = 0.01;

        /// Vector pointing straight up
        const UP: [f64; 3] = [0.0, 0.0, 1.0];

        for (x, y) in heights.iter_indices() {
                let xu = x as u32;
                let yu = y as u32;

                let normal = {
                   let right = if xu < self.size - 1 {
                       heights[x + 1][y]
                   } else {
                       heights[self.size as usize - 1][y]
                   };

                   let left = if x > 0 {
                       heights[x - 1][y]
                   } else {
                       heights[0][y]
                   };

                   let top = if yu < self.size - 1 {
                       heights[x][y + 1]
                   } else {
                       heights[x][self.size as usize - 1]
                   };

                   let bottom = if y > 0 {
                       heights[x][y - 1]
                   } else {
                       heights[x][0]
                   };

                   math::normalize(&[right - left, top - bottom , 2.0])
                };

                let cosa = math::dot(&normal, &UP);
                let sin_alpha = cosa.acos().sin().max(0.5);

                // local sediment capacity of the flow
                let capacity = self.soil_capacity
                               * self.velocity[x][y].magnitude()
                               * sin_alpha
                               * (self.water[x][y].min(0.01) / 0.01);

                if capacity > self.sediment[x][y] {
                    let d = KS * (capacity - self.sediment[x][y]);
                    heights[x][y] -= d;
                    self.sediment[x][y] += d;
                }

                // deposit onto ground
                else {
                    let d = KD * (self.sediment[x][y] - capacity);
                    heights[x][y] += d;
                    self.sediment[x][y] -= d;
                }
        }
    }
}

