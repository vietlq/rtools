//! `rcut` is a Rust replacement for GNU cut that supports UTF-8.
//! Implementation details are exported for reusability in case users
//! are interested in building their own char/word cutter.
//!

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{cmp, str};

extern crate clap;
use clap::{App, Arg, ArgGroup, SubCommand};

/// Cargo version specified in the Cargo.toml file
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Cargo version specified in the Cargo.toml file
pub fn version() -> &'static str {
    VERSION
}

/// Extract ranged pair having the pattern `(\d|\d+-|-\d+|\d+-\d+)`
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
pub fn extract_ranged_pairs(ranged_pairs_str: &str) -> Vec<(usize, usize)> {
    let unsorted_ranged_pairs: Vec<(usize, usize)> = ranged_pairs_str
        .split(",")
        .map(|char_part| str_to_ranged_pair(char_part))
        .filter(|(start_pos, end_pos)| start_pos <= end_pos)
        .collect();

    unsorted_ranged_pairs
}

/// Sort ranged pairs and merge those having adjacent or overlapping boundaries
pub fn merge_ranged_pairs(mut unsorted_ranged_pairs: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    // Without prior sorting, merging would be a bad idea
    unsorted_ranged_pairs.sort();

    let mut ranged_pairs: Vec<(usize, usize)> = vec![];

    for ranged_pair in &unsorted_ranged_pairs {
        if ranged_pairs.is_empty() {
            ranged_pairs.push(ranged_pair.clone());
        } else {
            let last_mut = ranged_pairs.last_mut().unwrap();

            // Merge 2 adjacently sorted intervals whenever possible
            if ranged_pair.0 - 1 > last_mut.1 {
                ranged_pairs.push(ranged_pair.clone());
            } else {
                last_mut.1 = cmp::max(last_mut.1, ranged_pair.1);
            }
        }
    }

    ranged_pairs
}

