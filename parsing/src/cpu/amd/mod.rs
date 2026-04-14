use self::parser::parse_csv;
use super::Cpu;

mod lexer;
mod parser;

// input.csv was obtained by going to https://www.amd.com/en/products/specifications/processors, and hitting the export icon above the cpu table

const INPUT_FILE: &str = include_str!("input.csv");

pub fn get_amd_cpus() -> Vec<Cpu<&'static str>> {
    let mut merged_vec: Vec<Cpu<&str>> = Vec::with_capacity(1024);
    merged_vec.append(&mut parse_csv(INPUT_FILE).unwrap());
    merged_vec
}

#[cfg(test)]
mod tests {
    use super::{parser::parse_csv, INPUT_FILE};

    #[test]
    fn it_work() {
        parse_csv(INPUT_FILE).unwrap();
    }
}
