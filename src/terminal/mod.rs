use std::io::{self, Read};

use termios::{Termios, ICANON, ECHO, TCSANOW, tcsetattr};

pub fn read_char() -> u8 {
    let stdin = 0; // couldn't get std::os::unix::io::FromRawFd to work on /dev/stdin or /dev/tty
    let termios = Termios::from_fd(stdin).unwrap();

    let mut new_termios = termios.clone();  // make a mutable copy of termios that we will modify
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode

    tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();

    let mut reader = io::stdin();
    let mut buffer = [0;1];  // read exactly one byte

    reader.read_exact(&mut buffer).unwrap();
    tcsetattr(stdin, TCSANOW, & termios).unwrap();  // reset the stdin to 

    buffer[0]
}