#![feature(test)]

extern crate test;
extern crate noise;

use noise::{SuperSimplex, Seedable, NoiseFn};
use std::num::ParseIntError;

type Faces = Vec<(u32, u32, u32, u32)>;
type Vertices = Vec<(f64, f64, f64)>;


/// Returns a vector of tuples containing indices for vertices
///
/// # Arguments
///
/// * `columns` - Columns for the grid
/// * `rows - Rows for the grid
///
fn grid_faces(columns: u32, rows: u32) -> Faces {
    let mut faces: Faces = Vec::with_capacity((columns * rows) as usize);

    for x in 0..columns - 1 {
        for y in 0..rows - 1 {
            faces.push((x * rows + y, (x + 1) * rows + y, (x + 1) * rows + 1 + y, x * rows + 1 + y))
        }
    }

    faces
}


/// Returns a vector of tuples containing coordinates for vertices
///
/// # Arguments
///
/// * `columns` - Columns for the grid
/// * `rows - Rows for the grid
/// * `z - Function to generate Z values
///
fn grid_vertices(columns: u32, rows: u32, z: &Fn(u32, u32) -> f64) -> Vertices {
    let half_x = f64::from(columns - 1) / 2.0;
    let half_y = f64::from(rows - 1) / 2.0;
    let mut verts: Vertices = Vec::with_capacity((columns * rows) as usize);

    for x in 0..columns {
        for y in 0..rows {
            verts.push((f64::from(x) - half_x, f64::from(y) - half_y, z(x, y)))
        }
    }

    verts
}


fn get_z(x: u32, y:u32, source: &NoiseFn<[f64; 3]>) -> f64 {
    let bound_low =  0.0;
    let bound_high =  1.0;
    let width = 128.0;
    let height = 128.0;

    let x_step = (bound_high - bound_low) / width;
    let y_step = (bound_high - bound_low) / height;

    let current_x = (bound_low + x_step) * x as f64;
    let current_y = (bound_low + y_step) * y as f64;

    source.get([current_x, current_y, 0.0])
}


fn terrain() -> Result<(Faces, Vertices), ParseIntError> {

    let rows = 128;
    let columns = 128;
    let seed = 1;
    let simplex_base = SuperSimplex::new().set_seed(seed);

    let z = |x, y| get_z(x, y, &simplex_base);

    let faces = grid_faces(columns, rows);
    let verts = grid_vertices(columns, rows, &z);

    println!("{:?}", verts);

    Ok((faces, verts))
}


#[cfg(test)]
mod tests {

    use super::*;
    use test::Bencher;

    #[test]
    fn test_faces() {
        let faces = grid_faces(4, 4);
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
        b.iter(|| grid_faces(128, 128));
    }


    #[test]
    fn test_vertices() {
        let z = |_, _| 0.0;
        let verts = grid_vertices(4, 4, &z);

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

        b.iter(|| grid_vertices(128, 128, &z));
     }


    #[bench]
    fn bench_terrain(b: &mut Bencher) {
        b.iter(|| terrain());
     }

}
