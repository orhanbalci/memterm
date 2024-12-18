use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use lazy_static::lazy_static;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::{char, UnicodeNormalization};
use unicode_width::UnicodeWidthChar;

use crate::charset::{LAT1_MAP, MAPS, VT100_MAP};
use crate::graphics::{BG_256, BG_AIXTERM, BG_ANSI, FG_256, FG_AIXTERM, FG_ANSI, FG_BG_256, TEXT};
use crate::modes::{DECAWM, DECCOLM, DECOM, DECSCNM, DECTCEM, IRM, LNM};
use crate::parser_listener::ParserListener;

#[derive(Clone, PartialEq, Debug)]
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
    pub fn new(columns: u32, lines: u32) -> Self {
        let mut screen = Screen {
            savepoints: Vec::new(),
            columns,
            lines,
            buffer: HashMap::new(),
            dirty: HashSet::new(),
            mode: _DEFAULT_MODE.clone(),
            margins: None,
            title: String::new(),
            icon_name: String::new(),
            charset: Charset::G0,
            g0_charset: LAT1_MAP.clone(),
            g1_charset: VT100_MAP.clone(),
            tabstops: HashSet::new(),
            cursor: Cursor {
                x: 0,
                y: 0,
                attr: CharOpts::default(),
                hidden: false,
            },
            saved_columns: None,
        };

        screen.reset();
        screen
    }

    ///A list of screen lines as unicode strings.
    pub fn display(&mut self) -> Vec<String> {
        let default_char = self.default_char();
        let render = |line: &mut HashMap<u32, CharOpts>| -> String {
            let mut result = String::new();
            let mut is_wide_char = false;
            for x in 0..self.columns {
                if is_wide_char {
                    is_wide_char = false;
                    continue;
                }
                let char = line.entry(x).or_insert(default_char.clone()).data.clone();
                is_wide_char = char
                    .chars()
                    .next()
                    .expect("can not read char")
                    .width()
                    .is_some_and(|s| s == 2);
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
        if top.or(Some(0)).expect("unexpected top value") == 0 && bottom.is_none() {
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
            margins_inner.top as i32
        } else {
            i32::max(
                0,
                i32::min(
                    top.expect("unexpected top value") as i32 - 1,
                    self.lines as i32 - 1,
                ),
            )
        };

        let bottom = if bottom.is_none() {
            margins_inner.bottom as i32
        } else {
            i32::max(
                0,
                i32::min(
                    bottom.expect("unexpected bottom value") as i32 - 1,
                    self.lines as i32 - 1,
                ),
            )
        };

        // Even though VT102 and VT220 require DECSTBM to ignore
        // regions of width less than 2, some programs (like aptitude
        // for example) rely on it. Practicality beats purity.
        if bottom - top >= 1 {
            self.margins = Some(Margins { top: top as u32, bottom: bottom as u32 });
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

    /// Returns an empty character with default foreground and background colors.
    pub fn default_char(&self) -> CharOpts {
        CharOpts {
            data: " ".to_owned(),
            fg: "default".to_owned(),
            bg: "default".to_owned(),
            reverse: self.mode.contains(&DECSCNM),
            ..CharOpts::default()
        }
    }
}

impl ParserListener for Screen {
    /// Fills screen with uppercase E's for screen focus and alignment.
    fn alignment_display(&mut self) {
        self.dirty.extend(0..self.lines);
        for y in 0..self.lines {
            let line = self.buffer.entry(y).or_insert_with(HashMap::new);
            for x in 0..self.columns {
                // TODO check this default, should be default_char on screen
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
        self.tabstops.clear();
        self.tabstops.extend((8..self.columns).step_by(8));

        self.cursor = Cursor {
            x: 0,
            y: 0,
            hidden: false,
            attr: self.default_char(),
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
            // Mark all lines as dirty
            self.dirty.extend(0..self.lines);
            let mut new_buffer: HashMap<u32, HashMap<u32, CharOpts>> = HashMap::new();
            // Copy lines before top margin unchanged
            for y in 0..top {
                if let Some(line) = self.buffer.get(&y) {
                    new_buffer.insert(y, line.clone());
                }
            }
            // Move lines up (decrement keys)
            for y in top..bottom {
                if let Some(line) = self.buffer.get(&(y + 1)) {
                    new_buffer.insert(y, line.clone());
                }
            }

            // Insert empty line at bottom
            new_buffer.insert(bottom, HashMap::new());

            // Copy lines after bottom margin unchanged
            for y in (bottom + 1)..self.lines {
                if let Some(line) = self.buffer.get(&y) {
                    new_buffer.insert(y, line.clone());
                }
            }

            // Replace old buffer with new one
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
            // Mark all lines as dirty
            self.dirty.extend(0..self.lines);
            let mut new_buffer: HashMap<u32, HashMap<u32, CharOpts>> = HashMap::new();

            // Copy lines before top margin unchanged
            for y in 0..top {
                if let Some(line) = self.buffer.get(&y) {
                    new_buffer.insert(y, line.clone());
                }
            }

            // Move lines within margins down
            for y in (top..=bottom).rev() {
                if let Some(line) = self.buffer.get(&y) {
                    new_buffer.insert(y + 1, line.clone());
                }
            }

            // Insert empty line at top margin
            new_buffer.insert(top, HashMap::new());

            // Copy lines after bottom margin unchanged
            for y in (bottom + 1)..self.lines {
                if let Some(line) = self.buffer.get(&y) {
                    new_buffer.insert(y, line.clone());
                }
            }

            self.buffer = new_buffer;
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
                    if c as usize > 255 {
                        c
                    } else {
                        self.g1_charset[c as usize]
                    }
                } else {
                    if c as usize > 255 {
                        c
                    } else {
                        self.g0_charset[c as usize]
                    }
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
        let default = self.default_char();

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
                        line.insert(x + count, default.clone());
                    }
                }
            }
            line.insert(x, default.clone());
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
        if self.cursor.x >= count.unwrap_or(1) {
            self.cursor.x -= count.unwrap_or(1);
        } else {
            self.cursor.x = 0;
        }
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
        let column: i32 = column.map(|a| if a == 0 { 1 } else { a }).unwrap_or(1) as i32 - 1;
        let mut line: i32 = line.map(|a| if a == 0 { 1 } else { a }).unwrap_or(1) as i32 - 1;

        // If origin mode (DECOM) is set, line number is relative to the top scrolling margin.
        if let Some(margins) = &self.margins {
            if self.mode.contains(&DECOM) {
                line += margins.top as i32;

                // Cursor is not allowed to move out of the scrolling region.
                if line < margins.top as i32 || line > margins.bottom as i32 {
                    return;
                }
            }
        }

        self.cursor.x = column as u32;
        self.cursor.y = line as u32;
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
        let count = count.map(|a| if a > 0 { a } else { 1 }).unwrap_or(1);

        let default_char = self.default_char();
        if let Some(line) = self.buffer.get_mut(&self.cursor.y) {
            for x in self.cursor.x..self.columns {
                if x + count <= self.columns {
                    if let Some(char_opts) = line.remove(&(x + count)) {
                        line.insert(x, char_opts);
                    } else {
                        line.insert(x, default_char.clone());
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
        let count = count.map(|a| if a > 0 { a } else { 1 }).unwrap_or(1);

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
        dbg!(mode_list.clone());
        if mode_list.iter().any(|m| *m == DECCOLM) {
            dbg!("DECCOLM");
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

        // Lines below follow the logic in set_mode.
        if mode_list.iter().any(|m| *m == DECCOLM) {
            if self.columns == 132 {
                if let Some(saved_columns) = self.saved_columns {
                    self.resize(None, Some(saved_columns));
                    self.saved_columns = None;
                }
            }
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
                    x.1.reverse = false;
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
            self.cursor.attr = self.default_char();
            return;
        }

        let mut attrs_list = attrs.to_vec();
        attrs_list.reverse();

        while let Some(attr) = attrs_list.pop() {
            match attr {
                0 => {
                    // Reset all attributes.
                    replace.extend(self.default_char().to_map());
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
                                if m < 16 {
                                    replace.insert(key.to_string(), FG_BG_256[m as usize].clone());
                                }
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
                        } else {
                            // consider panicing in a strict mode
                            // panic!("invalid mode for FG BG colors");
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

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    use super::{CharOpts, Screen};
    use crate::graphics::{BG_256, FG_256};
    use crate::modes::{DECAWM, DECCOLM, DECOM, DECSCNM, DECTCEM, IRM, LNM};
    use crate::parser::Parser;
    use crate::parser_listener::ParserListener;
    use crate::screen::Charset;

    /// Macro to create CharOpts with optional color
    macro_rules! co {
        (default) => {
            CharOpts::default()
        };
        ($c:literal) => {
            CharOpts { data: $c.to_string(), ..CharOpts::default() }
        };
        ($c:literal, fg = $color:literal) => {
            CharOpts {
                data: $c.to_string(),
                fg: $color.to_string(),
                ..CharOpts::default()
            }
        };
    }

    macro_rules! cv {
        ($(co !($($arg:tt)*)),+ $(,)?) => {
            vec![ $(co!($($arg)*)),+ ]
        };
    }

    pub fn update(screen: &mut Screen, lines: Vec<&str>, colored: Vec<u32>) {
        for (y, line) in lines.iter().enumerate() {
            for (x, char) in line.chars().enumerate() {
                let mut attrs = screen.default_char();
                if colored.contains(&(y as u32)) {
                    attrs.fg = "red".to_string();
                }
                attrs.data = char.to_string();
                screen
                    .buffer
                    .entry(y as u32)
                    .or_insert_with(HashMap::new)
                    .insert(x as u32, attrs);
            }
        }
    }

    pub fn tolist(screen: &Screen) -> Vec<Vec<CharOpts>> {
        let mut result = Vec::new();

        for y in 0..screen.lines {
            let mut line = Vec::new();
            for x in 0..screen.columns {
                let char_opts = screen
                    .buffer
                    .get(&y)
                    .and_then(|line| line.get(&x))
                    .cloned()
                    .unwrap_or_default();
                line.push(char_opts);
            }
            result.push(line);
        }

        result
    }
    #[test]
    fn initialize_char() {
        // List of fields in CharOpts struct
        let fields = vec![
            "data",
            "fg",
            "bg",
            "bold",
            "italics",
            "underscore",
            "strikethrough",
            "reverse",
            "blink",
        ];

        for field in fields.iter().skip(1) {
            let mut char_opts = CharOpts::default();
            match *field {
                "bold" => char_opts.bold = true,
                "italics" => char_opts.italics = true,
                "underscore" => char_opts.underscore = true,
                "strikethrough" => char_opts.strikethrough = true,
                "reverse" => char_opts.reverse = true,
                "blink" => char_opts.blink = true,
                _ => {}
            }

            match *field {
                "bold" => assert!(char_opts.bold),
                "italics" => assert!(char_opts.italics),
                "underscore" => assert!(char_opts.underscore),
                "strikethrough" => assert!(char_opts.strikethrough),
                "reverse" => assert!(char_opts.reverse),
                "blink" => assert!(char_opts.blink),
                _ => {}
            }
        }
    }

    #[test]
    fn remove_non_existant_attribute() {
        let mut screen = Screen::new(2, 2);

        let expected = vec![
            cv![co!(default), co!(default)],
            cv![co!(default), co!(default)],
        ];

        assert_eq!(tolist(&screen), expected);

        screen.select_graphic_rendition(&[24]); // underline-off.
        assert_eq!(tolist(&screen), expected);
        assert!(!screen.cursor.attr.underscore);
    }

    #[test]
    fn attributes() {
        let mut screen = Screen::new(2, 2);

        let default_char = CharOpts::default();
        let expected_initial = vec![
            cv![co!(default), co!(default)],
            cv![co!(default), co!(default)],
        ];

        assert_eq!(tolist(&screen), expected_initial);

        screen.select_graphic_rendition(&[1]); // bold.

        // Still default, since we haven't written anything.
        assert_eq!(tolist(&screen), expected_initial);
        assert!(screen.cursor.attr.bold);

        screen.draw("f");
        let expected_after_draw = vec![
            vec![
                CharOpts {
                    data: "f".to_string(),
                    fg: "default".to_string(),
                    bg: "default".to_string(),
                    bold: true,
                    ..default_char.clone()
                },
                default_char.clone(),
            ],
            vec![default_char.clone(), default_char.clone()],
        ];

        assert_eq!(tolist(&screen), expected_after_draw);
    }

    #[test]
    fn blink() {
        let mut screen = Screen::new(2, 2);

        let default_char = CharOpts::default();
        let expected_initial = vec![
            cv![co!(default), co!(default)],
            cv![co!(default), co!(default)],
        ];

        assert_eq!(tolist(&screen), expected_initial);

        screen.select_graphic_rendition(&[5]); // blink.

        screen.draw("f");
        let expected_after_draw = vec![
            vec![
                CharOpts {
                    data: "f".to_string(),
                    fg: "default".to_string(),
                    bg: "default".to_string(),
                    blink: true,
                    ..default_char.clone()
                },
                default_char.clone(),
            ],
            vec![default_char.clone(), default_char.clone()],
        ];

        assert_eq!(tolist(&screen), expected_after_draw);
    }

    #[test]
    fn colors() {
        let mut screen = Screen::new(2, 2);

        screen.select_graphic_rendition(&[30]); // Set foreground color to black.
        screen.select_graphic_rendition(&[40]); // Set background color to black.
        assert_eq!(screen.cursor.attr.fg, "black");
        assert_eq!(screen.cursor.attr.bg, "black");

        screen.select_graphic_rendition(&[31]); // Set foreground color to red.
        assert_eq!(screen.cursor.attr.fg, "red");
        assert_eq!(screen.cursor.attr.bg, "black");
    }

    #[test]
    fn colors256() {
        let mut screen = Screen::new(2, 2);

        // a) OK-case.
        screen.select_graphic_rendition(&[FG_256, 5, 0]);
        screen.select_graphic_rendition(&[BG_256, 5, 15]);
        assert_eq!(screen.cursor.attr.fg, "000000");
        assert_eq!(screen.cursor.attr.bg, "ffffff");
    }

    #[test]
    fn invalid_color() {
        //consider panicing in this cases
        let mut screen = Screen::new(2, 2);
        screen.select_graphic_rendition(&[48, 5, 100500]);
    }

    #[test]
    fn colors256_missing_attrs() {
        let mut screen = Screen::new(2, 2);

        // Test from https://github.com/selectel/pyte/issues/115
        screen.select_graphic_rendition(&[FG_256]);
        screen.select_graphic_rendition(&[BG_256]);

        assert_eq!(screen.cursor.attr, CharOpts::default());
    }

    #[test]
    fn colors24bit() {
        let mut screen = Screen::new(2, 2);

        // a) OK-case
        screen.select_graphic_rendition(&[38, 2, 0, 0, 0]);
        screen.select_graphic_rendition(&[48, 2, 255, 255, 255]);
        assert_eq!(screen.cursor.attr.fg, "000000");
        assert_eq!(screen.cursor.attr.bg, "ffffff");
    }

    #[test]
    fn colors24bit_invalid_color() {
        // consider panicing in this cases
        let mut screen = Screen::new(2, 2);
        screen.select_graphic_rendition(&[48, 2, 255]);
    }

    #[test]
    fn colors_aixterm() {
        let mut screen = Screen::new(2, 2);

        // a) foreground color.
        screen.select_graphic_rendition(&[94]);
        assert_eq!(screen.cursor.attr.fg, "brightblue");

        // b) background color.
        screen.select_graphic_rendition(&[104]);
        assert_eq!(screen.cursor.attr.bg, "brightblue");
    }

    #[test]
    fn colors_ignore_invalid() {
        let mut screen = Screen::new(2, 2);
        let default_attrs = screen.cursor.attr.clone();

        screen.select_graphic_rendition(&[100500]);
        assert_eq!(screen.cursor.attr, default_attrs);

        screen.select_graphic_rendition(&[38, 100500]);
        assert_eq!(screen.cursor.attr, default_attrs);

        screen.select_graphic_rendition(&[48, 100500]);
        assert_eq!(screen.cursor.attr, default_attrs);
    }

    #[test]
    fn reset_resets_colors() {
        let mut screen = Screen::new(2, 2);
        let expected_initial = vec![
            cv![co!(default), co!(default)],
            cv![co!(default), co!(default)],
        ];

        assert_eq!(tolist(&screen), expected_initial);

        screen.select_graphic_rendition(&[30]); // Set foreground color to black.
        screen.select_graphic_rendition(&[40]); // Set background color to black.
        assert_eq!(screen.cursor.attr.fg, "black");
        assert_eq!(screen.cursor.attr.bg, "black");

        screen.select_graphic_rendition(&[0]); // Reset all attributes.
        assert_eq!(screen.cursor.attr, CharOpts::default());
    }

    #[test]
    fn reset_works_between_attributes() {
        let mut screen = Screen::new(2, 2);

        let expected_initial = vec![
            cv![co!(default), co!(default)],
            cv![co!(default), co!(default)],
        ];

        assert_eq!(tolist(&screen), expected_initial);

        // Red fg, reset, red bg
        screen.select_graphic_rendition(&[31, 0, 41]);
        assert_eq!(screen.cursor.attr.fg, "default");
        assert_eq!(screen.cursor.attr.bg, "red");
    }

    #[test]
    fn multi_attribs() {
        let mut screen = Screen::new(2, 2);

        let expected_initial = vec![
            cv![co!(default), co!(default)],
            cv![co!(default), co!(default)],
        ];

        assert_eq!(tolist(&screen), expected_initial);

        screen.select_graphic_rendition(&[1]); // Set bold
        screen.select_graphic_rendition(&[3]); // Set italics

        assert!(screen.cursor.attr.bold);
        assert!(screen.cursor.attr.italics);
    }

    #[test]
    fn attributes_reset() {
        let mut screen = Screen::new(2, 2);
        screen.set_mode(&[LNM], false);

        let default_char = CharOpts::default();
        let expected_initial = vec![
            cv![co!(default), co!(default)],
            cv![co!(default), co!(default)],
        ];

        assert_eq!(tolist(&screen), expected_initial);

        // Set bold attribute and draw "foo"
        screen.select_graphic_rendition(&[1]);
        screen.draw("f");
        screen.draw("o");
        screen.draw("o");

        let bold_char = |c: &str| CharOpts {
            data: c.to_string(),
            bold: true,
            ..CharOpts::default()
        };

        let expected_after_foo = vec![
            vec![bold_char("f"), bold_char("o")],
            vec![bold_char("o"), default_char.clone()],
        ];

        assert_eq!(tolist(&screen), expected_after_foo);

        // Reset cursor position and attributes, then draw "f"
        screen.cursor_position(None, None);
        screen.select_graphic_rendition(&[0]);
        screen.draw("f");

        let normal_char = |c: &str| CharOpts { data: c.to_string(), ..CharOpts::default() };

        let expected_final = vec![
            vec![normal_char("f"), bold_char("o")],
            vec![bold_char("o"), default_char],
        ];

        assert_eq!(tolist(&screen), expected_final);
    }

    #[test]
    fn resize() {
        // Test initial resize behavior
        let mut screen = Screen::new(2, 2);
        screen.set_mode(&[DECOM], false);
        screen.set_margins(Some(0), Some(1));

        assert_eq!(screen.columns, 2);
        assert_eq!(screen.lines, 2);

        let expected_initial = vec![
            cv![co!(default), co!(default)],
            cv![co!(default), co!(default)],
        ];
        assert_eq!(tolist(&screen), expected_initial);

        // Test resize to larger dimensions
        screen.resize(Some(3), Some(3));
        assert_eq!(screen.columns, 3);
        assert_eq!(screen.lines, 3);

        let expected_larger = vec![
            cv![co!(default), co!(default), co!(default)],
            cv![co!(default), co!(default), co!(default)],
            cv![co!(default), co!(default), co!(default)],
        ];
        assert_eq!(tolist(&screen), expected_larger);
        assert!(screen.mode.contains(&DECOM));
        assert!(screen.margins.is_none());

        // Test resize back to original size
        screen.resize(Some(2), Some(2));
        assert_eq!(screen.columns, 2);
        assert_eq!(screen.lines, 2);
        assert_eq!(tolist(&screen), expected_initial);

        // Test quirks:
        // a) Adding columns to the right
        let mut screen = Screen::new(2, 2);
        update(&mut screen, vec!["bo", "sh"], vec![]);
        screen.resize(Some(2), Some(3));
        assert_eq!(screen.display(), vec!["bo ".to_string(), "sh ".to_string()]);

        // b) Removing columns from the right
        let mut screen = Screen::new(2, 2);
        update(&mut screen, vec!["bo", "sh"], vec![]);
        screen.resize(Some(2), Some(1));
        assert_eq!(screen.display(), vec!["b".to_string(), "s".to_string()]);

        // c) Adding rows at the bottom
        let mut screen = Screen::new(2, 2);
        update(&mut screen, vec!["bo", "sh"], vec![]);
        screen.resize(Some(3), Some(2));
        assert_eq!(
            screen.display(),
            vec!["bo".to_string(), "sh".to_string(), "  ".to_string()]
        );

        // d) Removing rows from the top
        let mut screen = Screen::new(2, 2);
        update(&mut screen, vec!["bo", "sh"], vec![]);
        screen.resize(Some(1), Some(2));
        assert_eq!(screen.display(), vec!["sh".to_string()]);
    }

    #[test]
    fn resize_same() {
        let mut screen = Screen::new(2, 2);
        screen.dirty.clear();
        screen.resize(Some(2), Some(2));
        assert!(screen.dirty.is_empty());
    }

    #[test]
    fn set_mode() {
        // Test DECCOLM mode
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![]);
        screen.cursor_position(Some(1), Some(1));
        screen.set_mode(&[DECCOLM], false);

        let default_char = screen.default_char();
        for line in tolist(&screen) {
            for char in line {
                assert_eq!(char, default_char);
            }
        }
        assert_eq!(screen.columns, 132);
        assert_eq!(screen.cursor.x, 0);
        assert_eq!(screen.cursor.y, 0);
        screen.reset_mode(&[DECCOLM], false);
        assert_eq!(screen.columns, 3);

        // Test DECOM mode
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![]);
        screen.cursor_position(Some(1), Some(1));
        screen.set_mode(&[DECOM], false);
        assert_eq!(screen.cursor.x, 0);
        assert_eq!(screen.cursor.y, 0);

        // Test DECSCNM mode
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![]);
        screen.set_mode(&[DECSCNM], false);
        for line in tolist(&screen) {
            for char in line {
                assert!(char.reverse);
            }
        }
        let default_char = screen.default_char();
        assert!(default_char.reverse);
        screen.reset_mode(&[DECSCNM], false);
        for line in tolist(&screen) {
            for char in line {
                assert!(!char.reverse);
            }
        }
        let default_char = screen.default_char();
        assert!(!default_char.reverse);

        // Test DECTCEM mode
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![]);
        screen.cursor.hidden = true;
        screen.set_mode(&[DECTCEM], false);
        assert!(!screen.cursor.hidden);
        screen.reset_mode(&[DECTCEM], false);
        assert!(screen.cursor.hidden);
    }

    #[test]
    fn draw() {
        // DECAWM on (default)
        let mut screen = Screen::new(3, 3);
        screen.set_mode(&[LNM], false);
        assert!(screen.mode.contains(&DECAWM));

        for ch in "abc".chars() {
            screen.draw(&ch.to_string());
        }

        assert_eq!(
            screen.display(),
            vec!["abc".to_string(), "   ".to_string(), "   ".to_string()]
        );
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 3));

        // One more character -- now we got a linefeed!
        screen.draw("a");
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 1));

        // DECAWM is off
        let mut screen = Screen::new(3, 3);
        screen.reset_mode(&[DECAWM], false);

        for ch in "abc".chars() {
            screen.draw(&ch.to_string());
        }

        assert_eq!(
            screen.display(),
            vec!["abc".to_string(), "   ".to_string(), "   ".to_string()]
        );
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 3));

        // No linefeed is issued on the end of the line ...
        screen.draw("a");
        assert_eq!(
            screen.display(),
            vec!["aba".to_string(), "   ".to_string(), "   ".to_string()]
        );
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 3));

        // IRM mode is on, expecting new characters to move the old ones
        // instead of replacing them
        screen.set_mode(&[IRM], false);
        screen.cursor_position(None, None);
        screen.draw("x");
        assert_eq!(
            screen.display(),
            vec!["xab".to_string(), "   ".to_string(), "   ".to_string()]
        );

        screen.cursor_position(None, None);
        screen.draw("y");
        assert_eq!(
            screen.display(),
            vec!["yxa".to_string(), "   ".to_string(), "   ".to_string()]
        );
    }

    #[test]
    fn draw_russian() {
        // Test from https://github.com/selectel/pyte/issues/65
        let screen = Arc::new(Mutex::new(Screen::new(20, 1)));
        let mut parser = Parser::new(screen.clone());

        // Feed the Russian text to the parser
        parser.feed("Нерусский текст".to_string());

        assert_eq!(
            screen.lock().unwrap().display(),
            vec!["Нерусский текст     ".to_string()]
        );
    }

    #[test]
    fn draw_multiple_chars() {
        let mut screen = Screen::new(10, 1);
        screen.draw("foobar");
        assert_eq!(screen.cursor.x, 6);
        assert_eq!(screen.display(), vec!["foobar    ".to_string()]);
    }
    #[test]
    fn draw_utf8() {
        let screen = Arc::new(Mutex::new(Screen::new(1, 1)));
        let mut parser = Parser::new(screen.clone());

        // Feed UTF-8 bytes for right double quotation mark (")
        parser.feed("\u{201D}".to_string()); // Unicode escape for "

        assert_eq!(screen.lock().unwrap().display(), vec!["”".to_string()]);
    }

    #[test]
    fn draw_width2() {
        let mut screen = Screen::new(10, 1);
        screen.draw("コンニチハ"); // Each character takes 2 columns
        assert_eq!(screen.cursor.x, screen.columns);
        assert_eq!(screen.display(), vec!["コンニチハ".to_string()]);
    }

    #[test]
    fn draw_width2_line_end() {
        let mut screen = Screen::new(10, 1);
        screen.draw(" コンニチハ"); // Space followed by 5 double-width characters
        assert_eq!(screen.cursor.x, screen.columns);
        assert_eq!(screen.display(), vec![" コンニチハ".to_string()]);
    }

    #[test]
    fn draw_width0_combining() {
        let mut screen = Screen::new(4, 2);

        // a) Test with no previous character
        screen.draw("\u{0308}"); // COMBINING DIAERESIS
        assert_eq!(
            screen.display(),
            vec!["    ".to_string(), "    ".to_string()]
        );

        // Draw "bad"
        screen.draw("bad");

        // b) Test with previous character on the same line
        screen.draw("\u{0308}"); // COMBINING DIAERESIS
        assert_eq!(
            screen.display(),
            vec!["bad̈ ".to_string(), "    ".to_string()]
        );

        // c) Test with previous character on the previous line
        screen.draw("!");
        screen.draw("\u{0308}"); // COMBINING DIAERESIS
        assert_eq!(
            screen.display(),
            vec!["bad̈!̈".to_string(), "    ".to_string()]
        );
    }

    #[test]
    fn draw_width0_irm() {
        let mut screen = Screen::new(10, 1);
        screen.set_mode(&[IRM], false); // Enable Insert Mode

        // Draw zero width space
        screen.draw("\u{200B}"); // ZERO WIDTH SPACE

        // Draw DELETE character
        screen.draw("\u{0007}"); // DELETE/BELL character

        // Check that screen is still empty (filled with spaces)
        assert_eq!(screen.display(), vec!["          ".to_string()]); // 10 spaces
    }

    #[test]
    fn draw_width0_irm_detailed() {
        let mut screen = Screen::new(10, 1);

        // Initial state check
        assert_eq!(screen.display(), vec!["          ".to_string()]); // 10 spaces
        assert_eq!(screen.cursor.x, 0);

        // Enable Insert Mode
        screen.set_mode(&[IRM], false);
        assert!(screen.mode.contains(&IRM));

        // Draw zero width space and verify no change
        screen.draw("\u{200B}"); // ZERO WIDTH SPACE
        assert_eq!(screen.display(), vec!["          ".to_string()]); // Still 10 spaces
        assert_eq!(screen.cursor.x, 0); // Cursor shouldn't move

        // Draw DELETE character and verify no change
        screen.draw("\u{0007}"); // DELETE/BELL character
        assert_eq!(screen.display(), vec!["          ".to_string()]); // Still 10 spaces
        assert_eq!(screen.cursor.x, 0); // Cursor shouldn't move

        // Final state verification
        assert_eq!(screen.display(), vec![" ".repeat(screen.columns as usize)]);
        assert!(screen.mode.contains(&IRM)); // IRM should still be enabled
    }

    #[test]
    fn draw_width0_decawm_off() {
        let mut screen = Screen::new(10, 1);

        // Turn off auto-wrap mode
        screen.reset_mode(&[DECAWM], false);
        assert!(!screen.mode.contains(&DECAWM));

        // Draw space followed by Japanese characters
        screen.draw(" コンニチハ");
        assert_eq!(screen.cursor.x, screen.columns);

        // Draw zero-width characters and verify cursor doesn't move
        screen.draw("\u{200B}"); // ZERO WIDTH SPACE
        assert_eq!(screen.cursor.x, screen.columns);

        screen.draw("\u{0007}"); // DELETE/BELL character
        assert_eq!(screen.cursor.x, screen.columns);
    }

    #[test]
    fn draw_width0_decawm_off_detailed() {
        let mut screen = Screen::new(10, 1);

        // Initial state check
        assert_eq!(screen.cursor.x, 0);
        assert!(screen.mode.contains(&DECAWM)); // DECAWM should be on by default

        // Turn off auto-wrap mode
        screen.reset_mode(&[DECAWM], false);
        assert!(!screen.mode.contains(&DECAWM));

        // Draw space followed by Japanese characters
        screen.draw(" コンニチハ");

        // Verify cursor is at end of line
        assert_eq!(screen.cursor.x, screen.columns);
        assert_eq!(screen.display(), vec![" コンニチハ".to_string()]);

        // Try to draw zero-width space
        screen.draw("\u{200B}"); // ZERO WIDTH SPACE
                                 // Verify cursor hasn't moved
        assert_eq!(screen.cursor.x, screen.columns);
        assert_eq!(screen.display(), vec![" コンニチハ".to_string()]);

        // Try to draw DELETE character
        screen.draw("\u{0007}"); // DELETE/BELL character
                                 // Verify cursor still hasn't moved
        assert_eq!(screen.cursor.x, screen.columns);
        assert_eq!(screen.display(), vec![" コンニチハ".to_string()]);

        // Final state verification
        assert_eq!(screen.cursor.x, screen.columns);
        assert!(!screen.mode.contains(&DECAWM));
    }

    #[test]
    fn draw_cp437() {
        let mut screen = Screen::new(5, 1);
        assert_eq!(screen.charset, Charset::G0);

        screen.define_charset("U", "(");
        // In Python this would be: "α ± ε".encode("cp437")
        // We're simulating feeding CP437 encoded bytes
        let cp437_text: [u8; 5] = [
            0xE0, // α
            0x20, // space
            0xF1, // ±
            0x20, // space
            0xEE, // ε
        ];

        for &byte in cp437_text.iter() {
            screen.draw(&(byte as char).to_string());
        }

        assert_eq!(screen.display(), vec!["α ± ε".to_string()]);
    }

    #[test]
    fn display_wcwidth() {
        let mut screen = Screen::new(10, 1);
        screen.draw("コンニチハ");

        assert_eq!(screen.display(), vec!["コンニチハ".to_string()]);
    }

    #[test]
    fn draw_with_carriage_return() {
        let line = "ipcs -s | grep nobody |awk '{print$2}'|xargs -n1 ipcrm sem ;ps aux|grep -P 'httpd|fcgi'|grep -v grep|awk '{print$2 \x0D}'|xargs kill -9;/etc/init.d/httpd startssl";

        let screen = Arc::new(Mutex::new(Screen::new(50, 3)));
        let mut parser = Parser::new(screen.clone());
        parser.feed(line.to_string());

        assert_eq!(
            screen.lock().unwrap().display(),
            vec![
                "ipcs -s | grep nobody |awk '{print$2}'|xargs -n1 i".to_string(),
                "pcrm sem ;ps aux|grep -P 'httpd|fcgi'|grep -v grep".to_string(),
                "}'|xargs kill -9;/etc/init.d/httpd startssl       ".to_string(),
            ]
        );
    }

    #[test]
    fn carriage_return() {
        let mut screen = Screen::new(3, 3);
        screen.cursor.x = 2;
        screen.cariage_return();

        assert_eq!(screen.cursor.x, 0);
    }

    #[test]
    fn index() {
        // a) Test basic index behavior
        let mut screen = Screen::new(2, 2);
        update(&mut screen, vec!["wo", "ot"], vec![1]);

        // Test indexing on non-last row
        screen.index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));

        let expected = vec![
            vec![
                CharOpts { data: "w".to_string(), ..CharOpts::default() },
                CharOpts { data: "o".to_string(), ..CharOpts::default() },
            ],
            vec![
                CharOpts {
                    data: "o".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
                CharOpts {
                    data: "t".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
            ],
        ];
        assert_eq!(tolist(&screen), expected);

        // b) Test indexing on last row
        screen.index();
        assert_eq!(screen.cursor.y, 1);

        let expected = vec![
            vec![
                CharOpts {
                    data: "o".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
                CharOpts {
                    data: "t".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
            ],
            vec![screen.default_char(), screen.default_char()],
        ];
        assert_eq!(tolist(&screen), expected);

        // c) Test with margins
        let mut screen = Screen::new(2, 5);
        update(&mut screen, vec!["bo", "sh", "th", "er", "oh"], vec![1, 2]);
        screen.set_margins(Some(2), Some(4));
        screen.cursor.y = 3;

        // First index
        screen.index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (3, 0));
        assert_eq!(
            screen.display(),
            vec![
                "bo".to_string(),
                "th".to_string(),
                "er".to_string(),
                "  ".to_string(),
                "oh".to_string()
            ]
        );

        let expected = vec![
            vec![
                CharOpts { data: "b".to_string(), ..CharOpts::default() },
                CharOpts { data: "o".to_string(), ..CharOpts::default() },
            ],
            vec![
                CharOpts {
                    data: "t".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
                CharOpts {
                    data: "h".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
            ],
            vec![
                CharOpts { data: "e".to_string(), ..CharOpts::default() },
                CharOpts { data: "r".to_string(), ..CharOpts::default() },
            ],
            vec![screen.default_char(), screen.default_char()],
            vec![
                CharOpts { data: "o".to_string(), ..CharOpts::default() },
                CharOpts { data: "h".to_string(), ..CharOpts::default() },
            ],
        ];
        assert_eq!(tolist(&screen), expected);

        // Second index
        screen.index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (3, 0));
        assert_eq!(
            screen.display(),
            vec![
                "bo".to_string(),
                "er".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "oh".to_string()
            ]
        );

        let expected = vec![
            vec![
                CharOpts { data: "b".to_string(), ..CharOpts::default() },
                CharOpts { data: "o".to_string(), ..CharOpts::default() },
            ],
            vec![
                CharOpts { data: "e".to_string(), ..CharOpts::default() },
                CharOpts { data: "r".to_string(), ..CharOpts::default() },
            ],
            vec![screen.default_char(), screen.default_char()],
            vec![screen.default_char(), screen.default_char()],
            vec![
                CharOpts { data: "o".to_string(), ..CharOpts::default() },
                CharOpts { data: "h".to_string(), ..CharOpts::default() },
            ],
        ];
        assert_eq!(tolist(&screen), expected);

        // Third index
        screen.index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (3, 0));
        assert_eq!(
            screen.display(),
            vec![
                "bo".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "oh".to_string()
            ]
        );

        let expected = vec![
            vec![
                CharOpts { data: "b".to_string(), ..CharOpts::default() },
                CharOpts { data: "o".to_string(), ..CharOpts::default() },
            ],
            vec![screen.default_char(), screen.default_char()],
            vec![screen.default_char(), screen.default_char()],
            vec![screen.default_char(), screen.default_char()],
            vec![
                CharOpts { data: "o".to_string(), ..CharOpts::default() },
                CharOpts { data: "h".to_string(), ..CharOpts::default() },
            ],
        ];
        assert_eq!(tolist(&screen), expected);

        // Fourth index (nothing should change)
        screen.index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (3, 0));
        assert_eq!(
            screen.display(),
            vec![
                "bo".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "oh".to_string()
            ]
        );
        assert_eq!(tolist(&screen), expected);
    }

    #[test]
    fn reverse_index() {
        // a) Test basic reverse index
        let mut screen = Screen::new(2, 2);
        update(&mut screen, vec!["wo", "ot"], vec![0]);

        // Test reverse indexing on first row
        screen.reverse_index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));

        let expected = vec![
            vec![screen.default_char(), screen.default_char()],
            vec![
                CharOpts {
                    data: "w".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
                CharOpts {
                    data: "o".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
            ],
        ];
        assert_eq!(tolist(&screen), expected);

        // b) Test second reverse index
        screen.reverse_index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));

        let expected = vec![
            vec![screen.default_char(), screen.default_char()],
            vec![screen.default_char(), screen.default_char()],
        ];
        assert_eq!(tolist(&screen), expected);

        // c) Test with margins
        let mut screen = Screen::new(2, 5);
        update(&mut screen, vec!["bo", "sh", "th", "er", "oh"], vec![2, 3]);
        screen.set_margins(Some(2), Some(4));
        screen.cursor.y = 1;

        // First reverse index
        screen.reverse_index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(
            screen.display(),
            vec![
                "bo".to_string(),
                "  ".to_string(),
                "sh".to_string(),
                "th".to_string(),
                "oh".to_string()
            ]
        );

        let expected = vec![
            vec![co!("b"), co!("o")],
            vec![screen.default_char(), screen.default_char()],
            vec![co!("s"), co!("h")],
            vec![
                CharOpts {
                    data: "t".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
                CharOpts {
                    data: "h".to_string(),
                    fg: "red".to_string(),
                    ..CharOpts::default()
                },
            ],
            vec![co!("o"), co!("h")],
        ];
        assert_eq!(tolist(&screen), expected);

        // Second reverse index
        screen.reverse_index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(
            screen.display(),
            vec![
                "bo".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "sh".to_string(),
                "oh".to_string()
            ]
        );

        let expected = vec![
            vec![co!("b"), co!("o")],
            vec![screen.default_char(), screen.default_char()],
            vec![screen.default_char(), screen.default_char()],
            vec![co!("s"), co!("h")],
            vec![co!("o"), co!("h")],
        ];
        assert_eq!(tolist(&screen), expected);

        // Third reverse index
        screen.reverse_index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(
            screen.display(),
            vec![
                "bo".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "oh".to_string()
            ]
        );

        let expected = vec![
            vec![co!("b"), co!("o")],
            vec![screen.default_char(), screen.default_char()],
            vec![screen.default_char(), screen.default_char()],
            vec![screen.default_char(), screen.default_char()],
            vec![co!("o"), co!("h")],
        ];
        assert_eq!(tolist(&screen), expected);

        // Fourth reverse index (nothing should change)
        screen.reverse_index();
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(
            screen.display(),
            vec![
                "bo".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "  ".to_string(),
                "oh".to_string()
            ]
        );
        assert_eq!(tolist(&screen), expected);
    }

    #[test]
    fn linefeed() {
        // Setup screen
        let mut screen = Screen::new(2, 2);
        update(&mut screen, vec!["bo", "sh"], vec![]);
        screen.set_mode(&[LNM], false);

        // a) Test with LNM on
        assert!(screen.mode.contains(&LNM));
        screen.cursor.x = 1;
        screen.cursor.y = 0;
        screen.linefeed();
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));

        // b) Test with LNM off
        screen.reset_mode(&[LNM], false);
        screen.cursor.x = 1;
        screen.cursor.y = 0;
        screen.linefeed();
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 1));
    }

    #[test]
    fn linefeed_margins() {
        // See issue #63 on GitHub.
        let mut screen = Screen::new(80, 24);
        screen.set_margins(Some(3), Some(27));
        screen.cursor_position(None, None);
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        screen.linefeed();
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
    }

    #[test]
    fn tabstops() {
        let mut screen = Screen::new(10, 10);

        // Check initial tabstops
        assert_eq!(screen.tabstops, {
            let mut stops = HashSet::new();
            stops.insert(8);
            stops
        });

        // Clear tabstops
        screen.clear_tab_stop(Some(3));
        assert!(screen.tabstops.is_empty());

        // Set new tabstops
        screen.cursor.x = 1;
        screen.set_tab_stop();
        screen.cursor.x = 8;
        screen.set_tab_stop();

        // Test tab behavior
        screen.cursor.x = 0;
        screen.tab();
        assert_eq!(screen.cursor.x, 1);
        screen.tab();
        assert_eq!(screen.cursor.x, 8);
        screen.tab();
        assert_eq!(screen.cursor.x, 9);
        screen.tab();
        assert_eq!(screen.cursor.x, 9);
    }

    #[test]
    fn clear_tabstops() {
        let mut screen = Screen::new(10, 10);
        screen.clear_tab_stop(Some(3));

        // a) Clear a tabstop at current cursor location
        screen.cursor.x = 1;
        screen.set_tab_stop();
        screen.cursor.x = 5;
        screen.set_tab_stop();
        screen.clear_tab_stop(None); // Clear at current cursor position (5)

        // Check only tabstop at 1 remains
        assert_eq!(screen.tabstops, {
            let mut stops = HashSet::new();
            stops.insert(1);
            stops
        });

        // Set and clear using explicit position
        screen.set_tab_stop();
        screen.clear_tab_stop(Some(0));

        // Check tabstop at 1 still remains
        assert_eq!(screen.tabstops, {
            let mut stops = HashSet::new();
            stops.insert(1);
            stops
        });

        // b) Clear all tabstops
        screen.set_tab_stop();
        screen.cursor.x = 9;
        screen.set_tab_stop();
        screen.clear_tab_stop(Some(3)); // 3 means clear all

        // Check all tabstops are cleared
        assert!(screen.tabstops.is_empty());
    }

    #[test]
    fn backspace() {
        let mut screen = Screen::new(2, 2);
        assert_eq!(screen.cursor.x, 0);

        // Test backspace at left edge
        screen.backspace();
        assert_eq!(screen.cursor.x, 0);

        // Test backspace from position 1
        screen.cursor.x = 1;
        screen.backspace();
        assert_eq!(screen.cursor.x, 0);
    }

    #[test]
    fn test_save_cursor() {
        // a) Test cursor position
        let mut screen = Screen::new(10, 10);
        screen.save_cursor();
        screen.cursor.x = 3;
        screen.cursor.y = 5;
        screen.save_cursor();
        screen.cursor.x = 4;
        screen.cursor.y = 4;

        // Restore and check last saved position
        screen.restore_cursor();
        assert_eq!(screen.cursor.x, 3);
        assert_eq!(screen.cursor.y, 5);

        // Restore and check initial position
        screen.restore_cursor();
        assert_eq!(screen.cursor.x, 0);
        assert_eq!(screen.cursor.y, 0);

        // b) Test modes
        let mut screen = Screen::new(10, 10);
        screen.set_mode(&[DECAWM, DECOM], false);
        screen.save_cursor();

        screen.reset_mode(&[DECAWM], false);

        screen.restore_cursor();
        assert!(screen.mode.contains(&DECAWM));
        assert!(screen.mode.contains(&DECOM));

        // c) Test attributes
        let mut screen = Screen::new(10, 10);
        screen.select_graphic_rendition(&[4]); // underscore
        screen.save_cursor();
        screen.select_graphic_rendition(&[24]); // no underscore

        assert_eq!(screen.cursor.attr, screen.default_char());

        screen.restore_cursor();

        assert_ne!(screen.cursor.attr, screen.default_char());
        assert_eq!(
            screen.cursor.attr,
            CharOpts { underscore: true, ..CharOpts::default() }
        );
    }

    #[test]
    fn test_restore_cursor_with_none_saved() {
        let mut screen = Screen::new(10, 10);
        screen.set_mode(&[DECOM], false);
        screen.cursor.x = 5;
        screen.cursor.y = 5;

        screen.restore_cursor();

        // Check cursor position resets to (0,0)
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        // Check DECOM mode is cleared
        assert!(!screen.mode.contains(&DECOM));
    }

    #[test]
    fn test_restore_cursor_out_of_bounds() {
        let mut screen = Screen::new(10, 10);

        // a) Test with origin mode off
        screen.cursor_position(Some(5), Some(5));
        screen.save_cursor();
        screen.resize(Some(3), Some(3));
        screen.reset();
        screen.restore_cursor();

        assert_eq!((screen.cursor.y, screen.cursor.x), (2, 2));

        // b) Test with origin mode on
        screen.resize(Some(10), Some(10));
        screen.cursor_position(Some(8), Some(8));
        screen.save_cursor();
        screen.resize(Some(5), Some(5));
        screen.reset();
        screen.set_mode(&[DECOM], false);
        screen.set_margins(Some(2), Some(3));
        screen.restore_cursor();

        assert_eq!((screen.cursor.y, screen.cursor.x), (2, 4));
    }

    #[test]
    fn test_insert_lines_basic() {
        // Basic insert without margins
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![1]);
        screen.insert_lines(None);

        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["   ", "sam", "is "]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![co!("s"), co!("a"), co!("m"),],
                vec![
                    co!("i", fg = "red"),
                    co!("s", fg = "red"),
                    co!(" ", fg = "red"),
                ],
            ]
        );
    }

    #[test]
    fn test_insert_multiple_lines() {
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![1]);
        screen.insert_lines(Some(2));

        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["   ", "   ", "sam"]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![co!("s"), co!("a"), co!("m"),],
            ]
        );
    }

    #[test]
    fn test_insert_lines_with_margins() {
        let mut screen = Screen::new(3, 5);
        update(
            &mut screen,
            vec!["sam", "is ", "foo", "bar", "baz"],
            vec![2, 3],
        );
        screen.set_margins(Some(1), Some(4));
        screen.cursor.y = 1;
        screen.insert_lines(Some(1));

        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(screen.display(), vec!["sam", "   ", "is ", "foo", "baz"]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![co!("s"), co!("a"), co!("m"),],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![co!("i"), co!("s"), co!(" "),],
                vec![
                    co!("f", fg = "red"),
                    co!("o", fg = "red"),
                    co!("o", fg = "red"),
                ],
                vec![co!("b"), co!("a"), co!("z"),],
            ]
        );
    }

    #[test]
    fn test_insert_lines_limited_margins() {
        let mut screen = Screen::new(3, 5);
        update(
            &mut screen,
            vec!["sam", "is ", "foo", "bar", "baz"],
            vec![2, 3],
        );
        screen.set_margins(Some(1), Some(3));
        screen.cursor.y = 1;
        screen.insert_lines(Some(1));

        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(screen.display(), vec!["sam", "   ", "is ", "bar", "baz"]);

        screen.insert_lines(Some(2));
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(screen.display(), vec!["sam", "   ", "   ", "bar", "baz"]);
    }

    #[test]
    fn test_insert_lines_overflow() {
        // Test inserting more lines than available within margins
        let mut screen = Screen::new(3, 5);
        update(
            &mut screen,
            vec!["sam", "is ", "foo", "bar", "baz"],
            vec![2, 3],
        );
        screen.set_margins(Some(2), Some(4));
        screen.cursor.y = 1;
        screen.insert_lines(Some(20));

        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(screen.display(), vec!["sam", "   ", "   ", "   ", "baz"]);
    }

    #[test]
    fn test_insert_lines_outside_margins() {
        // Test inserting when cursor is outside margins
        let mut screen = Screen::new(3, 5);
        update(
            &mut screen,
            vec!["sam", "is ", "foo", "bar", "baz"],
            vec![2, 3],
        );
        screen.set_margins(Some(2), Some(4));
        screen.insert_lines(Some(5));

        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["sam", "is ", "foo", "bar", "baz"]);
    }

    #[test]
    fn test_delete_lines_basic() {
        // Test basic line deletion without margins
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![1]);
        screen.delete_lines(None);

        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["is ", "foo", "   "]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![
                    co!("i", fg = "red"),
                    co!("s", fg = "red"),
                    co!(" ", fg = "red"),
                ],
                vec![co!("f"), co!("o"), co!("o"),],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
            ]
        );
    }

    #[test]
    fn test_delete_lines_with_margins() {
        let mut screen = Screen::new(3, 5);
        update(
            &mut screen,
            vec!["sam", "is ", "foo", "bar", "baz"],
            vec![2, 3],
        );
        screen.set_margins(Some(1), Some(4));
        screen.cursor.y = 1;
        screen.delete_lines(Some(1));

        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(screen.display(), vec!["sam", "foo", "bar", "   ", "baz"]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![co!("s"), co!("a"), co!("m"),],
                vec![
                    co!("f", fg = "red"),
                    co!("o", fg = "red"),
                    co!("o", fg = "red"),
                ],
                vec![
                    co!("b", fg = "red"),
                    co!("a", fg = "red"),
                    co!("r", fg = "red"),
                ],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![co!("b"), co!("a"), co!("z"),],
            ]
        );
    }

    #[test]
    fn test_delete_multiple_lines_with_margins() {
        let mut screen = Screen::new(3, 5);
        update(
            &mut screen,
            vec!["sam", "is ", "foo", "bar", "baz"],
            vec![2, 3],
        );
        screen.set_margins(Some(1), Some(4));
        screen.cursor.y = 1;
        screen.delete_lines(Some(2));

        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(screen.display(), vec!["sam", "bar", "   ", "   ", "baz"]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![co!("s"), co!("a"), co!("m"),],
                vec![
                    co!("b", fg = "red"),
                    co!("a", fg = "red"),
                    co!("r", fg = "red"),
                ],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![co!("b"), co!("a"), co!("z"),],
            ]
        );
    }

    #[test]
    fn test_delete_lines_overflow() {
        let mut screen = Screen::new(3, 5);
        update(
            &mut screen,
            vec!["sam", "is ", "foo", "bar", "baz"],
            vec![2, 3],
        );
        screen.set_margins(Some(1), Some(4));
        screen.cursor.y = 1;
        screen.delete_lines(Some(5));

        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 0));
        assert_eq!(screen.display(), vec!["sam", "   ", "   ", "   ", "baz"]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![co!("s"), co!("a"), co!("m"),],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![co!("b"), co!("a"), co!("z"),],
            ]
        );
    }

    #[test]
    fn test_delete_lines_outside_margins() {
        let mut screen = Screen::new(3, 5);
        update(
            &mut screen,
            vec!["sam", "is ", "foo", "bar", "baz"],
            vec![2, 3],
        );
        screen.set_margins(Some(2), Some(4));
        screen.cursor.y = 0;
        screen.delete_lines(Some(5));

        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["sam", "is ", "foo", "bar", "baz"]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![co!("s"), co!("a"), co!("m"),],
                vec![co!("i"), co!("s"), co!(" "),],
                vec![
                    co!("f", fg = "red"),
                    co!("o", fg = "red"),
                    co!("o", fg = "red"),
                ],
                vec![
                    co!("b", fg = "red"),
                    co!("a", fg = "red"),
                    co!("r", fg = "red"),
                ],
                vec![co!("b"), co!("a"), co!("z"),],
            ]
        );
    }

    #[test]
    fn test_delete_lines_default_count() {
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![1]);

        // First deletion
        screen.delete_lines(None);
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["is ", "foo", "   "]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![
                    co!("i", fg = "red"),
                    co!("s", fg = "red"),
                    co!(" ", fg = "red"),
                ],
                vec![co!("f"), co!("o"), co!("o"),],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
            ]
        );

        // Second deletion
        screen.delete_lines(None);
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["foo", "   ", "   "]);
        assert_eq!(
            tolist(&screen),
            vec![
                vec![co!("f"), co!("o"), co!("o"),],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
                vec![
                    screen.default_char(),
                    screen.default_char(),
                    screen.default_char()
                ],
            ]
        );
    }

    #[test]
    fn test_insert_characters_normal() {
        let mut screen = Screen::new(3, 4);
        update(&mut screen, vec!["sam", "is ", "foo", "bar"], vec![0]);

        // Save cursor position
        let cursor_x = screen.cursor.x;
        let cursor_y = screen.cursor.y;

        screen.insert_characters(Some(2));

        // Check cursor hasn't moved
        assert_eq!((screen.cursor.y, screen.cursor.x), (cursor_y, cursor_x));

        // Check first line
        assert_eq!(
            tolist(&screen)[0],
            vec![
                screen.default_char(),
                screen.default_char(),
                co!("s", fg = "red"),
            ]
        );
    }

    #[test]
    fn test_insert_characters_middle() {
        let mut screen = Screen::new(3, 4);
        update(&mut screen, vec!["sam", "is ", "foo", "bar"], vec![0]);

        screen.cursor.y = 2;
        screen.cursor.x = 1;
        screen.insert_characters(Some(1));

        assert_eq!(tolist(&screen)[2], cv![co!("f"), co!(default), co!("o")]);
    }

    #[test]
    fn insert_characters_overflow() {
        let mut screen = Screen::new(3, 4);
        update(&mut screen, vec!["sam", "is ", "foo", "bar"], vec![0]);

        screen.cursor.y = 3;
        screen.cursor.x = 1;
        screen.insert_characters(Some(10));

        assert_eq!(
            tolist(&screen)[3],
            vec![
                CharOpts { data: "b".to_string(), ..CharOpts::default() },
                screen.default_char(),
                screen.default_char(),
            ]
        );
    }

    #[test]
    fn insert_characters_default_count() {
        // Test with no count (should default to 1)
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![0]);

        screen.cursor_position(None, None);
        screen.insert_characters(None);

        assert_eq!(
            tolist(&screen)[0],
            cv![co!(default), co!("s", fg = "red"), co!("a", fg = "red"),]
        );

        // Test with explicit count of 1
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![0]);

        screen.cursor_position(None, None);
        screen.insert_characters(Some(1));

        assert_eq!(
            tolist(&screen)[0],
            cv![co!(default), co!("s", fg = "red"), co!("a", fg = "red")]
        );
    }

    #[test]
    fn test_delete_characters() {
        // Basic case
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![0]);
        screen.delete_characters(Some(2));
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["m  ", "is ", "foo"]);
        assert_eq!(
            tolist(&screen)[0],
            cv![co!("m", fg = "red"), co!(default), co!(default)]
        );

        // Delete at position (2,2)
        screen.cursor.y = 2;
        screen.cursor.x = 2;
        screen.delete_characters(None);
        assert_eq!((screen.cursor.y, screen.cursor.x), (2, 2));
        assert_eq!(screen.display(), vec!["m  ", "is ", "fo "]);

        // Delete at position (1,1)
        screen.cursor.y = 1;
        screen.cursor.x = 1;
        screen.delete_characters(Some(0));
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 1));
        assert_eq!(screen.display(), vec!["m  ", "i  ", "fo "]);

        // Extreme cases
        // 1. Delete from middle of line
        let mut screen = Screen::new(5, 1);
        update(&mut screen, vec!["12345"], vec![0]);
        screen.cursor.x = 1;
        screen.delete_characters(Some(3));
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 1));
        assert_eq!(screen.display(), vec!["15   "]);
        assert_eq!(
            tolist(&screen)[0],
            cv![
                co!("1", fg = "red"),
                co!("5", fg = "red"),
                co!(default),
                co!(default),
                co!(default)
            ]
        );

        // 2. Delete more than available
        let mut screen = Screen::new(5, 1);
        update(&mut screen, vec!["12345"], vec![0]);
        screen.cursor.x = 2;
        screen.delete_characters(Some(10));
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 2));
        assert_eq!(screen.display(), vec!["12   "]);
        assert_eq!(
            tolist(&screen)[0],
            cv![
                co!("1", fg = "red"),
                co!("2", fg = "red"),
                co!(default),
                co!(default),
                co!(default)
            ]
        );

        // 3. Delete from start
        let mut screen = Screen::new(5, 1);
        update(&mut screen, vec!["12345"], vec![0]);
        screen.delete_characters(Some(4));
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["5    "]);
        assert_eq!(
            tolist(&screen)[0],
            cv![
                co!("5", fg = "red"),
                co!(default),
                co!(default),
                co!(default),
                co!(default)
            ]
        );
    }

    #[test]
    fn test_erase_characters() {
        // Basic case
        let mut screen = Screen::new(3, 3);
        update(&mut screen, vec!["sam", "is ", "foo"], vec![0]);

        screen.erase_characters(Some(2));
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["  m", "is ", "foo"]);
        assert_eq!(
            tolist(&screen)[0],
            cv![co!(default), co!(default), co!("m", fg = "red")]
        );

        screen.cursor.y = 2;
        screen.cursor.x = 2;
        screen.erase_characters(None);
        assert_eq!((screen.cursor.y, screen.cursor.x), (2, 2));
        assert_eq!(screen.display(), vec!["  m", "is ", "fo "]);

        screen.cursor.y = 1;
        screen.cursor.x = 1;
        screen.erase_characters(Some(0));
        assert_eq!((screen.cursor.y, screen.cursor.x), (1, 1));
        assert_eq!(screen.display(), vec!["  m", "i  ", "fo "]);

        // Extreme cases
        // 1. Erase from middle
        let mut screen = Screen::new(5, 1);
        update(&mut screen, vec!["12345"], vec![0]);
        screen.cursor.x = 1;
        screen.erase_characters(Some(3));
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 1));
        assert_eq!(screen.display(), vec!["1   5"]);
        assert_eq!(
            tolist(&screen)[0],
            cv![
                co!("1", fg = "red"),
                co!(default),
                co!(default),
                co!(default),
                co!("5", fg = "red")
            ]
        );

        // 2. Erase more than available
        let mut screen = Screen::new(5, 1);
        update(&mut screen, vec!["12345"], vec![0]);
        screen.cursor.x = 2;
        screen.erase_characters(Some(10));
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 2));
        assert_eq!(screen.display(), vec!["12   "]);
        assert_eq!(
            tolist(&screen)[0],
            cv![
                co!("1", fg = "red"),
                co!("2", fg = "red"),
                co!(default),
                co!(default),
                co!(default)
            ]
        );

        // 3. Erase from start
        let mut screen = Screen::new(5, 1);
        update(&mut screen, vec!["12345"], vec![0]);
        screen.erase_characters(Some(4));
        assert_eq!((screen.cursor.y, screen.cursor.x), (0, 0));
        assert_eq!(screen.display(), vec!["    5"]);
        assert_eq!(
            tolist(&screen)[0],
            cv![
                co!(default),
                co!(default),
                co!(default),
                co!(default),
                co!("5", fg = "red")
            ]
        );
    }
}
