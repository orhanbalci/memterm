use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use lazy_static::lazy_static;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::{char, UnicodeNormalization};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::charset::{LAT1_MAP, MAPS, VT100_MAP};
use crate::graphics::{BG_256, BG_AIXTERM, BG_ANSI, FG_256, FG_AIXTERM, FG_ANSI, FG_BG_256, TEXT};
use crate::modes::{DECAWM, DECCOLM, DECOM, DECSCNM, DECTCEM, IRM, LNM};
use crate::parser_listener::ParserListener;

#[derive(Clone)]
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

impl CharOpts {
    fn clone_with_data(&self, data: String) -> Self {
        Self {
            data,
            fg: self.fg.clone(),
            bg: self.bg.clone(),
            bold: self.bold,
            italics: self.italics,
            underscore: self.underscore,
            strikethrough: self.strikethrough,
            reverse: self.reverse,
            blink: self.blink,
        }
    }

    fn update_from_map(&mut self, map: HashMap<String, String>) {
        for (key, value) in map {
            match key.as_str() {
                "data" => self.data = value,
                "fg" => self.fg = value,
                "bg" => self.bg = value,
                "bold" => self.bold = value.parse().unwrap_or(false),
                "italics" => self.italics = value.parse().unwrap_or(false),
                "underscore" => self.underscore = value.parse().unwrap_or(false),
                "strikethrough" => self.strikethrough = value.parse().unwrap_or(false),
                "reverse" => self.reverse = value.parse().unwrap_or(false),
                "blink" => self.blink = value.parse().unwrap_or(false),
                _ => {}
            }
        }
    }

    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("data".to_string(), self.data.clone());
        map.insert("fg".to_string(), self.fg.clone());
        map.insert("bg".to_string(), self.bg.clone());
        map.insert("bold".to_string(), self.bold.to_string());
        map.insert("italics".to_string(), self.italics.to_string());
        map.insert("underscore".to_string(), self.underscore.to_string());
        map.insert("strikethrough".to_string(), self.strikethrough.to_string());
        map.insert("reverse".to_string(), self.reverse.to_string());
        map.insert("blink".to_string(), self.blink.to_string());
        map
    }
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

#[derive(Clone)]
pub struct Cursor {
    pub x: u32,
    pub y: u32,
    pub attr: CharOpts,
    pub hidden: bool,
}

/// A container for screen's scroll margins
#[derive(Clone, Copy)]
pub struct Margins {
    pub top: u32,
    pub bottom: u32,
}

/// A container for savepoint, created on :data:`~pyte.escape.DECSC`.
pub struct Savepoint {
    pub cursor: Cursor,
    pub g0_charset: [char; 256],
    pub g1_charset: [char; 256],
    pub charset: Charset,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Charset {
    G0,
    G1,
}

pub struct Screen {
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
    pub g0_charset: [char; 256],
    pub g1_charset: [char; 256],
    pub tabstops: HashSet<u32>,
    pub cursor: Cursor,
    pub saved_columns: Option<u32>,
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

    /// Resize the screen to the given size.
    ///
    /// If the requested screen size has more lines than the existing
    /// screen, lines will be added at the bottom. If the requested
    /// size has less lines than the existing screen lines will be
    /// clipped at the top of the screen. Similarly, if the existing
    /// screen has less columns than the requested screen, columns will
    /// be added at the right, and if it has more columns will be
    /// clipped at the right.
    ///
    /// # Arguments
    ///
    /// * `lines` - number of lines in the new screen.
    /// * `columns` - number of columns in the new screen.
    ///
    /// <div class="warning">   If the requested screen size is identical to the current screen
    ///    size, the method does nothing.</div>
    pub fn resize(&mut self, lines: Option<u32>, columns: Option<u32>) {
        let lines = lines.or(Some(self.lines)).expect("can not read lines");
        let columns = columns
            .or(Some(self.columns))
            .expect("can not read columns");

        if lines == self.lines && columns == self.columns {
            return; // No changes.
        }

        self.dirty.extend(0..lines);

        if lines < self.lines {
            self.save_cursor();
            self.cursor_position(Some(0), Some(0));
            self.delete_lines(Some(self.lines - lines)); // Drop from the top.
            self.restore_cursor();
        }

        if columns < self.columns {
            for line in self.buffer.values_mut() {
                for x in columns..self.columns {
                    line.remove(&x);
                }
            }
        }

        (self.lines, self.columns) = (lines, columns);
        self.set_margins(None, None);
    }

