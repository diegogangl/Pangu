#![allow(dead_code)]

/// Terrain generation core


extern crate noise;
extern crate test;

use noise::{NoiseFn, Perlin, Point2, Seedable};

use super::math;
use super::curve::Curve;
use std::cmp::max;

pub type Faces = Vec<(u32, u32, u32, u32)>;
pub type Vertices = Vec<(f64, f64, f64)>;


/// Macro to scale a point
///
/// This macro can also apply Domain Warping
/// on the X and Y axis.
///
/// # Arguments
///
/// * `var` - The Point3 variable.
/// * `fac` - The scaling factor
/// * `warp` - Domain warping value
macro_rules! scale {
    ($var:ident, $fac:expr) => {
        [$var[0] * $fac, $var[1] * $fac]
    };

    ($var:ident, $fac:expr, $warp:expr) => {
        [
            $var[0] * $fac + $warp,
            $var[1] * $fac + $warp
        ]
    };
}


/// Macro to casue ridgedness (sharpness) on heights
///
/// This macro creates an expresion to assign a value.
///
/// # Arguments
///
/// * `self` - A procedural instance
/// * `signal` - The signal from the noise function
/// * `level` - Level at which the ridgedness should be activated
macro_rules! ridge {
    ($self:ident, $signal:ident) => {
        math::lerp($signal + ($signal.abs() * -1.0),
             $signal,
             $self.config.ridgedness)
    };
}

macro_rules! mountainess {
    ($self:ident, $signal:ident, $persistence:expr, $divisor:ident) => {
        (($signal + ($signal.abs() + $self.config.plains)) / $divisor)
            * $self.persistences[$persistence]
    };
}


#[derive(Clone, Debug)]
pub struct ProceduralConfig {
    /// The number of rows to use in the mesh grid
    pub rows: u32,

    /// The number of columns to use in the mesh grid
    pub columns: u32,

    /// Offsets for the coordinates passed to the noise
    /// function
    pub offset_x: f64,
    pub offset_y: f64,

    /// Z Rotation angle (in radians) for the noise
    pub rotation: f64,

    /// Scale for the noise function. Larger scales create
    /// smaller, more detailed noise while smaller values
    /// create larger, less detailed terrains.
    pub scale: f64,

    /// Size of the mesh object in scene units
    pub size: f64,

    /// Base seed for the noise function
    pub seed: u32,

    /// Make grid flat. Used for testing
    pub flat: bool,

    /// Roughness for the terrain
    pub roughness: f64,

    /// How plain the base terrain is
    pub plains: f64,

    /// At which height to clamp to generate a plateau
    pub plateau: f64,

    /// Mountainess
    pub mountainess: f64,

    /// Intensity of domain warping
    pub deformation: f64,

    /// Mixing between plains and mountains
    pub mix: f64,

    /// Ridgedness
    pub ridgedness: f64,

    /// Sea Floor
    pub sea_floor: f64,

    /// Maximum Height
    pub height: f64,

    /// Make the terrain seamless
    pub is_seamless: bool,

    /// Invert the terrain
    pub invert: bool,

    /// Use terraces
    pub terraces: bool,

    /// Invert Terraces
    pub terraces_invert: bool,

    /// Invert Terraces
    pub terraces_points: Vec<f64>,

    // Smooth out terrain
    pub smooth: bool,
    pub smooth_radial: bool,
    pub smooth_radial_fac: f64,
    pub smooth_radial_size: (f64, f64),
    pub smooth_linear_fac: (f64, f64),
    pub smooth_linear_start: (f64, f64),
    pub smooth_linear_invert: (bool, bool),
}


