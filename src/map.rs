#![allow(dead_code)]
#![allow(unused_macros)]

extern crate test;

use std::fmt;
use std::ops::{Index, IndexMut};

type Coords = (usize, usize);


/// Utility macro to generate a 2DMap quickly
///
/// # Arguments
///
/// * `x` - The rows to push into the map
macro_rules! map2D {
    () => (Map2D::new());
    ( $( $($x:expr),* );* ) => {{
            let mut tmp_vec2 = Map2D::new();

            $(
                tmp_vec2.push_row(vec![$( $x ),*]);
            )*

            tmp_vec2
        }};
}


/// Neighborhood types
///
/// Neighborhoods to use with iterators. They don't include
/// the center cell.
pub mod neighbors {

    // Moore Neighborhood
    pub const MOORE: [(isize, isize); 6] =
        [(-1, -1), (0, -1), (-1, 1), (1, 0), (0, 1), (1, 1)];

    // Rotated Von Neumann Neighborhood
    pub const VON_NEUMANN: [(isize, isize); 4] =
        [(-1, 1), (1, 1), (-1, -1), (1, -1)];
}


/// Represents a 2D vector for floating point values
///
/// This struct holds a 2D vector in row-major order, with
/// all rows stored contiguos in memory. X represents rows,
/// while Y represents columns.
///
/// ---  y=0    y=1   y=2     
/// x=0   1      2     3
/// x=1   4      5     6
/// x=2   7      8     9       
///
/// If the struct is constructed with `with_size` it will
/// be initialized with 0.0.
#[derive(Clone)]
pub struct Map2D<T> {
    // The vector containing the elements
    contents: Vec<T>,

    // Width of each row
    width: usize,
}


impl<T> Index<usize> for Map2D<T> 
where T: std::clone::Clone
{
    type Output = [T];

    /// Immutable index implementation
    ///
    /// This makes it possible to index the vector in 2D: map[x][y]
    ///
    /// # Arguments
    ///
    /// * `row` - The row to index
    ///
    /// # Panics
    ///
    /// Panics if the selected row is larger than the total rows.
    fn index(&self, row: usize) -> &[T] {
        assert!(row < self.height());
        let pos = row * self.width;

        &self.contents[pos..pos + self.width]
    }
}


impl<T> IndexMut<usize> for Map2D<T> 
where T: std::clone::Clone
{
    /// Mutable index implementation
    ///
    /// The mutable version of [the immutable index](Map2D::index)
    fn index_mut(&mut self, row: usize) -> &mut [T] {
        assert!(row < self.height());
        let pos = row * self.width;

        &mut self.contents[pos..pos + self.width]
    }
}


impl<T> fmt::Debug for Map2D<T> 
where T: std::fmt::Debug + std::clone::Clone {
    /// Debug format implementation
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\nWidth: {}\
             \nHeight: {}\
             \nTotal: {}\
             \nData: ",
            self.width,
            self.height(),
            self.contents.len()
        )?;

        for i in 0..self.height() {
            let row = &self[i];
            write!(f, "\n{:?}", row)?;
        }

        Ok(())
    }
}


