use super::map::Map2D;

pub mod empty;
pub mod invert;
pub mod seamless;
pub mod terraces;
pub mod smooth;
pub mod thermal;
pub mod water;


/// Common interface for modifiers
pub trait Modifier {

    /// Return if the modifier is enabled
    fn is_enabled(&self) -> bool;

    /// Apply the modifier to the heightmap
    ///
    /// # Arguments
    ///
    /// * `hmap` - Reference to the heightmap
    fn run(&mut self, hmap: &mut Map2D<f64>);
}
