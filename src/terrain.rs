#![allow(dead_code)]

use pyo3::prelude::*;
use pyo3::class::*;
use pyo3::types::PyDict;

/// Terrain generation core
extern crate noise;
extern crate test;

use super::map::Map2D;
use super::types;
use super::modifiers as mods;
use std::cmp::max;

pub type Faces = Vec<(u32, u32, u32, u32)>;
pub type Vertices = Vec<(f64, f64, f64)>;
pub type Heightmap = Vec<f64>;


/// Macro to extract values from a Python dictionary as
/// rust types.
///
/// # Arguments
///
/// * `params` - The parameters dictionary
/// * `key` - The key to look for in the dictionary
macro_rules! get {
    ($params:expr, $key:expr) => {
        match $params.get_item($key) {
            Some(v) => v.extract()?,
            None => panic!("Missing key {}!", $key),
        }
    };
}


/// Representation of a terrain
#[pyclass]
pub struct Terrain {

    // -------------------------------------------------------------------------
    // PUBLIC SETTINGS
    // -------------------------------------------------------------------------

    /// The number of rows to use in the mesh grid
    #[pyo3(get, set)]
    rows: u32,

    /// The number of columns to use in the mesh grid
    #[pyo3(get, set)]
    columns: u32,

    /// Base seed for the noise function
    #[pyo3(get, set)]
    seed: u32,

    /// Scale for the noise function. Larger scales create
    /// smaller, more detailed noise while smaller values
    /// create larger, less detailed terrains.
    #[pyo3(get, set)]
    realworld_scale: f64,

    /// Size of the mesh object in scene units
    #[pyo3(get, set)]
    size: f64,

    /// Maximum Height
    #[pyo3(get, set)]
    height: f64,

    /// Offsets for the coordinates passed to the noise
    /// function
    #[pyo3(get, set)]
    offset: (f64, f64),

    /// Z Rotation angle (in radians) for the noise
    #[pyo3(get, set)]
    rotation: f64,

    /// Make grid flat. Used for testing
    pub flat: bool,


    // -------------------------------------------------------------------------
    // PRIVATE PROPERTIES
    // -------------------------------------------------------------------------

    /// If rows != columns, then the terrain will have
    /// to be cut to this size. Otherwise it's value
    /// is zero.
    to_cut: (u32, u32),

    /// Steps to scale coordinates for the X and Y axis
    steps: (f64, f64),

    /// Upper bounds for noise coordinates. Used for seamless
    /// calculation. Lower bounds are always zero.
    limits_xy: (f64, f64),

    /// Generated height map
    hmap: Map2D<f64>,

    /// Terrain type. This is passed as an int from Python,
    /// and transformed into an enum value internally.
    terrain_type: Box<dyn types::TerrainType>,

    modifiers: Vec<Box<dyn mods::Modifier>>,
}


/// Implement public functions that can be called from Python
#[pymethods]
impl Terrain {

    /// Constructor
    #[new]
    fn new() -> Self {
        Self {
            rows: 64,
            columns: 64,
            to_cut: (0, 0),
            seed: 0,
            realworld_scale: 2.0,
            size: 5.0,
            height: 5.0,
            offset: (0.0, 0.0),
            rotation: 0.0,
            flat: false,
            steps: (0.0, 0.0),
            limits_xy: (0.0, 0.0),
            hmap: Map2D::new(),
            terrain_type: Box::new(types::SmoothHills::default()),
            modifiers: vec![],
        }
    }


