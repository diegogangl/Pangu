/// Calculate linear interpolation
///
/// # Arguments
///
/// * `a` - First value to interpolate
/// * `b` - Second value to interpolate
/// * `x` - "Mask" or control value for interpolation
pub fn lerp(a: f64, b: f64, x: f64) -> f64 {
    x.mul_add(a - b, b)
}


/// Remap a value to the range 0-n
///
/// This function does a linear conversion for a value
/// in the range [min..max] to [0..new_max].
///
/// # Arguments
///
/// * `v` - First value to interpolate
/// * `min` - Minimum value in the original range
/// * `max` - Maximum value in the original range
/// * `new_max` - Maximum value in the target range
pub fn map_on_zero(v: f64, min: f64, max: f64, new_max: f64) -> f64 {
    ((v - min) * new_max) / (max - min)
}
