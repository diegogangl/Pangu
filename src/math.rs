use std::f64::consts::PI;

/// Small math library to make life easier

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


/// Calculate cosine interpolation
///
/// # Arguments
///
/// * `a` - First value to interpolate
/// * `b` - Second value to interpolate
/// * `x` - "Mask" or control value for interpolation
pub fn cos_interp(a: f64, b: f64, x: f64) -> f64 {
    let mu = (1.0 - (x * PI).cos()) / 2.0;
    a * (1.0 - mu) + b * mu
}


/// Get a value from a percentage
///
/// # Arguments
///
/// * `percent` - The percentage (range 0..100)
/// * `total - Value represented by 100%
pub fn percent_to_value(percent: f64, total: f64) -> f64 {
    (percent / 100.0) * total
}


/// Clamp a value to a range
///
/// # Arguments
///
/// * `val` - Value to clamp
/// * `min` - Minimum value
/// * `max` - Maximum value
pub fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    debug_assert!(min <= max, "min must be less than or equal to max");

    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}


/// Calculate euclidean distance between two points
///
/// This function returns the *squared* distance.
///
/// # Arguments
///
/// * `p1` - Coordinates of the first point
/// * `p2` - Coordinates of the second point
pub fn distance<T>(p1: [T; 2], p2: [T; 2]) -> T
    where T: std::ops::Add<Output=T>
        + std::ops::Mul<Output=T>
        + std::ops::Sub<Output=T>
        + PartialOrd + Copy {

    let dist_x = if p1[0] > p2[0] { p1[0] - p2[0] } else { p2[0] - p1[0] };
    let dist_y = if p1[1] > p2[1] { p1[1] - p2[1] } else { p2[1] - p1[1] };

    (dist_x * dist_x) + (dist_y * dist_y)
}


/// Calculate magnitude of a 3D Vector
///
/// # Arguments
///
/// * `vector` - Slice of 3 values for the vector
pub fn magnitude(vector: &[f64]) -> f64 {

    vector.iter()
          .fold(0.0, |sum, val| sum + val.powi(2))
          .sqrt()
}


/// Normalize a 3D Vector
///
/// # Arguments
///
/// * `vector` - Slice of 3 values for the vector
pub fn normalize(vector: &[f64]) -> [f64; 3] {

    assert_eq!(vector.len(), 3);

    let mag = magnitude(vector);

    [vector[0] / mag,
     vector[1] / mag,
     vector[2] / mag]
}


/// Calculate the dot product of two vectors
///
/// # Arguments
///
/// * `a` - The first vector
/// * `b` - The first vector
pub fn dot(a: &[f64], b: &[f64]) -> f64 {

    assert_eq!(a.len(), b.len());

    let mut product = 0.0;

    for i in 0..a.len() {
        product += a[i] * b[i];
    }

    product
}


#[allow(dead_code)]
/// Adjust brighness and contrast for a value
///
/// This assumes the value is in the range 0..1,
/// brighness and contrast however can be any value.
/// The output is clamped back to 0..1, so it works like
/// an image editor brightness/contrast filter.
///
/// # Arguments
///
/// * `source` - The value
/// * `contrast` - The contrast factor
/// * `brightness` - The brightness factor
pub fn bright_contrast(source: f64, contrast: f64, brightness: f64) -> f64 {
    clamp((source - 0.5) * contrast + 0.5 + brightness, 0.0, 1.0)
}


/// Remap a value
///
/// Take a value between src.0 and src.1 and convert
/// it so it fits between dst.0 and dst.1
///
/// # Arguments
///
/// * `src` - The original range (min and max)
/// * `val` - The value to remap
/// * `dst` - The target range (min and max)
pub fn remap(src: [f64; 2], val: f64, dst: [f64; 2]) -> f64 {
    dst[0] + (val - src[0]) * (dst[1] - dst[0]) / (src[1] - src[0])
}


/// Performs cubic interpolation between two values bound between two other
/// values.
///
/// * `n0` - The value before the first value.
/// * `n1` - The first value.
/// * `n2` - The second value.
/// * `n3` - The value after the second value.
/// * `alpha` - The alpha value.
///
/// The alpha value should range from 0.0 to 1.0. If the alpha value is
/// 0.0, this function returns _n1_. If the alpha value is 1.0, this
/// function returns _n2_.
#[inline]
pub fn cubic(n0: f64, n1: f64, n2: f64, n3: f64, alpha: f64) -> f64
{
    let p = (n3 - n2) - (n0 - n1);
    let q = (n0 - n1) - p;

    p * alpha * alpha * alpha + q * alpha * alpha + (n2 - n0) * alpha + n1
}
