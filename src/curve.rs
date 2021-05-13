use super::math;


/// Represents a user-defined curve
#[derive(Clone, Debug, Default)]
pub struct Curve {
    points: Vec<ControlPoint>,
}

#[derive(Clone, Debug, Default, Copy)]
pub struct ControlPoint {
    pub input: f64,
    pub output: f64,
}

impl Curve {
    pub fn new() -> Self {
        Curve { points: Vec::with_capacity(4) }
    }


    /// Adds a control point to the curve.
    ///
    /// # Arguments
    ///
    /// * `control_point` - Value for the control point
    pub fn add_point(&mut self, input: f64, output: f64) -> &Self {
        let is_point_in_vector = self.points
                .iter()
                .any(|&x| (x.input - input).abs() < std::f64::EPSILON);

        if !is_point_in_vector {
            let index = self.points
                .iter()
                .position(|&x| x.input >= input)
                .unwrap_or_else(|| self.points.len());

            self.points.insert(index, ControlPoint {input, output});
            debug!("Added point at #{0}, {1}:{2}", index, input, output);
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
            .position(|&x| x.input >= value)
            .unwrap_or(length);


        (math::clamp(ind_pos as isize - 1, 0, (length - 1) as isize) as usize,
         math::clamp(ind_pos, 0, length - 1))
    }


    pub fn interpolate(&self, value: f64) -> f64 {

        // Confirm that there's at least 4 control points in the vector.
        assert!(self.points.len() >= 4);

        // Find the first element in the control point array that has a input
        // value larger than the output value from the source function
        let index_pos = self.points
                            .iter()
                            .position(|x| x.input > value)
                            .unwrap_or_else(|| self.points.len());

        // Ensure that the index is at least 2 and less than control_points.len()
        let index_pos = index_pos.clamp(2, self.points.len());

        // Find the four nearest control points so that we can perform cubic
        // interpolation.
        let index0 = (index_pos - 2).clamp(0, self.points.len() - 1);
        let index1 = (index_pos - 1).clamp(0, self.points.len() - 1);
        let index2 = index_pos.clamp(0, self.points.len() - 1);
        let index3 = (index_pos + 1).clamp(0, self.points.len() - 1);

        // If some control points are missing (which occurs if the value from
        // the source function is greater than the largest input value or less
        // than the smallest input value of the control point array), get the
        // corresponding output value of the nearest control point and exit.
        if index1 == index2 {
            return self.points[index1].output;
        }

        // Compute the alpha value used for cubic interpolation
        let input0 = self.points[index1].input;
        let input1 = self.points[index2].input;
        let alpha = (value - input0) / (input1 - input0);

        // Now perform the cubic interpolation and return.
        math::cubic(
            self.points[index0].output,
            self.points[index1].output,
            self.points[index2].output,
            self.points[index3].output,
            alpha,
        )
    }


    /// Get a control point
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the control point
    pub fn point(&self, index: usize) -> ControlPoint {
        self.points[index]
    }
}



