use std::ops::RangeInclusive;

use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::fmt::CSVReader;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, new)]
pub struct GradePoint {
    letter: String,
    grade_point: f64,
    conversion: RangeInclusive<u8>,
}

impl GradePoint {
    fn within(&self, value: &u8) -> bool {
        self.conversion.contains(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradePointAverageScale {
    pub scale: Vec<GradePoint>,
}
pub type GPAScale = GradePointAverageScale;

impl GradePointAverageScale {
    pub fn calc_gpa(&self, value: &u8) -> Option<GradePoint> {
        for grade in &self.scale {
            if grade.within(value) {
                return Some(grade.clone());
            }
        }
        return None;
    }
}

pub fn read_gpa_scale(mut rdr: CSVReader) -> GradePointAverageScale {
    let scale = rdr.deserialize().into_iter().flat_map(Result::ok).collect();
    GradePointAverageScale { scale }
}
