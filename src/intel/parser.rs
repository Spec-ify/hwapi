use crate::lexer::lex_csv;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub struct Cpu<'a> {
    name: &'a str,
    attributes: HashMap<&'a str, &'a str>,
}

pub fn parse_csv<'a>(csv: &'a str) -> Result<Vec<Cpu>, Box<dyn Error + '_>> {
    let lexer_output = lex_csv(csv)?;
    // the returned value
    let mut output: Vec<Cpu> = Vec::with_capacity(lexer_output.cpus.len());
    for (i, cpu_name) in lexer_output.cpus.iter().enumerate() {
        let mut cpu = Cpu {
            name: cpu_name,
            attributes: HashMap::with_capacity(256),
        };

        for record in lexer_output.records.clone() {
            println!("{:?}", record);
        // the first item in each record is the label, the items are in columns with the cpus
            let entry = record[i + 1];
            if entry != "" {
                cpu.attributes.insert(record[0], entry);
            }

        }
        output.push(cpu);

    }



    Ok(output)
}
