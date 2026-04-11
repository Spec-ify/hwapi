use super::lexer::lex_csv;
use crate::cpu::Cpu;
use std::collections::HashMap;
use std::error::Error;

pub fn parse_csv(csv: &'_ str) -> Result<Vec<Cpu<&str>>, Box<dyn Error + '_>> {
    let lexer_output = lex_csv(csv)?;
    // the returned value
    let mut output: Vec<Cpu<&str>> = Vec::new();
    for cpu_data in lexer_output.cpus {

        let mut cpu: Cpu<&str> = Cpu {
            // First entry is always name
            name: cpu_data[0],
            attributes: HashMap::new(),
        };

        for (index, &entry) in lexer_output.header.iter().enumerate() {
            // Skip name entry, name is already provided
            if entry != "Name" && !cpu_data[index].is_empty() {
                cpu.attributes.insert(lexer_output.header[index], cpu_data[index]);
            }
        }

        output.push(cpu);
        
    }

    Ok(output)
}
