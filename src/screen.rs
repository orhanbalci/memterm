use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use lazy_static::lazy_static;
use unicode_width::UnicodeWidthStr;

use crate::modes::{DECAWM, DECTCEM};

pub struct CharOpts {
    pub data: String,
    pub fg: String,
    pub bg: String,
    pub bold: bool,
    pub italics: bool,
    pub underscore: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub blink: bool,
}

impl Default for CharOpts {
    fn default() -> Self {
        Self {
            data: " ".to_owned(),
            fg: "default".to_owned(),
            bg: "default".to_owned(),
            bold: false,
            italics: false,
            underscore: false,
            strikethrough: false,
            reverse: false,
            blink: false,
        }
    }
}

pub struct Cursor {
    pub x: u32,
    pub y: u32,
    pub attr: CharOpts,
    pub hidden: bool,
}

/// A container for screen's scroll margins
pub struct Margins {
    pub top: u32,
    pub bottom: u32,
}

/// A container for savepoint, created on :data:`~pyte.escape.DECSC`.
pub struct Savepoint {
    pub cursor: Cursor,
    pub g0_charset: String,
    pub g1_charset: String,
    pub charset: u32,
    pub origin: bool,
    pub wrap: bool,
}

lazy_static! {
    static ref _DEFAULT_MODE: HashSet<u32> = {
        let mut m = HashSet::new();
        m.insert(DECAWM);
        m.insert(DECTCEM);
        m
    };
}

pub struct Screen {
    pub savepoints: Vec<Savepoint>,
    pub columns: u32,
    pub lines: u32,
    pub dirty: HashSet<u32>,
    pub margins: Option<Margins>,
    pub buffer: HashMap<u32, HashMap<u32, CharOpts>>,
}

impl Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Screen ({}, {})", self.columns, self.lines))
    }
}

impl Screen {
    ///A list of screen lines as unicode strings.
    pub fn display(&mut self) -> Vec<String> {
        let render = |line: &mut HashMap<u32, CharOpts>| -> String {
            let mut result = String::new();
            let mut is_wide_char = false;
            for x in 0..self.columns {
                if is_wide_char {
                    is_wide_char = false;
                    continue;
                }
                let char = line.entry(x).or_insert(CharOpts::default()).data.clone();
                is_wide_char = UnicodeWidthStr::width(
                    char.get(0..1)
                        .expect("at least 1 character empty string expected"),
                ) == 2;
                result.push_str(&char);
            }

            return result;
        };

        let mut result = Vec::new();
        for y in 0..self.lines {
            let line_render = render(
                &mut self
                    .buffer
                    .entry(y)
                    .or_insert(HashMap::<u32, CharOpts>::new()),
            );
            result.push(line_render);
        }

        return result;
    }
}