impl<T> Map2D<T> 
where T: std::clone::Clone
{
    /// Return a new Map2D with no size or elements
    pub fn new() -> Self {
        Map2D {
            contents: Vec::new(),
            width: 0,
        }
    }


    /// Return a new Map2D with a size
    ///
    /// Vector contents are set to 0.0
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the 2D vector
    /// * `height` - Height of the 2D vector
    pub fn with_size(width: usize, height: usize, initial: T) -> Self {
        assert!(width > 0);
        Map2D {
            contents: vec![initial; width * height],
            width: width,
        }
    }


    /// Return the width
    pub fn width(&self) -> usize {
        self.width
    }


    /// Return the height
    pub fn height(&self) -> usize {
        self.contents.len() / self.width
    }


    /// Add an entire row to the map
    ///
    /// # Arguments
    ///
    /// * `row` - Vector of f64 to push
    ///
    /// # Panics
    ///
    /// Panics if the length of the row is different than
    /// the rest of the map.
    pub fn push_row(&mut self, row: Vec<T>) {
        if self.width > 0 {
            assert_eq!(row.len(), self.width);
        } else {
            self.width = row.len();
        }

        self.contents.extend(row);
    }


    /// Check if coordinates are inside the map
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - y coordinate
    pub fn is_inside(&self, x: usize, y: usize) -> bool {
        x > 0 && x < self.width - 1 && y > 0 && y < self.height() - 1
    }


    /// Iterate over map indices
    ///
    /// Returns X and Y coordinates
    pub fn iter_indices(&self) -> impl Iterator<Item = (usize, usize)> {
        let height = self.height();

        (0..self.contents.len()).map(move |i| (i / height, i % height))
    }


    /// Get neighbor coordinates without checks
    ///
    /// This function takes an origin oordinate and a direction.
    /// There are no checks for whether this is inside the grid, or
    /// if the arithmetic will overflow. Use with caution!
    ///
    /// # Arguments
    ///
    /// * `origin` - Tuple of X and Y coordinates
    /// * `step` - Tuple of directions in X and Y (eg. -1, 1)
    ///
    /// # Panics
    ///
    /// This will panic if the arithmetic overflows
    pub fn find(&self, origin: Coords, step: (isize, isize)) -> Coords {
        let target_x: usize = if step.0 < 0 {
            origin.0.wrapping_sub(step.0.abs() as usize)
        } else {
            origin.0.wrapping_add(step.0 as usize)
        };

        let target_y: usize = if step.1 < 0 {
            origin.1.wrapping_sub(step.1.abs() as usize)
        } else {
            origin.1.wrapping_add(step.1 as usize)
        };

        (target_x, target_y)
    }


    /// Get neighbor coordinates safely
    ///
    /// This function takes a set of coordinates and a direction.
    /// If the target coordinates are found inside the map, it returns
    /// the neighbor coordinates. Otherwise it returns
    /// `None`. It also returns `None` in case of overflows.
    ///
    /// # Arguments
    ///
    /// * `origin` - Tuple of X and Y coordinates
    /// * `step` - Tuple of directions in X and Y (eg. -1, 1)
    pub fn safe_find(
        &self,
        origin: Coords,
        step: (isize, isize),
    ) -> Option<Coords> {
        let target_x = if step.0 < 0 {
            origin.0.checked_sub(step.0.abs() as usize)
        } else {
            origin.0.checked_add(step.0 as usize)
        };


        let target_y = if step.1 < 0 {
            origin.1.checked_sub(step.1.abs() as usize)
        } else {
            origin.1.checked_add(step.1 as usize)
        };


        match (target_x, target_y) {
            (Some(x), Some(y)) => {
                if self.is_inside(x, y) {
                    Some((x, y))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_neighbor() {
        let test_map = map2D![0.0, 1.0, 2.0;
                              3.0, 4.0, 5.0;
                              6.0, 7.0, 8.0];

        let fail = test_map.safe_find((0, 0), (0, -1));
        assert!(fail.is_none());

        let fail2 = test_map.safe_find((0, 0), (-1, 0));
        assert!(fail2.is_none());

        let fail3 = test_map.safe_find((0, 0), (-1, -1));
        assert!(fail3.is_none());

        let in_grid = test_map.safe_find((1, 1), (1, -1));
        assert!(in_grid.is_some());
        assert_eq!((2, 0), in_grid.unwrap());
    }


    #[test]
    fn test_is_inside() {
        let test_map = map2D![0.0, 1.0, 2.0;
                              3.0, 4.0, 5.0;
                              6.0, 7.0, 8.0];

        assert!(test_map.is_inside(1, 2));
        assert_eq!(false, test_map.is_inside(5, 6));
    }


    #[test]
    fn test_iter_indices() {
        let test_map = map2D![0.0, 1.0, 2.0;
                              3.0, 4.0, 5.0;
                              6.0, 7.0, 8.0];

        for (x, y) in test_map.iter_indices() {
            test_map[x][y] = 1.0;
        }


        for (x, y) in test_map.iter_indices() {
            assert_eq!(1.0, test_map[x][y]);
        }
    }
}
