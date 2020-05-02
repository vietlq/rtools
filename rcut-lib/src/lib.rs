//! `rcut` is a Rust replacement for GNU cut that supports UTF-8.
//! Implementation details are exported for reusability in case users
//! are interested in building their own char/word cutter.
//!

use std::{cmp, str};

extern crate rtools_traits;
use rtools_traits::{RtoolT, LineProcessorT};

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

/// Utility function to process ranged pairs (extract, and merge on demand)
pub fn prepare_ranged_pairs(no_merge: bool, ranged_pairs_str: &str) -> Vec<(usize, usize)> {
    let unsorted_ranged_pairs = extract_ranged_pairs(ranged_pairs_str);

    let ranged_pairs = if no_merge {
        unsorted_ranged_pairs
    } else {
        merge_ranged_pairs(unsorted_ranged_pairs)
    };

    ranged_pairs
}

pub trait CharContextT {
    fn ranged_pairs(&self) -> &Vec<(usize, usize)>;
}

pub trait FieldContextT {
    fn ranged_pairs(&self) -> &Vec<(usize, usize)>;

    fn delim(&self) -> &str;
}

pub struct CharContext<'a> {
    ranged_pairs: &'a Vec<(usize, usize)>,
}

impl<'a> CharContext<'a> {
    pub fn new(ranged_pairs: &'a Vec<(usize, usize)>) -> CharContext<'a> {
        CharContext {
            ranged_pairs: ranged_pairs,
        }
    }
}

impl CharContextT for CharContext<'_> {
    fn ranged_pairs(&self) -> &Vec<(usize, usize)> {
        self.ranged_pairs
    }
}

pub struct FieldContext<'a> {
    ranged_pairs: &'a Vec<(usize, usize)>,
    delim: &'a str,
}

impl<'a> FieldContext<'a> {
    pub fn new(ranged_pairs: &'a Vec<(usize, usize)>, delim: &'a str) -> FieldContext<'a> {
        FieldContext {
            ranged_pairs,
            delim,
        }
    }
}

impl FieldContextT for FieldContext<'_> {
    fn ranged_pairs(&self) -> &Vec<(usize, usize)> {
        self.ranged_pairs
    }

    fn delim(&self) -> &str {
        self.delim
    }
}

pub struct CharUtf8LineProcessor {}

