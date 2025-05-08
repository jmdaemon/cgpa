use std::{error::Error, io};

use csv::{ReaderBuilder, Trim};
use gpa::{create_csv_reader, read_gpa_scale, Grade, GradeScale};

fn main() -> Result<(), Box<dyn Error>> {
    // Parse a line in a csv file like
    // let data = "
    // ";
    // println!("Hello, world!");
    
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
    Ok(())
}

// To calculate a gpa we have a list of 

struct Record {
    title: String,
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
