use std::{error::Error, fmt::Debug, io::{self, Write}};

use csv::{Reader, ReaderBuilder, Trim};
use gpa::{read_gpa_scale, GradePoint, GradePointAverageScale};
use serde::{de::{self, DeserializeOwned, Visitor}, Deserialize, Deserializer};

fn main() -> Result<(), Box<dyn Error>> {

    // TODO:
    // Core:
    // - Updated prompts:
    // - File IO
    // - Simple CLI
    // Extra:
    // - Simple Calculator?
    // - Switch to pre/post weights (0-100% scale vs 10% or 20% etc)
    // - Prediction mode
    // - Experimental mode to calculate grade from assignment basis to final cumulative
    // grade
    // - C shared library bindings 
    
    // Read the specific gpa scaling
    let lines = gpa_scale().join("\n");
    let rdr = create_csv_reader(lines.as_bytes());
    show_lines::<GradePoint>(rdr)?;
    let rdr = create_csv_reader(lines.as_bytes());
    let scale = read_gpa_scale(rdr);

    // Prompt student for final grade & check with grade scale
    let input = 84u8;

    let g = scale.calc_gpa(&input);

    if let Some(grade) = g {
        println!("Student received grade of {:?}", grade);
    } else {
        println!("Grading was unsuccessful");
    }

    // Read the specific course grading 
    let lines = course_scale().join("\n");
    println!("{}", lines);

    let mut rdr = create_csv_reader(lines.as_bytes());
    for result in rdr.deserialize() {
        let record: CourseGrading = result?;
        println!("{:?}", record);
        println!("{:?}", record.percent.to_weight());
    }

    let rdr = create_csv_reader(lines.as_bytes());
    let weights = read_course_weights(rdr);

    println!("{:?}", weights);

    // Calculate the cumulative weighted gpa grading of the student 
    let mut longest = 0usize;
    for cg in &weights.weights {
        if cg.title.len() > longest {
            longest = cg.title.len();
        }
    }
    
    let mut grades = vec!();
    for weight in &weights.weights {
        // Get a line, parse the value into a number
        // let prompt = format!("Enter grade for {}: ", weight.title);
        // let prompt = Prompt::fmt_prompt_post_weight(longest, &weight.title);
        let prompt = Prompt::fmt_prompt_pre_weight(longest, &weight.title, weight.percent.value);
        let input = User::prompt(&prompt);
        
        let grade = input.parse::<u8>()?;
        grades.push(grade);
    }

    let weight_type = GradeWeightType::Post;
    let cumulative: f64 = match weight_type {
        GradeWeightType::Pre => {
            grades
            .into_iter()
            .zip(&weights.weights)
            .map(|(grade , grading)|
                // Formula:
                // 

                grade as f64 * grading.percent.to_weight()
                )
            .sum()
        }
        GradeWeightType::Post => {
            grades
            .into_iter()
            .map(|grade| grade as f64)
            .sum()
        }
    };

    let cumulative = cumulative.round() as u8;
    println!("{}", cumulative);

    // Show the grade of the student
    let lines = gpa_scale().join("\n");
    let mut rdr = create_csv_reader(lines.as_bytes());
    let scale = read_gpa_scale(rdr);

    // let g = scale.get_grade(&input);
    let g = scale.calc_gpa(&cumulative);

    if let Some(grade) = g {
        println!("Student received grade of {:?}", grade);
    } else {
        println!("Grading was unsuccessful");
    }


    // Parse a student's grading and calculate the end gpa based on the weights
    // let data = "
    // ";
    // TODO: Provide future projections

    // println!("Hello, world!");

    Ok(())
}

// Fixtures
// TODO: Use io::Cursor
fn gpa_scale() -> Vec<String> {
    // Letter, Grade Point, Conversion
    vec!(
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
    ).into_iter().map(String::from).collect::<Vec<String>>()
}

fn course_scale() -> Vec<String> {
    vec!(
        "3 Quizzes   , 10%",
        "2 Projects  , 20%",
        "5 Labs      , 20%",
        "Midterm Exam, 20%",
        "Final Exam  , 30%",
    ).into_iter().map(String::from).collect::<Vec<String>>()
}

fn show_lines<T>(mut rdr: Reader<&[u8]>) -> Result<(), Box<dyn Error>>
    where
        T: Debug + DeserializeOwned
{
    for result in rdr.deserialize() {
        let record: T = result?;
        println!("{:?}", record);
    }
    Ok(())
}

enum GradeWeightType {
    Pre,
    Post
}

struct Settings {
}

struct Prompt;

impl Prompt {
    // fn fmt_prompt_pre_weight(width: usize, s: &str, min: u8, max: u8) -> String {
    //     format!("Enter grade for: {:>0width$} [{:>2}-{:>2}%]: ", s, min, max, width = width)
    fn fmt_prompt_pre_weight(width: usize, s: &str, value: u8) -> String {
        format!("Enter grade for: {:>0width$} [0-{:>2}%]: ", s, value, width = width)
    }

    fn fmt_prompt_post_weight(width: usize, s: &str) -> String {
        format!("Enter grade for: {:>0width$} [0-100%]: ", s, width = width)
    }
}

struct User;

impl User {
    fn prompt(prompt: &str) -> String {
        print!("{prompt}");
        Self::input()
    }

    fn input() -> String {
        let input = &mut String::new();
        io::stdout().flush();
        io::stdin().read_line(input);
        input.to_string().trim().to_string()

        // let mut stdin = io::stdin();
        // let input = &mut String::new();
        // input.clear();
        // io::stdout().flush();
        // stdin.read_line(input);
        // input.to_string().trim().to_string()

    }
}

pub type CSVReader<'a> = Reader<&'a [u8]>;

pub fn create_csv_reader(content: &[u8]) -> CSVReader {
    ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .from_reader(content)
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
struct CourseGrading {
    title: String,
    percent: Percent,
}

#[derive(Debug, Clone)]
pub struct CourseGradeWeights {
    weights: Vec<CourseGrading>,
}

pub fn read_course_weights(mut rdr: CSVReader) -> CourseGradeWeights {
    let weights = rdr.deserialize()
        .into_iter()
        .flat_map(Result::ok)
        .collect();
    CourseGradeWeights { weights }
}

pub mod grade {
}

pub mod gpa {
    use std::ops::{Range, RangeInclusive};

    use csv::{Reader, ReaderBuilder, Trim};
    use serde::{Deserialize, Serialize};

    use crate::CSVReader;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GradePoint {
        letter: String,
        grade_point: f64,
        // conversion: Range<(u8, u8)>,
        conversion: RangeInclusive<u8>,
    }

    impl GradePoint {
        fn within(&self, value: &u8) -> bool {
            self.conversion.contains(value)
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GradePointAverageScale {
        scale: Vec<GradePoint>,
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
        let scale = rdr.deserialize()
            .into_iter()
            .flat_map(Result::ok)
            .collect();
        GradePointAverageScale { scale }
    }
}