    /// Set the terrain type and its settings for this terrain
    ///
    /// # Arguments
    ///
    /// * `params`: Dictionary with type of terrain and settings
    ///
    #[setter(terrain_type)]
    fn set_terrain_type(&mut self, params: &PyDict) -> PyResult<()> {

        self.terrain_type = match get!(params, "type") {
            0 => Box::new(types::Basic {
                    breakup: get!(params, "breakup"),
                    roughness: get!(params, "roughness"),
                    ..Default::default()
            }),

            1 => Box::new(types::SmoothHills {
                    difference: get!(params, "difference"),
                    flat: get!(params, "flat"),
                    detail: get!(params, "detail"),
                    twist: get!(params, "twist"),
                    ..Default::default()
            }),

            2 => Box::new(types::Mountains {
                    ridgedness: get!(params, "ridgedness"),
                    sharpness: get!(params, "sharpness"),
                    breakup: get!(params, "breakup"),
                    roughness: get!(params, "roughness"),
                    twist: get!(params, "twist"),
                    ..Default::default()
            }),

            _ => Box::new(types::Basic::default()),

        };

        self.terrain_type.set_seed(self.seed);
        Ok(())
    }


    /// Push a new modifier into the modifiers vector
    ///
    /// # Arguments
    ///
    /// * `params`: Dictionary with type of terrain and settings
    ///
    fn add_modifier(&mut self, params: &PyDict) -> PyResult<()> {

        debug!("Pushing modifier with params: {:?}", params);

        self.modifiers.push(match get!(params, "type") {
            "THERMAL" => Box::new(mods::ThermalErosion::new(params)?),
            "INVERT" => Box::new(mods::Invert::new(params)?),
            "SMOOTH" => Box::new(mods::Smooth::new(params)?),
            "SEAMLESS" => Box::new(mods::Seamless::new(params)?),

            _ => Box::new(mods::Empty::new(params)?),
        });

        Ok(())
    }


    /// Build a terrain mesh.
    /// Returns a tuple of Faces and Vertices.
    pub fn build_mesh(&mut self) -> (Faces, Vertices) {
        self.setup();
        self.hmap = self.heights();

        (self.faces(), self.vertices())
    }
}


/// Implement Python's magic functions
#[pyproto]
impl PyObjectProtocol for Terrain {
    /// Implementation for Python's __repr__
    ///
    /// This shows up when printing the terrain object
    fn __repr__(&self) -> PyResult<String> {
        Ok(
            format!("Terrain with seed: {}", self.seed)
        )
    }
}


/// Implement private functions
impl Terrain {

    /// Setup internal variables for the terrain.
    ///
    /// This must be called before generating a terrain for the first time,
    /// and after all the properties have been set.
    fn setup(&mut self) {
        let columns = f64::from(self.columns);
        let rows = f64::from(self.rows);

        // Calculate correct boundaries for the noise. Boundaries are
        // calculated fromt he ratio between rows and columns as well as
        // the scale setting.
        let limit_x = if columns > rows {
            self.realworld_scale
        } else {
            self.realworld_scale * (columns / rows)
        };


        let limit_y = if columns > rows {
            self.realworld_scale / (columns / rows)
        } else {
            self.realworld_scale
        };

        debug!("Bound limits are x: {:?}, y: {:?}", limit_x, limit_y);
        self.limits_xy = (limit_x, limit_y);


        // Calculate noise coordinates steps. These are used to fit
        // coordinates inside the boundaries.
        self.steps = (limit_x / columns, limit_y / rows);
        debug!("Calculated steps are: {:?}", self.steps);
    }


