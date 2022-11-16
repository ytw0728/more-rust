use std::{fs, io::{self, Write, BufWriter}};
use std::cmp;
use terminal_size::{Height, terminal_size};

use clap::Parser;
use crate::runner::arguments::Args;

mod runner;
mod terminal;

fn main() {
    let args = Args::parse();
    let contents = fs::read_to_string(&args.path)
        .expect("Could not read file.");
    let mut stdout = io::stdout();
    // (Buf) Wraps stdout in a buffer.
    let mut stdout = BufWriter::new(stdout);

    let size = terminal_size()
        .expect("Unable to get terminal size");

    let mut start_line = 0 as i32;
    let terminal_size = match size.1 {
        Height(h) => (h - 2) as i32,
    };
    let window_size = cmp::min(args.lines.unwrap_or(terminal_size), terminal_size);
    let total_size = contents.as_str().lines().into_iter().count() as i32;

    // clear screen
    for _ in 0..terminal_size + 1 {
        writeln!(stdout, "")
                .expect("Failed to write file content.");
    }
    stdout.flush()
            .expect("Failed to flush stdout.");

    loop {
        // writeln!(stdout, "\x1B[2J\x1B[1;1H") // all clear는 너무 버벅댄다.
        writeln!(stdout, "\x1b[1;1H")
                .expect("Failed to write file content.");

        for (line_no, line) in contents.as_str().lines().enumerate().filter(|&(i, _)|{
            (start_line..(start_line + window_size)).contains(&(i as i32))
        }) {
            writeln!(stdout, "\x1b[2K{}:\t{}", line_no + 1, line)
                .expect("Failed to write file content.");
        }

        write!(stdout, "\x1b[{};1H:", terminal_size + 2)
                .expect("Failed to write file content.");

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
                        start_line = cmp::max(start_line - 1, 0);
                        break Command::UP
                    },
                    // arrow up / ANSI escape sequences (27, 91, 65) ^[A
                    b'k' => {
                        start_line = cmp::max(start_line - 1, 0);
                        break Command::UP
                    }
                    // arrow right / ANSI escape sequences (27, 91, 67) ^[C
                    b'l' => {
                        start_line = cmp::min(start_line + 1, total_size);
                        break Command::DOWN
                    },
                    // arrow down / ANSI escape sequences (27, 91, 66) ^[B
                    b'j' => {
                        start_line = cmp::min(start_line + 1, total_size);
                        break Command::DOWN;
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