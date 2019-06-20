#![allow(dead_code)]

/// Terrain generation core


extern crate noise;
extern crate test;

use noise::{NoiseFn, Perlin, Point2, Seedable};

use super::math;
use super::config;
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


/// Representation of a terrain
#[derive(Clone, Debug)]
pub struct Procedural {
    /// Configuration for the procedural terrain
    config: config::Terrain,

    /// Perlin noises for the main octaves (re-used for the others)
    noise_fns: Vec<Perlin>,

    /// Persistence values
    persistences: Vec<f64>,

    /// Steps to scale coordinates for the X and Y axis
    steps: (f64, f64),

    /// Upper bounds for noise coordinates. Used for seamless
    /// calculation. Lower bounds are always zero.
    limits_xy: (f64, f64),
}


impl Procedural {
    pub fn new(conf: config::Terrain) -> Self {
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

        // All done!
        Procedural {
            config: conf,
            noise_fns: noise_fns,
            persistences: persistences,
            limits_xy: limits_xy,
            steps: steps,
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
        let ceiling = conf.height;

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

                if self.config.terraces.enabled {
                    z = self.config.terraces.run(z);
                }

                if z < floor {
                    z = floor;
                }

                if floor > 0.0 {
                    z -= floor;
                }

                // Smooth Modifier
                if self.config.smooth.enabled {
                    z *= self.config.smooth.run(x, y);
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
        let config = config::Terrain {
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
        let config = config::Terrain {
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

        let config = config::Terrain {
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

        let config = config::Terrain {
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
        let config = config::Terrain {
            rows: 4,
            columns: 4,
            size: 4.0,
            ..Default::default()
        };

        let steps = Procedural::new(config).steps;
        assert_eq!((0.5, 0.5), steps);

        let config = config::Terrain {
            rows: 8,
            columns: 4,
            size: 4.0,
            ..Default::default()
        };

        let steps = Procedural::new(config).steps;
        assert_eq!((0.25, 0.25), steps);

        let config = config::Terrain {
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
        let config = config::Terrain {
            rows: 4,
            columns: 4,
            rotation: 0.0,
            ..Default::default()
        };
        let values = Procedural::new(config).coords_for_noise(1.0, 1.0);
        assert_eq!((0.5, 0.5), values);

        let config = config::Terrain {
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
        let config = config::Terrain {
            rows: 128,
            columns: 128,
            ..Default::default()
        };
        let terrain = Procedural::new(config);
        b.iter(|| terrain.faces());
    }


    #[bench]
    fn verts(b: &mut Bencher) {
        let config = config::Terrain {
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
        let config = config::Terrain::default();
        let terrain = Procedural::new(config);

        b.iter(|| terrain.get_z([0.0, 0.0]));
    }

    #[bench]
    fn terrain(b: &mut Bencher) {
        let config = config::Terrain {
            rows: 128,
            columns: 128,
            ..Default::default()
        };
        let terrain = Procedural::new(config);
        b.iter(|| terrain.build_mesh());
    }
}
