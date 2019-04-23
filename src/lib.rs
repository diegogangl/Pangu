#![feature(test)]

mod land_fractal;
mod terrain;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;


type PyMesh = PyResult<(terrain::Faces, terrain::Vertices)>;


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


/// Glue function to generate a terrain. Builds a terrain::Procedural
/// and calls its build_mesh() function.
#[pyfunction]
fn procedural_terrain(params: &PyDict) -> PyMesh {
    let seed = get_param!(params, "seed", 0);
    let rows = get_param!(params, "rows", 64);
    let columns = get_param!(params, "columns", 64);
    let offset_x = get_param!(params, "offset_x", 0.0);
    let offset_y = get_param!(params, "offset_y", 0.0);


    Ok(terrain::Procedural::new().set_rows(rows)
                                 .set_columns(columns)
                                 .set_offset_x(offset_x)
                                 .set_offset_y(offset_y)
                                 .set_seed(seed)
                                 .build_mesh())
}


/// The pangu module to be used in Python
#[pymodule]
fn pangu(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(procedural_terrain))?;

    Ok(())
}
