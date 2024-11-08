use std::collections::btree_map::Keys;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use lazy_static::lazy_static;
use unicode_width::UnicodeWidthStr;

use crate::charset::{LAT1_MAP, MAPS, VT100_MAP};
use crate::modes::{DECAWM, DECCOLM, DECOM, DECSCNM, DECTCEM, LNM};
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
}

impl ParserListener for Screen {
    fn alignment_display(&self) {
        todo!()
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
    /// <div class="warning">User-defined charsets are currently not supported.</div>
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
    /// * Scrolling margins are reset to screen boundaries.
    /// * Cursor is moved to home location ``(0, 0)`` and its
    ///   attributes are set to defaults.
    /// * Screen is cleared, each character is reset to default char
    /// * Tabstops are reset to "every eight columns".
    /// * All lines are marked as dirty.
    ///
    /// <div class="warning">Neither VT220 nor VT102 manuals mention that terminal modes
    ///    and tabstops should be reset as well, thanks to
    ///    <code>xterm</code> -- we now know that.</div>
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

    // Set the current cursor position to whatever cursor is on top
    // of the stack.
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

    fn draw(&self, input: &str) {
        todo!()
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

    fn cursor_forward(&self, count: Option<u32>) {
        todo!()
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

    fn cursor_to_column(&self, character: Option<u32>) {
        todo!()
    }

    fn cursor_position(&mut self, line: Option<u32>, character: Option<u32>) {
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

    // Set (enable) a given list of modes.
    // :param list modes: modes to set, where each mode is a constant
    //    from :mod:`pyte.modes`.

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
            self.erase_in_display(Some(2));
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

    // Reset (disable) a given list of modes.
    // :param list modes: modes to reset -- hopefully, each mode is a
    //                   constant from :mod:`pyte.modes`.
    //
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
            self.erase_in_display(Some(2));
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

    fn select_graphic_rendition(&self, modes: &[u32]) {
        todo!()
    }

    /// Set terminal title.
    ///
    /// <div class="warning">This is an XTerm extension supported by the Linux terminal.</div>
    fn set_title(&mut self, title: &str) {
        self.title = title.to_owned();
    }

    /// Set icon name
    ///
    /// <div class="warning">This is an XTerm extension supported by the Linux terminal.</div>
    fn set_icon_name(&mut self, icon_name: &str) {
        self.icon_name = icon_name.to_owned();
    }
}