/// Extract parts of a UTF-8 encoded line
pub fn process_line_utf8(line: &str, ranged_pairs: &Vec<(usize, usize)>) -> Vec<u8> {
    let uchars: Vec<char> = line.chars().collect();
    let mut out_bytes: Vec<u8> = vec![];
    let char_count = &uchars.len();

    // Handle UTF-8
    // https://stackoverflow.com/questions/51982999/slice-a-string-containing-unicode-chars
    // https://crates.io/crates/unicode-segmentation

    for (start_pos, end_pos) in ranged_pairs {
        let mut char_pos: usize = start_pos.clone();

        while char_pos <= *char_count && char_pos <= *end_pos {
            let mut dst = [0; 8];
            out_bytes.extend(uchars[char_pos - 1].encode_utf8(&mut dst).as_bytes());
            char_pos += 1;
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

/// Process readable object: Send it via rcut pipeline
pub fn process_readable<R: std::io::Read, W: std::io::Write>(
    input: BufReader<R>,
    output: &mut BufWriter<W>,
    ascii_mode: bool,
    ranged_pairs: &Vec<(usize, usize)>,
) {
    if ascii_mode {
        process_lines(input, output, process_line_ascii, ranged_pairs);
    } else {
        process_lines(input, output, process_line_utf8, ranged_pairs);
    }
}

/// Process files: Send them via rcut pipeline
pub fn process_files<W: std::io::Write>(
    files: &Vec<&str>,
    writable: W,
    ascii_mode: bool,
    ranged_pairs: &Vec<(usize, usize)>,
) {
    let mut output = BufWriter::new(writable);

    for file in files {
        match File::open(file) {
            Ok(file) => {
                let input = BufReader::new(file);
                process_readable(input, &mut output, ascii_mode, ranged_pairs);
            }
            Err(err) => {
                eprintln!("Could not read the file `{}`. The error: {:?}", file, err);
            }
        }
    }
}

/// Generic line processor that delegates to concrete line processors
pub fn process_lines<F, R: Read, W: Write>(
    input: BufReader<R>,
    output: &mut BufWriter<W>,
    line_processor_fn: F,
    ranged_pairs: &Vec<(usize, usize)>,
) where
    F: Fn(&str, &Vec<(usize, usize)>) -> Vec<u8>,
{
    // Use higher order function instead of repeating the logic
    // https://doc.rust-lang.org/nightly/core/ops/trait.Fn.html
    // https://www.integer32.com/2017/02/02/stupid-tricks-with-higher-order-functions.html

    for line in input.lines() {
        let out_bytes = line_processor_fn(&line.unwrap(), &ranged_pairs);

        output.write(&out_bytes).unwrap();
    }
}

/// Cut and paste lines by ranges of characters
pub fn process_char_mode(
    ascii_mode: bool,
    ranged_pairs_str: &str,
    no_merge: bool,
    files: &Vec<&str>,
) {
    let unsorted_ranged_pairs = extract_ranged_pairs(ranged_pairs_str);

    let ranged_pairs = if no_merge {
        unsorted_ranged_pairs
    } else {
        merge_ranged_pairs(unsorted_ranged_pairs)
    };

    if files.is_empty() {
        process_readable(
            BufReader::new(std::io::stdin()),
            &mut BufWriter::new(std::io::stdout()),
            ascii_mode,
            &ranged_pairs,
        );
    } else {
        process_files(&files, &mut std::io::stdout(), ascii_mode, &ranged_pairs);
    }
}

/// Split lines into fields by delimiter, then cut and paste ranges of fields
pub fn process_field_mode(
    ascii_mode: bool,
    delim: &str,
    ranged_pairs_str: &str,
    no_merge: bool,
    files: &Vec<&str>,
) {
}

/// Perform operations similar to GNU cut
pub fn run() {
    // Prints each argument on a separate line
    for argument in std::env::args() {
        println!("{}", argument);
    }

    const _STR_CHARACTERS: &'static str = "characters";
    const _STR_DELIMITER: &'static str = "delimiter";
    const _STR_FIELDS: &'static str = "fields";
    const _STR_ASCII: &'static str = "ascii";
    const _STR_NO_MERGE: &'static str = "no-merge";

    let matches = App::new("rcut")
        .version(version())
        .about("Replacement for GNU cut. Written in Rust.")
        .author("Viet Le")
        .arg(
            Arg::with_name(_STR_CHARACTERS)
                .short("c")
                .long(_STR_CHARACTERS)
                .value_name("LIST")
                .help(
                    "Select only these ranges of **characters**.\n\
                       Ranges are comma-separated.\n\
                       Sample ranges: 5; 3-7,9; -5; 5-; 4,8-; -4,8.",
                )
                .next_line_help(true)
                .conflicts_with(_STR_DELIMITER)
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(_STR_DELIMITER)
                .short("d")
                .long(_STR_DELIMITER)
                .help(
                    "Split lines into fields delimited by given delimiter.\n\
                     Must be followed by list of fields. E.g. -f2,6-8.",
                )
                .next_line_help(true)
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(_STR_FIELDS)
                .short("f")
                .long(_STR_FIELDS)
                .value_name("LIST")
                .help(
                    "Select only these ranges of **fields**.\n\
                       Is dependent on the delimiter flag -d.\n\
                       Ranges are comma-separated.\n\
                       Sample ranges: 5; 3-7,9; -5; 5-; 4,8-; -4,8.",
                )
                .next_line_help(true)
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(_STR_ASCII)
                .short("a")
                .long(_STR_ASCII)
                .help("Turn on ASCII mode (the default mode is UTF-8).")
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name(_STR_NO_MERGE)
                .short("N")
                .long(_STR_NO_MERGE)
                .help(
                    "Do not sort and merge ranges.\n\
                    Think of it as cut-n-paste.\n\
                    Sort and merge by default.",
                )
                .next_line_help(true)
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("files")
                .help(
                    "The content of these files will be used.\n\
                     If no files given, STDIN will be used.",
                )
                .next_line_help(true)
                .required(false)
                .multiple(true),
        )
        .get_matches();

    let char_mode = matches.is_present(_STR_CHARACTERS);
    let field_mode = matches.is_present(_STR_DELIMITER);

    if !char_mode && !field_mode {
        eprintln!("Either -c/--characters or -d/--delimiter must be present!");
        std::process::exit(1);
    }

    if matches.is_present(_STR_FIELDS) && !field_mode {
        eprintln!("The flag -f/--fields is dependent on the flag -d/--delimiter!");
        std::process::exit(1);
    }

    if field_mode && !matches.is_present(_STR_FIELDS) {
        eprintln!("The flag -d/--delimiter requires presence of -f/--fields!");
        std::process::exit(1);
    }

    let ascii_mode = matches.is_present(_STR_ASCII);
    let no_merge = matches.is_present(_STR_NO_MERGE);

    // NOTE: Use `values_of` instead of `value_of`!!!!
    let files_it_opt = matches.values_of("files");
    let files = if files_it_opt.is_none() {
        vec![]
    } else {
        files_it_opt.unwrap().collect()
    };

    if field_mode {
        let delim = matches.value_of(_STR_DELIMITER).unwrap();
        let ranged_pairs_str = matches.value_of(_STR_FIELDS).unwrap();
        process_field_mode(ascii_mode, delim, ranged_pairs_str, no_merge, &files);
    } else {
        let ranged_pairs_str = matches.value_of(_STR_CHARACTERS).unwrap();
        process_char_mode(ascii_mode, ranged_pairs_str, no_merge, &files);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    const _STR_RANGES_01: &'static str = "9,4,7,3,12,5-15";
    const _STR_BIRDS: &'static str = "🦃🐔🐓🐣🐤🐥🐦🐧🕊🦅🦆🦢🦉🦚🦜";
    const _STR_BIRDS_OUTPUT: &'static str = "🕊🐣🐦🐓🦢🐤🐥🐦🐧🕊🦅🦆🦢🦉🦚🦜\n";
    const _STR_ALPHABET: &'static str = "abcdefghijklmnopqrstuvwxyz";
    const _STR_ALPHABET_OUTPUT: &'static str = "idgclefghijklmno\n";

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

    #[test]
    fn test_process_line_utf8() {
        let ranged_pairs = extract_ranged_pairs(_STR_RANGES_01);
        assert_eq!(
            _STR_BIRDS_OUTPUT.as_bytes().to_vec(),
            process_line_utf8(_STR_BIRDS, &ranged_pairs)
        );
    }

    #[test]
    fn test_process_line_ascii() {
        let ranged_pairs = extract_ranged_pairs(_STR_RANGES_01);
        assert_eq!(
            _STR_ALPHABET_OUTPUT.as_bytes().to_vec(),
            process_line_ascii(_STR_ALPHABET, &ranged_pairs)
        );
    }

    #[test]
    #[should_panic]
    fn test_process_line_ascii_panic() {
        let ranged_pairs = extract_ranged_pairs(_STR_RANGES_01);
        assert_eq!(
            _STR_BIRDS_OUTPUT.as_bytes().to_vec(),
            process_line_ascii(_STR_BIRDS, &ranged_pairs)
        );
    }

    #[test]
    fn test_process_lines_utf8_with_cursor() {
        // https://doc.rust-lang.org/std/io/struct.Cursor.html
        // https://stackoverflow.com/questions/41069865/how-to-create-an-in-memory-object-that-can-be-used-as-a-reader-writer-or-seek
        let input = BufReader::new(std::io::Cursor::new(_STR_BIRDS));
        let mut out_cursor = std::io::Cursor::new(Vec::<u8>::new());

        let ranged_pairs = extract_ranged_pairs(_STR_RANGES_01);
        // Let borrower of the output cursor expire before reacquiring the output cursor
        process_lines(
            input,
            &mut BufWriter::new(&mut out_cursor),
            process_line_utf8,
            &ranged_pairs,
        );

        out_cursor.seek(std::io::SeekFrom::Start(0)).unwrap();
        // Read the fake "file's" contents into a vector
        let mut out = Vec::new();
        out_cursor.read_to_end(&mut out).unwrap();
        assert_eq!(_STR_BIRDS_OUTPUT.as_bytes().to_vec(), out);
    }
}
