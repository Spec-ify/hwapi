use std::{collections::HashMap, fs};

mod lexer;
mod parser;

const FILE_PATH: &str = "input.csv";

fn read_file(path: &str) -> String {
    fs::read_to_string(path).expect("failed to read file")
}

#[cfg(test)]
mod tests {
    use crate::{parser::parse_csv, read_file, FILE_PATH};

    #[test]
    fn it_works() {
        let file_contents = read_file(FILE_PATH);
        println!("{:#?}", parse_csv(&file_contents).unwrap());
        //
        // let _ = parse_csv(&file_contents).unwrap();
    }
}
