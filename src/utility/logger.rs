
use std::io;
use colored::*;


pub enum Level {
    Info,
    Warn,
    Error,
}

pub struct Logger {}

impl Logger {

    #[allow(unused)]
    fn log(level: Level, label: &str, content: Option<&str>, inline: bool) {
        let prefix = match level {
            Level::Info  => "INFO".green(),
            Level::Warn  => "WARN".truecolor(255, 130, 0),
            Level::Error => "ERROR".truecolor(255, 20, 0),
        };
        let content = match content {
            Some(content) => format!("{}: {}", label.truecolor(140, 140, 140), content),
            None          => label.to_string()
        };
        match inline {
            true  => print!("[{}] {}", prefix, content),
            false => println!("[{}] {}", prefix, content),
        }
    }

    #[allow(unused)]
    pub fn info(label: &str) {
        Logger::log(Level::Info, label, None, false);
    }

    #[allow(unused)]
    pub fn warn(label: &str) {
        Logger::log(Level::Warn, label, None, false);
    }

    #[allow(unused)]
    pub fn err(label: &str) {
        Logger::log(Level::Error, label, None, false);
    }

    #[allow(unused)]
    pub fn info_long(label: &str, content: &str) {
        Logger::log(Level::Info, label, Some(content), false);
    }

    #[allow(unused)]
    pub fn warn_long(label: &str, content: &str) {
        Logger::log(Level::Warn, label, Some(content), false);
    }

    #[allow(unused)]
    pub fn err_long(label: &str, content: &str) {
        Logger::log(Level::Error, label, Some(content), false);
    }

    #[allow(unused)]
    pub fn input(label: &str) -> String {

        // input label
        Logger::log(Level::Info, label, Some(""), true);

        // read input
        std::io::Write::flush(&mut io::stdout()).unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()

    }

}

