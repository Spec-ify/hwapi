use std::collections::HashMap;

use levenshtein::levenshtein;
use log::{debug, trace};
use serde::Serialize;
mod amd;
mod intel;

use amd::get_amd_cpus;
use intel::get_intel_cpus;

/// A generic representation of a cpu
#[derive(Clone, Debug, Serialize)]
pub struct Cpu {
    /// Something like "Intel core i5-1234 processor"
    pub name: String,
    /// A list of attributes, examples might include a core count of 8, or whether or not a certain feature is enabled
    pub attributes: HashMap<String, String>,
}

#[derive(Clone)]
pub struct CpuCache {
    intel_cpus: Vec<Cpu>,
    amd_cpus: Vec<Cpu>,
    comparison_cache: HashMap<String, String>,
}

impl CpuCache {
    /// Create a new cache and parse the cpu databases into memory
    pub fn new() -> Self {
        let intel_cpus = get_intel_cpus();
        debug!("Intel CPU list deserialized");
        let amd_cpus = get_amd_cpus();
        debug!("Amd CPU list deserialized");

        Self {
            intel_cpus,
            amd_cpus,
            comparison_cache: HashMap::new(),
        }
    }

    /// Given a string that contains the inexact name of a cpu, try to find the best fit
    /// and return it. For example, it might take an input of "AMD Ryzen 5 3600 6-Core Processor",
    /// and return the entry with a `name` of "AMD Ryzen™ 5 3600".
    ///
    /// A mutable reference is required so that the comparison cache can be shared between calls
    pub fn find(&mut self, input: &str) -> Cpu {
        let input_model = find_model(input);
        // a list of CPUs, and the most likely
        let cpus = if input.contains("AMD") {
            &self.amd_cpus
        } else {
            &self.intel_cpus
        };
        // first see if a comparison has already been made
        if let Some(cpu_name) = self.comparison_cache.get(input) {
            return cpus
                .into_iter()
                .filter(|cpu| cpu.name == cpu_name.to_string())
                .nth(0)
                .unwrap()
                .clone();
        }
        // performing a full search if the cpu isn't found in the cache
        let mut best_fit = Cpu {
            name: "FUBAR".to_string(),
            attributes: HashMap::new(),
        };
        let mut best_score: usize = 10000;
        trace!("input model: {}", input_model);
        for cpu in cpus {
            let model: String = find_model(&cpu.name).to_string();
            // levenshtein distance is used to figure out how similar two strings are
            // 0 means that they're identical, the higher the number, the less similar they are
            let score = levenshtein(&input_model, &model);
            if score < best_score {
                best_score = score;
                best_fit = cpu.clone();

                trace!(
                    "Best fit of {} found, with a score of {}",
                    best_fit.name,
                    best_score
                );
            }
        }
        self.comparison_cache
            .insert(input.to_string(), best_fit.name.clone());
        debug!(
            "Given the input: {:?}, the CPU {:?} was found",
            input, best_fit.name
        );
        best_fit
    }
}

/// Search the input string for the section that refers to the model of a CPU.
/// For example, given an input string of "AMD Ryzen 5 3600", it would try to return "3600"
fn find_model(input: &str) -> String {
    let mut best_fit = "";
    let mut high_score: isize = -10;
    for token in input.split(' ') {
        let score = calculate_model_score(token);
        if score > high_score {
            best_fit = token;
            high_score = score;
        }
    }
    // because some edge cases exist where the model is either vaguely reported or split among multiple tokens, those are handled here
    // they are organized by blocks, each block should contain an explanation and a solution

    // 14th gen intel CPUs are reported in the form of `iX processor 14XYZ` by the database, but they're reported as
    // iX-14XYZ by the WMI. For now, this is handled by hacking iX and 14XYZ together if the case is detected
    {
        if input.contains("Intel") && best_fit.starts_with("14") {
            let tokens = input.split(' ');
            let i_tag = tokens.filter(|t| t.len() == 2 && t.starts_with('i')).nth(0);
            if let Some(t) = i_tag {
                return format!("{}-{}", t, best_fit);
            }
        }
    }

    // Ryzen PRO cpus have the same model, they're a different lineup though.
    // This is handled by taping PRO to the model
    {
        if input.contains("AMD") && input.contains("PRO") {
            return format!("PRO {}", best_fit);
        }
    }

    // Intel Core iX-123M processors are sometimes represented in the format of
    // iX CPU M 123
    {
        if input.contains("Intel") && input.contains(" M ") && best_fit.len() == 3 {
            let tokens = input.split(" ");
            let i_tag = tokens.filter(|t| t.len() == 2 && t.starts_with('i')).nth(0);
            if let Some(t) = i_tag {
                return format!("{}-{}M", t, best_fit);
            }
        }
    }

    best_fit.to_string()
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
        if c.is_ascii_digit() {
            score += 2;
        }
        if blacklist.contains(&c) {
            score -= 4;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::CpuCache;

    #[test]
    fn search_resilience() {
        let mut cache = CpuCache::new();
        // on the left is the name stored in the cache, on the right is the name collected from WMI data
        // these test cases should be filled out as time goes on with failing test cases
        // any test cases commented out currently fail
        let pairings = [
            (
                "AMD Ryzen™ 5 3400G with Radeon™ RX Vega 11 Graphics",
                "AMD Ryzen 5 3400G with Radeon Vega Graphics",
            ),
            (
                "AMD Ryzen™ 5 PRO 4650G",
                "AMD Ryzen 5 PRO 4650G with Radeon Graphics",
            ),
            (
                "Intel® Core™ i3-9100F Processor",
                "Intel(R) Core(TM) i3-9100F CPU @ 3.60GHz",
            ),
            ("AMD Ryzen™ 5 5600", "AMD Ryzen 5 5600 6-Core Processor"),
            ("AMD Ryzen™ 5 2600", "AMD Ryzen 5 2600 Six-Core Processor"),
            ("AMD Ryzen™ 5 7600", "AMD Ryzen 5 7600 6-Core Processor"),
            (
                "AMD Ryzen™ 5 7530U",
                "AMD Ryzen 5 7530U with Radeon Graphics",
            ),
            (
                "Intel® Core™ i9-9900K Processor",
                "Intel(R) Core(TM) i9-9900K CPU @ 3.60GHz",
            ),
            (
                "Intel® Core™ i7 processor 14700K",
                "Intel(R) Core(TM) i7-14700K",
            ),
            (
                "Intel® Core™ i7-620M Processor",
                "Intel(R) Core(TM) i7 CPU M 620 @ 2.67Ghz",
            ),
        ];

        for pairing in pairings {
            let found_cpu = cache.find(pairing.1);
            assert_eq!(found_cpu.name, pairing.0, "With an input of {:?}, a database result of {:?} was expected, while {:?} was returned instead.", pairing.1, pairing.0, found_cpu.name);
        }
    }
}