impl Default for ProceduralConfig {
    fn default() -> Self {
        ProceduralConfig {
            rows: Self::DEFAULT_ROWS,
            columns: Self::DEFAULT_COLUMNS,
            offset_x: Self::DEFAULT_OFFSET,
            offset_y: Self::DEFAULT_OFFSET,
            rotation: Self::DEFAULT_ROTATION,
            scale: Self::DEFAULT_SCALE,
            size: Self::DEFAULT_SIZE,
            seed: Self::DEFAULT_SEED,
            roughness: Self::DEFAULT_ROUGHNESS,
            plains: Self::DEFAULT_PLAINS,
            plateau: Self::DEFAULT_PLATEAU,
            deformation: Self::DEFAULT_DEFORMATION,
            mountainess: Self::DEFAULT_MOUNTAINESS,
            mix: Self::DEFAULT_MIX,
            ridgedness: Self::DEFAULT_RIDGEDNESS,
            sea_floor: Self::DEFAULT_SEA_FLOOR,
            height: Self::DEFAULT_HEIGHT,
            flat: false,
            is_seamless: Self::DEFAULT_SEAMLESS,
            invert: Self::DEFAULT_INVERT,
            terraces: Self::DEFAULT_TERRACES,
            terraces_invert: Self::DEFAULT_TERRACES_INVERT,
            terraces_points: Vec::new(),
            smooth: Self::DEFAULT_SMOOTH,
            smooth_radial: Self::DEFAULT_SMOOTH_RADIAL,
            smooth_radial_fac: Self::DEFAULT_SMOOTH_RADIAL_FAC,
            smooth_radial_size: Self::DEFAULT_SMOOTH_RADIAL_SIZE,
            smooth_linear_fac: Self::DEFAULT_SMOOTH_LINEAR_FAC,
            smooth_linear_start: Self::DEFAULT_SMOOTH_LINEAR_FAC,
            smooth_linear_invert: Self::DEFAULT_SMOOTH_LINEAR_INVERT,
        }
    }
}


impl ProceduralConfig {
    pub const DEFAULT_ROWS: u32 = 64;
    pub const DEFAULT_COLUMNS: u32 = 64;
    pub const DEFAULT_OFFSET: f64 = 0.0;
    pub const DEFAULT_ROTATION: f64 = 0.0;
    pub const DEFAULT_SCALE: f64 = 2.0;
    pub const DEFAULT_SIZE: f64 = 5.0;
    pub const DEFAULT_SEED: u32 = 0;
    pub const DEFAULT_ROUGHNESS: f64 = 0.1;
    pub const DEFAULT_PLAINS: f64 = 0.5;
    pub const DEFAULT_PLATEAU: f64 = 10.0;
    pub const DEFAULT_DEFORMATION: f64 = 0.1;
    pub const DEFAULT_MOUNTAINESS: f64 = 0.5;
    pub const DEFAULT_MIX: f64 = 0.5;
    pub const DEFAULT_RIDGEDNESS: f64 = 0.0;
    pub const DEFAULT_SEA_FLOOR: f64 = 0.0;
    pub const DEFAULT_HEIGHT: f64 = 3.0;
    pub const DEFAULT_SEAMLESS: bool = true;
    pub const DEFAULT_INVERT: bool = false;
    pub const DEFAULT_TERRACES: bool = false;
    pub const DEFAULT_TERRACES_INVERT: bool = true;
    pub const DEFAULT_SMOOTH: bool = false;
    pub const DEFAULT_SMOOTH_RADIAL: bool = true;
    pub const DEFAULT_SMOOTH_RADIAL_FAC: f64 = 0.0;
    pub const DEFAULT_SMOOTH_RADIAL_SIZE: (f64, f64) = (0.0, 0.0);
    pub const DEFAULT_SMOOTH_LINEAR_FAC: (f64, f64) = (0.0, 0.0);
    pub const DEFAULT_SMOOTH_LINEAR_START: (f64, f64) = (0.0, 0.0);
    pub const DEFAULT_SMOOTH_LINEAR_INVERT: (bool, bool) = (true, false);
}


/// Representation of a terrain
#[derive(Clone, Debug)]
pub struct Procedural {
    /// Configuration for the procedural terrain
    config: ProceduralConfig,

    /// Perlin noises for the main octaves (re-used for the others)
    noise_fns: Vec<Perlin>,

    /// Persistence values
    persistences: Vec<f64>,

    /// Steps to scale coordinates for the X and Y axis
    steps: (f64, f64),

    /// Upper bounds for noise coordinates. Used for seamless
    /// calculation. Lower bounds are always zero.
    limits_xy: (f64, f64),

    /// Curve for terrace effect
    terrace_curve: Curve,
}


