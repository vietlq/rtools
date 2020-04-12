//! `rcut` is a Rust replacement for GNU cut that supports UTF-8.
//! Implementation details are exported for reusability in case users
//! are interested in building their own char/word cutter.
//!

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::{cmp, str};

extern crate clap;
//use clap::{App, Arg, ArgGroup, SubCommand};
use clap::{App, Arg};

/// Cargo version specified in the Cargo.toml file
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Cargo version specified in the Cargo.toml file
pub fn version() -> &'static str {
    VERSION
}

/// Extract ranged pair having the pattern `(\d+-|-\d+|\d+-\d+)`
pub fn str_to_ranged_pair(char_part: &str) -> (usize, usize) {
    assert!(char_part != "-", "invalid range with no endpoint: -");

    let str_pos: Vec<&str> = char_part.split("-").collect();

    if str_pos.len() == 1 {
        let start_pos = char_part.parse::<usize>().unwrap();
        (start_pos, start_pos)
    } else {
        assert!(str_pos.len() == 2);

        let start_pos = if str_pos[0].is_empty() {
            1
        } else {
            str_pos[0].parse::<usize>().unwrap()
        };

        let end_pos = if str_pos[1].is_empty() {
            std::usize::MAX
        } else {
            str_pos[1].parse::<usize>().unwrap()
        };

        (start_pos, end_pos)
    }
}

/// Extract list of comma-separated ranged pairs
pub fn extract_ranged_pairs(char_pairs_str: &str) -> Vec<(usize, usize)> {
    let char_pairs: Vec<(usize, usize)> = char_pairs_str
        .split(",")
        .map(|char_part| str_to_ranged_pair(char_part))
        .filter(|(start_pos, end_pos)| start_pos <= end_pos)
        .collect();

    char_pairs
}

/// Sort ranged pairs and merge those having adjacent or overlapping boundaries
pub fn merge_ranged_pairs(mut char_pairs: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut ranged_pairs: Vec<(usize, usize)> = vec![];

    char_pairs.sort();

    for char_pair in &char_pairs {
        if ranged_pairs.is_empty() {
            ranged_pairs.push(char_pair.clone());
        } else {
            let last_mut = ranged_pairs.last_mut().unwrap();

            // Merge 2 adjacently sorted intervals whenever possible
            if char_pair.0 - 1 > last_mut.1 {
                ranged_pairs.push(char_pair.clone());
            } else {
                last_mut.1 = cmp::max(last_mut.1, char_pair.1);
            }
        }
    }

    ranged_pairs
}

/// Extract parts of a UTF-8 encoded line
pub fn process_line_utf8(line: &str, ranged_pairs: &Vec<(usize, usize)>) -> Vec<u8> {
    let uchars: Vec<char> = line.chars().collect();
    let mut out_bytes: Vec<u8> = vec![];
    let mut pair_idx: usize = 0;
    let mut char_pos: usize = ranged_pairs[pair_idx].0;
    let char_count = &uchars.len();
    let pair_count = ranged_pairs.len();
    let mut dst = [0; 8];

    // Handle UTF-8
    // https://stackoverflow.com/questions/51982999/slice-a-string-containing-unicode-chars
    // https://crates.io/crates/unicode-segmentation
    while char_pos <= *char_count && pair_idx < pair_count {
        let (start_pos, end_pos) = ranged_pairs[pair_idx];
        char_pos = cmp::max(start_pos, char_pos);

        if char_pos <= *char_count {
            out_bytes.extend(uchars[char_pos - 1].encode_utf8(&mut dst).as_bytes());
        }

        char_pos += 1;
        if end_pos < char_pos {
            pair_idx += 1;
        }
    }

    out_bytes.extend("\n".as_bytes());
    out_bytes
}

/// Extract parts of an ASCII encoded line
pub fn process_line_ascii(line: &str, ranged_pairs: &Vec<(usize, usize)>) -> Vec<u8> {
    let mut out_bytes: Vec<u8> = vec![];

    // Handle ASCII only
    for (start_pos, end_pos) in ranged_pairs {
        let len = &line.len();
        if *start_pos > *len {
            break;
        }

        // NOTE: This will panic if multi-byte characters are present
        let final_str = if *end_pos < *len {
            &line[start_pos - 1..*end_pos]
        } else {
            &line[start_pos - 1..]
        };

        out_bytes.extend(final_str.as_bytes());
    }

    out_bytes.extend("\n".as_bytes());
    out_bytes
}

pub enum InnerReadable {
    StdinType(std::io::Stdin),
    FileType(std::fs::File),
}

impl InnerReadable {
    fn from_stdin() -> InnerReadable {
        InnerReadable::StdinType(std::io::stdin())
    }

    fn from_file(file_name: &str) -> InnerReadable {
        InnerReadable::FileType(File::open(file_name).unwrap())
    }
}

// https://doc.rust-lang.org/std/io/type.Result.html
impl std::io::Read for InnerReadable {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self {
            InnerReadable::StdinType(inner_stdin) => inner_stdin.read(buf),
            InnerReadable::FileType(inner_file) => inner_file.read(buf),
        }
    }
}

/// Generic line processor that delegates to concrete line processors
pub fn process_lines<F>(
    input: InnerReadable,
    line_processor_fn: F,
    ranged_pairs: &Vec<(usize, usize)>,
) where
    F: Fn(&str, &Vec<(usize, usize)>) -> Vec<u8>,
{
    // Use higher order function instead of repeating the logic
    // https://doc.rust-lang.org/nightly/core/ops/trait.Fn.html
    // https://www.integer32.com/2017/02/02/stupid-tricks-with-higher-order-functions.html

    for line in BufReader::new(input).lines() {
        let out_bytes = line_processor_fn(&line.unwrap(), &ranged_pairs);

        std::io::stdout().write(&out_bytes).unwrap();
    }
}

