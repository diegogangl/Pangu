#![feature(test)]

mod land_fractal;
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
macro_rules! get_param {
    ($params:expr, $key:expr, $default_value:expr) => {
        match $params.get_item($key) {
            Some(item) => item.extract()?,
            None => $default_value,
        };
    };
}


/// Setup and return terrain
fn procedural_terrain(params: &PyDict) -> Result<terrain::Procedural, PyErr> {
    let seed = get_param!(params, "seed", 0);
    let rows = get_param!(params, "rows", 64);
    let columns = get_param!(params, "columns", 64);
    let size = get_param!(params, "size", 5.0);
    let scale = get_param!(params, "scale", 2.0);
    let offset_x = get_param!(params, "offset_x", 0.0);
    let offset_y = get_param!(params, "offset_y", 0.0);
    let offset_z = get_param!(params, "offset_z", 0.0);
    let rotation = get_param!(params, "rotation", 0.0);

    Ok(terrain::Procedural::new().set_rows(rows)
                                 .set_columns(columns)
                                 .set_size(size)
                                 .set_scale(scale)
                                 .set_offset_x(offset_x)
                                 .set_offset_y(offset_y)
                                 .set_offset_z(offset_z)
                                 .set_rotation(rotation)
                                 .set_seed(seed))
}


/// Glue function to generate a terrain. Builds a terrain::Procedural
/// and calls its build_mesh() function.
#[pyfunction]
fn terrain_mesh(params: &PyDict) -> PyMesh {
    let terrain = procedural_terrain(params)?;
    Ok(terrain.build_mesh())
}


/// Glue function to generate a terrain's vertices. Builds a
/// terrain::Procedural and calls its build_vertices() function.
#[pyfunction]
fn terrain_vertices(params: &PyDict) -> PyVerts {
    let terrain = procedural_terrain(params)?;
    Ok(terrain.build_vertices())
}


/// The pangu module to be used in Python
#[pymodule]
fn pangu(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(terrain_mesh))?;
    m.add_wrapped(wrap_pyfunction!(terrain_vertices))?;
    Ok(())
}
