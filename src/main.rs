
use amd::get_amd_cpus;
use intel::get_intel_cpus;
use levenshtein::levenshtein;

use serde_json::to_string;

mod amd;
mod cpu;
mod intel;

fn main() {
    // parse out intel and amd cpus into a big ol list
    let amd_cpus = get_amd_cpus();
    let intel_cpus = get_intel_cpus();

    // find the *most* similar cpu name
    let input_str = String::from("AMD Ryzen 5 3600 6-Core Processor");
    let input_model = find_model(&input_str);
    // find the most likely cpu
    let mut scoring: Vec<(usize, String)> = Vec::with_capacity(4096);
    let cpus = if input_str.contains("AMD") {
        amd_cpus
    } else {
        intel_cpus
    };

    for cpu in cpus {
        let model: String = find_model(&cpu.name).to_string();
        // levenshtein's similarity algorithm is used to find th the most likely product
        scoring.push((levenshtein(&input_model, &model), cpu.name));
    }
    scoring.sort_by_key(|k| k.0);
    // trim the last few entries so it fits on my screen
    scoring.drain(15..);
    println!("Searching for: {:?}", input_str);
    println!("{:#?}", scoring);

    // for cpu in amd_cpus {
    //     println!("{}\n{}\n", cpu.name, find_model(&cpu.name));
    // }
    // for cpu in intel_cpus {
    //     println!("{}\n{}", cpu.name, find_model(&cpu.name));
    // }
}

fn find_model(input: &str) -> &str {
    let mut best_fit = "";
    let mut high_score: isize = -10;
    for token in input.split(" ") {
        let score = calculate_model_score(token);
        if score > high_score {
            best_fit = token;
            high_score = score;
        }
    }
    best_fit
}

/// This function tries to determine the likelihood that the given token is the "model" of a cpu.
/// For example, with the string "Intel(R) Core(TM) i5-9400F CPU @ 2.90GHz", the token "i5-9400F"
/// would be given the highest score, while tokens like "Intel(R)" would ideally be given a significantly lower score 
fn calculate_model_score(token: &str) -> isize {
    // The theory is that any token that contains numbers is more likely to be a model number,
    // and any token that contains characters that aren't likely to exist in a model are less
    // likely to be a model number
    let mut score: isize = 0;
    let blacklist = ['.', '(', ')'];
    for c in token.chars() {
        if c.is_digit(10) {
            score += 2;
        }
        if blacklist.contains(&c) {
            score -= 4;
        }
    }
    score
}