    // Select top and bottom margins for the scrolling region.

    // :param int top: the smallest line number that is scrolled.
    // :param int bottom: the biggest line number that is scrolled.
    pub fn set_margins(&mut self, top: Option<u32>, bottom: Option<u32>) {
        // XXX 0 corresponds to the CSI with no parameters.
        if top.or(Some(0)).expect("unexpected bottom value") == 0 && bottom.is_none() {
            self.margins = None;
            return;
        }

        let margins_inner = self
            .margins
            .or(Some(Margins { top: 0, bottom: self.lines - 1 }))
            .expect("unexpected margins value");

        // Arguments are 1-based, while :attr:`margins` are zero
        // based -- so we have to decrement them by one. We also
        // make sure that both of them is bounded by [0, lines - 1].
        let top = if top.is_none() {
            margins_inner.top
        } else {
            u32::max(
                0,
                u32::min(top.expect("unexpected top value") - 1, self.lines - 1),
            )
        };

        let bottom = if bottom.is_none() {
            margins_inner.bottom
        } else {
            u32::max(
                0,
                u32::min(bottom.expect("unexpected bottom value") - 1, self.lines - 1),
            )
        };

        // Even though VT102 and VT220 require DECSTBM to ignore
        // regions of width less than 2, some programs (like aptitude
        // for example) rely on it. Practicality beats purity.
        if bottom - top >= 1 {
            self.margins = Some(Margins { top: top, bottom: bottom });
            // The cursor moves to the home position when the top and
            // bottom margins of the scrolling region (DECSTBM) changes.
            self.cursor_position(None, None);
        }
    }

    // Ensure the cursor is within horizontal screen bounds."""
    pub fn ensure_hbounds(&mut self) {
        self.cursor.x = u32::min(u32::max(0, self.cursor.x), self.columns - 1)
    }

    // Ensure the cursor is within vertical screen bounds.
    pub fn ensure_vbounds(&mut self, use_margins: Option<bool>) {
        let (top, bottom) = if (use_margins.unwrap_or(false) || self.mode.contains(&DECOM))
            && self.margins.is_some()
        {
            let Margins { top, bottom } = self.margins.unwrap();
            (top, bottom)
        } else {
            (0, self.lines - 1)
        };

        self.cursor.y = u32::min(u32::max(top, self.cursor.y), bottom)
    }

    /// Write to the process input.
    pub fn write_process_input(&self, _input: &str) {
        // Implementation for writing to the process input.
    }
}

impl ParserListener for Screen {
    /// Fills screen with uppercase E's for screen focus and alignment.
    fn alignment_display(&mut self) {
        self.dirty.extend(0..self.lines);
        for y in 0..self.lines {
            let line = self.buffer.entry(y).or_insert_with(HashMap::new);
            for x in 0..self.columns {
                let char_opts = line.entry(x).or_insert_with(CharOpts::default);
                char_opts.data = "E".to_string();
            }
        }
    }

    /// Define ``G0`` or ``G1`` charset.
    ///
    /// # Arguments
    /// * `code` - character set code, should be a character
    ///  from ``"B0UK"``, otherwise ignored.
    ///
    /// * `mode` - if ``"("`` ``G0`` charset is defined, if
    ///  ``")"`` we operate on ``G1``.
    ///
    /// **Warning:** User-defined charsets are currently not supported.
    fn define_charset(&mut self, code: &str, mode: &str) {
        if MAPS.keys().any(|&a| a == code) {
            if mode == "(" {
                self.g0_charset = MAPS
                    .get(code)
                    .expect(&format!("unexpected character map key {}", code))
                    .clone();
            } else if mode == ")" {
                self.g1_charset = MAPS
                    .get(code)
                    .expect(&format!("unexpected character map key {}", code))
                    .clone();
            }
        }
    }