/// Perform operations similar to GNU cut
pub fn run() {
    let matches = App::new("rcut")
        .version(version())
        .about("Replacement for GNU cut. Written in Rust.")
        .author("Viet Le")
        .arg(
            Arg::with_name("characters")
                .short("c")
                .long("characters")
                .value_name("LIST")
                .help(
                    "select only these ranges of characters\n\
                       ranges are comma-separated\n\
                       sample ranges: 5; 3-7,9; -5; 5-; 4,8-; -4,8",
                )
                .next_line_help(true)
                .required(true),
        )
        .arg(
            Arg::with_name("ascii")
                .short("a")
                .long("ascii")
                .help("turn on ASCII mode (the default mode is UTF-8)")
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("files")
                .help(
                    "the content of these files will be used\n\
                       if no file given, STDIN will be used",
                )
                .next_line_help(true)
                .required(false)
                .multiple(true),
        )
        .get_matches();

    let characters = matches.value_of("characters").unwrap();
    let ascii_mode = matches.is_present("ascii");
    let files = matches.value_of("files");

    println!("files = {:?}", files);

    let char_pairs = extract_ranged_pairs(characters);

    let ranged_pairs = merge_ranged_pairs(char_pairs);

    if ascii_mode {
        process_lines(
            InnerReadable::from_stdin(),
            process_line_ascii,
            &ranged_pairs,
        );
    } else {
        process_lines(
            InnerReadable::from_stdin(),
            process_line_utf8,
            &ranged_pairs,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_ranged_pair_valid_inputs() {
        assert_eq!(str_to_ranged_pair("1"), (1, 1));
        assert_eq!(str_to_ranged_pair("2"), (2, 2));
        assert_eq!(str_to_ranged_pair("-20"), (1, 20));
        assert_eq!(str_to_ranged_pair("20-"), (20, std::usize::MAX));
        assert_eq!(str_to_ranged_pair("3-7"), (3, 7));
    }

    #[test]
    #[should_panic]
    fn test_str_to_ranged_pair_empty_input() {
        str_to_ranged_pair("");
    }

    #[test]
    #[should_panic]
    fn test_str_to_ranged_pair_no_range() {
        str_to_ranged_pair("-");
    }

    #[test]
    #[should_panic]
    fn test_str_to_ranged_pair_invalid_char() {
        str_to_ranged_pair(";");
    }

    #[test]
    #[should_panic]
    fn test_str_to_ranged_pair_space() {
        str_to_ranged_pair(" ");
    }

    #[test]
    #[should_panic]
    fn test_str_to_ranged_pair_tab() {
        str_to_ranged_pair("\t");
    }

    #[test]
    fn test_extract_ranged_pairs_basic_valid_inputs() {
        assert_eq!(extract_ranged_pairs("1"), vec![(1, 1)]);
        assert_eq!(extract_ranged_pairs("1-8"), vec![(1, 8)]);
        assert_eq!(extract_ranged_pairs("5-9"), vec![(5, 9)]);
        assert_eq!(extract_ranged_pairs("9-5"), vec![]);
        assert_eq!(extract_ranged_pairs("-5"), vec![(1, 5)]);
        assert_eq!(extract_ranged_pairs("5-"), vec![(5, std::usize::MAX)]);
    }

    #[test]
    fn test_extract_ranged_pairs_ensure_no_sorting() {
        assert_eq!(
            extract_ranged_pairs("3,4,5-"),
            vec![(3, 3), (4, 4), (5, std::usize::MAX)]
        );
        assert_eq!(
            extract_ranged_pairs("5-,3,4"),
            vec![(5, std::usize::MAX), (3, 3), (4, 4)]
        );
        assert_eq!(
            extract_ranged_pairs("6-10,5-"),
            vec![(6, 10), (5, std::usize::MAX)]
        );
        assert_eq!(
            extract_ranged_pairs("7,6-10,5-"),
            vec![(7, 7), (6, 10), (5, std::usize::MAX)]
        );
    }

    #[test]
    #[should_panic]
    fn test_extract_ranged_pairs_empty() {
        extract_ranged_pairs("");
    }

    #[test]
    #[should_panic]
    fn test_extract_ranged_pairs_bad_range() {
        extract_ranged_pairs("-");
    }

    #[test]
    fn test_merge_ranged_pairs() {
        assert_eq!(
            merge_ranged_pairs(extract_ranged_pairs("3,4,5-")),
            vec![(3, std::usize::MAX)]
        );
        assert_eq!(
            merge_ranged_pairs(extract_ranged_pairs("3-4,5-")),
            vec![(3, std::usize::MAX)]
        );
        assert_eq!(
            merge_ranged_pairs(extract_ranged_pairs("3-5,5-")),
            vec![(3, std::usize::MAX)]
        );
        assert_eq!(
            merge_ranged_pairs(extract_ranged_pairs("3-6,5-")),
            vec![(3, std::usize::MAX)]
        );
        assert_eq!(
            merge_ranged_pairs(extract_ranged_pairs("7,6-10,5-")),
            vec![(5, std::usize::MAX)]
        );
        assert_eq!(
            merge_ranged_pairs(extract_ranged_pairs("3-7,8,2-10,12-20")),
            vec![(2, 10), (12, 20)]
        );
        assert_eq!(
            merge_ranged_pairs(extract_ranged_pairs("3-7,8,2-10,11-20")),
            vec![(2, 20)]
        );
    }
}
