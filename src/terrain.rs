#![allow(dead_code)]

extern crate noise;
extern crate test;

use noise::NoiseFn;

use super::land_fractal::LandFractal;
use std::cmp::max;

pub type Faces = Vec<(u32, u32, u32, u32)>;
pub type Vertices = Vec<(f64, f64, f64)>;


/// Representation of a terrain
#[derive(Clone, Copy, Debug, Default)]
pub struct Procedural {
    /// The number of rows to use in the mesh grid
    rows: u32,

    /// The number of columns to use in the mesh grid
    columns: u32,

    /// Offsets for the coordinates passed to the noise
    /// function
    offset_x: f64,
    offset_y: f64,
    offset_z: f64,

    /// Z Rotation angle (in radians) for the noise
    rotation: f64,

    /// Scale for the noise function. Larger scales create
    /// smaller, more detailed noise while smaller values
    /// create larger, less detailed terrains.
    scale: f64,

    /// Size of the mesh object in scene units
    size: f64,

    /// Base seed for the noise function
    seed: u32,

    /// Make grid flat. Used for testing
    flat: bool,

    /// Roughness for the terrain
    roughness: f64,
}


impl Procedural {
    const DEFAULT_ROWS: u32 = 64;
    const DEFAULT_COLUMNS: u32 = 64;
    const DEFAULT_SEED: u32 = 0;
    const DEFAULT_OFFSET: f64 = 0.0;
    const DEFAULT_SIZE: f64 = 20.0;
    const DEFAULT_SCALE: f64 = 2.0;
    const DEFAULT_ROTATION: f64 = 0.0;
    const DEFAULT_FLAT: bool = false;
    const DEFAULT_ROUGHNESS: f64 = 0.5;

    pub fn new() -> Self {
        Procedural { rows: Self::DEFAULT_ROWS,
                     columns: Self::DEFAULT_COLUMNS,
                     offset_x: Self::DEFAULT_OFFSET,
                     offset_y: Self::DEFAULT_OFFSET,
                     offset_z: Self::DEFAULT_OFFSET,
                     rotation: Self::DEFAULT_ROTATION,
                     size: Self::DEFAULT_SIZE,
                     scale: Self::DEFAULT_SCALE,
                     seed: Self::DEFAULT_SEED,
                     flat: Self::DEFAULT_FLAT,
                     roughness: Self::DEFAULT_ROUGHNESS, }
    }


    setter!(set_rows, rows, u32);
    setter!(set_columns, columns, u32);
    setter!(set_seed, seed, u32);
    setter!(set_offset_x, offset_x, f64);
    setter!(set_offset_y, offset_y, f64);
    setter!(set_offset_z, offset_z, f64);
    setter!(set_size, size, f64);
    setter!(set_scale, scale, f64);
    setter!(set_rotation, rotation, f64);
    setter!(set_flat, flat, bool);
    setter!(set_roughness, roughness, f64);