impl Procedural {
    pub fn new(conf: ProceduralConfig) -> Self {
        let columns = f64::from(conf.columns);
        let rows = f64::from(conf.rows);

        // Calculate correct boundaries for the noise. Boundaries are
        // calculated fromt he ratio between rows and columns as well as
        // the scale setting.
        let limit_x = if columns > rows {
            conf.scale
        } else {
            conf.scale * (columns / rows)
        };


        let limit_y = if columns > rows {
            conf.scale / (columns / rows)
        } else {
            conf.scale
        };

        debug!("Bound limits are x: {:?}, y: {:?}", limit_x, limit_y);
        let limits_xy = (limit_x, limit_y);


        // Calculate noise coordinates steps. These are used to fit
        // coordinates inside the boundaries.
        let steps = (limit_x / columns, limit_y / rows);
        debug!("Calculated steps are: {:?}", steps);


        // Setup Perlin noise functions for the octaves. Each octave
        // has a different seed based on the seed setting.
        let mut noise_fns = Vec::with_capacity(6);

        for i in 0..6 {
            noise_fns.push(Perlin::new().set_seed(conf.seed + i));
        }


        // Setup persistence values for the octaves. These are used
        // in the main noise function.
        let base = 1.0 - conf.plains;

        let persistences = vec![
            base,
            conf.mountainess,
            base * conf.mountainess,
            // Final octaves
            conf.roughness / 2.0,
            conf.roughness / 4.0,
            conf.roughness / 8.0,
            // Blend terrain
            base.powi(2),
            base / 2.0,
        ];

        debug!("Calculated persistences: {:?}", persistences);


        let terrace_curve = if conf.terraces {
            debug!("Adding control points for terrace");

            let mut curve = Curve::new();

            for p in &conf.terraces_points {
               let point = math::percent_to_value(*p, conf.height);
               curve.add_point(point);
            }

            curve

        } else {
            Curve::new()
        };


        // All done!
        Procedural {
            config: conf,
            noise_fns: noise_fns,
            persistences: persistences,
            limits_xy: limits_xy,
            steps: steps,
            terrace_curve: terrace_curve,
        }
    }


    /// Generate list of faces for the terrain mesh
    ///
    /// Returns the a vector of tuples containing the indices
    /// for the four vertices of each face.
    fn faces(&self) -> Faces {
        let conf = &self.config;

        let capacity = (conf.columns * conf.rows) as usize;
        let mut faces: Faces = Vec::with_capacity(capacity);

        for x in 0..conf.columns - 1 {
            for y in 0..conf.rows - 1 {
                faces.push((
                    x * conf.rows + y,
                    (x + 1) * conf.rows + y,
                    (x + 1) * conf.rows + 1 + y,
                    x * conf.rows + 1 + y,
                ))
            }
        }

        faces
    }


