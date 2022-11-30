use std::{fs, io::{BufReader, Seek, Read, BufRead, self}};

type EOF = bool;
pub trait FileRangeReader
where Self: Sized {
    fn read_range(self, file: &fs::File) -> io::Result<(Vec<String>, EOF, Self)>;
}


#[derive(Clone, Copy)]
pub struct FileCursorItem {
    pub cursor: u64,
    pub line_index: u64
}
#[derive(Clone, Copy)]
pub struct FileCursor {
    pub window_size: usize,
    pub prev_line: FileCursorItem,
    pub prev_page: FileCursorItem,
    pub current_line: FileCursorItem,
    pub next_line: FileCursorItem,
    pub next_page: FileCursorItem,
}
impl FileCursor {
    pub fn new(window_size: usize) -> Self {
        FileCursor {
            window_size,
            prev_line: FileCursorItem{ cursor: 0, line_index: 0 },
            prev_page: FileCursorItem{ cursor: 0, line_index: 0 },
            current_line: FileCursorItem{ cursor: 0, line_index: 0 },
            next_line: FileCursorItem{ cursor: 0, line_index: 0 },
            next_page: FileCursorItem{ cursor: 0, line_index: 0 },
        }
    }
    pub fn from(source: Self) -> Self {
        FileCursor {
            window_size: source.window_size,
            prev_line: source.prev_line,
            prev_page: source.prev_page,
            current_line: source.current_line,
            next_line: source.next_line,
            next_page: source.next_page,
        }
    }
    pub fn to_prev_line(self) -> Self {
        let mut next = Self::from(self);
        next.current_line = self.prev_line;
        next
    }
    pub fn to_prev_page(self) -> Self {
        let mut next = Self::from(self);
        next.current_line = self.prev_page;
        next
    }
    pub fn to_next_line(self) -> Self {
        let mut next = Self::from(self);
        next.current_line = self.next_line;
        next
    }
    pub fn to_next_page(self) -> Self {
        let mut next = Self::from(self);
        next.current_line = self.next_page;
        next
    }
}

const VIRTUAL_PAGE_SIZE: u64 = 1024 * 8;
impl FileRangeReader for FileCursor {
    fn read_range(self, file: &fs::File) -> io::Result<(Vec<String>, EOF, Self)> {
        let mut file = BufReader::new(file);
        
        let virtual_page_start = std::cmp::max(0, self.current_line.cursor as i64 - VIRTUAL_PAGE_SIZE as i64) as u64;
        let virtual_page_end = self.current_line.cursor + VIRTUAL_PAGE_SIZE;

        match file.seek(std::io::SeekFrom::Start(virtual_page_start)) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        let mut lines = file.by_ref().take(virtual_page_end).lines();
        let contents = lines.by_ref().map(
            |res| {
                match res {
                    Ok(line) => line,
                    Err(_) => String::from(""),
                }
            }
        ).collect::<Vec<String>>();

        let accumulated_lengths= contents.iter()
            .map(|line| line.as_bytes().len() as i32)
            .scan(0, |sum, val| {
                *sum += val + 1;
                Some(*sum)
            })
            .collect::<Vec<i32>>();

        let virtual_total_length = accumulated_lengths.len();
        let actual_first_index = accumulated_lengths.iter().enumerate().find(
            |(_, &size)| {
                if virtual_page_start == 0 {
                    size >= self.current_line.cursor as i32
                } else {
                    size >= VIRTUAL_PAGE_SIZE as i32
                }
            }
        ).unwrap_or((0, &0)).0;
        let virtual_last_index = accumulated_lengths.iter().enumerate().filter(
        |&(i, _)| i as i32 <= virtual_total_length as i32 - self.window_size as i32
            ).last().unwrap_or((0, &0)).0;

        let current = actual_first_index;
        let prev_page = std::cmp::max(current as i32 - self.window_size as i32, 0) as usize;
        let prev_line = std::cmp::max(current as i32 - 1, 0) as usize;
        let next_line = std::cmp::min(current as i32 + 1, virtual_last_index as i32) as usize;
        let next_page = std::cmp::min(current as i32 + self.window_size as i32, virtual_last_index as i32) as usize;

        let eof = virtual_total_length == 0 || virtual_last_index == current;
        Ok(
            (
                contents.get(current..std::cmp::min(contents.len(), current + self.window_size)).unwrap_or(&[]).to_vec(),
                eof,
                FileCursor {
                    window_size: self.window_size,
                    prev_page: FileCursorItem {
                        cursor: *accumulated_lengths.get(prev_page).unwrap_or(&0) as u64,
                        line_index: self.current_line.line_index - current.abs_diff(prev_page) as u64,
                    },
                    prev_line: FileCursorItem {
                        cursor: *accumulated_lengths.get(prev_line).unwrap_or(&0) as u64,
                        line_index: self.current_line.line_index - current.abs_diff(prev_line) as u64
                    },
                    current_line: FileCursorItem {
                        cursor: *accumulated_lengths.get(current).unwrap_or(&0) as u64,
                        line_index: self.current_line.line_index,
                    },
                    next_line: FileCursorItem {
                        cursor: *accumulated_lengths.get(next_line).unwrap_or(&0) as u64,
                        line_index: self.current_line.line_index + current.abs_diff(next_line) as u64,
                    },
                    next_page: FileCursorItem {
                        cursor: *accumulated_lengths.get(next_page).unwrap_or(&0) as u64,
                        line_index: self.current_line.line_index + current.abs_diff(next_page) as u64,
                    },
                }
            )
        )
    }
}