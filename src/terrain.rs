#![allow(dead_code)]

extern crate noise;
extern crate test;

use noise::{NoiseFn,  Seedable, Constant};

use super::land_fractal::LandFractal;
use std::cmp::max;

pub type Faces = Vec<(u32, u32, u32, u32)>;
pub type Vertices = Vec<(f64, f64, f64)>;


/// Representation of a terrain
#[derive(Clone, Copy, Debug, Default)]
pub struct Procedural {
    rows: u32,
    columns: u32,
    offset_x: f64,
    offset_y: f64,
    seed: u32,
    step_x: f64,
    step_y: f64,
    size: f64,
    scale: f64,
}


impl Procedural {
    const DEFAULT_ROWS: u32 = 64;
    const DEFAULT_COLUMNS: u32 = 64;
    const DEFAULT_SEED: u32 = 0;
    const DEFAULT_OFFSET_X: f64 = 0.0;
    const DEFAULT_OFFSET_Y: f64 = 0.0;
    const DEFAULT_STEP: f64 = 1.0;
    const DEFAULT_SIZE: f64 = 20.0;
    const DEFAULT_SCALE: f64 = 2.0;

    pub fn new() -> Self {
        Procedural { rows: Self::DEFAULT_ROWS,
                     columns: Self::DEFAULT_COLUMNS,
                     offset_x: Self::DEFAULT_OFFSET_X,
                     offset_y: Self::DEFAULT_OFFSET_Y,
                     step_x: Self::DEFAULT_STEP,
                     step_y: Self::DEFAULT_STEP,
                     size: Self::DEFAULT_SIZE,
                     scale: Self::DEFAULT_SCALE,
                     seed: Self::DEFAULT_SEED }
    }


    /// Sets the rows of the terrain grid.
    pub fn set_rows(self, rows: u32) -> Self {
        Procedural { rows, ..self }
    }


    /// Sets the columns of the terrain grid.
    pub fn set_columns(self, columns: u32) -> Self {
        Procedural { columns, ..self }
    }


    /// Sets the seed of the noise functions.
    pub fn set_seed(self, seed: u32) -> Self {
        Procedural { seed, ..self }
    }


    /// Sets the offset for the X axis
    pub fn set_offset_x(self, offset_x: f64) -> Self {
        Procedural { offset_x, ..self }
    }


    /// Sets the offset for the Y axis
    pub fn set_offset_y(self, offset_y: f64) -> Self {
        Procedural { offset_y, ..self }
    }


    /// Sets the object size
    pub fn set_size(self, size: f64) -> Self {
        Procedural { size, ..self }
    }


    /// Sets the real world scale of the terrain
    pub fn set_scale(self, scale: f64) -> Self {
        Procedural { scale, ..self }
    }


    /// Returns the faces of the terrain mesh as a vector of tuples
    /// containing four vertex indices.
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


    /// Returns the 3D coordinates for the terrain mesh as a vector
    /// of tuples.
    fn vertices(&self, z: &NoiseFn<[f64; 2]>) -> Vertices {
        let half_x = f64::from(self.columns - 1) / 2.0;
        let half_y = f64::from(self.rows - 1) / 2.0;

        let capacity = (self.columns * self.rows) as usize;
        let mut verts: Vertices = Vec::with_capacity(capacity);

        let scale = f64::from(max(self.rows, self.columns)) * (1.0 / self.size);

        for x in 0..self.columns {
            for y in 0..self.rows {
                let x = f64::from(x);
                let y = f64::from(y);

                let x_for_noise = self.step_x * (x + self.offset_x);
                let y_for_noise = self.step_y * (y + self.offset_y);

                verts.push(((x - half_x) / scale, (y - half_y) / scale,
                            z.get([x_for_noise, y_for_noise])));
            }
        }

        verts
    }


    /// Pre-calculate useful numbers for noise generation
    pub fn setup(self) -> Self {
        let columns = f64::from(self.columns);
        let rows = f64::from(self.rows);

        let ratio = columns / rows;

        let x_bounds = if columns > rows { self.scale } else { self.scale * ratio };
        let y_bounds = if columns > rows { self.scale / ratio } else { self.scale };

        let step_x = x_bounds / columns;
        let step_y = y_bounds / rows;

        Procedural { step_x, step_y, ..self }
    }


    /// Build and return a plane mesh. This is a grid with Z coordinates
    /// set to zero). Useful for testing and benching.
    pub fn build_plane(&self) -> (Faces, Vertices) {
        let z = Constant::new(0.0);
        (self.faces(), self.vertices(&z))
    }


    /// Build and return a terrain mesh. The return is a tuple of Faces
    /// and Vertices.
    pub fn build_mesh(&self) -> (Faces, Vertices) {

        let z_scale = self.size / 10.0;

        let noise = LandFractal::new().set_seed(self.seed).set_z_scale(z_scale);
        (self.faces(), self.vertices(&noise)
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn faces() {
        let faces = Procedural::new().set_rows(4).set_columns(4).faces();

        let expected = vec![(0, 4, 5, 1),
                            (1, 5, 6, 2),
                            (2, 6, 7, 3),
                            (4, 8, 9, 5),
                            (5, 9, 10, 6),
                            (6, 10, 11, 7),
                            (8, 12, 13, 9),
                            (9, 13, 14, 10),
                            (10, 14, 15, 11)];

        assert_eq!(faces, expected);
    }


    #[test]
    fn vertices() {
        let z = Constant::new(0.0);
        let verts = Procedural::new().set_rows(4).set_columns(4).set_size(4.0).vertices(&z);

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

        assert_eq!(verts, expected);
    }

}


mod benches {
    #[allow(unused_imports)]
    use test::Bencher;
    use super::*;


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
