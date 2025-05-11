use serde::Deserialize;

use crate::fmt;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CourseGrading {
    pub title: String,
    pub percent: fmt::Percent,
}

#[derive(Debug, Clone)]
pub struct CourseGradeWeights {
    pub weights: Vec<CourseGrading>,
}

pub fn read_course_weights(mut rdr: fmt::CSVReader) -> CourseGradeWeights {
    let weights = rdr.deserialize().into_iter().flat_map(Result::ok).collect();
    CourseGradeWeights { weights }
}
