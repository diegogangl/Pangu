#![allow(dead_code)]

extern crate noise;
extern crate test;

use noise::{Constant, NoiseFn, Seedable};

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
}


impl Procedural {
    const DEFAULT_ROWS: u32 = 64;
    const DEFAULT_COLUMNS: u32 = 64;
    const DEFAULT_SEED: u32 = 0;
    const DEFAULT_OFFSET_X: f64 = 0.0;
    const DEFAULT_OFFSET_Y: f64 = 0.0;
    const DEFAULT_SIZE: f64 = 20.0;
    const DEFAULT_SCALE: f64 = 2.0;
    const DEFAULT_ROTATION: f64 = 0.0;

    pub fn new() -> Self {
        Procedural { rows: Self::DEFAULT_ROWS,
                     columns: Self::DEFAULT_COLUMNS,
                     offset_x: Self::DEFAULT_OFFSET_X,
                     offset_y: Self::DEFAULT_OFFSET_Y,
                     rotation: Self::DEFAULT_ROTATION,
                     size: Self::DEFAULT_SIZE,
                     scale: Self::DEFAULT_SCALE,
                     seed: Self::DEFAULT_SEED }
    }


    /// Sets the rows of the terrain grid.
    pub fn set_rows(mut self, rows: u32) -> Self {
        self.rows = rows;
        self
    }


    /// Sets the columns of the terrain grid.
    pub fn set_columns(mut self, columns: u32) -> Self {
        self.columns = columns;
        self
    }


    /// Sets the seed of the noise functions.
    pub fn set_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }


    /// Sets the offset for the X axis
    pub fn set_offset_x(mut self, offset_x: f64) -> Self {
        self.offset_x = offset_x;
        self
    }


    /// Sets the offset for the Y axis
    pub fn set_offset_y(mut self, offset_y: f64) -> Self {
        self.offset_y = offset_y;
        self
    }


    /// Sets the object size
    pub fn set_size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }


    /// Sets the real world scale of the terrain
    pub fn set_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }


    /// Sets the rotation angle
    pub fn set_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }


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
    fn vertices(&self, z: &NoiseFn<[f64; 2]>) -> Vertices {
        let half_x = f64::from(self.columns - 1) / 2.0;
        let half_y = f64::from(self.rows - 1) / 2.0;

        let capacity = (self.columns * self.rows) as usize;
        let mut verts: Vertices = Vec::with_capacity(capacity);

        let scale = f64::from(max(self.rows, self.columns)) * (1.0 / self.size);
        let steps = self.calculate_steps();

        for x in 0..self.columns {
            for y in 0..self.rows {
                let x = f64::from(x) - half_x;
                let y = f64::from(y) - half_y;

                let noise_coords = self.coords_for_noise(x, y, steps);

                verts.push((x / scale,
                            y / scale,
                            z.get([noise_coords.0, noise_coords.1])));
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
        let x2 = if self.rotation > 0.0 {
            let rotated = x * self.rotation.cos() - y * self.rotation.sin();
            steps.0 * (rotated + self.offset_x)
        } else {
            steps.0 * (x + self.offset_x)
        };

        let y2 = if self.rotation > 0.0 {
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
        LandFractal::new().set_seed(self.seed)
                          .set_z_scale(self.size / 10.0)
    }


    /// Build a plane mesh. This is a grid with Z coordinates
    /// set to zero. Only useful for testing and benching.
    pub fn build_plane(&self) -> (Faces, Vertices) {
        let z = Constant::new(0.0);
        (self.faces(), self.vertices(&z))
    }


    /// Build a terrain mesh.
    /// Returns a tuple of Faces and Vertices.
    pub fn build_mesh(self) -> (Faces, Vertices) {
        (self.faces(), self.vertices(&self.get_noise_fn()))
    }


    /// Build a terrain mesh.
    /// Returns a tuple of Faces and Vertices.
    pub fn build_vertices(self) -> Vertices {
        self.vertices(&self.get_noise_fn())
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
        let z = Constant::new(0.0);
        let terrain = Procedural::new().set_rows(4).set_columns(4).set_size(4.0);
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

        assert_eq!(expected, terrain.vertices(&z));

        let longer = terrain.set_rows(8);
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

        assert_eq!(expected, longer.vertices(&z));

        let taller = terrain.set_columns(8);
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

        assert_eq!(expected, taller.vertices(&z));
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
        let z = Constant::new(0.0);

        let terrain = Procedural::new().set_rows(128).set_columns(128);
        b.iter(|| terrain.vertices(&z));
    }


    #[bench]
    fn terrain(b: &mut Bencher) {
        let terrain = Procedural::new().set_rows(128).set_columns(128);
        b.iter(|| terrain.build_mesh());
    }
}
