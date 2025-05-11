use std::io::{self, Write};

pub struct Prompt;

impl Prompt {
    pub fn fmt_prompt_post_weight(width: usize, s: &str, value: u8) -> String {
        format!(
            "{} {:>0width$} [0-{:>2}%]: ",
            " ".repeat(3),
            s,
            value,
            width = width
        )
    }

    pub fn fmt_prompt_pre_weight(width: usize, s: &str) -> String {
        format!("{} {:>0width$} [0-100%]: ", " ".repeat(3), s, width = width)
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
    }
}
