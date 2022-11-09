use std::{fs, io::{self, Write, BufWriter}};
use std::cmp;
use terminal_size::{Height, terminal_size};

use clap::Parser;
use crate::runner::arguments::Args;
mod runner;

fn main() {
    let args = Args::parse();
    let contents = fs::read_to_string(&args.path)
        .expect("Could not read file.");
    let mut stdout = io::stdout();
    // (Buf) Wraps stdout in a buffer.
    let mut stdout = BufWriter::new(stdout);

    let size = terminal_size()
        .expect("Unable to get terminal size");

    let start_line = 0;
    let window_size = cmp::min(args.lines,  if let Height(h) = size.1 { i32::from(h) } else { 256 });

    for (line_no, line) in contents.as_str().lines().enumerate().filter(|&(i, _)|{
        (start_line..(start_line + window_size as usize)).contains(&i)
    }) {
            writeln!(stdout, "{}: {}", line_no + 1, line);
    }
}
