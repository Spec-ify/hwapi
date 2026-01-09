use super::lexer::lex_csv;
use crate::cpu::Cpu;
use std::collections::HashMap;
use std::error::Error;

pub fn parse_csv(csv: &'_ str) -> Result<Vec<Cpu<&str>>, Box<dyn Error + '_>> {
    let lexer_output = lex_csv(csv)?;
    // the returned value
    let mut output: Vec<Cpu<&str>> = Vec::new();
    for (i, cpu_name) in lexer_output.cpus.iter().enumerate() {
        let mut cpu: Cpu<&str> = Cpu {
            name: cpu_name,
            attributes: HashMap::new(),
        };
        for record in lexer_output.records.clone() {
            // println!("{:#?}", record[0].to_string());
            // the first item in each record is the label, the items are in columns with the cpus
            // println!("{:#?}: {:#?}, {:#?}", record[0].to_string(), record[1].to_string(), record[2].to_string());
            let entry = record[i + 1];
            if !entry.is_empty() {
                cpu.attributes.insert(record[0], entry);
            }
        }
        output.push(cpu);
    }

    Ok(output)
}
