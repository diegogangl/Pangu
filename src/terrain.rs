#![allow(dead_code)]

extern crate noise;
extern crate test;

use noise::{NoiseFn, Perlin, Point3, Seedable};

use super::utils::linear_interp;
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
/// * `fac - The scaling factor
/// * `warp - Domain warping value
macro_rules! scale {
    ($var:ident, $fac:expr) => {
        [$var[0] * $fac, $var[1] * $fac, $var[2] * $fac]
    };

    ($var:ident, $fac:expr, $warp:expr) => {
        [
            $var[0] * $fac + $warp,
            $var[1] * $fac + $warp,
            $var[2] * $fac,
        ]
    };
}


/// Macro casue ridgedness (sharp points) on heights
///
/// This macro creates an expresion to assign a value.
///
/// # Arguments
///
/// * `self` - A procedural instance
/// * `signal` - The signal from the noise function
/// * `level` - Level at which the ridgedness should be activated
macro_rules! ridge {
    ($self:ident, $signal:ident, $level:expr) => {
       if $self.config.ridgedness >= $level {
            $signal.abs() * -1.0
       } else {
            $signal
       }
    };
}


#[derive(Clone, Copy, Debug)]
pub struct ProceduralConfig {
    /// The number of rows to use in the mesh grid
    pub rows: u32,

    /// The number of columns to use in the mesh grid
    pub columns: u32,

    /// Offsets for the coordinates passed to the noise
    /// function
    pub offset_x: f64,
    pub offset_y: f64,
    pub offset_z: f64,

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
    pub ridgedness: u8,
}