/// Extract chars from a UTF-8 line within given ranges
pub fn process_line_by_char_utf8(line: &str, ranged_pairs: &Vec<(usize, usize)>) -> Vec<u8> {
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

impl<C: CharContextT> LineProcessorT<C> for CharUtf8LineProcessor {
    /// Extract parts of a UTF-8 encoded line
    fn process(&self, line: &str, context: &C) -> Vec<u8> {
        process_line_by_char_utf8(line, context.ranged_pairs())
    }
}

pub struct ByteLineProcessor {}

/// Extract bytes from a line within given ranges
pub fn process_line_by_byte(line: &str, ranged_pairs: &Vec<(usize, usize)>) -> Vec<u8> {
    let mut out_bytes: Vec<u8> = vec![];
    let bytes = line.as_bytes();
    let len = &bytes.len();

    // Handle ASCII/single-bytes only
    for (start_pos, end_pos) in ranged_pairs {
        if *start_pos > *len {
            break;
        }

        // NOTE: This will panic if multi-byte characters are present
        let final_bytes = if *end_pos < *len {
            &bytes[start_pos - 1..*end_pos]
        } else {
            &bytes[start_pos - 1..]
        };

        out_bytes.extend(final_bytes);
    }

    out_bytes.extend("\n".as_bytes());
    out_bytes
}

impl<C: CharContextT> LineProcessorT<C> for ByteLineProcessor {
    /// Extract parts of an ASCII encoded line
    fn process(&self, line: &str, context: &C) -> Vec<u8> {
        process_line_by_byte(line, context.ranged_pairs())
    }
}

pub struct CharProcessor {}

impl<C: CharContextT, P: LineProcessorT<C>> RtoolT<C, P> for CharProcessor {}

pub struct FieldUtf8LineProcessor {}

/// Extract fields from a UTF-8 line within given ranges
pub fn process_line_by_field_utf8(
    line: &str,
    ranged_pairs: &Vec<(usize, usize)>,
    delim: &str,
) -> Vec<u8> {
    let mut out_bytes: Vec<u8> = vec![];
    let delim = delim;

    let fields: Vec<&str> = line.split(delim).collect();
    let mut has_written = false;

    for (start_pos, end_pos) in ranged_pairs {
        let len = &fields.len();
        if *start_pos > *len {
            break;
        }

        let extracted_fields = if *end_pos < *len {
            &fields[start_pos - 1..*end_pos]
        } else {
            &fields[start_pos - 1..]
        };

        for field in extracted_fields {
            // Delimiter sits between fields
            if has_written {
                out_bytes.extend(delim.as_bytes());
            } else {
                has_written = true;
            }

            out_bytes.extend(field.as_bytes());
        }
    }

    out_bytes.extend("\n".as_bytes());
    out_bytes
}

impl<C: FieldContextT> LineProcessorT<C> for FieldUtf8LineProcessor {
    /// Extract parts of an ASCII encoded line
    fn process(&self, line: &str, context: &C) -> Vec<u8> {
        process_line_by_field_utf8(line, context.ranged_pairs(), context.delim())
    }
}

pub struct FieldProcessor {}

impl<C: FieldContextT, P: LineProcessorT<C>> RtoolT<C, P> for FieldProcessor {}

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
        let char_processor = CharUtf8LineProcessor {};
        let ranged_pairs = extract_ranged_pairs(_STR_RANGES_01);
        assert_eq!(
            _STR_BIRDS_OUTPUT.as_bytes().to_vec(),
            char_processor.process(
                _STR_BIRDS,
                &CharContext {
                    ranged_pairs: &ranged_pairs
                }
            )
        );
    }

    #[test]
    fn test_process_line_ascii() {
        let char_processor = ByteLineProcessor {};
        let ranged_pairs = extract_ranged_pairs(_STR_RANGES_01);
        assert_eq!(
            _STR_ALPHABET_OUTPUT.as_bytes().to_vec(),
            char_processor.process(
                _STR_ALPHABET,
                &CharContext {
                    ranged_pairs: &ranged_pairs
                }
            )
        );
    }

    #[test]
    #[should_panic]
    fn test_process_line_ascii_panic() {
        let char_processor = ByteLineProcessor {};
        let ranged_pairs = extract_ranged_pairs(_STR_RANGES_01);
        assert_eq!(
            _STR_BIRDS_OUTPUT.as_bytes().to_vec(),
            char_processor.process(
                _STR_BIRDS,
                &CharContext {
                    ranged_pairs: &ranged_pairs
                }
            )
        );
    }

    #[test]
    fn test_process_lines_utf8_with_cursor() {
        // https://doc.rust-lang.org/std/io/struct.Cursor.html
        // https://stackoverflow.com/questions/41069865/how-to-create-an-in-memory-object-that-can-be-used-as-a-reader-writer-or-seek
        let input = BufReader::new(std::io::Cursor::new(_STR_BIRDS));
        let mut out_cursor = std::io::Cursor::new(Vec::<u8>::new());

        let ranged_pairs = extract_ranged_pairs(_STR_RANGES_01);
        let char_processor = CharProcessor {};
        // Let borrower of the output cursor expire before reacquiring the output cursor
        char_processor.process_lines(
            &CharUtf8LineProcessor {},
            input,
            &mut BufWriter::new(&mut out_cursor),
            &CharContext {
                ranged_pairs: &ranged_pairs,
            },
        );

        out_cursor.seek(std::io::SeekFrom::Start(0)).unwrap();
        // Read the fake "file's" contents into a vector
        let mut out = Vec::new();
        out_cursor.read_to_end(&mut out).unwrap();
        assert_eq!(_STR_BIRDS_OUTPUT.as_bytes().to_vec(), out);
    }

    #[test]
    fn test_process_ascii_fields_for_line_ignored_delim() {
        let line_processor = FieldUtf8LineProcessor {};
        let line = "1234";
        let delim = ":";
        let ranged_pairs: Vec<(usize, usize)> = vec![(2, 2), (4, 6)];
        assert_eq!(
            vec![10],
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &ranged_pairs
                }
            )
        );
    }

    #[test]
    fn test_process_ascii_fields_for_line_leading_delim() {
        let line_processor = FieldUtf8LineProcessor {};
        let line = ":1234";
        let delim = ":";
        let ranged_pairs: Vec<(usize, usize)> = vec![(2, 2), (4, 6)];
        assert_eq!(
            "1234\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &ranged_pairs
                }
            )
        );
    }

    #[test]
    fn test_process_ascii_fields_for_line_trailing_delim() {
        let line_processor = FieldUtf8LineProcessor {};
        let line = "1234:";
        let delim = ":";
        let ranged_pairs: Vec<(usize, usize)> = vec![(2, 2), (4, 6)];
        assert_eq!(
            "\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &ranged_pairs
                }
            )
        );
    }

    #[test]
    fn test_process_ascii_fields_for_line_1st_field_empty() {
        let line_processor = FieldUtf8LineProcessor {};
        let line = ":1:2:3";
        let delim = ":";
        assert_eq!(
            ":2\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (3, 3)]
                },
            )
        );
        assert_eq!(
            ":2:3\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (3, 3), (4, 4)]
                }
            )
        );
        assert_eq!(
            ":3\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (4, 4)]
                }
            )
        );
        assert_eq!(
            ":2:3\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (3, 4)]
                }
            )
        );
        assert_eq!(
            ":2:3\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (3, 5)]
                }
            )
        );
    }

    #[test]
    fn test_process_utf8_fields_for_line_1st_field_empty() {
        let line_processor = FieldUtf8LineProcessor {};
        let line = ":🐣:🐥:🐓";
        let delim = ":";
        assert_eq!(
            ":🐥\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (3, 3)]
                }
            )
        );
        assert_eq!(
            ":🐥:🐓\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (3, 3), (4, 4)]
                }
            )
        );
        assert_eq!(
            ":🐓\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (4, 4)]
                }
            )
        );
        assert_eq!(
            ":🐥:🐓\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (3, 4)]
                }
            )
        );
        assert_eq!(
            ":🐥:🐓\n".as_bytes().to_vec(),
            line_processor.process(
                line,
                &FieldContext {
                    delim,
                    ranged_pairs: &vec![(1, 1), (3, 5)]
                }
            )
        );
    }
}
