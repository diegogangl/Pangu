use super::math;

/// Represents a user-defined curve
#[derive(Clone, Debug, Default)]
pub struct Curve {
    points: Vec<f64>,
}


impl Curve {
    pub fn new() -> Self {
        Curve { points: Vec::new() }
    }


    /// Adds a control point to the curve.
    ///
    /// # Arguments
    ///
    /// * `control_point` - Value for the control point
    pub fn add_point(&mut self, control_point: f64) -> &Self {
        let is_point_in_vector = self.points
                .iter()
                .any(|&x| (x - control_point).abs() < std::f64::EPSILON);

        if !is_point_in_vector {
            let index = self.points
                .iter()
                .position(|&x| x >= control_point)
                .unwrap_or_else(|| self.points.len());

            self.points.insert(index, control_point);
            debug!("Added control point {0} at #{1}", control_point, index);
        }

        self
    }


    /// Get the index to the two nearest control points to value
    ///
    /// # Arguments
    ///
    /// * `value` - Value to find points for
    pub fn points_near(&self, value: f64) -> (usize, usize) {
        let length = self.points.len();

        let ind_pos = self.points
            .iter()
            .position(|&x| x >= value)
            .unwrap_or(length);


        (math::clamp(ind_pos as isize - 1, 0, (length - 1) as isize) as usize,
         math::clamp(ind_pos, 0, length - 1))
    }


    /// Get a control point
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the control point
    pub fn point(&self, index: usize) -> f64 {
        self.points[index]
    }
}

