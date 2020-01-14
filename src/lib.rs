#![feature(test)]
#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

mod math;
mod curve;
mod terrain;
mod modifiers;
mod config;
mod map;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;


type PyMesh = PyResult<(terrain::Faces, terrain::Vertices)>;
type PyVerts = PyResult<terrain::Vertices>;


/// Glue function to generate a terrain. Builds a terrain::Procedural
/// and calls its build_mesh() function.
#[pyfunction]
fn terrain_mesh(params: &PyDict) -> PyMesh {
    let config = config::Terrain::from_dict(params)?;
    Ok(terrain::Procedural::new(config).build_mesh())
}


/// Glue function to generate a terrain's vertices. Builds a
/// terrain::Procedural and calls its build_vertices() function.
#[pyfunction]
fn terrain_vertices(params: &PyDict) -> PyVerts {
    let config = config::Terrain::from_dict(params)?;
    Ok(terrain::Procedural::new(config).build_vertices())
}


/// The pangu module to be used in Python
#[pymodule]
fn pangu(_py: Python, m: &PyModule) -> PyResult<()> {

    if !cfg!(target_os = "macos") || cfg!(debug_assertions) {
        let log_config = Config {time: Some(Level::Trace),..Default::default() };
        TermLogger::init(LevelFilter::Debug, log_config).unwrap();
    }

    m.add_wrapped(wrap_pyfunction!(terrain_mesh))?;
    m.add_wrapped(wrap_pyfunction!(terrain_vertices))?;
    Ok(())
}
