#![feature(test)]
#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

mod math;
mod curve;
mod terrain;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use pyo3::PyErr;

type PyMesh = PyResult<(terrain::Faces, terrain::Vertices)>;
type PyVerts = PyResult<terrain::Vertices>;


/// Macro to extract item values from Python dictionary as simple
/// rust types.
///
/// # Arguments
///
/// * `params` - The parameters dictionary
/// * `key` - The key to look for in the dictionary
/// * `default_value` - Value to use when the key is not found
macro_rules! param {
    ($params:expr, $key:expr, $default_value:ident) => {
        match $params.get_item($key) {
            Some(item) => item.extract()?,
            None => terrain::ProceduralConfig::$default_value,
        };
    };

    // Vecs can't have default parameters
    ($params:expr, $key:expr) => {
        match $params.get_item($key) {
            Some(item) => item.extract()?,
            None => Vec::new(),
        };
    };
}


/// Setup and return configuration
fn get_config(params: &PyDict) -> Result<terrain::ProceduralConfig, PyErr> {
    debug!("Start Terrain Generation =====================================");

    let config = terrain::ProceduralConfig {
        seed: param!(params, "seed", DEFAULT_SEED),
        rows: param!(params, "rows", DEFAULT_ROWS),
        columns: param!(params, "columns", DEFAULT_COLUMNS),
        size: param!(params, "size", DEFAULT_SIZE),
        scale: param!(params, "scale", DEFAULT_SCALE),
        offset_x: param!(params, "offset_x", DEFAULT_OFFSET),
        offset_y: param!(params, "offset_y", DEFAULT_OFFSET),
        rotation: param!(params, "rotation", DEFAULT_OFFSET),
        roughness: param!(params, "roughness", DEFAULT_OFFSET),
        plains: param!(params, "plains", DEFAULT_OFFSET),
        plateau: param!(params, "plateau", DEFAULT_OFFSET),
        deformation: param!(params, "deformation", DEFAULT_OFFSET),
        mountainess: param!(params, "mountainess", DEFAULT_MOUNTAINESS),
        mix: param!(params, "mix", DEFAULT_MIX),
        ridgedness: param!(params, "ridgedness", DEFAULT_RIDGEDNESS),
        sea_floor: param!(params, "sea_floor", DEFAULT_SEA_FLOOR),
        height: param!(params, "height", DEFAULT_HEIGHT),
        is_seamless: param!(params, "seamless", DEFAULT_SEAMLESS),
        invert: param!(params, "invert", DEFAULT_INVERT),
        terraces: param!(params, "terraces", DEFAULT_TERRACES),
        terraces_invert: param!(params, "terraces_invert", DEFAULT_TERRACES_INVERT),
        terraces_points: param!(params, "terraces_points"),
        flat: false,
        smooth: param!(params, "smooth", DEFAULT_SMOOTH),
        smooth_radial: param!(params, "smooth_radial", DEFAULT_SMOOTH_RADIAL),
        smooth_radial_fac: param!(params, "smooth_radial_fac", DEFAULT_SMOOTH_RADIAL_FAC),
        smooth_radial_size: param!(params, "smooth_radial_size", DEFAULT_SMOOTH_RADIAL_SIZE),
        smooth_linear_fac: param!(params, "smooth_linear_fac", DEFAULT_SMOOTH_LINEAR_FAC),
        smooth_linear_start: param!(params, "smooth_linear_start", DEFAULT_SMOOTH_LINEAR_START),
        smooth_linear_invert: param!(params, "smooth_linear_invert", DEFAULT_SMOOTH_LINEAR_INVERT),
    };

    Ok(config)
}


/// Glue function to generate a terrain. Builds a terrain::Procedural
/// and calls its build_mesh() function.
#[pyfunction]
fn terrain_mesh(params: &PyDict) -> PyMesh {
    let config = get_config(params)?;
    Ok(terrain::Procedural::new(config).build_mesh())
}


/// Glue function to generate a terrain's vertices. Builds a
/// terrain::Procedural and calls its build_vertices() function.
#[pyfunction]
fn terrain_vertices(params: &PyDict) -> PyVerts {
    let config = get_config(params)?;
    Ok(terrain::Procedural::new(config).build_vertices())
}


/// The pangu module to be used in Python
#[pymodule]
fn pangu(_py: Python, m: &PyModule) -> PyResult<()> {

    let log_config = Config {time: Some(Level::Trace),..Default::default() };
    TermLogger::init(LevelFilter::Debug, log_config).unwrap();

    m.add_wrapped(wrap_pyfunction!(terrain_mesh))?;
    m.add_wrapped(wrap_pyfunction!(terrain_vertices))?;
    Ok(())
}
