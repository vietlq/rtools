extern crate clap;
//use clap::{App, Arg, ArgGroup, SubCommand};
use clap::{App, Arg};

extern crate regex;
use regex::Regex;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new("rcut")
        .version(VERSION)
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
    println!("characters = {}", characters);

    let char_regex = Regex::new(r"^\d+(\-\d+)?(,\d+(\-\d+)?)?$").unwrap();
    if char_regex.is_match(characters) {
        println!("Good list of characters :)")
    } else {
        println!("Bad list of characters!")
    }
}
