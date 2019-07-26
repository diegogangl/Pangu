#![allow(dead_code)]
#![allow(unused_macros)]

extern crate test;

use std::ops::{Index, IndexMut};
use std::fmt;

#[derive(Clone)]
pub struct Map2D {
    contents: Vec<f64>,
    width: usize
}

impl Index<usize> for Map2D  {
    type Output = [f64];
    fn index(&self, row: usize) -> &[f64] {
        assert!(row < self.height());
        let pos = row * self.width;

        &self.contents[pos..pos + self.width]
    }
}


impl IndexMut<usize> for Map2D {
    fn index_mut(&mut self, row: usize) -> &mut [f64] {
        assert!(row < self.height());
        let pos = row * self.width;

        &mut self.contents[pos..pos + self.width]
    }
}


impl fmt::Debug for Map2D {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\nWidth: {}\
                   \nHeight: {}\
                   \nTotal: {}\
                   \nData: ",
                   self.width, self.height(),
                   self.contents.len())?;

        for i in 0..self.height() {
            let row = &self[i];
            write!(f, "\n{:?}", row)?;
        }

        Ok(())
    }

}


impl Map2D {
    pub fn new() -> Self {
        Map2D { contents: Vec::new(), width: 0 }
    }

    pub fn with_size(width: usize, height: usize) -> Self {
        assert!(width > 0);
        Map2D { contents: vec![0.0; width * height], 
                width: width }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.contents.len() / self.width
    }

    pub fn push_row(&mut self, row: Vec<f64>) {
        if self.width > 0 {
            assert_eq!(row.len(), self.width);
        } else {
            self.width = row.len();
        }

        self.contents.extend(row);
    }

    pub fn pop_row(&mut self) {
        for _ in 0..self.width {
            self.contents.pop();
        }
    }

    pub fn is_inside(&self, x: usize, y: usize) -> bool {
        x > 0 && x < self.width - 1 && y > 0 && y < self.height() - 1
    }

    pub fn iter_indices(&self) -> impl Iterator<Item = (usize, usize)> {
        let height = self.height();
    
        (0..self.contents.len())
            .map(move |i| (i / height, i % height))
    }

    pub fn neighbor(&self, src: (usize, usize), dir: (isize, isize)) -> Option<(f64, usize, usize)> {

        let target_x = if dir.0 < 0 {
                src.0.checked_sub(dir.0.abs() as usize)
            } else {
                src.0.checked_add(dir.0 as usize)
            };

        let target_y = if dir.1 < 0 {
                src.1.checked_sub(dir.1.abs() as usize)
            } else {
                src.1.checked_add(dir.1 as usize)
            };

        match (target_x, target_y) {
            (Some(x), Some(y)) => { 
                if self.is_inside(x, y) {
                    Some((self.index(x)[y], x, y)) 
                } else {
                    None
                }
            },
            _ => None
        }
    }
}


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



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_neighbor() {
        let test_map = map2D![0.0, 1.0, 2.0;
                              3.0, 4.0, 5.0;
                              6.0, 7.0, 8.0];

        let fail = test_map.neighbor((0, 0), (0, -1));
        assert!(fail.is_none());

        let fail2 = test_map.neighbor((0, 0), (-1, 0));
        assert!(fail2.is_none());

        let fail3 = test_map.neighbor((0, 0), (-1, -1));
        assert!(fail3.is_none());

        let in_grid = test_map.neighbor((1, 1), (1, -1));
        assert!(in_grid.is_some());

        println!("{:?}", test_map);

        for (x,y) in test_map.iter_indices() {
            let value = test_map[x][y];
            println!("x: {}, y: {}, value: {:?}", x, y, value);
        }

    }
}


mod benches {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;


    #[bench]
    fn neighbor(b: &mut Bencher) {
        let test_map = map2D![0.0, 1.0, 2.0;
                              3.0, 4.0, 5.0;
                              6.0, 7.0, 8.0];

        b.iter(|| test_map.neighbor((1,1), (-1, 1)));
    }

}