impl Default for ProceduralConfig {
    fn default() -> Self {
        ProceduralConfig {
            rows: Self::DEFAULT_ROWS,
            columns: Self::DEFAULT_COLUMNS,
            offset_x: Self::DEFAULT_OFFSET,
            offset_y: Self::DEFAULT_OFFSET,
            offset_z: Self::DEFAULT_OFFSET,
            rotation: Self::DEFAULT_ROTATION,
            scale: Self::DEFAULT_SCALE,
            size: Self::DEFAULT_SIZE,
            seed: Self::DEFAULT_SEED,
            roughness: Self::DEFAULT_ROUGHNESS,
            plains: Self::DEFAULT_PLAINS,
            plateau: Self::DEFAULT_PLATEAU,
            deformation: Self::DEFAULT_DEFORMATION,
            mountainess: Self::DEFAULT_MOUNTAINESS,
            mix: 0.5,
            ridgedness: Self::DEFAULT_RIDGEDNESS,
            flat: false,
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
    pub const DEFAULT_RIDGEDNESS: u8 = 0;
}


/// Representation of a terrain
#[derive(Clone, Debug)]
pub struct Procedural {
    /// Configuration for the procedural terrain
    config: ProceduralConfig,

    /// Perlin noises for the main octaves (re-used for the others)
    noise_fns: Vec<Perlin>,

    /// Scale on Z (height) of the terrain
    z_scale: f64,

    /// Persistence values
    persistences: Vec<f64>,

    /// Steps to scale coordinates for the X and Y axis
    steps: (f64, f64),
}


impl Procedural {
    pub fn new(config: ProceduralConfig) -> Self {
        Procedural {
            config: config,
            noise_fns: Self::setup_noise_fns(config.seed),
            persistences: Self::setup_persistences(&config),
            z_scale: config.size / 20.0,
            steps: Self::calculate_steps(&config),
        }
    }


    /// Setup the Perlin noise functions for the octaves
    ///
    /// # Arguments
    ///
    /// * `seed` - The base seed for the noises
    ///
    /// Returns a vector of Perlin noise functions with
    /// different seeds
    fn setup_noise_fns(seed: u32) -> Vec<Perlin> {
        let mut noise_fns = Vec::with_capacity(6);

        for i in 0..6 {
            noise_fns.push(Perlin::new().set_seed(seed + i));
        }

        noise_fns
    }


    /// Setup persistence values
    ///
    /// # Arguments
    ///
    /// * `conf` - Procedural terrain configuration
    fn setup_persistences(conf: &ProceduralConfig) -> Vec<f64> {
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


        persistences
    }


    /// Generate list of faces for the terrain mesh
    ///
    /// Returns the a vector of tuples containing the indices
    /// for the four vertices of each face.
    fn faces(&self) -> Faces {
        let conf = self.config;

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
    /// # Arguments
    ///
    /// * `z` - Noise function to generate the terrain
    ///
    /// Returns the 3D coordinates for the mesh as a vector
    /// of tuples.
    fn vertices(&self) -> Vertices {
        let conf = self.config;

        let half_x = f64::from(conf.columns - 1) / 2.0;
        let half_y = f64::from(conf.rows - 1) / 2.0;

        let capacity = (conf.columns * conf.rows) as usize;
        let mut verts: Vertices = Vec::with_capacity(capacity);

        let scale = f64::from(max(conf.rows, conf.columns)) * (1.0 / conf.size);

        for x in 0..conf.columns {
            for y in 0..conf.rows {
                let x = f64::from(x) - half_x;
                let y = f64::from(y) - half_y;

                let co = self.coords_for_noise(x, y);
                let z = if conf.flat {
                    0.0
                } else {
                    let val = self.get_z([co.0, co.1, conf.offset_z]);
                    if val > conf.plateau {
                        conf.plateau
                    } else {
                        val
                    }
                };

                verts.push((x / scale, y / scale, z));
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
    /// * `steps`: Steps to scale the coordinates for X and Y
    fn coords_for_noise(&self, x: f64, y: f64) -> (f64, f64) {
        let conf = self.config;

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


    /// Calculate correct boundaries for the noise and the steps
    /// to make coordinates fit in the bounds. Boundaries are
    /// calculated from the ratio between rows and columns as
    /// well as the scale field.
    /// Returns a tuple with the X and Y steps.
    fn calculate_steps(conf: &ProceduralConfig) -> (f64, f64) {
        let columns = f64::from(conf.columns);
        let rows = f64::from(conf.rows);

        let ratio = columns / rows;

        let x_bounds = if columns > rows {
            conf.scale
        } else {
            conf.scale * ratio
        };


        let y_bounds = if columns > rows {
            conf.scale / ratio
        } else {
            conf.scale
        };

        (x_bounds / columns, y_bounds / rows)
    }


    /// Get noise value
    ///
    /// # Arguments
    /// * `point` - The coordinates in 3D space for the noise
    fn get_z(&self, point: Point3<f64>) -> f64 {
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
        result = ridge!(self, signal, 5);


        //---------------------------------------------------------------------
        // Large features of the terrain

        let octave1_scale = 1.5;
        current_point = scale!(point, octave1_scale, domain);

        let mut signal = self.noise_fns[1].get(current_point);
        signal = ((signal + (signal.abs() + self.config.plains)) / divisor)
                   * self.persistences[1];

        result += ridge!(self, signal, 4);


        //---------------------------------------------------------------------
        // Larger details

        let octave2_scale = 2.0;
        current_point = scale!(current_point, octave2_scale, domain);

        let mut signal = self.noise_fns[2].get(current_point);
        signal = ((signal + (signal.abs() + self.config.plains)) / divisor)
                   * self.persistences[2];

        result += ridge!(self, signal, 3);

        //---------------------------------------------------------------------
        // Medium details

        let octave3_scale = 2.0;
        current_point = scale!(current_point, octave3_scale, domain);

        let mut signal = self.noise_fns[3].get(current_point);
        signal = ((signal + (signal.abs() + self.config.plains)) / divisor)
                   * self.persistences[3];

        result += ridge!(self, signal, 2);


        //---------------------------------------------------------------------
        // Small details

        let octave4_scale = 1.2;

        current_point = scale!(current_point, octave4_scale, domain);
        let signal = self.noise_fns[4].get(current_point) * self.persistences[4];
        result += ridge!(self, signal, 1);


        //---------------------------------------------------------------------
        // Fine details
        let octave5_scale = 1.4;

        current_point = scale!(current_point, octave5_scale, domain);
        result += self.noise_fns[5].get(current_point) * self.persistences[5];


        //---------------------------------------------------------------------
        // BLEND NOISE
        //---------------------------------------------------------------------

        //---------------------------------------------------------------------
        // Basic shape of the terrain

        let signal = self.noise_fns[3].get(point) * self.persistences[6];
        blend = ridge!(self, signal, 4);


        //---------------------------------------------------------------------
        // Extra-details

        let blend1_scale = 2.0;

        current_point = scale!(point, blend1_scale, domain);
        let signal = self.noise_fns[1].get(current_point) * self.persistences[7];
        blend += ridge!(self, signal, 5);

        // Make sure there are no holes in the ground when using a high
        // plains setting
        mask += self.config.plains;

        if self.config.ridgedness > 3 {
            mask *= -1.0;
        }

        linear_interp(result, blend, mask) * self.z_scale
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
        assert_eq!((0.5, 0.5), Procedural::calculate_steps(&config));

        let config = ProceduralConfig {
            rows: 8,
            columns: 4,
            size: 4.0,
            ..Default::default()
        };
        assert_eq!((0.25, 0.25), Procedural::calculate_steps(&config));

        let config = ProceduralConfig {
            rows: 4,
            columns: 8,
            size: 4.0,
            ..Default::default()
        };
        assert_eq!((0.25, 0.25), Procedural::calculate_steps(&config));
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

        b.iter(|| terrain.get_z([0.0, 0.0, 0.0]));
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
