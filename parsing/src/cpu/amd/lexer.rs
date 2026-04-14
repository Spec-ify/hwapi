use std::error::Error;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::sequence::{delimited, terminated};
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
    // stripping BOM bytes from start of file
    let header_line = delimited(
        tag("\u{feff}"),
        take_until("\n"),
        tag("\n"),
    )(input)?;

    let split_input: Vec<&str> = header_line.1.split("\",\"").map(|s| s.trim_matches(|c| c == '"' || c == ',' || c == '\n')).collect();

    Ok(("", split_input))
}

// CPUs are formatted according to the header
fn read_record(input: &str) -> IResult<&str, Vec<Vec<&str>>> {

    // Removing the header from the input
    // File terminates with double newlines
    let cpu_records= delimited(
        // read a whole line, this is discarded
        terminated(take_until::<_, _, nom::error::Error<_>>("\n"), tag("\n")),
        // read until two newlines in a row
        alt((take_until("\n\n"), take_until("\r\n\r\n"))),
        // discard the rest
        alt((tag("\n\n"), tag("\r\n\r\n"))),
    )(input)?;

    let cpu_list: Vec<&str> = cpu_records.1.split('\n').collect();
    let mut cpu_record: Vec<Vec<&str>> = Vec::with_capacity(1024);

    for cpu in cpu_list {
        let cpu_vector: Vec<&str> = cpu.split("\",\"").map(|s| s.trim_matches(|c| c == '"' || c == ',' || c == '\n')).collect();
        cpu_record.push(cpu_vector);
    }

    Ok(("", cpu_record))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_read_header() {
        let mock_header = "\"Name\",\"Family\",\"Series\",\"Form Factor\",\"# of CPU Cores\",\"# of Threads\",\"Max. Boost Clock\",\"Base Clock\",\"L2 Cache\"\n";
        let output: (&str, Vec<&str>) = read_file_header(mock_header).unwrap();
        assert_eq!(output.1[output.1.len() - 1], "L2 Cache");
    }

    #[test]
    fn basic_read_record() {
        let mock_record = "\"AMD Ryzen™ AI 9 HX PRO 475\",\"Ryzen PRO\",\"Ryzen AI PRO 400 Series\",\n\"AMD Ryzen™ AI 9 HX PRO 470\",\"Ryzen PRO\",\"Ryzen AI PRO 400 Series\",\n\n";
        let output = read_record(mock_record).unwrap();
        assert_eq!(output.1[0][2], "Ryzen AI PRO 400 Series");
    }
}