use std::collections::{HashMap, HashSet};

use log::{debug, error};
use nom::bytes::complete::{take_until, take_while};
use serde::Serialize;
mod amd;
mod intel;

use amd::get_amd_cpus;
use intel::get_intel_cpus;

/// A generic representation of a cpu. T is the string type, there are massive gains by using zero copy for the intel cpu database, but that's a lot more work
/// for the amd CPU database.
///
/// I know it's awful, leave me alone. -Arc
#[derive(Clone, Debug, Serialize)]
pub struct Cpu<T> {
    /// Something like "Intel core i5-1234 processor"
    pub name: T,
    /// A list of attributes, examples might include a core count of 8, or whether or not a certain feature is enabled
    pub attributes: HashMap<T, T>,
}

#[derive(PartialEq, Clone)]
struct IndexEntry {
    /// The primary identifier for a processor, like:
    /// - `14900` in `i9-14900k`
    model: String,
    /// An identifier applied directly to the beginning of the processor number, like:
    /// - `i7-` in `i7-7600`
    prefix: String,
    /// Identifiers applied to the end of the processor number, like:
    /// - `F` in `i5-11400F`
    suffix: String,
    /// Similar to modifiers, but they're not directly a part of the processor, like:
    /// `PRO` in `Ryzen 5 PRO 5600`
    tags: HashSet<String>,
    /// The index of the corresponding cpu in the "full" vec
    index: usize,
}

#[derive(Clone)]
pub struct CpuCache<'a> {
    intel_cpus: Vec<Cpu<&'a str>>,
    intel_index: Vec<IndexEntry>,
    amd_cpus: Vec<Cpu<String>>,
    amd_index: Vec<IndexEntry>,
}

impl CpuCache<'_> {
    /// Create a new cache and parse the cpu databases into memory
    pub fn new() -> Self {
        let intel_cpus = get_intel_cpus();
        debug!("Intel CPU list deserialized");
        let mut intel_index: Vec<IndexEntry> = Vec::with_capacity(2048);
        for (i, cpu) in intel_cpus.iter().enumerate() {
            match generate_index_entry(&cpu.name, i) {
                Ok(idx) => {
                    intel_index.push(idx);
                }
                Err(e) => {
                    error!("index will not be complete because generation generation failed for cpu: {:?} with error {:?}", cpu.name, e);
                }
            }
        }
        debug!("Index generated for Intel CPUs");
        let amd_cpus = get_amd_cpus();
        debug!("Amd CPU list deserialized");
        let mut amd_index: Vec<IndexEntry> = Vec::with_capacity(2048);
        for (i, cpu) in amd_cpus.iter().enumerate() {
            match generate_index_entry(&cpu.name, i) {
                Ok(idx) => {
                    amd_index.push(idx);
                }
                Err(e) => {
                    error!("index will not be complete because generation generation failed for cpu: {:?} with error {:?}", cpu.name, e);
                }
            }
        }

        Self {
            intel_cpus,
            intel_index,
            amd_cpus,
            amd_index,
        }
    }

    /// Given a string that contains the inexact name of a cpu, try to find the best fit
    /// and return it. For example, it might take an input of "AMD Ryzen 5 3600 6-Core Processor",
    /// and return the entry with a `name` of "AMD Ryzen™ 5 3600".
    ///
    /// A mutable reference is required so that the comparison cache can be shared between calls
    pub fn find<'a>(
        &'a mut self,
        input: &'a str,
    ) -> Result<Cpu<String>, Box<dyn std::error::Error + '_>> {
        let index = if input.contains("AMD") {
            &self.amd_index
        } else {
            &self.intel_index
        };
        let idx_for_input = generate_index_entry(input, 0)?;
        // first look for an index entry that has an exact match for the processor model number
        let similar_cpus = index.iter().filter(|idx| idx.model == idx_for_input.model);
        // now find the closest fit among all similar cpus
        // a higher score indicates a closer match
        let mut best_score = -100;
        let mut best_idx_match: Option<&IndexEntry> = None;
        for idx_entry in similar_cpus {
            let mut score: i32 = 0;
            // if the prefix doesn't match, dock points
            if idx_for_input.prefix != idx_entry.prefix {
                score -= 10;
            }
            // if the suffix doesn't match, dock points
            if idx_for_input.suffix != idx_entry.suffix {
                score -= 10;
            }
            // for every matching tag that both entries have, give points
            // points are not currently docked if the entry is missing tags that the input has
            for tag in &idx_for_input.tags {
                if idx_entry.tags.contains(tag) {
                    score += 5;
                }
            }
            // update the best fit if a better fit was found
            if score > best_score {
                best_score = score;
                best_idx_match = Some(idx_entry);
            }
        }
        // let cpus: &Vec<Cpu<_>> = if input.contains("AMD") {
        //     &self.amd_cpus
        // } else {
        //     &self.intel_cpus
        // };
        match best_idx_match {
            None => {
                error!("When searching for cpu {:?}, no cpus were found with a matching model number of: {:?}", input, idx_for_input.model);
                return Err(Box::from("No close matches found"));
            }
            Some(idx_entry) => {
                if input.contains("AMD") {
                    return Ok(self.amd_cpus[idx_entry.index].clone());
                }
                // intel requires some work to un-zerocopy data
                let found_cpu = &self.intel_cpus[idx_entry.index];
                return Ok(Cpu {
                    name: found_cpu.name.to_string(),
                    attributes: found_cpu
                        .attributes
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                });
            }
        }
    }
}

/// Take the input model name, and try to parse it into an [IndexEntry] with an index of `index`.
fn generate_index_entry<'name>(
    name: &str,
    index: usize,
) -> Result<IndexEntry, Box<dyn std::error::Error + '_>> {
    let model_token = find_model(name);
    // find the prefix, if one exists
    let prefix_combinator: (&str, &str) =
        match take_until::<_, _, nom::error::Error<_>>("-")(model_token.as_str()) {
            Ok(p) => p,
            Err(_) => {
                // assume there's no prefix
                ("", &model_token)
            }
        };
    let model_combinator: (&str, &str) = match take_while::<_, _, nom::error::Error<_>>(
        |c: char| c.is_ascii_digit(),
    )(prefix_combinator.1)
    {
        Ok(m) => m,
        Err(_) => {
            return Err(Box::from(format!("index generation failed for cpu {:?} because no base 10 digits were found after the prefix", name)));
        }
    };
    // tags are considered anything *but* the name, and a few keywords
    let blacklist = ["Intel", "AMD", "Processor", name];
    let mut tags: HashSet<String> = HashSet::new();
    for tag in name.split(' ').filter(|t| !blacklist.contains(t)) {
        tags.insert(tag.to_string());
    }

    Ok(IndexEntry {
        model: String::from(model_combinator.0),
        prefix: String::from(prefix_combinator.0),
        // just whatever is leftover after the combinator
        suffix: String::from(model_combinator.1),
        tags,
        index,
    })
}

/// Search the input string for the section that refers to the model of a CPU.
/// For example, given an input string of "AMD Ryzen 5 3600", it would try to return "3600".
/// This function does return the whole token associated with a model, so prefixes and suffixes
/// are included
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
            let found_cpu = cache.find(pairing.1).unwrap();
            assert_eq!(found_cpu.name, pairing.0, "With an input of {:?}, a database result of {:?} was expected, while {:?} was returned instead.", pairing.1, pairing.0, found_cpu.name);
        }
    }
}
