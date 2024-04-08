use crate::cpu::Cpu;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

// input.json was obtained by going to https://www.amd.com/en/products/specifications/processors, and hitting the export icon -> json
// the site will probably freeze for like 30 seconds, but it'll eventually spit out a json
const FILE_CONTENTS: &str = include_str!("input.json");

#[derive(Deserialize)]
struct AmdJson {
    data: Vec<HashMap<String, serde_json::Value>>,
}

pub fn get_amd_cpus() -> Vec<Cpu<String>> {
    let deserialized_json = deserialize_json();
    process_json(deserialized_json)
}

/// Read the json from the file into an [AmdJson]
fn deserialize_json() -> AmdJson {
    serde_json::from_str(FILE_CONTENTS).unwrap()
}

/// Take a struct directly from the json and format it into a vec of [Cpu]
fn process_json(json: AmdJson) -> Vec<Cpu<String>> {
    let mut output: Vec<Cpu<String>> = Vec::with_capacity(json.data.len());
    for raw_data in json.data {
        // first strip out some stray keys that appear to be cruft
        // fun fact, some of the models are not a string, but a *number*
        // so fuck you too, amd
        let name = match raw_data["Model"].clone() {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            _ => {
                panic!("Model of unexpected type encountered")
            }
        };
        let mut output_cpu = Cpu {
            name,
            attributes: HashMap::new(),
        };
        for key in raw_data.keys() {
            // stripping out keys that are either garbage, or already handled (model)
            let blacklist = ["0", "", "Model"];
            if blacklist.contains(&key.as_str()) {
                continue;
            }
            if raw_data[key] == *"" {
                continue;
            }
            // sometimes attributes are a number instead of a string, deal with that now
            match raw_data[key].clone() {
                Value::String(s) => {
                    // // entries with a space are wrapped in quotes, so strip those
                    // let mut stripped_key = key.clone();
                    // // println!("{:?}", key);
                    // if key.contains("\"") {
                    //     stripped_key = s[1..s.len() - 2].to_string();

                    // }
                    output_cpu.attributes.insert(key.to_string(), s);
                }
                Value::Number(n) => {
                    output_cpu.attributes.insert(key.to_string(), n.to_string());
                }
                _ => {
                    panic!("Unexpected type found in amd json");
                }
            }
            // output_cpu
            //     .attributes
            //     .insert(key.to_string(), raw_data[key]);
        }
        output.push(output_cpu)
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{process_json, AmdJson};
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn basic_process_json() {
        let mut mock_json = AmdJson {
            data: vec![HashMap::new()],
        };

        mock_json.data[0].insert("0".to_string(), Value::String("on".to_string()));
        mock_json.data[0].insert("".to_string(), Value::String("0".to_string()));
        mock_json.data[0].insert(
            "Model".to_string(),
            Value::String("MOS Systems 6502".to_string()),
        );
        mock_json.data[0].insert("foo".to_string(), Value::String("bar".to_string()));

        let output = process_json(mock_json);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].name, "MOS Systems 6502".to_string());
    }
}
