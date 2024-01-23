use super::lexer::lex_csv;
use crate::cpu::Cpu;
use std::collections::HashMap;
use std::error::Error;

pub fn parse_csv(csv: &'_ str) -> Result<Vec<Cpu>, Box<dyn Error + '_>> {
    let lexer_output = lex_csv(csv)?;
    // the returned value
    let mut output: Vec<Cpu> = Vec::with_capacity(lexer_output.cpus.len());
    for (i, cpu_name) in lexer_output.cpus.iter().enumerate() {
        let mut cpu = Cpu {
            name: cpu_name.to_string(),
            attributes: HashMap::with_capacity(256),
        };
        for record in lexer_output.records.clone() {
            // the first item in each record is the label, the items are in columns with the cpus
            let entry = record[i + 1];
            if !entry.is_empty() {
                cpu.attributes
                    .insert(record[0].to_string(), entry.to_string());
            }
        }
        output.push(cpu);
    }

    Ok(output)
}
