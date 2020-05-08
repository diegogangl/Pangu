/// Macro to extract values from a Python dictionary as
/// rust types.
///
/// # Arguments
///
/// * `params` - The parameters dictionary
/// * `key` - The key to look for in the dictionary

#[macro_export]
macro_rules! get {
    ($params:expr, $key:expr) => {
        match $params.get_item($key) {
            Some(v) => v.extract()?,
            None => panic!("Missing key {}!", $key),
        }
    };
}
