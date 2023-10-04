use core::ops::Range;
use std::io::SeekFrom;

pub struct LineInfo {
    pub start: usize,
    pub number: usize,
}

pub fn create<I: Iterator<Item = u8>>(iter: &mut I, start: usize) -> LineInfo {
    let mut line_number = 1;
    let mut line_start = 0;
    let mut offset = 0;
    while let Some(c) = iter.next() {
        if offset == start {
            break;
        }

        offset += 1;

        if c == b'\n' {
            line_number += 1;
            line_start = offset;
        }
    }
    LineInfo {
        start: line_start,
        number: line_number,
    }
}

pub fn print_line<I: Iterator<Item = u8> + std::io::Seek, W: std::fmt::Write>(iter: &mut I, start: usize, write: &mut W) {
    iter.seek(SeekFrom::Start(start as u64)).unwrap();

    while let Some(c) = iter.next() {
        if c != b'\n' && c != b'\r' {
            write!(write, "{}", c as char).unwrap();
        } else {
            break;
        }
    }

    writeln!(write).unwrap();
}

pub fn mark_range<W: std::fmt::Write>(line_start: usize, range: Range<usize>, write: &mut W) {
    for _ in line_start..range.start {
        write!(write, " ").unwrap();
    }
    for _ in range {
        write!(write, "^").unwrap();
    }
    writeln!(write).unwrap();
}
