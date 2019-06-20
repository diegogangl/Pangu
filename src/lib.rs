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
macro_rules! get {
    ($params:expr, $key:expr) => {
        $params.get_item($key).unwrap().extract()?
    };
}


/// Setup and return configuration
fn get_config(params: &PyDict) -> Result<terrain::ProceduralConfig, PyErr> {
    debug!("[Start Terrain Generation]");

    let config = terrain::ProceduralConfig {
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
