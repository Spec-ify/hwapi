use core::slice;
use std::error::Error;

use nom::bytes::complete::{tag, take_until};
use nom::sequence::{delimited, terminated};
use nom::IResult;

/// Everything deserialized by the lexer
#[derive(Debug)]
pub struct LexerOutput<'a> {
    /// The timestamp of the csv file
    pub timestamp: &'a str,
    /// A list of all cpus in the file
    pub cpus: Vec<&'a str>,
    /// the deserialized records in the file
    pub records: Vec<Vec<&'a str>>,
}

pub fn lex_csv<'a>(input: &'a str) -> Result<LexerOutput, Box<dyn Error + 'a>> {
    let header_combinator_output = read_file_header(input).expect("Failed to read file header");
    let cpu_combinator_output =
        read_cpu_record(header_combinator_output.0).expect("Failed to parse list of cpus");

    // sections are now repeatedly read from the csv until no more can be read
    let mut serialized_records: Vec<&str> = Vec::with_capacity(256);
    let mut iterated_output =
        read_section(cpu_combinator_output.0).expect("Failed to read first section");
    while !iterated_output.0.is_empty() {
        serialized_records.push(iterated_output.1);
        iterated_output = read_section(iterated_output.0)?;
    }
    // iterate over all of the serialized records and deserialize them
    let mut deserialized_records: Vec<Vec<&str>> = Vec::with_capacity(256);
    for serialized_record in serialized_records {
        // this variable is iterated over until all sections are read from
        let mut iterated_output = read_record(serialized_record)?;
        while !iterated_output.0.is_empty() {
            deserialized_records.push(iterated_output.1);
            iterated_output = read_record(iterated_output.0)?;
        }
    }

    Ok(LexerOutput {
        timestamp: header_combinator_output.1,
        cpus: cpu_combinator_output.1,
        records: deserialized_records,
    })
}

/// The header of the file contains some generic boilerplate and the date of the report generation
fn read_file_header(input: &str) -> IResult<&str, &str> {
    delimited(
        // apparently the csv starts with a 0 width space?
        tag("\u{feff}ARK | Intel® Product Specification Comparison\n"),
        take_until("\n"),
        tag("\n ,"),
    )(input)
}

/// The list of CPUs has formatting that varies slightly from a normal record
fn read_cpu_record(input: &str) -> IResult<&str, Vec<&str>> {
    let cpu_record = take_until(" \n")(input)?;
    let split_input: Vec<&str> = cpu_record.1.split(" ,").collect();
    // .strip_prefix is used because take_until includes the "until" part
    // in the remainder instead of dropping it
    Ok((cpu_record.0.strip_prefix(" \n").unwrap(), split_input))
}

fn read_record(input: &str) -> IResult<&str, Vec<&str>> {
    let record: (&str, &str) = take_until("\n")(input)?;
    let split_input: Vec<&str> = record.1.split(',').collect();
    // .strip_prefix is used because take_until includes the "until" part
    // in the remainder instead of dropping it
    Ok((record.0.strip_prefix('\n').unwrap(), split_input))
}

/// The majority of the actual data organized into sections, this function reads a "block", or
/// everything underneath a header, returning a serialized group of records
fn read_section(input: &str) -> IResult<&str, &str> {
    // read a line and discard it, the actual "header", then read until there are two newlines in a row
    // this is nasty and could definitely be cleaned up a lot
    let combinator_output: Result<(&str, &str), nom::Err<_>> = delimited(
        // read a whole line, this is discarded
        terminated(take_until::<_, _, nom::error::Error<_>>("\n"), tag("\n")),
        // read until two newlines in a row
        // preceded(tag("\n"), take_until("\n\n")),
        take_until("\n\n"),
        tag("\n\n"),
    )(input);
    // the very last section in the file needs special handling because it does not have two newlines at the end
    // this is probably fallible and not ideal
    match combinator_output {
        Ok(o) => {
            // because there's not really a "take_until_inclusive"
            // TODO: take a look at the nom source code for take_until, and write
            // a combinator for `take_until_incl` or whatever
            let raw_ptr: *const u8 = o.1.as_ptr();
            let str_len = o.1.len();
            // SAFETY: the combinator above asserts that there's at least one `\n`
            // in front of the subslice
            let output_str: &str = unsafe {
                let slice: &[u8] = slice::from_raw_parts(raw_ptr, str_len + 1);
                std::str::from_utf8(slice).unwrap()
            };
            Ok((o.0, output_str))
        }
        Err(_) => Ok(("", input)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_read_header() {
        let mock_header = "\u{feff}ARK | Intel® Product Specification Comparison\n01/14/2024 01:06:53 PM\n ,Intel® Xeon® D-1823NT Processor";
        let output = read_file_header(mock_header).unwrap();
        assert_eq!(output.1, "01/14/2024 01:06:53 PM");
    }

    #[test]
    fn basic_read_cpu_record() {
        let mock_cpu_record = "ab ,cd ,ef \nfoo";
        let output = read_cpu_record(mock_cpu_record);
        assert_eq!(output, Ok(("foo", vec!["ab", "cd", "ef"])));
    }

    #[test]
    fn basic_read_record() {
        let mock_record = "ab,,cd\nef";
        let output = read_record(mock_record);
        assert_eq!(output, Ok(("ef", vec!["ab", "", "cd"])));
    }

    #[test]
    fn basic_read_section() {
        let mock_section = "heading\nab\ncd\n\notherheading";
        let output = read_section(mock_section);
        assert_eq!(output, Ok(("otherheading", "ab\ncd\n")));
    }
}
