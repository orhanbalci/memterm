use std::collections::HashMap;

use crate::parser_listener::ParserListener;

pub struct Counter {
    pub counts: HashMap<&'static str, i32>,
    pub last_params: HashMap<&'static str, Vec<u32>>, // Store numeric parameters
    pub last_strings: HashMap<&'static str, String>,  // Store string parameters
    pub last_private: Option<bool>,                   // Store private flag
}

impl Counter {
    pub fn new() -> Self {
        Counter {
            counts: HashMap::new(),
            last_params: HashMap::new(),
            last_strings: HashMap::new(),
            last_private: None,
        }
    }

    fn increment(&mut self, name: &'static str) {
        *self.counts.entry(name).or_insert(0) += 1;
    }

    fn save_params(&mut self, name: &'static str, params: &[u32]) {
        self.last_params.insert(name, params.to_vec());
    }

    fn save_string(&mut self, name: &'static str, s: &str) {
        self.last_strings.insert(name, s.to_string());
    }

    pub fn get_count(&self, name: &str) -> i32 {
        *self.counts.get(name).unwrap_or(&0)
    }

    pub fn get_last_params(&self, name: &str) -> Option<&Vec<u32>> {
        self.last_params.get(name)
    }

    pub fn get_last_string(&self, name: &str) -> Option<&String> {
        self.last_strings.get(name)
    }

    pub fn get_last_private(&self) -> Option<bool> {
        self.last_private
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

    fn cursor_back(&mut self, count: Option<u32>) {
        self.increment("cursor_back");
        self.save_params("cursor_back", &[count.unwrap_or(1)]);
    }

    fn cursor_down(&mut self, count: Option<u32>) {
        self.increment("cursor_down");
        self.save_params("cursor_down", &[count.unwrap_or(1)]);
    }

    fn cursor_down1(&mut self, count: Option<u32>) {
        self.increment("cursor_down1");
        self.save_params("cursor_down1", &[count.unwrap_or(1)]);
    }

    fn cursor_forward(&mut self, count: Option<u32>) {
        self.increment("cursor_forward");
        self.save_params("cursor_forward", &[count.unwrap_or(1)]);
    }

    fn cursor_position(&mut self, line: Option<u32>, column: Option<u32>) {
        self.increment("cursor_position");
        let mut params = vec![];
        if line.is_some() {
            params.push(line.unwrap());
        }

        if column.is_some() {
            params.push(column.unwrap());
        }

        self.save_params("cursor_position", params.as_slice());
    }

    fn cursor_to_column(&mut self, column: Option<u32>) {
        self.increment("cursor_to_column");
        self.save_params("cursor_to_column", &[column.unwrap_or(1)]);
    }

    fn cursor_to_line(&mut self, line: Option<u32>) {
        self.increment("cursor_to_line");
        self.save_params("cursor_to_line", &[line.unwrap_or(1)]);
    }

    fn cursor_up(&mut self, count: Option<u32>) {
        self.increment("cursor_up");
        self.save_params("cursor_up", &[count.unwrap_or(1)]);
    }

    fn cursor_up1(&mut self, count: Option<u32>) {
        self.increment("cursor_up1");
        self.save_params("cursor_up1", &[count.unwrap_or(1)]);
    }

    fn delete_characters(&mut self, count: Option<u32>) {
        self.increment("delete_characters");
        self.save_params("delete_characters", &[count.unwrap_or(1)]);
    }

    fn delete_lines(&mut self, count: Option<u32>) {
        self.increment("delete_lines");
        self.save_params("delete_lines", &[count.unwrap_or(1)]);
    }

    fn draw(&mut self, string: &str) {
        self.increment("draw");
        self.save_string("draw", string);
    }

    fn erase_characters(&mut self, count: Option<u32>) {
        self.increment("erase_characters");
        self.save_params("erase_characters", &[count.unwrap_or(1)]);
    }

    fn erase_in_display(&mut self, how: Option<u32>, private: Option<bool>) {
        self.increment("erase_in_display");
        self.save_params("erase_in_display", &[how.unwrap_or(0)]);
        self.last_private = private;
    }

    fn erase_in_line(&mut self, how: Option<u32>, private: Option<bool>) {
        self.increment("erase_in_line");
        self.save_params("erase_in_line", &[how.unwrap_or(0)]);
        self.last_private = private;
    }

    fn index(&mut self) {
        self.increment("index");
    }

    fn insert_characters(&mut self, count: Option<u32>) {
        self.increment("insert_characters");
        self.save_params("insert_characters", &[count.unwrap_or(1)]);
    }

    fn insert_lines(&mut self, count: Option<u32>) {
        self.increment("insert_lines");
        self.save_params("insert_lines", &[count.unwrap_or(1)]);
    }

    fn report_device_attributes(&mut self, mode: Option<u32>, private: Option<bool>) {
        self.increment("report_device_attributes");
        self.save_params("report_device_attributes", &[mode.unwrap_or(0)]);
        self.last_private = private;
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

    fn set_icon_name(&mut self, icon_name: &str) {
        self.increment("set_icon_name");
        self.save_string("set_icon_name", icon_name);
    }

    fn set_title(&mut self, title: &str) {
        self.increment("set_title");
        self.save_string("set_title", title);
    }

    fn tab(&mut self) {
        self.increment("tab");
    }

    fn clear_tab_stop(&mut self, how: Option<u32>) {
        self.increment("clear_tab_stop");
        self.save_params("clear_tab_stop", &[how.unwrap_or(0)]);
    }

    fn define_charset(&mut self, code: &str, mode: &str) {
        self.increment("define_charset");
        self.save_string("define_charset_code", code);
        self.save_string("define_charset_mode", mode);
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

    fn set_mode(&mut self, modes: &[u32], private: bool) {
        self.increment("set_mode");
        self.save_params("set_mode", modes);
        self.last_private = Some(private);
    }

    fn reset_mode(&mut self, modes: &[u32], private: bool) {
        self.increment("reset_mode");
        self.save_params("reset_mode", modes);
        self.last_private = Some(private);
    }

    fn select_graphic_rendition(&mut self, modes: &[u32]) {
        self.increment("select_graphic_rendition");
        self.save_params("select_graphic_rendition", modes);
    }
}
