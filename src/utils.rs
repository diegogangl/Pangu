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

