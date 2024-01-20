use self::parser::parse_csv;
use super::cpu::Cpu;

mod lexer;
mod parser;

// This csv was obtained by going to https://ark.intel.com/content/www/us/en/ark/search/featurefilter.html?productType=873,
// selecting a filter that includes every single processor (eg, 1 core to [max] cores) -> Compare all -> Compare -> Export Comparison
// note: you need to actually manually load everything in for the csv to contain everything
const FILE_CONTENTS: &str = include_str!("input.csv");

pub fn get_intel_cpus() -> Vec<Cpu> {
    parse_csv(FILE_CONTENTS).unwrap()
}

#[cfg(test)]
mod tests {
    use super::{parser::parse_csv, FILE_CONTENTS};

    #[test]
    fn it_works() {
        parse_csv(FILE_CONTENTS).unwrap();
    }
}
