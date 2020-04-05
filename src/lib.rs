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
        .get_matches();
    let characters = matches.value_of("characters").unwrap();

    let mut char_pairs: Vec<(usize, usize)> = characters
        .split(",")
        .map(|char_part| char_part_to_pair(char_part))
        .filter(|(p1, p2)| p1 <= p2)
        .collect();
    char_pairs.sort();

    let mut merged_pairs: Vec<(usize, usize)> = vec![];
    for char_pair in &char_pairs {
        if merged_pairs.is_empty() {
            merged_pairs.push(char_pair.clone());
        } else {
            let last_mut = merged_pairs.last_mut().unwrap();
            if char_pair.0 <= last_mut.1 {
                last_mut.1 = cmp::max(last_mut.1, char_pair.1);
            } else {
                merged_pairs.push(char_pair.clone());
            }
        }
    }

    let f = BufReader::new(io::stdin());
    for line in f.lines() {
        let rline = line.unwrap();
        let uchars: Vec<_> = rline.chars().collect();
        let mut pair_idx: usize = 0;
        let mut char_pos: usize = merged_pairs[pair_idx].0;
        let char_count = &uchars.len();
        let pair_count = merged_pairs.len();
        let mut dst = [0; 8];

        // Handle UTF-8
        while char_pos <= *char_count && pair_idx < pair_count {
            let (p1, p2) = merged_pairs[pair_idx];
            char_pos = cmp::max(p1, char_pos);

            if char_pos <= *char_count {
                std::io::stdout()
                    .write(&uchars[char_pos - 1].encode_utf8(&mut dst).as_bytes())
                    .unwrap();
            }

            char_pos += 1;
            if p2 < char_pos {
                pair_idx += 1;
            }
        }

        /*
        // Handle ASCII only
        for (p1, p2) in &merged_pairs {
            let len = &rline.len();
            if *p1 > *len {
                break;
            }
            // TODO: Handle UTF-8
            // https://stackoverflow.com/questions/51982999/slice-a-string-containing-unicode-chars
            // https://crates.io/crates/unicode-segmentation
            let final_str = if *p2 < *len {
                &rline[p1 - 1..*p2]
            } else {
                &rline[p1 - 1..]
            };
            std::io::stdout().write(final_str.as_bytes()).unwrap();
        }
        */

        std::io::stdout().write("\n".as_bytes()).unwrap();
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
