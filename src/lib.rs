#![feature(test)]
#![allow(dead_code)]

extern crate noise;
extern crate test;

use noise::{NoiseFn, Seedable, SuperSimplex};
use std::num::ParseIntError;

type Faces = Vec<(u32, u32, u32, u32)>;
type Vertices = Vec<(f64, f64, f64)>;


/// Representation of a terrain
#[derive(Clone, Copy, Debug, Default)]
struct Terrain {
    rows: u32,
    columns: u32,
    seed: u32,
}


impl Terrain {
    pub const DEFAULT_ROWS: u32 = 64;
    pub const DEFAULT_COLUMNS: u32 = 64;
    pub const DEFAULT_SEED: u32 = 0;

    pub fn new() -> Self {
        Terrain { rows: Self::DEFAULT_ROWS,
                  columns: Self::DEFAULT_COLUMNS,
                  seed: Self::DEFAULT_SEED }
    }


    /// Sets the rows of the terrain grid.
    pub fn set_rows(self, rows: u32) -> Self {
        Terrain { rows, ..self }
    }


    /// Sets the columns of the terrain grid.
    pub fn set_columns(self, columns: u32) -> Self {
        Terrain { columns, ..self }
    }


    /// Sets the seed of the noise functions.
    pub fn set_seed(self, seed: u32) -> Self {
        Terrain { seed, ..self }
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
    fn vertices(&self, z: &Fn(u32, u32) -> f64) -> Vertices {
        let half_x = f64::from(self.columns - 1) / 2.0;
        let half_y = f64::from(self.rows - 1) / 2.0;

        let capacity = (self.columns * self.rows) as usize;
        let mut verts: Vertices = Vec::with_capacity(capacity);

        for x in 0..self.columns {
            for y in 0..self.rows {
                verts.push((f64::from(x) - half_x, f64::from(y) - half_y, z(x, y)))
            }
        }

        verts
        }

}



#[cfg(test)]
mod tests {

    use super::*;
    use test::Bencher;

    #[test]
    fn test_faces() {
        let faces = Terrain::new().set_rows(4).set_columns(4).faces();

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


    #[bench]
    fn bench_faces(b: &mut Bencher) {
        let terrain = Terrain::new().set_rows(128).set_columns(128);
        b.iter(|| terrain.faces() );
    }

    #[test]
    fn test_vertices() {
        let z = |_, _| 0.0;
        let verts = Terrain::new().set_rows(4).set_columns(4).vertices(&z);

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


    #[bench]
    fn bench_verts(b: &mut Bencher) {
        let z = |_, _| 0.0;

        let terrain = Terrain::new().set_rows(128).set_columns(128);
        b.iter(|| terrain.vertices(&z));
    }
}