    /// Generate list of vertices for the terrain mesh
    ///
    /// Returns the 3D coordinates for the mesh as a vector
    /// of tuples.
    fn vertices(&self) -> Vertices {
        let conf = &self.config;

        let half_x = f64::from(conf.columns - 1) / 2.0;
        let half_y = f64::from(conf.rows - 1) / 2.0;

        let capacity = (conf.columns * conf.rows) as usize;
        let mut verts: Vertices = Vec::with_capacity(capacity);

        debug!("Allocated vec with capacity: {:?}", capacity);

        let scale = f64::from(max(conf.rows, conf.columns)) * (1.0 / conf.size);
        debug!("Scale: {:?}", scale);

        let mut heights_min = 0.0;
        let mut heights_max = 1.0;

        let floor = conf.sea_floor * conf.height;
        let ceiling = conf.plateau * conf.height;

        debug!("Calculated floor: {:?}", floor);
        debug!("Calculated ceiling: {:?}", ceiling);

        // Convenience for seamless calculations
        let x_extent = self.limits_xy.0;
        let y_extent = self.limits_xy.1;

        for x in 0..conf.columns {
            for y in 0..conf.rows {
                let x = f64::from(x) - half_x;
                let y = f64::from(y) - half_y;

                let co = self.coords_for_noise(x, y);
                let mut z = if conf.flat {
                    0.0

                // Make seamless
                } else if conf.is_seamless {
                    let sw = self.get_z([co.0, co.1]);
                    let se = self.get_z([co.0 + x_extent, co.1]);
                    let nw = self.get_z([co.0, co.1 + y_extent]);
                    let ne = self.get_z([co.0 + x_extent, co.1 + y_extent]);

                    let x_blend = 1.0 - ((co.0 + 1.0) / x_extent);
                    let y_blend = 1.0 - ((co.1 + 1.0) / y_extent);

                    let y0 = math::lerp(se, sw, x_blend);
                    let y1 = math::lerp(ne, nw, x_blend);

                    let val = math::lerp(y1, y0, y_blend);

                    // Keep track of min/max for normalization
                    if val > heights_max {
                        heights_max = val;
                    }

                    if val < heights_min {
                        heights_min = val;
                    }

                    val
                } else {
                    let val = self.get_z([co.0, co.1]);

                    // Keep track of min/max for normalization
                    if val > heights_max {
                        heights_max = val;
                    }

                    if val < heights_min {
                        heights_min = val;
                    }

                    val
                };

                if conf.invert {
                    z *= -1.0;
                }

                verts.push((x / scale, y / scale, z));
            }
        }

        // Normalization
        for x in 0..conf.columns {
            for y in 0..conf.rows {
                let i = (y * conf.columns + x) as usize;
                let mut z = verts[i].2;

                z = math::map_on_zero(
                    z,
                    heights_min,
                    heights_max,
                    self.config.height,
                );

                if self.config.terraces {
                    z = self.terrace(z);
                }

                // Restrict to plateau and sea floor
                if z > ceiling {
                    z = ceiling;
                }

                if z < floor {
                    z = floor;
                }

                if floor > 0.0 {
                    z -= floor;
                }

                // Smooth Modifier
                if self.config.smooth {
                    z *= if self.config.smooth_radial {
                            self.smooth_radial(x, y)
                         } else {
                            self.smooth_linear(x, y)
                         };
                }

                verts[i] = (verts[i].0, verts[i].1, z);
            }
        }

        verts
    }


    /// Adjust X, Y coordinates for the noise function
    ///
    /// Takes care of rotating, offsetting and scaling the
    /// coordinates to the noise bounds.
    ///
    /// # Arguments
    ///
    /// * `x`: Value for X axis
    /// * `y`: Value for y axis
    fn coords_for_noise(&self, x: f64, y: f64) -> (f64, f64) {
        let conf = &self.config;

        let x2 = if conf.rotation != 0.0 {
            let rotated = x * conf.rotation.cos() - y * conf.rotation.sin();
            self.steps.0 * (rotated + conf.offset_x)
        } else {
            self.steps.0 * (x + conf.offset_x)
        };

        let y2 = if conf.rotation != 0.0 {
            let rotated = x * conf.rotation.sin() + y * conf.rotation.cos();
            self.steps.1 * (rotated + conf.offset_y)
        } else {
            self.steps.1 * (y + conf.offset_y)
        };

        (x2, y2)
    }


