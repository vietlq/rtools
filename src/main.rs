use std::process;

extern crate clap;
//use clap::{App, Arg, ArgGroup, SubCommand};
use clap::{App, Arg};

extern crate regex;
use regex::Regex;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn handle_pos_parse(s: &str) -> u32 {
    match s.parse::<u32>() {
        Ok(n) => n,
        Err(_) => std::u32::MAX
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

    let char_regex = Regex::new(r"^\d+(\-(\d+)?)?(,\d+(\-(\d+)?)?)?$").unwrap();
    if char_regex.is_match(characters) {
        println!("Good list of characters :)");
    } else {
        println!("Bad list of characters!");
        process::exit(1);
    }

    for char_part in characters.split(",") {
        let positions: Vec<u32> = char_part.split("-")
            .map(|s| handle_pos_parse(s)).collect();
        println!("char_part = {}", char_part);
        for pos in positions {
            println!("pos = {}", pos);
        }
    }
}
