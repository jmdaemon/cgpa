use std::io::{self, Write};

pub struct Prompt;

impl Prompt {
    // fn fmt_prompt_pre_weight(width: usize, s: &str, min: u8, max: u8) -> String {
    //     format!("Enter grade for: {:>0width$} [{:>2}-{:>2}%]: ", s, min, max,
    // width = width)
    pub fn fmt_prompt_post_weight(width: usize, s: &str, value: u8) -> String {
        format!(
            "Enter grade for: {:>0width$} [0-{:>2}%]: ",
            s,
            value,
            width = width
        )
    }

    pub fn fmt_prompt_pre_weight(width: usize, s: &str) -> String {
        format!("Enter grade for: {:>0width$} [0-100%]: ", s, width = width)
    }
}

pub struct TUI;

impl TUI {
    pub fn prompt(prompt: &str) -> String {
        print!("{prompt}");
        Self::input()
    }

    pub fn input() -> String {
        let input = &mut String::new();
        let _ = io::stdout().flush();
        let _ = io::stdin().read_line(input);
        input.to_string().trim().to_string()

        // let mut stdin = io::stdin();
        // let input = &mut String::new();
        // input.clear();
        // io::stdout().flush();
        // stdin.read_line(input);
        // input.to_string().trim().to_string()
    }
}
