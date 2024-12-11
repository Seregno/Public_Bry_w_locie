use std::fs;
use wg_2024::config::Config;

pub trait FromFile {
    fn from_file(path: &str) -> Self;
}

impl FromFile for Config {
    fn from_file(path: &str) -> Self {
        let file_str = fs::read_to_string(path).unwrap();
        toml::from_str(&file_str).unwrap()
    }
}
