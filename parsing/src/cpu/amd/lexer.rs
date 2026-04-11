use std::error::Error;

use nom::bytes::complete::{take_until};
use nom::IResult;

/// Everything deserialized by the lexer
#[derive(Debug)]
#[allow(unused)]
pub struct LexerOutput<'a> {
    /// Names of all attributes
    pub header: Vec<&'a str>,
    /// A list of all cpus in the file
    pub cpus: Vec<Vec<&'a str>>
}

pub fn lex_csv<'a>(input: &'a str) -> Result<LexerOutput<'a>, Box<dyn Error + 'a>> {
    let header_combinator_output = read_file_header(input).expect("Failed to read AMD CSV header");
    let cpu_combinator_output = read_record(input).expect("Failed to read AMD CPU data");
    
    Ok(LexerOutput {
        header: header_combinator_output.1,
        cpus: cpu_combinator_output.1,
    })
}

/// Header of CSV contains the title of each data entry
fn read_file_header(input: &str) -> IResult<&str, Vec<&str>> {
    let header_line: (&str, &str) = take_until("\n")(input)?;
    let split_input: Vec<&str> = header_line.1.split("\",\"").map(|s| s.trim_matches('"')).collect();
    // .strip_prefix is used because take_until includes the "until" part
    // in the remainder instead of dropping it
    Ok(("", split_input))
}

// CPUs are formatted according to the header
fn read_record(input: &str) -> IResult<&str, Vec<Vec<&str>>> {
    // File terminates with double newlines
    let cpu_records: (&str, &str) = take_until("\n\n")(input)?;
    let cpu_list: Vec<&str> = cpu_records.1.split('\n').collect();
    let mut cpu_record: Vec<Vec<&str>> = Vec::with_capacity(1024);

    for cpu in cpu_list {
        let cpu_vector: Vec<&str> = cpu.split("\",\"").map(|s| s.trim_matches('"')).collect();
        cpu_record.push(cpu_vector);
    }
    // .strip_prefix is used because take_until includes the "until" part
    // in the remainder instead of dropping it
    Ok(("", cpu_record))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_read_header() {
        let mock_header = "\"Name\",\"Family\",\"Series\",\"Form Factor\",\"# of CPU Cores\",\"# of Threads\",\"Max. Boost Clock\",\"Base Clock\",\"L2 Cache\"";
        let output: (&str, Vec<&str>) = read_file_header(mock_header).unwrap();
        assert_eq!(output.1[output.1.len() - 1], "01/14/2024 01:06:53 PM");
    }

    #[test]
    fn basic_read_record() {
        let mock_record = "\"AMD Ryzen™ AI 9 HX PRO 475\",\"Ryzen PRO\",\"Ryzen AI PRO 400 Series\",\n\"AMD Ryzen™ AI 9 HX PRO 470\",\"Ryzen PRO\",\"Ryzen AI PRO 400 Series\",\n\n";
        let output = read_record(mock_record).unwrap();
        assert_eq!(output.1[0][2], "Ryzen AI PRO 400 Series");
    }
}