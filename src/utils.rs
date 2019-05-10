/// Macro to generate public setter methods
///
/// # Arguments
///
/// * `name` - Name of the setter function
/// * `attr` - Name of the field to set
/// * `attr_type` - Expected type for the field
macro_rules! setter {
    ($name:ident, $attr:ident, $attr_type:ty) => {
        pub fn $name(mut self, value: $attr_type) -> Self {
            self.$attr = value;
            self
        }
    }
}


/// Calculate linear interpolation
///
/// # Arguments
/// * `a` - First value to interpolate
/// * `b` - Second value to interpolate
/// * `x` - "Mask" or control value for interpolation
pub fn linear_interp(a: f64, b: f64, x: f64) -> f64 {
   x.mul_add(a - b, b)
}