    /// Generate list of faces for the terrain mesh
    ///
    /// Returns the a vector of tuples containing the indices
    /// for the four vertices of each face.
    fn faces(&self) -> Faces {
        let capacity = (self.columns * self.rows) as usize;
        let mut faces: Faces = Vec::with_capacity(capacity);

        for x in 0..self.columns - 1 {
            for y in 0..self.rows - 1 {
                faces.push((x * self.rows + y,
                            (x + 1) * self.rows + y,
                            (x + 1) * self.rows + 1 + y,
                            x * self.rows + 1 + y))
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
        let half_x = f64::from(self.columns - 1) / 2.0;
        let half_y = f64::from(self.rows - 1) / 2.0;

        let capacity = (self.columns * self.rows) as usize;
        let mut verts: Vertices = Vec::with_capacity(capacity);

        let scale = f64::from(max(self.rows, self.columns)) * (1.0 / self.size);
        let steps = self.calculate_steps();
        let z_fn = self.get_noise_fn();

        for x in 0..self.columns {
            for y in 0..self.rows {
                let x = f64::from(x) - half_x;
                let y = f64::from(y) - half_y;

                let noise_coords = self.coords_for_noise(x, y, steps);
                let z = if self.flat {
                    0.0
                } else {
                    z_fn.get([noise_coords.0, noise_coords.1, self.offset_z])
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
    fn coords_for_noise(self, x: f64, y: f64, steps: (f64, f64)) -> (f64, f64) {
        let x2 = if self.rotation != 0.0 {
            let rotated = x * self.rotation.cos() - y * self.rotation.sin();
            steps.0 * (rotated + self.offset_x)
        } else {
            steps.0 * (x + self.offset_x)
        };

        let y2 = if self.rotation != 0.0 {
            let rotated = x * self.rotation.sin() + y * self.rotation.cos();
            steps.1 * (rotated + self.offset_y)
        } else {
            steps.1 * (y + self.offset_y)
        };

        (x2, y2)
    }


    /// Calculate correct boundaries for the noise and the steps
    /// to make coordinates fit in the bounds. Boundaries are
    /// calculated from the ratio between rows and columns as
    /// well as the scale field.
    /// Returns a tuple with the X and Y steps.
    fn calculate_steps(self) -> (f64, f64) {
        let columns = f64::from(self.columns);
        let rows = f64::from(self.rows);

        let ratio = columns / rows;

        let x_bounds = if columns > rows {
            self.scale
        } else {
            self.scale * ratio
        };


        let y_bounds = if columns > rows {
            self.scale / ratio
        } else {
            self.scale
        };

        (x_bounds / columns, y_bounds / rows)
    }


    /// Get the noise function with the right settings
    fn get_noise_fn(self) -> LandFractal {
        LandFractal::new(self.seed).set_roughness(self.roughness)
                                   .set_z_scale(self.size / 20.0)
    }

    /// Build a terrain mesh.
    /// Returns a tuple of Faces and Vertices.
    pub fn build_mesh(self) -> (Faces, Vertices) {
        (self.faces(), self.vertices())
    }


    /// Build a terrain mesh.
    /// Returns a tuple of Faces and Vertices.
    pub fn build_vertices(self) -> Vertices {
        self.vertices()
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn faces() {
        let terrain = Procedural::new().set_rows(4).set_columns(4);

        let expected = vec![(0, 4, 5, 1),
                            (1, 5, 6, 2),
                            (2, 6, 7, 3),
                            (4, 8, 9, 5),
                            (5, 9, 10, 6),
                            (6, 10, 11, 7),
                            (8, 12, 13, 9),
                            (9, 13, 14, 10),
                            (10, 14, 15, 11)];

        assert_eq!(expected, terrain.faces());
    }


    #[test]
    fn vertices() {
        let terrain = Procedural::new().set_rows(4)
                                       .set_columns(4)
                                       .set_size(4.0)
                                       .set_flat(true);

        let expected = vec![(-1.5, -1.5, 0.0),
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
                            (1.5, 1.5, 0.0)];

        assert_eq!(expected, terrain.vertices());

        let longer = terrain.set_rows(8).set_flat(true);

        let expected = vec![(-0.75, -1.75, 0.0),
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
                            (0.75, 1.75, 0.0)];

        assert_eq!(expected, longer.vertices());

        let taller = terrain.set_columns(8).set_flat(true);

        let expected = vec![(-1.75, -0.75, 0.0),
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
                            (1.75, 0.75, 0.0)];

        assert_eq!(expected, taller.vertices());
    }


    #[test]
    fn steps_calculation() {
        let terrain = Procedural::new().set_rows(4).set_columns(4).set_size(4.0);
        assert_eq!((0.5, 0.5), terrain.calculate_steps());

        let longer = terrain.set_rows(8);
        assert_eq!((0.25, 0.25), longer.calculate_steps());

        let taller = terrain.set_columns(8);
        assert_eq!((0.25, 0.25), taller.calculate_steps());
    }


    #[test]
    fn rotation() {
        let terrain = Procedural::new().set_rows(4)
                                       .set_columns(4)
                                       .set_rotation(0.0);

        let values = terrain.coords_for_noise(1.0, 1.0, terrain.calculate_steps());
        assert_eq!((0.5, 0.5), values);

        let terrain = Procedural::new().set_rows(4)
                                       .set_columns(4)
                                       .set_rotation(1.0);

        let values = terrain.coords_for_noise(1.0, 1.0, terrain.calculate_steps());

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
        let terrain = Procedural::new().set_rows(128).set_columns(128);
        b.iter(|| terrain.faces());
    }


    #[bench]
    fn verts(b: &mut Bencher) {
        let terrain = Procedural::new().set_rows(128)
                                       .set_columns(128)
                                       .set_flat(true);

        b.iter(|| terrain.vertices());
    }


    #[bench]
    fn terrain(b: &mut Bencher) {
        let terrain = Procedural::new().set_rows(128).set_columns(128);
        b.iter(|| terrain.build_mesh());
    }
}