    /// Get noise value
    ///
    /// # Arguments
    /// * `point` - The coordinates in 3D space for the noise
    fn get_z(&self, point: Point2<f64>) -> f64 {
        let mut result;
        let mut domain;
        let mut blend;
        let mut current_point;
        let divisor = 1.0 + self.config.plains;


        //---------------------------------------------------------------------
        // BLEND MASK
        //---------------------------------------------------------------------
        let mask_control = self.config.mix;
        current_point = scale!(point, mask_control);

        let mut mask = self.noise_fns[0].get(current_point);


        //---------------------------------------------------------------------
        // DOMAIN WARPING
        //---------------------------------------------------------------------

        let domain_scale = 1.5;

        current_point = scale!(point, domain_scale);
        domain = self.noise_fns[1].get(current_point);

        current_point = scale!(current_point, domain_scale);
        domain += self.noise_fns[1].get(current_point) * 0.5;

        current_point = scale!(current_point, domain_scale);
        domain += self.noise_fns[2].get(current_point) * 0.25;

        domain *= self.config.deformation;


        //---------------------------------------------------------------------
        // BASE FRACTAL NOISE
        //---------------------------------------------------------------------

        //---------------------------------------------------------------------
        // Basic shape of the terrain

        let signal = self.noise_fns[0].get(point) * self.persistences[0];
        result = ridge!(self, signal);


        //---------------------------------------------------------------------
        // Large features of the terrain

        current_point = scale!(point, 1.5, domain);

        result += {
            let mut signal = self.noise_fns[1].get(current_point);
            signal = mountainess!(self, signal, 1, divisor);
            ridge!(self, signal)
        };


        //---------------------------------------------------------------------
        // Larger details

        current_point = scale!(current_point, 2.0, domain);

        result += {
            let mut signal = self.noise_fns[2].get(current_point);
            signal = mountainess!(self, signal, 2, divisor);
            ridge!(self, signal)
        };

        //---------------------------------------------------------------------
        // Medium details

        current_point = scale!(current_point, 2.0, domain);

        result += {
            let signal = self.noise_fns[3].get(current_point);
            mountainess!(self, signal, 3, divisor) * result
        };


        //---------------------------------------------------------------------
        // Small details

        current_point = scale!(current_point, 1.2, domain);

        result += self.noise_fns[4].get(current_point)
                  * self.persistences[4]
                  * result;


        //---------------------------------------------------------------------
        // Fine details

        current_point = scale!(current_point, 1.4, domain);
        result += self.noise_fns[5].get(current_point)
                  * self.persistences[5]
                  * result;


        //---------------------------------------------------------------------
        // BLEND NOISE
        //---------------------------------------------------------------------

        //---------------------------------------------------------------------
        // Basic shape of the terrain

        let signal = self.noise_fns[3].get(point) * self.persistences[6];
        blend = ridge!(self, signal);


        //---------------------------------------------------------------------
        // Extra-details

        current_point = scale!(point, 2.0, domain);

        blend += {
            let signal =
                self.noise_fns[1].get(current_point) * self.persistences[7];
            ridge!(self, signal)
        };

        // Make sure there are no holes in the ground when using a high
        // plains setting
        mask += self.config.plains;
        math::lerp(result, blend, mask)
    }


    /// Create a circular smooth effect
    ///
    /// Only works in square terrains for now
    ///
    /// # Arguments
    ///
    /// * `x` - X index of the current height point
    /// * `y` - Y index of the current height point
    fn smooth_radial(&self, x: u32, y: u32) -> f64 {

        if self.config.columns != self.config.rows {
            return 1.0
        }

        let cols = f64::from(self.config.columns);
        let rows = f64::from(self.config.rows);

        let center_x = cols / 2.0;
        let center_y = rows / 2.0;

        let max_dist = math::distance(center_x, center_y,
                                cols - self.config.smooth_radial_size.0,
                                rows - self.config.smooth_radial_size.1);

        let dist = math::distance(center_x, center_y, x as f64, y as f64);
        let normalized = (dist / max_dist).min(1.0);

        // Normalized with a power of <1 creates pointy terrains
        math::lerp(0.0, 1.0, normalized.powf(self.config.smooth_radial_fac - normalized))
    }


    /// Create a smooth effect on one or two axis
    ///
    /// # Arguments
    ///
    /// * `x` - X index of the current height point
    /// * `y` - Y index of the current height point
    fn smooth_linear(&self, x: u32, y: u32) -> f64 {
        let mut multiplier = 1.0;

        if self.config.smooth_linear_fac.0 > 0.0 {
            let cols = f64::from(self.config.columns);

            let fac = if self.config.smooth_linear_invert.0 {
               (cols - (x as f64 + self.config.smooth_linear_start.0)) / cols
            } else {
               ((cols + (x as f64 - self.config.smooth_linear_start.0)) / cols) - 1.0
            };

            if fac > 0.0 {
                multiplier = fac.powf(self.config.smooth_linear_fac.0);
            } else {
                multiplier = 0.0;
            };
        };

        if self.config.smooth_linear_fac.1 > 0.0 {
            let rows = f64::from(self.config.rows);

            let fac = if self.config.smooth_linear_invert.1 {
               (rows - (y as f64 + self.config.smooth_linear_start.1)) / rows
            } else {
               ((rows + (y as f64 - self.config.smooth_linear_start.1)) / rows) - 1.0
            };

            if fac > 0.0 {
                multiplier *= fac.powf(self.config.smooth_linear_fac.1);
            } else {
                multiplier *= 0.0;
            };
        };

        multiplier
    }


