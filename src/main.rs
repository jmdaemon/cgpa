use std::{error::Error, io};

use csv::{ReaderBuilder, Trim};
use gpa::{create_csv_reader, read_gpa_scale, Grade, GradeScale};
use serde::{de::{self, Visitor}, Deserialize, Deserializer};

fn main() -> Result<(), Box<dyn Error>> {
    
    // Read the specific gpa scaling
    // Letter, Grade Point, Conversion
    let lines = vec!(
        "A+, 4.33, 90,100",
        "A , 4.00, 85,89",
        "A-, 3.67, 80,84",
        "B+, 3.33, 76,79",
        "B , 3.00, 72,75",
        "B-, 2.67, 68,71",
        "C+, 2.33, 64,67",
        "C , 2.00, 60,63",
        "C-, 1.67, 56,59",
        "D , 1.00, 50,55",
        "F , 0.00,  0,49",
    ).into_iter().map(String::from).collect::<Vec<String>>();

    let lines = lines.join("\n");

    let mut rdr = create_csv_reader(lines.as_bytes());
    for result in rdr.deserialize() {
        let record: Grade = result?;
        println!("{:?}", record);
    }
    let scale = read_gpa_scale(rdr);

    // Parse the course grading
    let lines= vec!(
        "3 Quizzes   , 10%",
        "2 Projects  , 20%",
        "5 Labs      , 20%",
        "Midterm Exam, 20%",
        "Final Exam  , 30%",
    );

    let lines = lines.join("\n");
    println!("{}", lines);

    let mut rdr = 
        ReaderBuilder::new()
            .has_headers(false)
            .trim(Trim::All)
            .from_reader(lines.as_bytes());

    for result in rdr.deserialize() {
        let record: Grading = result?;
        println!("{:?}", record);
        println!("{:?}", record.percent.to_weight());
    }

    let weights= rdr.deserialize()
        .into_iter()
        .flat_map(Result::ok)
        .collect();
    let weighs = GradeWeights { weights };


    // Parse a student's grading and calculate the end gpa based on the weights
    // let data = "
    // ";
    // println!("Hello, world!");

    Ok(())
}

// To calculate a gpa we have a list of 

#[derive(Debug, Clone)]
struct Percent {
    percent: String,
    value: u8,
}

impl Percent {
    fn to_weight(&self) -> f64 {
        self.value as f64 / 100.0
    }
}

struct PercentVisitor;
impl<'de> Visitor<'de> for PercentVisitor {
    type Value = Percent;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a whole number percent between 0-100 (e.g 55%)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error, {
                let s = v.split("%")
                    .collect::<Vec<&str>>()
                    .join("")
                    .to_string();
                let value = s.parse::<u8>().expect("Error: Unable to parse percent");
                Ok(Percent { percent: v.to_owned(), value })
    }
}

impl<'de> Deserialize<'de> for Percent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de> {
        deserializer.deserialize_str(PercentVisitor)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Grading {
    title: String,
    percent: Percent,
}

struct GradeWeights {
    weights: Vec<Grading>,
}

pub mod gpa {
    use std::ops::Range;

    use csv::{Reader, ReaderBuilder, Trim};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Grade {
        letter: String,
        grade_point: f64,
        // conversion: Range<(u8, u8)>,
        conversion: Range<u8>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GradeScale {
        scale: Vec<Grade>,
    }

    pub type GradeScaleReader<'a> = Reader<&'a [u8]>;

    pub fn create_csv_reader(content: &[u8]) -> GradeScaleReader {
        ReaderBuilder::new()
            .has_headers(false)
            .trim(Trim::All)
            .from_reader(content)
    }

    pub fn read_gpa_scale(mut rdr: GradeScaleReader) -> GradeScale {
        let scale = rdr.deserialize()
            .into_iter()
            .flat_map(Result::ok)
            .collect();
        GradeScale { scale }
    }
}
