use self::parser::parse_csv;
use super::Cpu;

mod lexer;
mod parser;

// This csv was obtained by going to https://ark.intel.com/content/www/us/en/ark/search/featurefilter.html?productType=873,
// selecting a filter that includes every single processor (eg, 1 core to [max] cores) -> Compare all -> Compare -> Export Comparison

// update: I have found the best way to actually get every cpu is to go through the process of loading every cpu onto your webpage, find the table element containing
// all of the cpus -> right click -> create global variable -> iterate over `temp1.children` and extract`element.getAttribute("data-product-id")
// from there, you can generate a list of urls based off of <https://ark.intel.com/content/www/us/en/ark/compare.html?productIds=> and that list of ids
// you generated earlier.
// You can also apparently extract an sqlite DB from the android app <https://github.com/issy/intel-ark-api>, that may be worth looking into,
// although when i checked, it looked slightly out of date

// this list should contain every intel core processor (not ultra) up till 14th gen
// TODO: make this list contain *every* intel processor
const CHUNKS: [&str; 6] = [
    include_str!("chunks/1.csv"),
    include_str!("chunks/2.csv"),
    include_str!("chunks/3.csv"),
    include_str!("chunks/4.csv"),
    include_str!("chunks/5.csv"),
    include_str!("chunks/6.csv"),
];

pub fn get_intel_cpus() -> Vec<Cpu> {
    let mut merged_vec: Vec<Cpu> = Vec::with_capacity(1024);
    for chunk in CHUNKS {
        merged_vec.append(&mut parse_csv(chunk).unwrap());
    }
    merged_vec
}

#[cfg(test)]
mod tests {
    use super::{parser::parse_csv, CHUNKS};

    #[test]
    fn it_work() {
        parse_csv(CHUNKS[0]).unwrap();
    }
}