    /// Create a terrace effect
    ///
    /// # Arguments
    /// * `value - A height value from the terrain
    fn terrace(&self, value: f64) -> f64 {
        // Get indices of the nearest two points
        let indexes = self.terrace_curve.points_near(value);

        // If some control points are missing get the output value
        // of the nearest control point and return. This can
        // happen when value < lowest_point or value > highest_point
        if indexes.0 == indexes.1 {
            return self.terrace_curve.point(indexes.1);
        }

        // Get values and calculate alpha parameter for lerping
        let mut input_0 = self.terrace_curve.point(indexes.0);
        let mut input_1 = self.terrace_curve.point(indexes.1);
        let mut alpha = (value - input_0) / (input_1 - input_0);

        if self.config.terraces_invert {
            alpha = 1.0 - alpha;
            std::mem::swap(&mut input_0, &mut input_1);
        }

        math::lerp(input_1, input_0, alpha.powi(2))
    }


    /// Build a terrain mesh.
    /// Returns a tuple of Faces and Vertices.
    pub fn build_mesh(&self) -> (Faces, Vertices) {
        (self.faces(), self.vertices())
    }


    /// Build a terrain mesh.
    /// Returns a tuple of Faces and Vertices.
    pub fn build_vertices(&self) -> Vertices {
        self.vertices()
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn faces() {
        let config = ProceduralConfig {
            rows: 4,
            columns: 4,
            ..Default::default()
        };
        let faces = Procedural::new(config).faces();

        let expected = vec![
            (0, 4, 5, 1),
            (1, 5, 6, 2),
            (2, 6, 7, 3),
            (4, 8, 9, 5),
            (5, 9, 10, 6),
            (6, 10, 11, 7),
            (8, 12, 13, 9),
            (9, 13, 14, 10),
            (10, 14, 15, 11),
        ];

        assert_eq!(expected, faces);
    }


    #[test]
    fn vertices() {
        let config = ProceduralConfig {
            rows: 4,
            columns: 4,
            size: 4.0,
            flat: true,
            ..Default::default()
        };
        let verts = Procedural::new(config).vertices();

        let expected = vec![
            (-1.5, -1.5, 0.0),
            (-1.5, -0.5, 0.0),
            (-1.5, 0.5, 0.0),
            (-1.5, 1.5, 0.0),
            (-0.5, -1.5, 0.0),
            (-0.5, -0.5, 0.0),
            (-0.5, 0.5, 0.0),
            (-0.5, 1.5, 0.0),
            (0.5, -1.5, 0.0),
            (0.5, -0.5, 0.0),
            (0.5, 0.5, 0.0),
            (0.5, 1.5, 0.0),
            (1.5, -1.5, 0.0),
            (1.5, -0.5, 0.0),
            (1.5, 0.5, 0.0),
            (1.5, 1.5, 0.0),
        ];

        assert_eq!(expected, verts);

        let config = ProceduralConfig {
            rows: 8,
            columns: 4,
            size: 4.0,
            flat: true,
            ..Default::default()
        };
        let verts = Procedural::new(config).vertices();

        let expected = vec![
            (-0.75, -1.75, 0.0),
            (-0.75, -1.25, 0.0),
            (-0.75, -0.75, 0.0),
            (-0.75, -0.25, 0.0),
            (-0.75, 0.25, 0.0),
            (-0.75, 0.75, 0.0),
            (-0.75, 1.25, 0.0),
            (-0.75, 1.75, 0.0),
            (-0.25, -1.75, 0.0),
            (-0.25, -1.25, 0.0),
            (-0.25, -0.75, 0.0),
            (-0.25, -0.25, 0.0),
            (-0.25, 0.25, 0.0),
            (-0.25, 0.75, 0.0),
            (-0.25, 1.25, 0.0),
            (-0.25, 1.75, 0.0),
            (0.25, -1.75, 0.0),
            (0.25, -1.25, 0.0),
            (0.25, -0.75, 0.0),
            (0.25, -0.25, 0.0),
            (0.25, 0.25, 0.0),
            (0.25, 0.75, 0.0),
            (0.25, 1.25, 0.0),
            (0.25, 1.75, 0.0),
            (0.75, -1.75, 0.0),
            (0.75, -1.25, 0.0),
            (0.75, -0.75, 0.0),
            (0.75, -0.25, 0.0),
            (0.75, 0.25, 0.0),
            (0.75, 0.75, 0.0),
            (0.75, 1.25, 0.0),
            (0.75, 1.75, 0.0),
        ];

        assert_eq!(expected, verts);

        let config = ProceduralConfig {
            rows: 4,
            columns: 8,
            size: 4.0,
            flat: true,
            ..Default::default()
        };

        let verts = Procedural::new(config).vertices();

        let expected = vec![
            (-1.75, -0.75, 0.0),
            (-1.75, -0.25, 0.0),
            (-1.75, 0.25, 0.0),
            (-1.75, 0.75, 0.0),
            (-1.25, -0.75, 0.0),
            (-1.25, -0.25, 0.0),
            (-1.25, 0.25, 0.0),
            (-1.25, 0.75, 0.0),
            (-0.75, -0.75, 0.0),
            (-0.75, -0.25, 0.0),
            (-0.75, 0.25, 0.0),
            (-0.75, 0.75, 0.0),
            (-0.25, -0.75, 0.0),
            (-0.25, -0.25, 0.0),
            (-0.25, 0.25, 0.0),
            (-0.25, 0.75, 0.0),
            (0.25, -0.75, 0.0),
            (0.25, -0.25, 0.0),
            (0.25, 0.25, 0.0),
            (0.25, 0.75, 0.0),
            (0.75, -0.75, 0.0),
            (0.75, -0.25, 0.0),
            (0.75, 0.25, 0.0),
            (0.75, 0.75, 0.0),
            (1.25, -0.75, 0.0),
            (1.25, -0.25, 0.0),
            (1.25, 0.25, 0.0),
            (1.25, 0.75, 0.0),
            (1.75, -0.75, 0.0),
            (1.75, -0.25, 0.0),
            (1.75, 0.25, 0.0),
            (1.75, 0.75, 0.0),
        ];

        assert_eq!(expected, verts);
    }


    #[test]
    fn steps_calculation() {
        let config = ProceduralConfig {
            rows: 4,
            columns: 4,
            size: 4.0,
            ..Default::default()
        };

        let steps = Procedural::new(config).steps;
        assert_eq!((0.5, 0.5), steps);

        let config = ProceduralConfig {
            rows: 8,
            columns: 4,
            size: 4.0,
            ..Default::default()
        };

        let steps = Procedural::new(config).steps;
        assert_eq!((0.25, 0.25), steps);

        let config = ProceduralConfig {
            rows: 4,
            columns: 8,
            size: 4.0,
            ..Default::default()
        };

        let steps = Procedural::new(config).steps;
        assert_eq!((0.25, 0.25), steps);
    }


    #[test]
    fn rotation() {
        let config = ProceduralConfig {
            rows: 4,
            columns: 4,
            rotation: 0.0,
            ..Default::default()
        };
        let values = Procedural::new(config).coords_for_noise(1.0, 1.0);
        assert_eq!((0.5, 0.5), values);

        let config = ProceduralConfig {
            rows: 4,
            columns: 4,
            rotation: 1.0,
            ..Default::default()
        };
        let values = Procedural::new(config).coords_for_noise(1.0, 1.0);

        assert!(values.0.fract() - (1505.0) < 1e-10);
        assert!(values.1.fract() - (69088.0) < 1e-10);
    }
}


mod benches {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;


    #[bench]
    fn faces(b: &mut Bencher) {
        let config = ProceduralConfig {
            rows: 128,
            columns: 128,
            ..Default::default()
        };
        let terrain = Procedural::new(config);
        b.iter(|| terrain.faces());
    }


    #[bench]
    fn verts(b: &mut Bencher) {
        let config = ProceduralConfig {
            rows: 128,
            columns: 128,
            flat: true,
            ..Default::default()
        };

        let terrain = Procedural::new(config);

        b.iter(|| terrain.vertices());
    }

    #[bench]
    fn get_noise(b: &mut Bencher) {
        let config = ProceduralConfig::default();
        let terrain = Procedural::new(config);

        b.iter(|| terrain.get_z([0.0, 0.0]));
    }

    #[bench]
    fn terrain(b: &mut Bencher) {
        let config = ProceduralConfig {
            rows: 128,
            columns: 128,
            ..Default::default()
        };
        let terrain = Procedural::new(config);
        b.iter(|| terrain.build_mesh());
    }
}
