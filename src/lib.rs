use std::io::prelude::*;
use std::io::BufReader;
use std::{cmp, io, str};

extern crate clap;
//use clap::{App, Arg, ArgGroup, SubCommand};
use clap::{App, Arg};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn char_part_to_pair(char_part: &str) -> (usize, usize) {
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

fn extract_char_pairs(char_pairs_str: &str) -> Vec<(usize, usize)> {
    let mut char_pairs: Vec<(usize, usize)> = char_pairs_str
        .split(",")
        .map(|char_part| char_part_to_pair(char_part))
        .filter(|(start_pos, end_pos)| start_pos <= end_pos)
        .collect();

    char_pairs.sort();

    char_pairs
}

fn merge_char_pairs(char_pairs: &Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut ranged_pairs: Vec<(usize, usize)> = vec![];

    for char_pair in char_pairs {
        if ranged_pairs.is_empty() {
            ranged_pairs.push(char_pair.clone());
        } else {
            let last_mut = ranged_pairs.last_mut().unwrap();
            if char_pair.0 <= last_mut.1 {
                last_mut.1 = cmp::max(last_mut.1, char_pair.1);
            } else {
                ranged_pairs.push(char_pair.clone());
            }
        }
    }

    ranged_pairs
}

fn process_line_utf8(line: &str, ranged_pairs: &Vec<(usize, usize)>) -> Vec<u8> {
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

fn process_line_ascii(line: &str, ranged_pairs: &Vec<(usize, usize)>) -> Vec<u8> {
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

fn process_lines_utf8(ranged_pairs: &Vec<(usize, usize)>) {
    let f = BufReader::new(io::stdin());
    for line in f.lines() {
        let out_bytes = process_line_utf8(&line.unwrap(), &ranged_pairs);

        std::io::stdout().write(&out_bytes).unwrap();
    }
}

fn process_lines_ascii(ranged_pairs: &Vec<(usize, usize)>) {
    let f = BufReader::new(io::stdin());
    for line in f.lines() {
        let out_bytes = process_line_ascii(&line.unwrap(), &ranged_pairs);

        std::io::stdout().write(&out_bytes).unwrap();
    }
}

pub fn do_cut() {
    let matches = App::new("rcut")
        .version(VERSION)
        .about("Replacement for GNU cut. Written in Rust.")
        .author("Viet Le")
        .arg(
            Arg::with_name("characters")
                .short("c")
                .long("characters")
                .value_name("LIST")
                .help("select only these characters")
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
        .get_matches();

    let characters = matches.value_of("characters").unwrap();
    let ascii_mode = matches.is_present("ascii");

    let char_pairs = extract_char_pairs(characters);

    let ranged_pairs = merge_char_pairs(&char_pairs);

    if ascii_mode {
        process_lines_ascii(&ranged_pairs);
    } else {
        process_lines_utf8(&ranged_pairs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_part_to_pair_valid_inputs() {
        assert_eq!(char_part_to_pair("-"), (1, std::usize::MAX));
        assert_eq!(char_part_to_pair("1"), (1, 1));
        assert_eq!(char_part_to_pair("2"), (2, 2));
        assert_eq!(char_part_to_pair("-20"), (1, 20));
        assert_eq!(char_part_to_pair("20-"), (20, std::usize::MAX));
        assert_eq!(char_part_to_pair("3-7"), (3, 7));
    }

    #[test]
    #[should_panic]
    fn test_char_part_to_pair_empty_input() {
        char_part_to_pair("");
    }

    #[test]
    #[should_panic]
    fn test_char_part_to_pair_invalid_char() {
        char_part_to_pair(";");
    }
}
