use std::error::Error;
use std::fmt::Debug;
use std::fs;
use std::path::Path;

use cgpa::course::{read_course_weights, CourseScale};
use cgpa::fmt;
use cgpa::gpa::{read_gpa_scale, GPAScale};
use cgpa::tui::{Prompt, TUI};
use clap::Parser;
use cli::{GradeType, CLI};
use log::{debug, error, info, trace, warn};
use simple_logger::SimpleLogger;

// Extra:
// - Simple Calculator?
// - Switch to pre/post weights (0-100% scale vs 10% or 20% etc)
// - Prediction mode
// - Experimental mode to calculate grade from assignment basis to final
//   cumulative
// grade
// - C shared library bindings?
fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::parse();

    // Enable app debug info
    if cli.global_opts.debug {
        SimpleLogger::new()
            // .with_level(log::LevelFilter::Debug)
            .init().unwrap();
    }

    // Set a default weight type
    let weight_type = match cli.app_opts.weight_type {
        GradeType { before: true, after: false, } => {
            info!("[CLI] Set opt weight_type: pre");
            GradeWeightType::Pre
        },
        GradeType { before: false, after: true, } | _ => {
            info!("[CLI] Set opt weight_type: post");
            GradeWeightType::Post
        }
    };
    let fp_gpa_scale = cli.app_opts.gpa_scale;
    let fp_course_scale = cli.app_opts.course_scale;

    let gpa_scale = load_gpa_scale(&fp_gpa_scale)?;
    let course_scale = load_course_scale(&fp_course_scale)?;

    // Calculate the cumulative weighted gpa grading of the student
    let prompts = prep_user_prompts(&weight_type, &course_scale);
    let grades = get_user_grades(prompts);
    let cumulative = calc_user_grade(weight_type, &course_scale, grades);

    // Show the grade of the student
    show_gpa(&cumulative, &gpa_scale);

    Ok(())
}

fn calc_user_grade(weight_type: GradeWeightType, course_scale: &CourseScale, grades: Vec<u8>) -> u8 {
    info!("Calculating student gpa");
    let cumulative: f64 = match weight_type {
        GradeWeightType::Pre => {
            grades
                .into_iter()
                .zip(&course_scale.weights)
                .map(|(grade , grading)|
                grade as f64 * grading.percent.to_weight())
                .sum()
        }
        GradeWeightType::Post => grades.into_iter().map(|grade| grade as f64).sum(),
    };
    debug!("Cumulative GPA: {}", cumulative);

    let cumulative = cumulative.round() as u8;
    cumulative
}

fn get_user_grades(prompts: Vec<String>) -> Vec<u8> {
    println!("Please enter the grades for: ");

    let grades: Vec<_> = prompts.into_iter()
        .map(|p| TUI::prompt(&p))
        .map(|input| input.parse::<u8>())
        .collect();

    info!("Validating user input");
    let grades = grades.into_iter()
        .inspect(|res| warn!("{:?}", res))
        .flat_map(Result::ok)
        .collect::<Vec<u8>>();
    grades
}

fn prep_user_prompts(weight_type: &GradeWeightType, course_scale: &CourseScale) -> Vec<String> {
    info!("Preparing user prompts...");
    let longest_title = course_scale.weights.iter().map(|cg| cg.title.len()).max().unwrap_or(0);

    let prompts = &course_scale.weights.iter()
        .map(|cg| {
            match *weight_type {
                GradeWeightType::Pre => Prompt::fmt_prompt_pre_weight(longest_title, &cg.title),
                GradeWeightType::Post => Prompt::fmt_prompt_post_weight(longest_title, &cg.title, cg.percent.value),
        }
    }).collect::<Vec<String>>();
    prompts.iter().for_each(|p| trace!("{:?}", p));
    prompts.to_owned()
}

fn load_course_scale(fp_course_scale: &Path) -> Result<CourseScale, Box<dyn Error>> {
    info!("Loading course grading scale...");
    let lines = fs::read_to_string(fp_course_scale)?;
    let rdr = fmt::create_csv_reader(lines.as_bytes());
    let course_scale = read_course_weights(rdr);
    course_scale.weights.iter().for_each(|cg| trace!("{:?}", cg));
    course_scale.weights.iter().for_each(|cg| trace!("{:?}", cg.percent.to_weight()));
    info!("Course Scale Loaded!");
    Ok(course_scale)
}

