use pyo3::types::PyDict;
use pyo3::PyErr;
use pyo3::prelude::*;

use super::modifiers::*;


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

    ///  Terraces modifier
    pub terraces: Terraces,

    // Smooth modifier
    pub smooth: Smooth,

    // Thermal Erosion modifier
    pub thermal: ThermalErosion,

    // Hydraulic Erosion modifier
    pub water: WaterErosion,
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
            terraces: Terraces::default(),
            smooth: Smooth::default(),
            thermal: ThermalErosion::default(),
            water: WaterErosion::with_capacity(1),
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
        let height = get!(params, "height");
        let columns: u32 = get!(params, "columns");
        let rows: u32 = get!(params, "rows");

        let terraces = if get!(params, "terraces") {
            let points = get!(params, "terraces_points");
            let mut t = Terraces::from_list(points, height);
            t.invert = get!(params, "terraces_invert");

            t
        } else {
            Terraces::default()
        };

        let smooth = if get!(params, "smooth") {
            Smooth {
                enabled: true,
                style: if get!(params, "smooth_radial") {
                            SmoothStyle::RADIAL
                        } else {
                            SmoothStyle::LINEAR
                        },
                radial_fac: get!(params, "smooth_radial_fac"),
                radial_size: get!(params, "smooth_radial_size"),
                linear_fac: get!(params, "smooth_linear_fac"),
                linear_start: get!(params, "smooth_linear_start"),
                linear_invert: get!(params, "smooth_linear_invert"),
                rows: rows as f64,
                columns: columns as f64,
            }
        } else {
            Smooth::default()
        };


        let thermal = if get!(params, "thermal") {
            ThermalErosion {
                enabled: true,
                talus: get!(params, "thermal_talus"),
                iterations: get!(params, "thermal_iterations"),
            }
        } else {
            ThermalErosion::default()
        };


        let water = if  get!(params, "water") {
            let capacity = (rows * columns) as usize;
            let mut w = WaterErosion::with_capacity(capacity);

            w.enabled = true;
            w.iterations =  get!(params, "water_iterations");
            w.evaporation =  get!(params, "water_evaporation");
            w.rain_rate =  get!(params, "water_rain");
            w.soil_capacity = get!(params, "water_soil");

            let springs: Vec<&PyDict> = get!(params, "water_springs");

            for s in springs {
               w.springs.push(Spring {
                    x: get!(s, "x"),
                    y: get!(s, "y"),
                    radius: get!(s, "radius"),
                    amount: get!(s, "amount"),
               });
            }

            w
        } else {
            WaterErosion::with_capacity(1)
        };


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
            height: height,
            is_seamless: get!(params, "seamless"),
            invert: get!(params, "invert"),
            terraces: terraces,
            flat: false,
            smooth: smooth,
            thermal: thermal,
            water: water,
        };

        Ok(config)

    }
}
