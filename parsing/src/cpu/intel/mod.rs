use self::parser::parse_csv;
use super::Cpu;

mod lexer;
mod parser;

// This csv was obtained by going to https://ark.intel.com/content/www/us/en/ark/search/featurefilter.html?productType=873,
// selecting a filter that includes every single processor (eg, 1 core to [max] cores) -> Compare all -> Compare -> Export Comparison

// update: I have found the best way to actually get every cpu is to go through the process of loading every cpu onto your webpage, find the table element containing
// all of the cpus -> right click -> Store as global variable -> iterate over `temp1.children` and extract`element.getAttribute("data-product-id")
// from there, you can generate a list of urls based off of <https://ark.intel.com/content/www/us/en/ark/compare.html?productIds=> and that list of ids
// you generated earlier.
// You can also apparently extract an sqlite DB from the android app <https://github.com/issy/intel-ark-api>, that may be worth looking into,
// although when i checked, it looked slightly out of date
/*
Below is copy-pasteable javascript to export stuff
```javascript
// (in chrome) open inspect element, select a table element, and look for the `tbody`
// that contains all of the cpu `tr`s, right click -> store as global variable.
// this should create `temp1` in your terminal
// then paste the below in:
let ids = [];
for (const element of temp1.children) {
    ids.push(element.getAttribute("data-product-id"));
}
console.log(ids.join(","));
```
Below is copy-pasteable javascript to generate a list of urls from that
wall of ids you found (i used deno)
```
let ids = [PASTE_IDS_HERE];
// https://stackoverflow.com/questions/8495687/split-array-into-chunks
 let chunks = ids.reduce((all,one,i) => {
   const ch = Math.floor(i/200);
   all[ch] = [].concat((all[ch]||[]),one);
   return all
}, []);

for (const chunk of chunks) {
    console.log("https://ark.intel.com/content/www/us/en/ark/compare.html?productIds=" + chunk.join(","));
}
```
*/

// this list should contain every intel processor till the beginning of 2024
const CHUNKS: [&str; 16] = [
    include_str!("chunks/1.csv"),
    include_str!("chunks/2.csv"),
    include_str!("chunks/3.csv"),
    include_str!("chunks/4.csv"),
    include_str!("chunks/5.csv"),
    include_str!("chunks/6.csv"),
    include_str!("chunks/7.csv"),
    include_str!("chunks/8.csv"),
    include_str!("chunks/9.csv"),
    include_str!("chunks/10.csv"),
    include_str!("chunks/11.csv"),
    include_str!("chunks/12.csv"),
    include_str!("chunks/13.csv"),
    include_str!("chunks/14.csv"),
    include_str!("chunks/15.csv"),
    include_str!("chunks/16.csv"),
];

pub fn get_intel_cpus() -> Vec<Cpu<&'static str>> {
    let mut merged_vec: Vec<Cpu<&str>> = Vec::with_capacity(1024);
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
        parse_csv(CHUNKS[15]).unwrap();
    }
}
