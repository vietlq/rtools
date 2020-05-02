extern crate clap;
use clap::{App, Arg};

use rcut::{
    prepare_ranged_pairs, version, ByteLineProcessor, CharContext, CharProcessor,
    CharUtf8LineProcessor, FieldContext, FieldProcessor, FieldUtf8LineProcessor, RtoolT,
};

/// Perform operations similar to GNU cut
pub fn do_rcut(input_args: &Vec<&str>) {
    const _STR_BYTES: &'static str = "bytes";
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
            Arg::with_name(_STR_BYTES)
                .short("b")
                .long(_STR_BYTES)
                .value_name("LIST")
                .help(
                    "Select only these ranges of **bytes**.\n\
                       Ranges are comma-separated.\n\
                       Sample ranges: 5; 3-7,9; -5; 5-; 4,8-; -4,8.",
                )
                .next_line_help(true)
                .conflicts_with_all(&vec![_STR_DELIMITER, _STR_CHARACTERS])
                .required(false)
                .takes_value(true),
        )
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
                .conflicts_with_all(&vec![_STR_DELIMITER, _STR_BYTES])
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
        .get_matches_from(input_args);

    let byte_mode = matches.is_present(_STR_BYTES);
    let char_mode = matches.is_present(_STR_CHARACTERS);
    let field_mode = matches.is_present(_STR_DELIMITER);

    if !byte_mode && !char_mode && !field_mode {
        eprintln!("One of -b/--bytes or -c/--characters or -d/--delimiter must be present!");
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
        let ranged_pairs = prepare_ranged_pairs(no_merge, ranged_pairs_str);
        let field_processor = FieldProcessor {};
        let context = FieldContext::new(&ranged_pairs, delim);
        field_processor.process(&FieldUtf8LineProcessor {}, &files, &context);
    } else {
        let ranged_pairs_str = if char_mode {
            matches.value_of(_STR_CHARACTERS).unwrap()
        } else {
            matches.value_of(_STR_BYTES).unwrap()
        };

        let ranged_pairs = prepare_ranged_pairs(no_merge, ranged_pairs_str);
        let char_processor = CharProcessor {};
        let context = CharContext::new(&ranged_pairs);

        if ascii_mode || byte_mode {
            char_processor.process(&ByteLineProcessor {}, &files, &context);
        } else {
            char_processor.process(&CharUtf8LineProcessor {}, &files, &context);
        }
    };
}

fn main() {
    let input_args: Vec<_> = std::env::args().collect();
    let input_args = input_args.iter().map(|s| s.as_str()).collect();

    do_rcut(&input_args)
}
