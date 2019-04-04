#![feature(test)]

extern crate test;

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
    (0..columns - 1).flat_map(|x| {
                        (0..rows - 1).map(move |y| {
                                         (x * rows + y,
                                          (x + 1) * rows + y,
                                          (x + 1) * rows + 1 + y,
                                          x * rows + 1 + y)
                                     })
                    })
                    .collect::<Faces>()
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

    (0..columns).flat_map(|x| {
                    (0..rows).map(move |y| {
                                 (f64::from(x) - half_x, f64::from(y) - half_y, z(x, y))
                             })
                })
                .collect::<Vertices>()
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
        b.iter(|| {
             for _ in 1..100 {
                 grid_faces(128, 128);
             }
         });
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

        b.iter(|| {
             for _ in 1..100 {
                 grid_vertices(128, 128, &z);
             }
         });
    }

}