    /// Reset the terminal to its initial state.
    ///
    /// - Scrolling margins are reset to screen boundaries.
    /// - Cursor is moved to home location `(0, 0)` and its attributes are set to defaults.
    /// - Screen is cleared, each character is reset to default char.
    /// - Tabstops are reset to "every eight columns".
    /// - All lines are marked as dirty.
    ///
    /// **Warning**
    /// Neither VT220 nor VT102 manuals mention that terminal modes and tabstops should be reset as well.
    /// Thanks to `xterm` -- we now know that.
    fn reset(&mut self) {
        self.dirty.clear();
        self.dirty.extend(0..self.lines);
        self.buffer.clear();
        self.margins = None;

        self.mode = _DEFAULT_MODE.clone();

        self.title = "".to_owned();
        self.icon_name = "".to_owned();

        self.charset = Charset::G0;
        self.g0_charset = LAT1_MAP.clone();
        self.g1_charset = VT100_MAP.clone();

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

    /// Move the cursor down one line in the same column. If the
    /// cursor is at the last line, create a new line at the bottom.
    fn index(&mut self) {
        let Margins { top, bottom } = self
            .margins
            .or(Some(Margins { top: 0, bottom: self.lines - 1 }))
            .expect("unexpected margin found");

        if self.cursor.y == bottom {
            // TODO: mark only the lines within margins?
            self.dirty.extend(0..self.lines);
            let mut new_buffer: HashMap<u32, HashMap<u32, CharOpts>> = HashMap::new();

            self.buffer.iter().for_each(|(&outer_key, inner_map)| {
                if outer_key >= top {
                    new_buffer.insert(outer_key + 1, (*inner_map).clone());
                } else if outer_key < bottom {
                    new_buffer.insert(outer_key + 1, (*inner_map).clone());
                }
                new_buffer.insert(outer_key, (*inner_map).clone());
            });
            new_buffer.remove(&bottom);
            new_buffer.insert(bottom, HashMap::new());
            self.buffer = new_buffer;
        } else {
            self.cursor_down(None);
        }
    }

    // Perform an index and, if LNM is set, a  carriage return.
    fn linefeed(&mut self) {
        self.index();
        if self.mode.contains(&LNM) {
            self.cariage_return();
        }
    }

    // Move the cursor up one line in the same column. If the cursor
    // at the first line, create a new line at the top.
    fn reverse_index(&mut self) {
        let (top, bottom) = match &self.margins {
            Some(margins) => (margins.top, margins.bottom),
            None => (0, self.lines - 1),
        };

        if self.cursor.y == top {
            // TODO: mark only the lines within margins?
            for i in 0..self.lines {
                self.dirty.insert(i);
            }
            for y in (top + 1..=bottom).rev() {
                if let Some(line) = self.buffer.get(&(y - 1)).cloned() {
                    self.buffer.insert(y, line);
                }
            }
            self.buffer.remove(&top);
        } else {
            self.cursor_up(None);
        }
    }

    // Set a horizontal tab stop at cursor position.
    fn set_tab_stop(&mut self) {
        self.tabstops.insert(self.cursor.x);
    }

    //  Push the current cursor position onto the stack.
    fn save_cursor(&mut self) {
        self.savepoints.push(Savepoint {
            cursor: self.cursor.clone(),
            g0_charset: self.g0_charset.clone(),
            g1_charset: self.g1_charset.clone(),
            charset: self.charset,
            origin: self.mode.contains(&DECOM),
            wrap: self.mode.contains(&DECAWM),
        })
    }

    /// Set the current cursor position to whatever cursor is on top
    /// of the stack.
    fn restore_cursor(&mut self) {
        if self.savepoints.len() > 0 {
            let savepoint = self
                .savepoints
                .pop()
                .expect("can not retrieve last savepoint");

            self.g0_charset = savepoint.g0_charset.clone();
            self.g1_charset = savepoint.g1_charset.clone();
            self.charset = savepoint.charset;

            if savepoint.origin {
                self.set_mode(&[DECOM], false)
            }
            if savepoint.wrap {
                self.set_mode(&[DECAWM], false)
            }

            self.cursor = savepoint.cursor;
            self.ensure_hbounds();
            self.ensure_vbounds(Some(true));
        } else {
            // If nothing was saved, the cursor moves to home position;
            // origin mode is reset. :todo: DECAWM?
            self.reset_mode(&[DECOM], false);
            self.cursor_position(None, None);
        }
    }

    /// Select ``G1`` character set.
    fn shift_out(&mut self) {
        self.charset = Charset::G1;
    }

    /// Select ``G0`` character set.
    fn shift_in(&mut self) {
        self.charset = Charset::G0;
    }

    /// Bell stub -- the actual implementation should probably be by the end-user.
    fn bell(&mut self) {}

    /// Move cursor to the left one or keep it in its position if
    /// it's at the beginning of the line already.
    fn backspace(&mut self) {
        self.cursor_back(None);
    }

    /// Move to the next tab space, or the end of the screen if there
    /// aren't anymore left.
    fn tab(&mut self) {
        // Convert HashSet to a Vec
        let mut vec: Vec<_> = self.tabstops.iter().collect();
        // Sort the Vec
        vec.sort();

        let mut column: u32 = 0;
        for &stop in vec.iter() {
            if self.cursor.x < *stop {
                column = *stop;
                break;
            }
        }

        if column == 0 {
            column = self.columns - 1;
        }

        self.cursor.x = column;
    }

    /// Move the cursor to the beginning of the current line.
    fn cariage_return(&mut self) {
        self.cursor.x = 0;
    }

    /// Display decoded characters at the current cursor position and
    /// advances the cursor if `DECAWM` is set.
    ///
    /// # Parameters
    /// - `data`: Text to display.
    ///
    /// # Version
    /// - Changed in version 0.5.0: Character width is taken into account.
    ///   Specifically, zero-width and unprintable characters do not affect
    ///   screen state. Full-width characters are rendered into two consecutive
    ///   character containers.
    fn draw(&mut self, data: &str) {
        let data = data
            .chars()
            .map(|c| {
                if self.charset == Charset::G1 {
                    self.g1_charset[c as usize]
                } else {
                    self.g0_charset[c as usize]
                }
            })
            .collect::<String>();

        for char in data.chars() {
            let char_width = char.width().unwrap_or(0);

            // If this was the last column in a line and auto wrap mode is
            // enabled, move the cursor to the beginning of the next line,
            // otherwise replace characters already displayed with newly
            // entered.
            if self.cursor.x == self.columns {
                if self.mode.contains(&DECAWM) {
                    self.dirty.insert(self.cursor.y);
                    self.cariage_return();
                    self.linefeed();
                } else if char_width > 0 {
                    self.cursor.x = self.cursor.x.saturating_sub(char_width as u32);
                }
            }

            // If Insert mode is set, new characters move old characters to
            // the right, otherwise terminal is in Replace mode and new
            // characters replace old characters at cursor position.
            if self.mode.contains(&IRM) && char_width > 0 {
                self.insert_characters(Some(char_width as u32));
            }

            let line = self
                .buffer
                .entry(self.cursor.y)
                .or_insert_with(HashMap::new);
            if char_width == 1 {
                line.insert(
                    self.cursor.x,
                    self.cursor.attr.clone_with_data(char.to_string()),
                );
            } else if char_width == 2 {
                line.insert(
                    self.cursor.x,
                    self.cursor.attr.clone_with_data(char.to_string()),
                );
                if self.cursor.x + 1 < self.columns {
                    line.insert(
                        self.cursor.x + 1,
                        self.cursor.attr.clone_with_data("".to_string()),
                    );
                }
            } else if char_width == 0 && is_combining_mark(char) {
                if self.cursor.x > 0 {
                    if let Some(last) = line.get_mut(&(self.cursor.x - 1)) {
                        last.data = last.data.nfc().collect::<String>() + &char.to_string();
                    }
                } else if self.cursor.y > 0 {
                    if let Some(last) = self
                        .buffer
                        .get_mut(&(self.cursor.y - 1))
                        .and_then(|l| l.get_mut(&(self.columns - 1)))
                    {
                        last.data = last.data.nfc().collect::<String>() + &char.to_string();
                    }
                }
            } else {
                break; // Unprintable character or doesn't advance the cursor.
            }

            // .. note:: We can't use `cursor_forward()`, because that
            //           way, we'll never know when to linefeed.
            if char_width > 0 {
                self.cursor.x = std::cmp::min(self.cursor.x + char_width as u32, self.columns);
            }
        }

        self.dirty.insert(self.cursor.y);
    }

    /// Insert the indicated # of blank characters at the cursor
    /// position. The cursor does not move and remains at the beginning
    /// of the inserted blank characters. Data on the line is shifted
    /// forward.
    ///
    /// # Arguments
    ///
    /// * `count` - number of characters to insert.
    fn insert_characters(&mut self, count: Option<u32>) {
        self.dirty.insert(self.cursor.y);

        let count = count.unwrap_or(1);
        let line = self
            .buffer
            .get_mut(&self.cursor.y)
            .expect("can not retrieve line");
        for x in (self.cursor.x..self.columns + 1).rev() {
            if x + count <= self.columns {
                let x_val = line.get(&x);
                match x_val {
                    Some(val) => {
                        line.insert(x + count, val.clone());
                    }
                    None => {
                        line.insert(x + count, CharOpts::default());
                    }
                }
            }
            line.insert(x, CharOpts::default());
        }
    }

    fn cursor_up(&mut self, count: Option<u32>) {
        let top = match &self.margins {
            Some(margins) => margins.top,
            None => 0,
        };
        let count = count.unwrap_or(1);
        self.cursor.y = self.cursor.y.saturating_sub(count).max(top);
    }

    fn cursor_down(&mut self, count: Option<u32>) {
        let bottom = match &self.margins {
            Some(margins) => margins.bottom,
            None => self.lines - 1,
        };
        let count = count.unwrap_or(1);
        self.cursor.y = (self.cursor.y + count).min(bottom);
    }

    fn cursor_down1(&mut self, count: Option<u32>) {
        self.cursor_down(count);
        self.cariage_return();
    }

    /// Move cursor right the indicated number of columns. Cursor stops
    /// at the right margin.
    ///
    /// # Parameters
    /// - `count`: Number of columns to skip.
    fn cursor_forward(&mut self, count: Option<u32>) {
        self.cursor.x += count.unwrap_or(1);
        self.ensure_hbounds();
    }

    /// Move cursor left the indicated # of columns. Cursor stops
    /// at left margin.
    ///
    /// # Arguements
    ///
    /// * `count` - number of columns to skip
    fn cursor_back(&mut self, count: Option<u32>) {
        // Handle the case when we've just drawn in the last column
        // and would wrap the line on the next :meth:`draw()` call.
        if self.cursor.x == self.columns {
            self.cursor.x -= 1
        }

        self.cursor.x -= count.unwrap_or(1);
        self.ensure_hbounds();
    }

    fn cursor_up1(&mut self, count: Option<u32>) {
        self.cursor_up(count);
        self.cariage_return();
    }

    fn cursor_to_column(&mut self, character: Option<u32>) {
        self.cursor.x = character.unwrap_or(1) - 1;
        self.ensure_hbounds();
    }

    fn cursor_position(&mut self, line: Option<u32>, column: Option<u32>) {
        let column = column.unwrap_or(1) - 1;
        let mut line = line.unwrap_or(1) - 1;

        // If origin mode (DECOM) is set, line number is relative to the top scrolling margin.
        if let Some(margins) = &self.margins {
            if self.mode.contains(&DECOM) {
                line += margins.top;

                // Cursor is not allowed to move out of the scrolling region.
                if line < margins.top || line > margins.bottom {
                    return;
                }
            }
        }

        self.cursor.x = column;
        self.cursor.y = line;
        self.ensure_hbounds();
        self.ensure_vbounds(None);
    }

    /// Erases display in a specific way.
    ///
    /// Character attributes are set to cursor attributes.
    ///
    /// # Parameters
    ///
    /// - `how`: Defines the way the line should be erased in:
    ///     - `0`: Erases from cursor to end of screen, including cursor position.
    ///     - `1`: Erases from beginning of screen to cursor, including cursor position.
    ///     - `2` and `3`: Erases complete display. All lines are erased and changed to single-width. Cursor does not move.
    /// - `private`: When `true`, only characters marked as erasable are affected. **Not implemented**.
    ///
    /// # Version
    ///
    /// This method accepts any number of positional arguments as some `clear` implementations include a `;` after the first parameter causing the stream to assume a `0` second parameter.
    fn erase_in_display(&mut self, how: Option<u32>, _private: Option<bool>) {
        let interval: std::ops::Range<u32> = match how {
            Some(0) => self.cursor.y + 1..self.lines,
            Some(1) => 0..self.cursor.y,
            Some(2 | 3) => 0..self.lines,
            _ => 0..0, // Handle invalid `how` values
        };

        self.dirty.extend(interval.clone());
        for y in interval.clone() {
            let line = &mut self.buffer.get_mut(&y).expect("can not retrieve line");
            for x in 0..line.len() {
                line.insert(x as u32, self.cursor.attr.clone());
            }
        }

        if how == Some(0) || how == Some(1) {
            self.erase_in_line(how, None);
        }
    }

    fn erase_in_line(&mut self, how: Option<u32>, _private: Option<bool>) {
        self.dirty.insert(self.cursor.y);

        let how = how.unwrap_or(0);
        let interval: Box<dyn Iterator<Item = u32>> = match how {
            0 => Box::new(self.cursor.x..self.columns),
            1 => Box::new(0..=self.cursor.x),
            2 => Box::new(0..self.columns),
            _ => {
                panic!("invalid eras_in_line parameter");
            } // Handle invalid `how` values if necessary
        };

        let line = self
            .buffer
            .get_mut(&self.cursor.y)
            .expect("can not retrieve line");
        for x in interval {
            line.insert(x, self.cursor.attr.clone());
        }
    }

    /// Insert the indicated number of lines at the line with the cursor.
    /// Lines displayed at and below the cursor move down. Lines moved
    /// past the bottom margin are lost.
    ///
    /// # Parameters
    ///
    /// - `count`: Number of lines to insert.
    fn insert_lines(&mut self, count: Option<u32>) {
        let count = count.unwrap_or(1);
        let Margins { top, bottom } = self
            .margins
            .unwrap_or(Margins { top: 0, bottom: self.lines - 1 });

        // If cursor is outside scrolling margins, do nothing.
        if top <= self.cursor.y && self.cursor.y <= bottom {
            self.dirty.extend(self.cursor.y..self.lines);
            for y in (self.cursor.y as u32..=bottom as u32).rev() {
                if y + count <= bottom as u32 {
                    if let Some(line) = self.buffer.remove(&y) {
                        self.buffer.insert(y + count, line);
                    }
                } else {
                    self.buffer.remove(&y);
                }
            }

            self.cariage_return();
        }
    }

    fn delete_lines(&mut self, count: Option<u32>) {
        let count = count.unwrap_or(1);
        let Margins { top, bottom } = self
            .margins
            .unwrap_or(Margins { top: 0, bottom: self.lines - 1 });

        // If cursor is outside scrolling margins -- do nothing.
        if top <= self.cursor.y && self.cursor.y <= bottom {
            self.dirty.extend(self.cursor.y..self.lines);
            for y in self.cursor.y..=bottom {
                if y + count <= bottom {
                    if let Some(line) = self.buffer.remove(&(y + count)) {
                        self.buffer.insert(y, line);
                    }
                } else {
                    self.buffer.remove(&y);
                }
            }

            self.cariage_return();
        }
    }

    /// Delete the indicated number of characters, starting with the
    /// character at the cursor position. When a character is deleted,
    /// all characters to the right of the cursor move left. Character
    /// attributes move with the characters.
    ///
    /// # Parameters
    /// - `count`: Number of characters to delete.
    fn delete_characters(&mut self, count: Option<u32>) {
        self.dirty.insert(self.cursor.y);
        let count = count.unwrap_or(1);

        if let Some(line) = self.buffer.get_mut(&self.cursor.y) {
            for x in self.cursor.x..self.columns {
                if x + count <= self.columns {
                    if let Some(char_opts) = line.remove(&(x + count)) {
                        line.insert(x, char_opts);
                    } else {
                        line.insert(x, CharOpts::default());
                    }
                } else {
                    line.remove(&x);
                }
            }
        }
    }

    /// Erase the indicated number of characters, starting with the
    /// character at the cursor position. Character attributes are set
    /// to cursor attributes. The cursor remains in the same position.
    ///
    /// # Parameters
    /// - `count`: Number of characters to erase.
    ///
    /// # Note
    /// Using cursor attributes for character attributes may seem
    /// illogical, but if you recall that a terminal emulator emulates
    /// a typewriter, it starts to make sense. The only way a typewriter
    /// could erase a character is by typing over it.
    fn erase_characters(&mut self, count: Option<u32>) {
        self.dirty.insert(self.cursor.y);
        let count = count.unwrap_or(1);

        if let Some(line) = self.buffer.get_mut(&self.cursor.y) {
            for x in self.cursor.x..std::cmp::min(self.cursor.x + count, self.columns) {
                line.insert(x, self.cursor.attr.clone());
            }
        }
    }
    /// Report terminal identity.
    ///
    /// # Parameters
    /// - `mode`: Mode for reporting terminal identity.
    /// - `private`: When `true`, the method does nothing. This behavior is consistent with the VT220 manual.
    ///
    /// # Version
    /// - Added in version 0.5.0
    /// - Changed in version 0.7.0: If `private` keyword argument is set, the method does nothing.
    fn report_device_attributes(&mut self, mode: Option<u32>, private: Option<bool>) {
        // We only implement "primary" DA which is the only DA request
        // VT102 understood, see `VT102ID` in `linux/drivers/tty/vt.c`.
        if mode.unwrap_or(0) == 0 && !private.unwrap_or(false) {
            self.write_process_input("\x1B[?6c");
        }
    }

    /// Move cursor to a specific line in the current column.
    ///
    /// # Parameters
    /// - `line`: Line number to move the cursor to.
    fn cursor_to_line(&mut self, line: Option<u32>) {
        self.cursor.y = line.unwrap_or(1) - 1;

        // If origin mode (DECOM) is set, line numbers are relative to
        // the top scrolling margin.
        if self.mode.contains(&DECOM) {
            if let Some(margins) = self.margins {
                self.cursor.y += margins.top;
            }

            // FIXME: should we also restrict the cursor to the scrolling
            // region?
        }

        self.ensure_vbounds(None);
    }

    /// Clear a horizontal tab stop.
    ///
    /// # Parameters
    /// - `how`: Defines the way the tab stop should be cleared:
    ///     - `0` or nothing: Clears a horizontal tab stop at the cursor position.
    ///     - `3`: Clears all horizontal tab stops.
    fn clear_tab_stop(&mut self, how: Option<u32>) {
        match how.unwrap_or(0) {
            0 => {
                // Clears a horizontal tab stop at cursor position, if it's
                // present, or silently fails if otherwise.
                self.tabstops.remove(&self.cursor.x);
            }
            3 => {
                // Clears all horizontal tab stops.
                self.tabstops.clear();
            }
            _ => {
                // Handle invalid `how` values if necessary.
            }
        }
    }

    /// Set (enable) a given list of modes.
    ///
    /// # Arguments
    ///
    /// - `modes`: A list of modes to set, where each mode is a constant from the `pyte::modes` module.
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// let modes = vec![Mode::Insert, Mode::Replace];
    /// set_modes(modes);
    /// ```
    ///
    /// # Note
    ///
    /// Each mode should be a constant from the `modes` module.
    fn set_mode(&mut self, modes: &[u32], private: bool) {
        // mode_list = list(modes)
        // Private mode codes are shifted, to be distinguished from non
        // private ones.
        let mut mode_list = Vec::from(modes);
        if private {
            mode_list = modes.iter().map(|m| m << 5).collect::<Vec<_>>();
            if mode_list.iter().any(|m| *m == DECSCNM) {
                self.dirty.extend(0..self.lines);
            }
        }

        self.mode.extend(mode_list.iter());

        // When DECOLM mode is set, the screen is erased and the cursor
        // moves to the home position.
        if mode_list.iter().any(|m| *m == DECCOLM) {
            self.saved_columns = Some(self.columns);
            self.resize(None, Some(132));
            self.erase_in_display(Some(2), None);
            self.cursor_position(None, None);
        }

        // According to VT520 manual, DECOM should also home the cursor.
        if mode_list.iter().any(|m| *m == DECOM) {
            self.cursor_position(None, None);
        }

        // Mark all displayed characters as reverse.
        if mode_list.iter().any(|m| *m == DECSCNM) {
            for line in self.buffer.values_mut() {
                // line.default = self.default_char;
                for x in line.iter_mut() {
                    x.1.reverse = true;
                }
            }

            self.select_graphic_rendition(&[7]); // +reverse.
        }

        // # Make the cursor visible.
        if mode_list.iter().any(|m| *m == DECTCEM) {
            self.cursor.hidden = false;
        }
    }

    /// Reset (disable) a given list of modes.
    ///
    /// # Arguments
    ///
    /// - `modes`: A list of modes to reset. Each mode should ideally be a constant from the `pyte::modes` module.
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// let modes = vec![Mode::Insert, Mode::Replace];
    /// reset_modes(modes);
    /// ```
    ///
    /// # Note
    ///
    /// Make sure that each mode is a constant from the `modes` module.
    fn reset_mode(&mut self, modes: &[u32], is_private: bool) {
        let mut mode_list = Vec::from(modes);
        // Private mode codes are shifted, to be distinguished from non
        // private ones.
        if is_private {
            mode_list = modes.iter().map(|m| m << 5).collect::<Vec<_>>();
            if mode_list.iter().any(|m| *m == DECSCNM) {
                self.dirty.extend(0..self.lines);
            }
        }

        // retain mode mode_list difference
        self.mode = self
            .mode
            .iter()
            .filter(|&&x| !mode_list.iter().any(|&y| x == y))
            .cloned()
            .collect();

        // Lines below follow the logic in :meth:`set_mode`.
        if mode_list.iter().any(|m| *m == DECCOLM) {
            self.saved_columns = Some(self.columns);
            self.resize(None, Some(132));
            self.erase_in_display(Some(2), None);
            self.cursor_position(None, None);
        }

        // According to VT520 manual, DECOM should also home the cursor.
        if mode_list.iter().any(|m| *m == DECOM) {
            self.cursor_position(None, None);
        }

        // Mark all displayed characters as reverse.
        if mode_list.iter().any(|m| *m == DECSCNM) {
            for line in self.buffer.values_mut() {
                // line.default = self.default_char;
                for x in line.iter_mut() {
                    x.1.reverse = true;
                }
            }

            self.select_graphic_rendition(&[27]); // +reverse.
        }

        // Hide the cursor.
        if mode_list.iter().any(|m| *m == DECTCEM) {
            self.cursor.hidden = true;
        }
    }

    /// Set display attributes.
    ///
    /// # Parameters
    /// - `attrs`: A list of display attributes to set.
    fn select_graphic_rendition(&mut self, attrs: &[u32]) {
        let mut replace = HashMap::new();

        // Fast path for resetting all attributes.
        if attrs.is_empty() || (attrs.len() == 1 && attrs[0] == 0) {
            self.cursor.attr = CharOpts::default();
            return;
        }

        let mut attrs_list = attrs.to_vec();
        attrs_list.reverse();

        while let Some(attr) = attrs_list.pop() {
            match attr {
                0 => {
                    // Reset all attributes.
                    replace.extend(CharOpts::default().to_map());
                }
                attr if FG_ANSI.contains_key(&attr) => {
                    replace.insert("fg".to_string(), FG_ANSI[&attr].clone());
                }
                attr if BG_ANSI.contains_key(&attr) => {
                    replace.insert("bg".to_string(), BG_ANSI[&attr].clone());
                }
                attr if TEXT.contains_key(&attr) => {
                    let attr_str = &TEXT[&attr];
                    replace.insert(
                        attr_str[1..].to_string(),
                        attr_str.starts_with('+').to_string(),
                    );
                }
                attr if FG_AIXTERM.contains_key(&attr) => {
                    replace.insert("fg".to_string(), FG_AIXTERM[&attr].clone());
                }
                attr if BG_AIXTERM.contains_key(&attr) => {
                    replace.insert("bg".to_string(), BG_AIXTERM[&attr].clone());
                }
                attr if attr == FG_256 || attr == BG_256 => {
                    let key = if attr == FG_256 { "fg" } else { "bg" };
                    if let Some(n) = attrs_list.pop() {
                        if n == 5 {
                            if let Some(m) = attrs_list.pop() {
                                replace.insert(key.to_string(), FG_BG_256[m as usize].clone());
                            }
                        } else if n == 2 {
                            if let (Some(r), Some(g), Some(b)) =
                                (attrs_list.pop(), attrs_list.pop(), attrs_list.pop())
                            {
                                replace.insert(
                                    key.to_string(),
                                    format!("{:02x}{:02x}{:02x}", r, g, b),
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        self.cursor.attr.update_from_map(replace);
    }

    /// Set terminal title.
    ///
    /// **Warning:** This is an XTerm extension supported by the Linux terminal.
    fn set_title(&mut self, title: &str) {
        self.title = title.to_owned();
    }

    /// Set icon name
    ///
    /// **Warning:** This is an XTerm extension supported by the Linux terminal.
    fn set_icon_name(&mut self, icon_name: &str) {
        self.icon_name = icon_name.to_owned();
    }
}
