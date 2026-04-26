use std::io::{self, IsTerminal};

#[derive(Debug, Clone, Copy)]
pub struct Logger {
    color: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            color: io::stdout().is_terminal(),
        }
    }

    pub fn info(&self, topic: &str, message: &str) {
        self.line("ok", topic, message);
    }

    pub fn warn(&self, topic: &str, message: &str) {
        self.line("warn", topic, message);
    }

    fn line(&self, level: &str, topic: &str, message: &str) {
        if self.color {
            let level_color = match level {
                "ok" => "\x1b[38;5;114m",
                "warn" => "\x1b[38;5;221m",
                _ => "\x1b[38;5;250m",
            };
            println!(
                "\x1b[38;5;75mwave\x1b[0m {level_color}{level:<4}\x1b[0m \x1b[38;5;246m{topic:<8}\x1b[0m {message}"
            );
        } else {
            println!("wave {level:<4} {topic:<8} {message}");
        }
    }
}
