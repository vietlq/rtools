//! `rtools-traits` library provides common traits for tools
//! that reimplement GNU tools.
//!

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::str;

pub trait LineProcessorT<C> {
    fn process(&self, line: &str, context: &C) -> Vec<u8>;
}

pub trait RtoolT<C, P: LineProcessorT<C>> {
    /// Generic line processor that delegates to concrete line processors
    fn process_lines<R: Read, W: Write>(
        &self,
        line_processor: &P,
        input: BufReader<R>,
        output: &mut BufWriter<W>,
        context: &C,
    ) {
        for line in input.lines() {
            let out_bytes = line_processor.process(&line.unwrap(), context);

            output.write(&out_bytes).unwrap();
        }
    }

    /// Process readable object: Send input to the line processor
    fn process_readable<R: std::io::Read, W: std::io::Write>(
        &self,
        line_processor: &P,
        input: BufReader<R>,
        output: &mut BufWriter<W>,
        context: &C,
    ) {
        self.process_lines(line_processor, input, output, &context);
    }

    /// Process files: Send them to the line processor
    fn process_files<W: std::io::Write>(
        &self,
        line_processor: &P,
        files: &Vec<&str>,
        writable: W,
        context: &C,
    ) {
        // TODO: What can we do about encodings? ASCII vs UTF-8 vs X
        let mut output = BufWriter::new(writable);

        for file in files {
            match File::open(file) {
                Ok(file) => {
                    let input = BufReader::new(file);
                    self.process_readable(line_processor, input, &mut output, context);
                }
                Err(err) => {
                    eprintln!("Could not read the file `{}`. The error: {:?}", file, err);
                }
            }
        }
    }

    /// Read lines from the input files or STDIN and send them to the processor. Results go to defined output
    fn process<W: std::io::Write>(&self, line_processor: &P, files: &Vec<&str>, writable: &mut W, context: &C) {
        // TODO: What can we do about encodings? ASCII vs UTF-8 vs X
        // TODO: Add a method to take BufWriter<W> instead of assuming STDOUT
        if files.is_empty() {
            self.process_readable(
                line_processor,
                BufReader::new(std::io::stdin()),
                &mut BufWriter::new(writable),
                context,
            );
        } else {
            self.process_files(line_processor, &files, writable, context);
        }
    }

    /// Read lines from the input files or STDIN and write results to STDOUT
    fn process_to_stdout(&self, line_processor: &P, files: &Vec<&str>, context: &C) {
        self.process(line_processor, files, &mut BufWriter::new(std::io::stdout()), context);
    }
}