    /// Generate list of faces for the terrain mesh
    ///
    /// Returns the a vector of tuples containing the indices
    /// for the four vertices of each face.
    fn faces(&self) -> Faces {
        let columns = if self.to_cut.1 > 0 {
            self.to_cut.1 - 1
        } else {
            self.columns - 1
        };

        let rows = if self.to_cut.0 > 0 {
            self.to_cut.0 - 1
        } else {
            self.rows - 1
        };


        let multiplier = if self.to_cut.0 > 0 {
            self.to_cut.0
        } else {
            self.rows
        };

        let capacity = (columns * rows) as usize;
        let mut faces: Faces = Vec::with_capacity(capacity);

        for x in 0..columns {
            for y in 0..rows {
                faces.push((
                        x * multiplier + y,
                        (x + 1) * multiplier + y,
                        (x + 1) * multiplier + 1 + y,
                        x * multiplier + 1 + y,
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

        let columns = if self.to_cut.1 > 0 {
            self.to_cut.1
        } else {
            self.columns
        };

        let rows = if self.to_cut.0 > 0 {
            self.to_cut.0
        } else {
            self.rows
        };

        let capacity = (columns * rows) as usize;
        let mut verts: Vertices = Vec::with_capacity(capacity);

        debug!("Allocated vertices with capacity: {:?}", capacity);

        // Used to scale the mesh
        let scale = max(rows, columns) as f64
            * (1.0 / self.size);

        debug!("Scale: {:?}", scale);

        // Used to center the mesh in the scene
        let half_x = ((rows - 1) as f64) / 2.0;
        let half_y = ((columns - 1) as f64) / 2.0;

        for y in 0..columns as usize {
            for x in 0..rows as usize {
                let scaled_x = ((x as f64) - half_x) / scale;
                let scaled_y = ((y as f64) - half_y) / scale;

                verts.push((scaled_x, scaled_y, self.hmap[x][y]));
            }
        }

        verts
    }


    /// Generate the heightmap for the terrain
    ///
    /// Returns a flat Vector with values in the range [0..1]
    fn heights(&mut self) -> Map2D<f64> {

        // Convenience
        let columns = self.columns;
        let rows = self.rows;

        // Keep track of height range for normalization
        let mut heights_min = 0.0;
        let mut heights_max = 1.0;

        // Allocation
        let capacity = (columns * rows) as usize;
        let mut hmap = Map2D::with_size(columns as usize, rows as usize, 0.0);

        debug!("Allocated heightmap with capacity: {:?}", capacity);
        debug!("Allocated heightmap with width: {:?} ", hmap.width());
        debug!("Allocated heightmap with height: {:?}", hmap.height());

        // Initial Generation
        for (x, y) in hmap.iter_indices() {
            let co = self.coords_for_noise(x as f64, y as f64);
            let z = self.terrain_type.height_at(co);

            // Keep track of min/max for normalization
            if z > heights_max {
                heights_max = z;
            }

            if z < heights_min {
                heights_min = z;
            }

            hmap[x][y] = z
        }

        // Run all modifiers
        for modifier in &self.modifiers {
            modifier.run(&mut hmap);
        }

        hmap
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
    fn coords_for_noise(&self, x: f64, y: f64) -> [f64; 2] {

        let x2 = if self.rotation != 0.0 {
            let rotated = x * self.rotation.cos() - y * self.rotation.sin();
            self.steps.0 * (rotated + self.offset.0)
        } else {
            self.steps.0 * (x + self.offset.0)
        };

        let y2 = if self.rotation != 0.0 {
            let rotated = x * self.rotation.sin() + y * self.rotation.cos();
            self.steps.1 * (rotated + self.offset.1)
        } else {
            self.steps.1 * (y + self.offset.1)
        };

        [x2, y2]
    }
}


// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn faces() {
//         let mut terrain = Terrain::new();
//         terrain.rows = 4;
//         terrain.columns = 4;

//         let expected = vec![
//             (0, 4, 5, 1),
//             (1, 5, 6, 2),
//             (2, 6, 7, 3),
//             (4, 8, 9, 5),
//             (5, 9, 10, 6),
//             (6, 10, 11, 7),
//             (8, 12, 13, 9),
//             (9, 13, 14, 10),
//             (10, 14, 15, 11),
//         ];

//         assert_eq!(expected, terrain.faces());
//     }


    // #[test]
    // fn vertices() {
    //     let config = config::Terrain {
    //         rows: 4,
    //         columns: 4,
    //         size: 4.0,
    //         flat: true,
    //         ..Default::default()
    //     };
    //     let verts = Procedural::new(config).vertices();


    //     let expected = vec![
    //         (-1.5, -1.5, 0.0),
    //         (-0.5, -1.5, 0.0),
    //         (0.5, -1.5, 0.0),
    //         (1.5, -1.5, 0.0),
    //         (-1.5, -0.5, 0.0),
    //         (-0.5, -0.5, 0.0),
    //         (0.5, -0.5, 0.0),
    //         (1.5, -0.5, 0.0),
    //         (-1.5, 0.5, 0.0),
    //         (-0.5, 0.5, 0.0),
    //         (0.5, 0.5, 0.0),
    //         (1.5, 0.5, 0.0),
    //         (-1.5, 1.5, 0.0),
    //         (-0.5, 1.5, 0.0),
    //         (0.5, 1.5, 0.0),
    //         (1.5, 1.5, 0.0),
    //     ];

    //     assert_eq!(expected, verts);

    //     let config = config::Terrain {
    //         rows: 8,
    //         columns: 4,
    //         size: 4.0,
    //         flat: true,
    //         ..Default::default()
    //     };
    //     let verts = Procedural::new(config).vertices();

    //     let expected = vec![
    //         (-1.75, -0.75, 0.0),
    //         (-1.25, -0.75, 0.0),
    //         (-0.75, -0.75, 0.0),
    //         (-0.25, -0.75, 0.0),
    //         (0.25, -0.75, 0.0),
    //         (0.75, -0.75, 0.0),
    //         (1.25, -0.75, 0.0),
    //         (1.75, -0.75, 0.0),
    //         (-1.75, -0.25, 0.0),
    //         (-1.25, -0.25, 0.0),
    //         (-0.75, -0.25, 0.0),
    //         (-0.25, -0.25, 0.0),
    //         (0.25, -0.25, 0.0),
    //         (0.75, -0.25, 0.0),
    //         (1.25, -0.25, 0.0),
    //         (1.75, -0.25, 0.0),
    //         (-1.75, 0.25, 0.0),
    //         (-1.25, 0.25, 0.0),
    //         (-0.75, 0.25, 0.0),
    //         (-0.25, 0.25, 0.0),
    //         (0.25, 0.25, 0.0),
    //         (0.75, 0.25, 0.0),
    //         (1.25, 0.25, 0.0),
    //         (1.75, 0.25, 0.0),
    //         (-1.75, 0.75, 0.0),
    //         (-1.25, 0.75, 0.0),
    //         (-0.75, 0.75, 0.0),
    //         (-0.25, 0.75, 0.0),
    //         (0.25, 0.75, 0.0),
    //         (0.75, 0.75, 0.0),
    //         (1.25, 0.75, 0.0),
    //         (1.75, 0.75, 0.0),
    //     ];


    //     assert_eq!(expected, verts);

    //     let config = config::Terrain {
    //         rows: 4,
    //         columns: 8,
    //         size: 4.0,
    //         flat: true,
    //         ..Default::default()
    //     };

    //     let verts = Procedural::new(config).vertices();

    //     let expected = vec![
    //         (-0.75, -1.75, 0.0),
    //         (-0.25, -1.75, 0.0),
    //         (0.25, -1.75, 0.0),
    //         (0.75, -1.75, 0.0),
    //         (-0.75, -1.25, 0.0),
    //         (-0.25, -1.25, 0.0),
    //         (0.25, -1.25, 0.0),
    //         (0.75, -1.25, 0.0),
    //         (-0.75, -0.75, 0.0),
    //         (-0.25, -0.75, 0.0),
    //         (0.25, -0.75, 0.0),
    //         (0.75, -0.75, 0.0),
    //         (-0.75, -0.25, 0.0),
    //         (-0.25, -0.25, 0.0),
    //         (0.25, -0.25, 0.0),
    //         (0.75, -0.25, 0.0),
    //         (-0.75, 0.25, 0.0),
    //         (-0.25, 0.25, 0.0),
    //         (0.25, 0.25, 0.0),
    //         (0.75, 0.25, 0.0),
    //         (-0.75, 0.75, 0.0),
    //         (-0.25, 0.75, 0.0),
    //         (0.25, 0.75, 0.0),
    //         (0.75, 0.75, 0.0),
    //         (-0.75, 1.25, 0.0),
    //         (-0.25, 1.25, 0.0),
    //         (0.25, 1.25, 0.0),
    //         (0.75, 1.25, 0.0),
    //         (-0.75, 1.75, 0.0),
    //         (-0.25, 1.75, 0.0),
    //         (0.25, 1.75, 0.0),
    //         (0.75, 1.75, 0.0),
    //     ];

    //     assert_eq!(expected, verts);
    // }


    // #[test]
    // fn steps_calculation() {
    //     let config = config::Terrain {
    //         rows: 4,
    //         columns: 4,
    //         size: 4.0,
    //         ..Default::default()
    //     };

    //     let steps = Procedural::new(config).steps;
    //     assert_eq!((0.5, 0.5), steps);

    //     let config = config::Terrain {
    //         rows: 8,
    //         columns: 4,
    //         size: 4.0,
    //         ..Default::default()
    //     };

    //     let steps = Procedural::new(config).steps;
    //     assert_eq!((0.25, 0.25), steps);

    //     let config = config::Terrain {
    //         rows: 4,
    //         columns: 8,
    //         size: 4.0,
    //         ..Default::default()
    //     };

    //     let steps = Procedural::new(config).steps;
    //     assert_eq!((0.25, 0.25), steps);
    // }


    // #[test]
    // fn rotation() {
    //     let config = config::Terrain {
    //         rows: 4,
    //         columns: 4,
    //         rotation: 0.0,
    //         ..Default::default()
    //     };
    //     let values = Procedural::new(config).coords_for_noise(1.0, 1.0);
    //     assert_eq!([0.5, 0.5], values);

    //     let config = config::Terrain {
    //         rows: 4,
    //         columns: 4,
    //         rotation: 1.0,
    //         ..Default::default()
    //     };
    //     let values = Procedural::new(config).coords_for_noise(1.0, 1.0);

    //     assert!(values[0].fract() - (1505.0) < 1e-10);
    //     assert!(values[1].fract() - (69088.0) < 1e-10);
    // }
// }


// mod benches {
//     use super::*;
//     #[allow(unused_imports)]
//     use test::Bencher;


//     #[bench]
//     fn faces(b: &mut Bencher) {
//         let config = config::Terrain {
//             rows: 128,
//             columns: 128,
//             ..Default::default()
//         };
//         let terrain = Procedural::new(config);
//         b.iter(|| terrain.faces());
//     }


//     #[bench]
//     fn verts(b: &mut Bencher) {
//         let config = config::Terrain {
//             rows: 128,
//             columns: 128,
//             flat: true,
//             ..Default::default()
//         };

//         let mut terrain = Procedural::new(config);

//         b.iter(|| terrain.vertices());
//     }


//     #[bench]
//     fn get_smooth_hills(b: &mut Bencher) {
//         let config = config::Terrain::default();
//         let terrain = Procedural::new(config);

//         b.iter(|| terrain.hills_z([0.0, 0.0]));
//     }


//     #[bench]
//     fn get_mountainous(b: &mut Bencher) {
//         let config = config::Terrain::default();
//         let terrain = Procedural::new(config);

//         b.iter(|| terrain.mountain_z([0.0, 0.0]));
//     }


//     #[bench]
//     fn terrain(b: &mut Bencher) {
//         let config = config::Terrain {
//             rows: 128,
//             columns: 128,
//             ..Default::default()
//         };
//         let mut terrain = Procedural::new(config);
//         b.iter(|| terrain.build_mesh());
//     }
// }
