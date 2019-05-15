#![feature(test)]
#[macro_use]

mod utils;
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
}


/// Setup and return configuration
fn get_config(params: &PyDict) -> Result<terrain::ProceduralConfig, PyErr> {
    let config = terrain::ProceduralConfig {
        seed: param!(params, "seed", DEFAULT_SEED),
        rows: param!(params, "rows", DEFAULT_ROWS),
        columns: param!(params, "columns", DEFAULT_COLUMNS),
        size: param!(params, "size", DEFAULT_SIZE),
        scale: param!(params, "scale", DEFAULT_SCALE),
        offset_x: param!(params, "offset_x", DEFAULT_OFFSET),
        offset_y: param!(params, "offset_y", DEFAULT_OFFSET),
        offset_z: param!(params, "offset_z", DEFAULT_OFFSET),
        rotation: param!(params, "rotation", DEFAULT_OFFSET),
        roughness: param!(params, "roughness", DEFAULT_OFFSET),
        plains: param!(params, "plains", DEFAULT_OFFSET),
        plateau: param!(params, "plateau", DEFAULT_OFFSET),
        deformation: param!(params, "deformation", DEFAULT_OFFSET),
        flat: false,
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
    m.add_wrapped(wrap_pyfunction!(terrain_mesh))?;
    m.add_wrapped(wrap_pyfunction!(terrain_vertices))?;
    Ok(())
}