fn load_gpa_scale(fp_gpa_scale: &Path) -> Result<GPAScale, Box<dyn Error>> {
    info!("Loading gpa grading scale...");
    let lines = fs::read_to_string(fp_gpa_scale)?;
    let rdr = fmt::create_csv_reader(lines.as_bytes());
    let gpa_scale = read_gpa_scale(rdr);
    gpa_scale.scale.iter().for_each(|gp| trace!("{:?}", gp));
    info!("GPA Scale Loaded!");
    Ok(gpa_scale)
}

/// Show a student's gpa for a course
fn show_gpa(grade: &u8, scale: &GPAScale) {
    if let Some(gpa) = scale.calc_gpa(grade) {
        println!("Grade: {}%", grade);
        println!("GPA  : {:?}", gpa);
    } else {
        let err = "Error: Unable to calculate course gpa for student.";
        eprintln!("{}", err);
        error!("{}", err);
    }
}

#[derive(Debug)]
enum GradeWeightType {
    Pre,
    Post,
}

pub mod cli {
    use std::path::PathBuf;

    use clap::{ArgGroup, Args, Parser};

    #[derive(Debug, Args)]
    pub struct GlobalOpts {
        /// Toggle verbose app information
        #[arg(short, long, default_value_t = false)]
        pub debug: bool,

        /// Silence all app errors & warnings
        #[arg(help = "Disable app warnings")]
        #[arg(short, long, default_value_t = false)]
        pub quiet: bool,
    }

    #[derive(Debug, Args)]
    pub struct AppOpts {
        /// Set grading weight type
        #[command(flatten)]
        pub weight_type: GradeType,

        /// Set gpa grading scale from input file
        #[arg(short, long, aliases = ["gpa"])]
        pub gpa_scale: PathBuf,

        /// Set course grading scale from input file
        #[arg(short, long, aliases = ["course"])]
        pub course_scale: PathBuf,

        /// Set student grades from input file
        pub grades: Option<PathBuf>,
    }

    // NOTE: clap-rs issue 2621
    #[derive(Debug, Args)]
    #[clap(group(
        ArgGroup::new("grade_type")
            .multiple(false)
            .args(&["before", "after"]),
            ))]
    pub struct GradeType {
        /// Calculate gpa before course scaling is applied
        #[clap(short, long, default_value_t = true)]
        pub before: bool,
        /// Calculate gpa after course scaling is applied
        #[clap(short, long)]
        pub after: bool,
    }

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

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use cgpa::*;
    use super::*;

    // Fixtures
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

    #[test]
    fn test_gpa_scale() {
        let a_plus = gpa::GradePoint::new("A+".to_string(), 4.33, 90..=100);
        let a = gpa::GradePoint::new("A".to_string(), 4.00, 85..=89);

        info!("Loaded the gpa grading scale");
        let lines = gpa_scale().join("\n");
        let rdr = fmt::create_csv_reader(lines.as_bytes());
        let scale = read_gpa_scale(rdr);
        scale.scale.iter().for_each(|gp| trace!("{:?}", gp));

        assert_eq!(scale.scale.get(0), Some(&a_plus));

        // Check that the bounds are correct for the grading scales
        assert_eq!(scale.calc_gpa(&90), Some(a_plus.clone()));
        assert_eq!(scale.calc_gpa(&100), Some(a_plus.clone()));
        assert_eq!(scale.calc_gpa(&89), Some(a));
    }

    #[test]
    fn test_course_scale() {
        let quizzes = course::CourseGrading {
            title: "3 Quizzes".to_string(),
            percent: fmt::Percent { percent: "10%".to_string(), value: 10 },
        };

        info!("Loading course grading scale...");
        let lines = course_scale().join("\n");
        let rdr = fmt::create_csv_reader(lines.as_bytes());
        let weights = read_course_weights(rdr);
        weights.weights.iter().for_each(|cg| trace!("{:?}", cg));
        weights.weights.iter().for_each(|cg| trace!("{:?}", cg.percent.to_weight()));
        info!("Course Scale Loaded!");

        assert_eq!(weights.weights.get(0), Some(&quizzes));
    }
}
