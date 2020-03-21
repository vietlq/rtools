extern crate clap;
use clap::{App, Arg, ArgGroup, SubCommand};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new("rcut")
        .version(VERSION)
        .arg(
            Arg::with_name("characters")
                .short("c")
                .long("characters")
                .value_name("LIST")
                .help("select only these characters"),
        )
        .get_matches();
    let characters = matches.value_of("characters").unwrap();
    println!("characters = {}", characters);
}
