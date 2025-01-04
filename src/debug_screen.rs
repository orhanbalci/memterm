use crate::parser_listener::ParserListener;

pub struct DebugScreen {
    pub output: Vec<String>,
}

impl DebugScreen {
    pub fn new() -> Self {
        DebugScreen { output: Vec::new() }
    }

    fn record(&mut self, s: String) {
        println!("{}", s);
        self.output.push(s);
    }
}

impl ParserListener for DebugScreen {
    fn alignment_display(&mut self) {
        self.record("[\"alignment_display\"]".to_string());
    }

    fn define_charset(&mut self, code: &str, mode: &str) {
        self.record(format!("[\"define_charset\", [{}, {}]]", code, mode));
    }

    fn reset(&mut self) {
        self.record("[\"reset\"]".to_string());
    }

    fn index(&mut self) {
        self.record("[\"index\"]".to_string());
    }

    fn linefeed(&mut self) {
        self.record("[\"linefeed\"]".to_string());
    }

    fn reverse_index(&mut self) {
        self.record("[\"reverse_index\"]".to_string());
    }

    fn set_tab_stop(&mut self) {
        self.record("[\"set_tab_stop\"]".to_string());
    }

    fn save_cursor(&mut self) {
        self.record("[\"save_cursor\"]".to_string());
    }

    fn restore_cursor(&mut self) {
        self.record("[\"restore_cursor\"]".to_string());
    }

    fn bell(&mut self) {
        self.record("[\"bell\"]".to_string());
    }

    fn backspace(&mut self) {
        self.record("[\"backspace\"]".to_string());
    }

    fn tab(&mut self) {
        self.record("[\"tab\"]".to_string());
    }

    fn cariage_return(&mut self) {
        self.record("[\"carriage_return\"]".to_string());
    }

    fn draw(&mut self, input: &str) {
        self.record(format!("[\"draw\", {:?}]", input));
    }

    fn insert_characters(&mut self, count: Option<u32>) {
        self.record(format!("[\"insert_characters\", {:?}]", count));
    }

    fn cursor_up(&mut self, count: Option<u32>) {
        self.record(format!("[\"cursor_up\", {:?}]", count));
    }

    fn cursor_down(&mut self, count: Option<u32>) {
        self.record(format!("[\"cursor_down\", {:?}]", count));
    }

    fn cursor_forward(&mut self, count: Option<u32>) {
        self.record(format!("[\"cursor_forward\", {:?}]", count));
    }

    fn cursor_back(&mut self, count: Option<u32>) {
        self.record(format!("[\"cursor_back\", {:?}]", count));
    }

    fn cursor_down1(&mut self, count: Option<u32>) {
        self.record(format!("[\"cursor_down1\", {:?}]", count));
    }

    fn cursor_up1(&mut self, count: Option<u32>) {
        self.record(format!("[\"cursor_up1\", {:?}]", count));
    }

    fn cursor_to_column(&mut self, character: Option<u32>) {
        self.record(format!("[\"cursor_to_column\", {:?}]", character));
    }

    fn cursor_position(&mut self, line: Option<u32>, character: Option<u32>) {
        self.record(format!(
            "[\"cursor_position\", {:?}, {:?}]",
            line, character
        ));
    }

    fn erase_in_display(&mut self, how: Option<u32>, private: Option<bool>) {
        self.record(format!("[\"erase_in_display\", {:?}, {:?}]", how, private));
    }

    fn erase_in_line(&mut self, how: Option<u32>, private: Option<bool>) {
        self.record(format!("[\"erase_in_line\", {:?}, {:?}]", how, private));
    }

    fn insert_lines(&mut self, count: Option<u32>) {
        self.record(format!("[\"insert_lines\", {:?}]", count));
    }

    fn delete_lines(&mut self, count: Option<u32>) {
        self.record(format!("[\"delete_lines\", {:?}]", count));
    }

    fn delete_characters(&mut self, count: Option<u32>) {
        self.record(format!("[\"delete_characters\", {:?}]", count));
    }

    fn erase_characters(&mut self, count: Option<u32>) {
        self.record(format!("[\"erase_characters\", {:?}]", count));
    }

    fn report_device_attributes(&mut self, mode: Option<u32>, private: Option<bool>) {
        self.record(format!(
            "[\"report_device_attributes\", {:?}, {:?}]",
            mode, private
        ));
    }

    fn cursor_to_line(&mut self, line: Option<u32>) {
        self.record(format!("[\"cursor_to_line\", {:?}]", line));
    }

    fn clear_tab_stop(&mut self, how: Option<u32>) {
        self.record(format!("[\"clear_tab_stop\", {:?}]", how));
    }

    fn set_mode(&mut self, modes: &[u32], is_private: bool) {
        self.record(format!("[\"set_mode\", {:?}, {:?}]", modes, is_private));
    }

    fn reset_mode(&mut self, modes: &[u32], is_private: bool) {
        self.record(format!("[\"reset_mode\", {:?}, {:?}]", modes, is_private));
    }

    fn select_graphic_rendition(&mut self, modes: &[u32]) {
        self.record(format!("[\"select_graphic_rendition\", {:?}]", modes));
    }

    fn shift_out(&mut self) {
        self.record("[\"shift_out\"]".to_string());
    }

    fn shift_in(&mut self) {
        self.record("[\"shift_in\"]".to_string());
    }

    fn set_title(&mut self, title: &str) {
        self.record(format!("[\"set_title\", {:?}]", title));
    }

    fn set_icon_name(&mut self, icon_name: &str) {
        self.record(format!("[\"set_icon_name\", {:?}]", icon_name));
    }
}
