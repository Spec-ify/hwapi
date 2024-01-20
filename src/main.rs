
use amd::get_amd_cpus;
use intel::get_intel_cpus;
use levenshtein::levenshtein;

use serde_json::to_string;

mod amd;
mod cpu;
mod intel;

fn main() {
    // parse out intel and amd cpus into a big ol list
    let mut amd_cpus = get_amd_cpus();
    let mut intel_cpus = get_intel_cpus();

    // find the *most* similar cpu name
    let mut scoring: Vec<(usize, String)> = Vec::with_capacity(4096);
    let mut input_str = String::from("AMD Ryzen 5 3600 6-Core Processor");
    // special handling because words like "processor" in the input string throw *everything* off
    let blacklist = ["Processor", "with Radeon", "Graphics", "AMD Ryzen", "Intel"];
    for entry in blacklist {
        input_str = input_str.replace(entry, "");
    }

    // for cpu in cpus {
    //     let name: String= cpu.name.clone();
    //     // levenshtein's similarity algorithm is used to find th the most likely product
    //     scoring.push((levenshtein(&input_str, &cpu.name), name));
    // }
    // scoring.sort_by_key(|k| k.0);
    // // trim the last few entries so it fits on my screen
    // scoring.drain(15..);
    // println!("Searching for: Intel(R) Core(TM) i5-9400F CPU @ 2.90GHz");
    // println!("{:#?}", scoring);
    // for cpu in intel_cpus {
    //     println!("{}", cpu.name);
    // }
    intel_cpus.append(&mut amd_cpus);
    println!("{}", serde_json::to_string(&intel_cpus).unwrap());
}