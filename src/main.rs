use std::{cmp, io, process};
use std::io::{BufReader};
use std::io::prelude::*;

extern crate clap;
//use clap::{App, Arg, ArgGroup, SubCommand};
use clap::{App, Arg};

extern crate regex;
use regex::Regex;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn handle_pos_parse(s: &str) -> usize {
    match s.parse::<usize>() {
        Ok(n) => n,
        Err(_) => std::usize::MAX,
    }
}

fn char_part_to_pair(char_part: &str) -> (usize, usize) {
    let positions: Vec<usize> = char_part.split("-").map(|s| handle_pos_parse(s)).collect();
    match positions.len() {
        1 => (positions[0], positions[0]),
        2 => (positions[0], positions[1]),
        _ => panic!("Invalid input!"),
    }
}

fn main() {
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

    let char_regex = Regex::new(r"^\d+(\-(\d+)?)?(,\d+(\-(\d+)?)?)*$").unwrap();
    if !char_regex.is_match(characters) {
        println!("Bad input for the flag -c");
        process::exit(1);
    }

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
        for (p1, p2) in &merged_pairs {
            let len = &rline.len();
            if *p1 > *len {
                break;
            }
            // TODO: Handle UTF-8
            let final_str = if *p2 < *len {
                &rline[p1 - 1 .. *p2]
            } else {
                &rline[p1 - 1 ..]
            };
            std::io::stdout().write(final_str.as_bytes()).unwrap();
        }
        std::io::stdout().write("\n".as_bytes()).unwrap();
    }
}
