use crate::parser_listener::ParserListener;

pub struct ParserPrinter {}

impl ParserListener for ParserPrinter {
    fn alignment_display(&self) {
        println!("alignment display");
    }

    fn define_charset(&mut self, code: &str, mode: &str) {
        println!("defining charset code {} mode {}", code, mode);
    }

    fn reset(&mut self) {
        println!("reset");
    }

    fn index(&mut self) {
        println!("index");
    }

    fn linefeed(&mut self) {
        println!("linefeed");
    }

    fn reverse_index(&mut self) {
        println!("reverse_index");
    }

    fn set_tab_stop(&mut self) {
        println!("set_tab_stop");
    }

    fn save_cursor(&mut self) {
        println!("save_cursor");
    }

    fn restore_cursor(&mut self) {
        println!("restore_cursor");
    }

    fn bell(&mut self) {
        println!("bell");
    }

    fn backspace(&mut self) {
        println!("backspace");
    }

    fn tab(&mut self) {
        println!("tab");
    }

    fn cariage_return(&mut self) {
        println!("carriage return")
    }

    fn draw(&self, input: &str) {
        println!("draw input {}", input);
    }

    fn insert_characters(&mut self, count: Option<u32>) {
        println!("insert_characters count {:?}", count);
    }

    fn cursor_up(&mut self, count: Option<u32>) {
        println!("cursor up count {:?} ", count);
    }

    fn cursor_down(&mut self, count: Option<u32>) {
        println!("cursor down count {:?}", count);
    }

    fn cursor_forward(&self, count: Option<u32>) {
        println!("cursor forward count {:?}", count);
    }

    fn cursor_back(&mut self, count: Option<u32>) {
        println!("cursor back count {:?}", count);
    }

    fn cursor_down1(&mut self, count: Option<u32>) {
        println!("cursor down count {:?}", count);
    }

    fn cursor_up1(&mut self, count: Option<u32>) {
        println!("cursor up1 count {:?}", count);
    }

    fn cursor_to_column(&mut self, character: Option<u32>) {
        println!("cursor to column character {:?}", character);
    }

    fn cursor_position(&mut self, line: Option<u32>, character: Option<u32>) {
        println!("cursor position");
    }

    fn erase_in_display(&mut self, how: Option<u32>, private: Option<bool>) {
        println!("erase in display");
    }

    fn erase_in_line(&mut self, how: Option<u32>, private: Option<bool>) {
        println!("erase in line");
    }

    fn insert_lines(&mut self, count: Option<u32>) {
        println!("insert lines")
    }

    fn delete_lines(&mut self, count: Option<u32>) {
        println!("delete lines");
    }

    fn delete_characters(&mut self, count: Option<u32>) {
        println!("delete characters");
    }

    fn erase_characters(&mut self, count: Option<u32>) {
        println!("erase characters");
    }

    fn report_device_attributes(&mut self, mode: Option<u32>, private: Option<bool>) {
        println!("report device attributes");
    }

    fn cursor_to_line(&self, count: Option<u32>) {
        println!("cursor to line");
    }

    fn clear_tab_stop(&self, option: Option<u32>) {
        println!("clear tab stop");
    }

    fn set_mode(&mut self, modes: &[u32], is_private: bool) {
        println!("set mode");
    }

    fn reset_mode(&mut self, modes: &[u32], is_private: bool) {
        println!("reset mode");
    }

    fn select_graphic_rendition(&self, modes: &[u32]) {
        println!("select graphic rendition");
    }

    fn shift_out(&mut self) {
        println!("shift out");
    }

    fn shift_in(&mut self) {
        println!("shift in");
    }

    fn set_title(&mut self, title: &str) {
        println!("set_title {}", title);
    }

    fn set_icon_name(&mut self, icon_name: &str) {
        println!("set icon_name {}", icon_name);
    }
}
