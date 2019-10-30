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


/// Type of terrain to generate
///
/// This option affects the settings and noise
/// function used.
#[derive(Clone, Debug)]
pub enum TerrainType {
    SmoothHills,
    Mountainous,
}


/// Settings for the Smooth Hills Terrain type
///
/// These settings are only specific to this terrain
/// type.
#[derive(Clone, Debug)]
pub struct SmoothHills {

    // General scale (first octave)
    pub difference: f64,

    // Flat area between hills
    pub flat: f64,

    // Noise on hills
    pub detail: f64,

    // Amount of domain warping to apply
    pub twist: f64,
}

impl Default for SmoothHills {
    fn default() -> Self {
        SmoothHills {
            difference: 0.0,
            flat: 0.0,
            detail: 0.0,
            twist: 0.0,
        }
    }
}



/// Settings for the Mountainous Terrain type
///
/// These settings are only specific to this terrain
/// type.
#[derive(Clone, Debug)]
pub struct Mountainous {
    // Ridgedness (spikey-ness) of the mountains
    pub ridgedness: f64,

    // Sharpness of the medium terrain features
    pub sharpness: f64,

    // Number of mountains (scale at the 3rd octave)
    pub breakup: f64,

    // Terrain roughness (persistence of higher octaves)
    pub roughness: f64,

    // Amount of domain warping to apply
    pub twist: f64,
}

impl Default for Mountainous {
    fn default() -> Self {
        Mountainous {
            ridgedness: 0.0,
            sharpness: 0.0,
            breakup: 0.0,
            roughness: 0.0,
            twist: 0.0,
        }
    }
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

    /// If rows != columns, then the terrain will have
    /// to be cut to this size. Otherwise it's value 
    /// is zero.
    pub to_cut: (u32, u32),

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

    // Settings for Smooth Hills 
    pub hills: SmoothHills,

    // Settings for Mountainous terrain
    pub mountains: Mountainous,

    // Type of terrain to generate
    pub terrain_type: TerrainType,

    // Seamless modifier
    pub seamless: Seamless,
}


impl Default for Terrain {
    fn default() -> Self {
        Terrain {
            rows: 64,
            columns: 64,
            to_cut: (0, 0),
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: 0.0,
            scale: 2.0,
            size: 5.0,
            seed: 0,
            height: 3.0,
            flat: false,
            is_seamless: false,
            invert: false,
            terraces: Terraces::default(),
            smooth: Smooth::default(),
            thermal: ThermalErosion::default(),
            water: WaterErosion::with_size(1, 1),
            hills: SmoothHills::default(),
            mountains: Mountainous::default(),
            terrain_type: TerrainType::SmoothHills,
            seamless: Seamless::default(),
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
        let mut columns: u32 = get!(params, "columns");
        let mut rows: u32 = get!(params, "rows");

        let to_cut = if rows != columns {
            (rows, columns)
        } else {
            (0, 0)
        };

        // Make sure terrains are square
        if rows != columns {
            let square = columns.max(rows);

            columns = square;
            rows = square;
        };

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
            let mut w = WaterErosion::with_size(rows as usize, columns as usize);

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
            WaterErosion::with_size(1, 1)
        };

        let hills = if get!(params, "type_hills") {
            SmoothHills {
                difference: get!(params, "hills_difference"),
                flat: get!(params, "hills_flat"),
                detail: get!(params, "hills_detail"),
                twist: get!(params, "hills_twist"),
            } 
        } else {
            SmoothHills::default()
        };

        let mountains = if get!(params, "type_mountains") {
            Mountainous {
                ridgedness: get!(params, "mountains_ridgedness"),
                sharpness: get!(params, "mountains_sharpness"),
                breakup: get!(params, "mountains_breakup"),
                roughness: get!(params, "mountains_roughness"),
                twist: get!(params, "mountains_twist"),
            }
        } else {
            Mountainous::default()
        };

        let terrain_type = if get!(params, "type_mountains") {
            TerrainType::Mountainous
        } else {
            TerrainType::SmoothHills
        };


        let seamless = if get!(params, "seamless") {
            Seamless { 
                enabled: true,
                fade: get!(params, "seamless_fade"),
            }
        } else {
            Seamless::default() 
        };

        let config = Terrain {
            seed: get!(params, "seed"),
            rows,
            columns,
            to_cut,
            size: get!(params, "size"),
            scale: get!(params, "scale"),
            offset_x: get!(params, "offset_x"),
            offset_y: get!(params, "offset_y"),
            rotation: get!(params, "rotation"),
            height,
            is_seamless: get!(params, "seamless"),
            invert: get!(params, "invert"),
            terraces,
            flat: false,
            smooth,
            thermal,
            water,
            hills,
            mountains,
            terrain_type,
            seamless,
        };

        Ok(config)

    }
}
