#![feature(test)]
#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

#[macro_use]
mod utils;
mod math;
mod curve;
mod terrain;
mod modifiers;
mod map;
mod types;

use pyo3::prelude::*;


/// The pangu module to be used in Python
#[pymodule]
fn pangu(_py: Python, m: &PyModule) -> PyResult<()> {

    // Ignore silently if we can't find a Term to log
    let mut config_builder = ConfigBuilder::new();
    config_builder.set_time_level(LevelFilter::Trace);

    let _ = TermLogger::init(LevelFilter::Debug,
                             config_builder.build(),
                             TerminalMode::Mixed);

    m.add_class::<terrain::Terrain>()?;

    Ok(())
}
