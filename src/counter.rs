use std::collections::HashMap;

use crate::parser_listener::ParserListener;

pub struct Counter {
    pub counts: HashMap<&'static str, i32>,
}

impl Counter {
    pub fn new() -> Self {
        Counter { counts: HashMap::new() }
    }

    fn increment(&mut self, name: &'static str) {
        *self.counts.entry(name).or_insert(0) += 1;
    }

    pub fn get_count(&self, name: &str) -> i32 {
        *self.counts.get(name).unwrap_or(&0)
    }
}

impl ParserListener for Counter {
    fn alignment_display(&mut self) {
        self.increment("alignment_display");
    }
    fn backspace(&mut self) {
        self.increment("backspace");
    }
    fn bell(&mut self) {
        self.increment("bell");
    }

    fn linefeed(&mut self) {
        self.increment("linefeed");
    }
    fn cursor_back(&mut self, _: Option<u32>) {
        self.increment("cursor_back");
    }
    fn cursor_down(&mut self, _: Option<u32>) {
        self.increment("cursor_down");
    }
    fn cursor_down1(&mut self, _: Option<u32>) {
        self.increment("cursor_down1");
    }
    fn cursor_forward(&mut self, _: Option<u32>) {
        self.increment("cursor_forward");
    }
    fn cursor_position(&mut self, _: Option<u32>, _: Option<u32>) {
        self.increment("cursor_position");
    }
    fn cursor_to_column(&mut self, _: Option<u32>) {
        self.increment("cursor_to_column");
    }
    fn cursor_to_line(&mut self, _: Option<u32>) {
        self.increment("cursor_to_line");
    }
    fn cursor_up(&mut self, _: Option<u32>) {
        self.increment("cursor_up");
    }
    fn cursor_up1(&mut self, _: Option<u32>) {
        self.increment("cursor_up1");
    }
    fn delete_characters(&mut self, _: Option<u32>) {
        self.increment("delete_characters");
    }
    fn delete_lines(&mut self, _: Option<u32>) {
        self.increment("delete_lines");
    }
    fn draw(&mut self, _: &str) {
        self.increment("draw");
    }
    fn erase_characters(&mut self, _: Option<u32>) {
        self.increment("erase_characters");
    }
    fn erase_in_display(&mut self, _: Option<u32>, _: Option<bool>) {
        self.increment("erase_in_display");
    }
    fn erase_in_line(&mut self, _: Option<u32>, _: Option<bool>) {
        self.increment("erase_in_line");
    }
    fn index(&mut self) {
        self.increment("index");
    }
    fn insert_characters(&mut self, _: Option<u32>) {
        self.increment("insert_characters");
    }
    fn insert_lines(&mut self, _: Option<u32>) {
        self.increment("insert_lines");
    }
    fn report_device_attributes(&mut self, _: Option<u32>, _: Option<bool>) {
        self.increment("report_device_attributes");
    }
    fn reverse_index(&mut self) {
        self.increment("reverse_index");
    }
    fn save_cursor(&mut self) {
        self.increment("save_cursor");
    }
    fn restore_cursor(&mut self) {
        self.increment("restore_cursor");
    }
    fn set_icon_name(&mut self, _: &str) {
        self.increment("set_icon_name");
    }
    fn set_title(&mut self, _: &str) {
        self.increment("set_title");
    }
    fn tab(&mut self) {
        self.increment("tab");
    }
    fn clear_tab_stop(&mut self, _: Option<u32>) {
        self.increment("clear_tab_stop");
    }

    fn define_charset(&mut self, _code: &str, _mode: &str) {
        self.increment("define_charset");
    }

    fn reset(&mut self) {
        self.increment("reset");
    }

    fn set_tab_stop(&mut self) {
        self.increment("set_tab_stop");
    }

    fn shift_out(&mut self) {
        self.increment("shift_out");
    }

    fn shift_in(&mut self) {
        self.increment("shift_in");
    }

    fn cariage_return(&mut self) {
        self.increment("cariage_return");
    }

    fn set_mode(&mut self, _modes: &[u32], _is_private: bool) {
        self.increment("set_mode");
    }

    fn reset_mode(&mut self, _modes: &[u32], _is_private: bool) {
        self.increment("reset_mode");
    }

    fn select_graphic_rendition(&mut self, _modes: &[u32]) {
        self.increment("select_graphic_rendition");
    }
    // Implement other ParserListener methods similarly
}
