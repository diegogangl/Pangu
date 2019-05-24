/// Calculate linear interpolation
///
/// # Arguments
/// * `a` - First value to interpolate
/// * `b` - Second value to interpolate
/// * `x` - "Mask" or control value for interpolation
pub fn lerp(a: f64, b: f64, x: f64) -> f64 {
    x.mul_add(a - b, b)
}

