use pyo3::types::PyDict;
use pyo3::PyErr;
use pyo3::prelude::*;


/// Macro to extract values from a Python dictionary as
/// rust types.
///
/// # Arguments
///
/// * `params` - The parameters dictionary
/// * `key` - The key to look for in the dictionary
macro_rules! get {
    ($params:expr, $key:expr) => {
        $params.get_item($key).unwrap().extract()?
    };
}


/// Settings for a Procedural Terrain
///
/// This is only a structure to hold parameters for
/// the terrain, it doesn't do anything on it's own.
#[derive(Clone, Debug)]
pub struct Terrain {
    /// The number of rows to use in the mesh grid
    pub rows: u32,

    /// The number of columns to use in the mesh grid
    pub columns: u32,

    /// Offsets for the coordinates passed to the noise
    /// function
    pub offset_x: f64,
    pub offset_y: f64,

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

    /// Mountainess
    pub mountainess: f64,

    /// Intensity of domain warping
    pub deformation: f64,

    /// Mixing between plains and mountains
    pub mix: f64,

    /// Ridgedness
    pub ridgedness: f64,

    /// Sea Floor
    pub sea_floor: f64,

    /// Maximum Height
    pub height: f64,

    /// Make the terrain seamless
    pub is_seamless: bool,

    /// Invert the terrain
    pub invert: bool,

    /// Use terraces
    pub terraces: bool,

    /// Invert Terraces
    pub terraces_invert: bool,

    /// Invert Terraces
    pub terraces_points: Vec<f64>,

    // Smooth out terrain
    pub smooth: bool,
    pub smooth_radial: bool,
    pub smooth_radial_fac: f64,
    pub smooth_radial_size: (f64, f64),
    pub smooth_linear_fac: (f64, f64),
    pub smooth_linear_start: (f64, f64),
    pub smooth_linear_invert: (bool, bool),
}


impl Default for Terrain {
    fn default() -> Self {
        Terrain {
            rows: 64,
            columns: 64,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,
            scale: 2.0,
            size: 5.0,
            seed: 0,
            roughness: 0.1,
            plains:0.5,
            deformation: 0.1,
            mountainess: 0.5,
            mix: 0.5,
            ridgedness: 0.0,
            sea_floor: 0.0,
            height: 3.0,
            flat: false,
            is_seamless: false,
            invert: false,
            terraces: false,
            terraces_invert: false,
            terraces_points: Vec::new(),
            smooth: false,
            smooth_radial: true,
            smooth_radial_fac: 0.0,
            smooth_radial_size: (0.0, 0.0),
            smooth_linear_fac: (0.0, 0.0),
            smooth_linear_start: (0.0, 0.0),
            smooth_linear_invert: (true, false),
        }
    }
}


impl Terrain {

    /// Build config from parameters in a Python Dictionary
    ///
    /// # Arguments
    ///
    /// * `params`: The parameters dictionary
    pub fn from_dict(params: &PyDict) -> Result<Self, PyErr> {

        let config = Terrain {
            seed: get!(params, "seed"),
            rows: get!(params, "rows"),
            columns: get!(params, "columns"),
            size: get!(params, "size"),
            scale: get!(params, "scale"),
            offset_x: get!(params, "offset_x"),
            offset_y: get!(params, "offset_y"),
            rotation: get!(params, "rotation"),
            roughness: get!(params, "roughness"),
            plains: get!(params, "plains"),
            deformation: get!(params, "deformation"),
            mountainess: get!(params, "mountainess"),
            mix: get!(params, "mix"),
            ridgedness: get!(params, "ridgedness"),
            sea_floor: get!(params, "sea_floor"),
            height: get!(params, "height"),
            is_seamless: get!(params, "seamless"),
            invert: get!(params, "invert"),
            terraces: get!(params, "terraces"),
            terraces_invert: get!(params, "terraces_invert"),
            terraces_points: get!(params, "terraces_points"),
            flat: false,
            smooth: get!(params, "smooth"),
            smooth_radial: get!(params, "smooth_radial"),
            smooth_radial_fac: get!(params, "smooth_radial_fac"),
            smooth_radial_size: get!(params, "smooth_radial_size"),
            smooth_linear_fac: get!(params, "smooth_linear_fac"),
            smooth_linear_start: get!(params, "smooth_linear_start"),
            smooth_linear_invert: get!(params, "smooth_linear_invert"),
        };

        Ok(config)

    }
}
