#![feature(iter_advance_by)]

macro_rules! ascii {
    ($($xx:literal/$yy:literal), *) => {
        unsafe { std::str::from_utf8_unchecked(&[$(($xx << 4) + $yy),*]) }
    };
}

pub(crate) use ascii;
pub mod charset;
pub mod control;
pub mod modes;
pub mod parser;
pub mod parser_listener;
pub mod parser_printer;
pub mod screen;
