use std::error::Error;
use std::fmt::Debug;

use cgpa::course::{read_course_weights, CourseGrading};
use cgpa::fmt;
use cgpa::gpa::{read_gpa_scale, GradePoint};
use cgpa::tui::{Prompt, TUI};
use clap::Parser;
use cli::{GradeType, CLI};
use csv::Reader;
use serde::de::DeserializeOwned;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::parse();

    // Set a default weight type
    match cli.app_opts.weight_type {
        GradeType { before: _, after: true, } => {
            println!("Set post weight to after");
        }
        GradeType { before: true, after: _, } | _ => {
            println!("Set post weight to before");
        }
    }

    // TODO:
    // Core:
    // - Updated prompts:
    // - File IO
    // - Simple CLI
    // Extra:
    // - Simple Calculator?
    // - Switch to pre/post weights (0-100% scale vs 10% or 20% etc)
    // - Prediction mode
    // - Experimental mode to calculate grade from assignment basis to final
    //   cumulative
    // grade
    // - C shared library bindings

    // Read the specific gpa scaling
    let lines = gpa_scale().join("\n");
    let rdr = fmt::create_csv_reader(lines.as_bytes());
    show_lines::<GradePoint>(rdr)?;
    let rdr = fmt::create_csv_reader(lines.as_bytes());
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

    let mut rdr = fmt::create_csv_reader(lines.as_bytes());
    for result in rdr.deserialize() {
        let record: CourseGrading = result?;
        println!("{:?}", record);
        println!("{:?}", record.percent.to_weight());
    }

    let rdr = fmt::create_csv_reader(lines.as_bytes());
    let weights = read_course_weights(rdr);

    println!("{:?}", weights);

    // Calculate the cumulative weighted gpa grading of the student
    let mut longest = 0usize;
    for cg in &weights.weights {
        if cg.title.len() > longest {
            longest = cg.title.len();
        }
    }

    let mut grades = vec![];
    for weight in &weights.weights {
        // Get a line, parse the value into a number
        // let prompt = format!("Enter grade for {}: ", weight.title);
        // let prompt = Prompt::fmt_prompt_post_weight(longest, &weight.title);
        let prompt = Prompt::fmt_prompt_pre_weight(longest, &weight.title, weight.percent.value);
        let input = TUI::prompt(&prompt);

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

                grade as f64 * grading.percent.to_weight())
                .sum()
        }
        GradeWeightType::Post => grades.into_iter().map(|grade| grade as f64).sum(),
    };

    let cumulative = cumulative.round() as u8;
    println!("{}", cumulative);

    // Show the grade of the student
    let lines = gpa_scale().join("\n");
    let rdr = fmt::create_csv_reader(lines.as_bytes());
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
    vec![
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
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<String>>()
}

fn course_scale() -> Vec<String> {
    vec![
        "3 Quizzes   , 10%",
        "2 Projects  , 20%",
        "5 Labs      , 20%",
        "Midterm Exam, 20%",
        "Final Exam  , 30%",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<String>>()
}

fn show_lines<T>(mut rdr: Reader<&[u8]>) -> Result<(), Box<dyn Error>>
where
    T: Debug + DeserializeOwned,
{
    for result in rdr.deserialize() {
        let record: T = result?;
        println!("{:?}", record);
    }
    Ok(())
}

#[derive(Debug)]
enum GradeWeightType {
    Pre,
    Post,
}

pub mod cli {
    // struct Settings {}

    use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};

    #[derive(Debug, Args)]
    pub struct GlobalOpts {
        /// Toggle to display detailed app runtime logging info
        #[arg(help = "Toggle verbose information")]
        #[arg(short, long, default_value_t = false)]
        pub verbose: bool,

        /// Silence all app errors & warnings
        #[arg(help = "Disable app warnings")]
        #[arg(short, long, default_value_t = false)]
        pub quiet: bool,
    }

    #[derive(Debug, Args)]
    pub struct AppOpts {
        /// Toggle to display detailed app runtime logging info
        #[command(flatten)]
        pub weight_type: GradeType,
    }

    // NOTE: clap-rs issue 2621
    #[derive(Debug, Args)]
    #[clap(group(
        ArgGroup::new("grade_type")
            .multiple(false)
            .args(&["before", "after"]),
            ))]
    pub struct GradeType {
        #[clap(short, long, default_value_t = true)]
        pub before: bool,
        #[clap(short, long)]
        pub after: bool,
    }
    // pub enum GradeType {
    //     #[clap(short)]
    //     Before,
    //     #[clap(short)]
    //     After,
    // }

    // cli
    // -g : GPA Scale
    // -c : Course Scale
    // -b : Before / pre weight
    // -a : After  / post weight
    // [arg] : Optional Student Grades
    #[derive(Debug, Parser)]
    #[command(about, author, version)]
    pub struct CLI {
        #[clap(flatten)]
        pub global_opts: GlobalOpts,

        #[clap(flatten)]
        pub app_opts: AppOpts,
    }
}
