
use std::io;
use colored::*;


pub enum Level {
    Info,
    Warn,
    Error,
}

pub struct Logger {}

impl Logger {

    fn log(level: Level, label: &str, content: Option<&str>, inline: bool) {
        let sign = match level {
            Level::Info  => ">".green(),
            Level::Warn  => "!".truecolor(255, 130, 0),
            Level::Error => "X".truecolor(255, 20, 0),
        };
        let prefix = match level {
            Level::Warn  => "WARNING ".truecolor(255, 130, 0).bold().to_string(),
            Level::Error => "ERROR ".truecolor(255, 20, 0).bold().to_string(),
            _     => "".to_string(),
        };
        let content = match content {
            Some(content) => format!("{}: {}", label.truecolor(140, 140, 140), content),
            None          => label.to_string()
        };
        match inline {
            true => print!("[{}] {}{}", sign, prefix, content),
            false  => println!("[{}] {}{}", sign, prefix, content),
        }
    }

    pub fn info(label: &str) {
        Logger::log(Level::Info, label, None, false);
    }

    pub fn warn(label: &str) {
        Logger::log(Level::Warn, label, None, false);
    }

    pub fn err(label: &str) {
        Logger::log(Level::Error, label, None, false);
    }

    pub fn info_long(label: &str, content: &str) {
        Logger::log(Level::Info, label, Some(content), false);
    }

    pub fn warn_long(label: &str, content: &str) {
        Logger::log(Level::Warn, label, Some(content), false);
    }

    pub fn err_long(label: &str, content: &str) {
        Logger::log(Level::Error, label, Some(content), false);
    }

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

