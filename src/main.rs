use std::{fs, io::{self, Write, BufWriter}};

use file::{FileCursor, FileRangeReader};


use clap::Parser;
use terminal_size::{terminal_size, Height};
use crate::runner::arguments::Args;

mod file;
mod runner;
mod terminal;

fn main() {
    let args = Args::parse();
    let file = fs::File::open(&args.path)
        .unwrap_or_else(|_| panic!("Could not open file, {}.", args.path));

    let stdout = io::stdout();
    // (Buf) Wraps stdout in a buffer.
    let mut stdout = BufWriter::new(stdout);

    
    let size = terminal_size()
        .expect("Unable to get terminal size");

    // let mut start_line = 0_i32;
    let terminal_size = match size.1 {
        Height(h) => (h - 2) as i32,
    };
    let window_size = std::cmp::min(args.lines.unwrap_or(terminal_size), terminal_size);

    // clear screen
    for _ in 0..terminal_size + 1 {
        writeln!(stdout)
                .expect("Failed to write file content.");
    }
    stdout.flush()
            .expect("Failed to flush stdout.");

    let mut cursor = FileCursor::new(window_size as usize);        
    loop {
        // writeln!(stdout, "\x1B[2J\x1B[1;1H") // all clear는 너무 버벅댄다.
        writeln!(stdout, "\x1b[1;1H")
                .expect("Failed to write file content.");

        let (contents, eof, next_cursor) = cursor.read_range(&file)
        .expect("Failed to read file contents.");
        cursor = next_cursor;

        for (line_no, line) in contents.iter().enumerate() {
            writeln!(stdout, "\x1b[2K{}:\t{}", cursor.current_line.line_index + line_no as u64 + 1, line)
            .expect("Failed to write file content.");
        }

        if eof {
            write!(stdout, "\x1b[{};1H\x1b[2KEND", terminal_size + 2)
                .expect("Failed to write file content.");
        } else {
            write!(stdout, "\x1b[{};1H\x1b[2K:", terminal_size + 2)
                .expect("Failed to write file content.");
        }

        stdout.flush()
            .expect("Failed to flush stdout.");

        match 
            loop {
                match terminal::read_char() {
                    b'q' => {
                        break Command::QUIT
                    },
                    // arrow left / ANSI escape sequences (27, 91, 68) ^[D
                    b'h' => {
                        cursor = cursor.to_prev_line();
                        break Command::UP
                    },
                    // arrow up / ANSI escape sequences (27, 91, 65) ^[A
                    b'k' => {
                        cursor = cursor.to_prev_line();
                        break Command::UP
                    }
                    // arrow right / ANSI escape sequences (27, 91, 67) ^[C
                    b'l' => {
                        cursor = cursor.to_next_line();
                        break Command::DOWN
                    },
                    // arrow down / ANSI escape sequences (27, 91, 66) ^[B
                    b'j' => {
                        cursor = cursor.to_next_line();
                        break Command::DOWN;
                    },
                    b'f' => {
                        cursor = cursor.to_next_page();
                        break Command::DOWN
                    },
                    b'b' => {
                        cursor = cursor.to_prev_page();
                        break Command::UP
                    },
                    _ => ()
                }
            }
        {
            Command::QUIT => break,
            _ => continue,
        }
    }
}

enum Command {
    QUIT,
    UP,
    DOWN,
}