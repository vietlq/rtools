use std::{cmp, io, process};
use std::io::{BufReader};
use std::io::prelude::*;

extern crate clap;
//use clap::{App, Arg, ArgGroup, SubCommand};
use clap::{App, Arg};

extern crate regex;
use regex::Regex;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn handle_pos_parse(s: &str) -> u32 {
    match s.parse::<u32>() {
        Ok(n) => n,
        Err(_) => std::u32::MAX,
    }
}

fn char_part_to_pair(char_part: &str) -> (u32, u32) {
    let positions: Vec<u32> = char_part.split("-").map(|s| handle_pos_parse(s)).collect();
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

    let mut char_pairs: Vec<(u32, u32)> = characters
        .split(",")
        .map(|char_part| char_part_to_pair(char_part))
        .filter(|(p1, p2)| p1 <= p2)
        .collect();
    char_pairs.sort();
    for char_pair in &char_pairs {
        println!("char_pair = {:?}", char_pair);
    }
    let mut merged_pairs: Vec<(u32, u32)> = vec![];
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
    println!("merged_pairs = {:?}", merged_pairs);

    let f = BufReader::new(io::stdin());
    for line in f.lines() {
        println!("line = {}", line.unwrap());
    }
}
