use nom::bytes::complete::{tag, take, take_until};
use nom::sequence::{delimited, terminated};
use nom::IResult;
use std::collections::HashMap;

// the input file was obtained from
// https://github.com/MicrosoftDocs/windows-driver-docs/blob/staging/windows-driver-docs-pr/debugger/bug-check-code-reference2.md
const FILE_INPUT: &str = include_str!("./input.md");

#[derive(PartialEq, Debug, Clone)]
pub struct Code {
    /// The code associated with a particular bugcheck, eg `1` for `APC_INDEX_MISMATCH`
    code: u64,
    /// The name of a given code, for example `APC_INDEX_MISMATCH`
    name: String,
    /// A link to the associate microsoft documentation for that code
    url: String,
}

/// An interface for fetching and storing bugcheck codes
pub struct CodeCache {
    /// Lookup happens from a code, and it returns a tuple containing a (name, url) pair
    codes: HashMap<u64, (String, String)>,
}

impl CodeCache {
    pub fn new() -> Self {
        let mut codes = HashMap::new();
        let table = read_header(FILE_INPUT).unwrap().0;
        let mut parser_output = read_record(table);
        while let Ok(o) = parser_output {
            codes.insert(o.1.code, (o.1.name, o.1.url));
            parser_output = read_record(o.0);
        }
        Self { codes }
    }

    /// Fetch the associated name and url for a bugcheck code, if it exists
    pub fn get<C: Into<u64>>(&self, code: C) -> Option<&(String, String)> {
        self.codes.get(&code.into())
    }

    /// Iterate over all keyvalue pairs of the bugcheck library
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, u64, (String, String)> {
        self.codes.iter()
    }
}

impl Default for CodeCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Read the file up to the start of the actual bugcheck table
/// actual list
fn read_header(input: &str) -> IResult<&str, &str> {
    take_until("| 0x000")(input)
}

/// Read a single bugcheck code and the url from the table
fn read_record(input: &str) -> IResult<&str, Code> {
    let code_combinator = delimited(tag("| "), take(10_u16), tag(" | "))(input)?;
    let md_combinator = parse_md_link(code_combinator.0)?;
    let (name, url) = md_combinator.1;

    // Read cleanly to the next line
    let cleanup_combinator = terminated(take_until("|\n"), tag("|\n"))(md_combinator.0)?;
    Ok((
        cleanup_combinator.0,
        Code {
            code: u64::from_str_radix(&code_combinator.1.replace("0x", ""), 16).unwrap(),
            url,
            name,
        },
    ))
}

/// Convert a markdown link to a tuple containing the name of the code and a link to microsoft documentation
fn parse_md_link(input: &str) -> IResult<&str, (String, String)> {
    // let link_combinator = take_until("bug-check")(input)?;
    let name_combinator = delimited(tag("[**"), take_until("**]"), tag("**]"))(input)?;
    let file_combinator = delimited(tag("("), take_until(".md)"), tag(".md)"))(name_combinator.0)?;
    let resource = file_combinator.1.replace(".md)", "");
    let name = name_combinator.1.replace("\\", "");

    Ok((
        file_combinator.0,
        (
            name,
            format!(
                "https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/{resource}"
            ),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_table_record() {
        let table_record =
            "| 0x00000001 | [**APC\\_INDEX\\_MISMATCH**](bug-check-0x1--apc-index-mismatch.md)         |\n";
        let expected_code = Code {
            code: 1,
            name: String::from("APC_INDEX_MISMATCH"),
            url: String::from("https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/bug-check-0x1--apc-index-mismatch")
        };
        let combinator_output = read_record(table_record).unwrap();
        assert!(
            combinator_output.0.is_empty(),
            "Combinator leftovers should be empty, is instead {:?}",
            combinator_output.0
        );
        assert_eq!(combinator_output.1, expected_code);
    }

    #[test]
    fn do_thing() {
        let cache = CodeCache::new();
        let output = cache.get(1_u64).unwrap();
        assert_eq!(output, &("APC_INDEX_MISMATCH".to_string(), "https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/bug-check-0x1--apc-index-mismatch".to_string()));
    }
}
