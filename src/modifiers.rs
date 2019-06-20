use super::math;
use super::curve::Curve;


/// Terraces modifier
///
/// Creates a terraces-like effect by flattening certain areas
/// of the terrain. These areas are defined by their height,
/// according to the control points in the curve.
#[derive(Clone, Debug)]
pub struct Terraces {
    /// Enable the modifier
    pub enabled: bool,

    /// Invert Slope
    pub invert: bool,

    /// Control points for the terraces
    pub curve: Curve,
}


impl Default for Terraces {
    fn default() -> Self {
        Terraces {
            enabled: false,
            invert: false,
            curve: Curve::new(),
        }
    }
}


impl Terraces {

    /// Return a Terrace modifier with points from list
    ///
    /// # Arguments
    ///
    /// * `points` - A list of control points. Range: [0..100]
    /// * `height` - The terrain's height
    pub fn from_list(points: Vec<f64>, height: f64) -> Self {
        debug!("Adding control points for terrace");

        let mut curve = Curve::new();

        points.iter().for_each(|p| {
           let point = math::percent_to_value(*p, height);
           curve.add_point(point);
        });

        Terraces {
            enabled: true,
            invert: false,
            curve: curve,
        }
    }


    /// Calculate the terrace effect
    ///
    /// # Arguments
    /// * `value - A height value from the terrain
    pub fn run(&self, value: f64) -> f64 {

        // Get indices of the nearest two points
        let indexes = self.curve.points_near(value);

        // If some control points are missing get the output value
        // of the nearest control point and return. This can
        // happen when value < lowest_point or value > highest_point
        if indexes.0 == indexes.1 {
            return self.curve.point(indexes.1);
        }

        // Get values and calculate alpha parameter for lerping
        let mut input_0 = self.curve.point(indexes.0);
        let mut input_1 = self.curve.point(indexes.1);
        let mut alpha = (value - input_0) / (input_1 - input_0);

        if self.invert {
            alpha = 1.0 - alpha;
            std::mem::swap(&mut input_0, &mut input_1);
        }

        math::lerp(input_1, input_0, alpha.powi(2))
    }
}

