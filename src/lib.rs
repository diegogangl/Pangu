#![feature(test)]

mod land_fractal;
mod terrain;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;


type PyMesh = PyResult<(terrain::Faces, terrain::Vertices)>;


/// Glue function to generate a terrain. Builds a terrain::Procedural
/// and calls its build_mesh() function.
#[pyfunction]
fn procedural_terrain(params: &PyDict) -> PyMesh {
    let seed = match params.get_item("seed") {
        Some(item) => item.extract()?,
        None => 0,
    };

    Ok(terrain::Procedural::new().set_rows(128)
                                 .set_columns(128)
                                 .set_seed(seed)
                                 .build_mesh())
}


/// The pangu module to be used in Python
#[pymodule]
fn pangu(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(procedural_terrain))?;

    Ok(())
}
