use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use lazy_static::lazy_static;
use unicode_width::UnicodeWidthStr;

use crate::charset::{LAT1_MAP, VT100_MAP};
use crate::modes::{DECAWM, DECTCEM};
use crate::parser_listener::ParserListener;

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

pub enum Charset {
    G0,
    G1,
}

pub struct Screen<'a> {
    pub savepoints: Vec<Savepoint>,
    pub columns: u32,
    pub lines: u32,
    pub dirty: HashSet<u32>,
    pub margins: Option<Margins>,
    pub buffer: HashMap<u32, HashMap<u32, CharOpts>>,
    pub mode: HashSet<u32>,
    pub title: String,
    pub icon_name: String,
    pub charset: Charset,
    pub g0_charset: &'a [char; 256],
    pub g1_charset: &'a [char; 256],
    pub tabstops: HashSet<u32>,
    pub cursor: Cursor,
    pub saved_columns: Option<u32>,
}

impl<'a> Display for Screen<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Screen ({}, {})", self.columns, self.lines))
    }
}

impl<'a> Screen<'a> {
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

impl<'a> ParserListener for Screen<'a> {
    fn alignment_display(&self) {
        todo!()
    }

    fn define_charset(&self, code: &str, mode: &str) {
        todo!()
    }

    ///Reset the terminal to its initial state.
    ///* Scrolling margins are reset to screen boundaries.
    ///* Cursor is moved to home location -- ``(0, 0)`` and its
    ///  attributes are set to defaults.
    ///* Screen is cleared -- each character is reset to default char
    ///* Tabstops are reset to "every eight columns".
    ///* All lines are marked as dirty.
    ///
    ///.. note::
    ///
    ///   Neither VT220 nor VT102 manuals mention that terminal modes
    ///   and tabstops should be reset as well, thanks to
    ///   :manpage:`xterm` -- we now know that.
    fn reset(&mut self) {
        self.dirty.clear();
        self.dirty.extend(0..self.lines);
        self.buffer.clear();
        self.margins = None;

        self.mode = _DEFAULT_MODE.clone();

        self.title = "".to_owned();
        self.icon_name = "".to_owned();

        self.charset = Charset::G0;
        self.g0_charset = &LAT1_MAP;
        self.g1_charset = &VT100_MAP;

        // From ``man terminfo`` -- "... hardware tabs are initially
        // set every `n` spaces when the terminal is powered up. Since
        // we aim to support VT102 / VT220 and linux -- we use n = 8.
        self.dirty.clear();
        self.dirty.extend((8..self.columns).step_by(8));

        self.cursor = Cursor {
            x: 0,
            y: 0,
            hidden: false,
            attr: CharOpts::default(),
        };
        self.cursor_position(None, None);

        self.saved_columns = None
    }

    fn index(&self) {
        todo!()
    }

    fn linefeed(&self) {
        todo!()
    }

    fn reverse_index(&self) {
        todo!()
    }

    fn set_tab_stop(&self) {
        todo!()
    }

    fn save_cursor(&self) {
        todo!()
    }

    fn restore_cursor(&self) {
        todo!()
    }

    fn shift_out(&self) {
        todo!()
    }

    fn shift_in(&self) {
        todo!()
    }

    fn bell(&self) {
        todo!()
    }

    fn backspace(&self) {
        todo!()
    }

    fn tab(&self) {
        todo!()
    }

    fn cariage_return(&self) {
        todo!()
    }

    fn draw(&self, input: &str) {
        todo!()
    }

    fn insert_characters(&self, count: Option<u32>) {
        todo!()
    }

    fn cursor_up(&self, count: Option<u32>) {
        todo!()
    }

    fn cursor_down(&self, count: Option<u32>) {
        todo!()
    }

    fn cursor_forward(&self, count: Option<u32>) {
        todo!()
    }

    fn cursor_back(&self, count: Option<u32>) {
        todo!()
    }

    fn cursor_down1(&self, count: Option<u32>) {
        todo!()
    }

    fn cursor_up1(&self, count: Option<u32>) {
        todo!()
    }

    fn cursor_to_column(&self, character: Option<u32>) {
        todo!()
    }

    fn cursor_position(&self, line: Option<u32>, character: Option<u32>) {
        todo!()
    }

    fn erase_in_display(&self, erase_page: Option<u32>) {
        todo!()
    }

    fn erase_in_line(&self, erase_line: Option<u32>) {
        todo!()
    }

    fn insert_lines(&self, count: Option<u32>) {
        todo!()
    }

    fn delete_lines(&self, count: Option<u32>) {
        todo!()
    }

    fn delete_characters(&self, count: Option<u32>) {
        todo!()
    }

    fn erase_characters(&self, count: Option<u32>) {
        todo!()
    }

    fn report_device_attributes(&self, attribute: Option<u32>) {
        todo!()
    }

    fn cursor_to_line(&self, count: Option<u32>) {
        todo!()
    }

    fn clear_tab_stop(&self, option: Option<u32>) {
        todo!()
    }

    fn set_mode(&self, modes: &[u32]) {
        todo!()
    }

    fn reset_mode(&self, modes: &[u32]) {
        todo!()
    }

    fn select_graphic_rendition(&self, modes: &[u32]) {
        todo!()
    }
}
