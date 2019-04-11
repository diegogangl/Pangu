#![feature(test)]

mod terrain;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;


/// Glue function to generate a terrain. Builds a terrain::Procedural
/// and calls its build_mesh() function.
#[pyfunction]
fn procedural_terrain() -> (terrain::Faces, terrain::Vertices) {

    terrain::Procedural::new().build_mesh()

}


/// The pangu module to be used in Python
#[pymodule]
fn pangu(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(procedural_terrain))?;

    Ok(())
}
