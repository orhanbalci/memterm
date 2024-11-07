use crate::parser_listener::ParserListener;

pub struct ParserPrinter {}

impl ParserListener for ParserPrinter {
    fn alignment_display(&self) {
        println!("alignment display");
    }

    fn define_charset(&mut self, code: &str, mode: &str) {
        println!("defining charset");
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

    fn save_cursor(&self) {
        println!("save_cursor");
    }

    fn restore_cursor(&self) {
        println!("restore_cursor");
    }

    fn bell(&self) {
        println!("bell");
    }

    fn backspace(&self) {
        println!("backspace");
    }

    fn tab(&self) {
        println!("tab");
    }

    fn cariage_return(&mut self) {
        println!("carriage return")
    }

    fn draw(&self, input: &str) {
        println!("draw");
    }

    fn insert_characters(&self, count: Option<u32>) {
        println!("insert_characters");
    }

    fn cursor_up(&self, count: Option<u32>) {
        println!("cursor up");
    }

    fn cursor_down(&self, count: Option<u32>) {
        println!("cursor down");
    }

    fn cursor_forward(&self, count: Option<u32>) {
        println!("cursor forward");
    }

    fn cursor_back(&self, count: Option<u32>) {
        println!("cursor back");
    }

    fn cursor_down1(&self, count: Option<u32>) {
        println!("cursor down");
    }

    fn cursor_up1(&self, count: Option<u32>) {
        println!("cursor up1");
    }

    fn cursor_to_column(&self, character: Option<u32>) {
        println!("cursor to column");
    }

    fn cursor_position(&self, line: Option<u32>, character: Option<u32>) {
        println!("cursor position");
    }

    fn erase_in_display(&self, erase_page: Option<u32>) {
        println!("erase in display");
    }

    fn erase_in_line(&self, erase_line: Option<u32>) {
        println!("erase in line");
    }

    fn insert_lines(&self, count: Option<u32>) {
        println!("insert lines")
    }

    fn delete_lines(&self, count: Option<u32>) {
        println!("delete lines");
    }

    fn delete_characters(&self, count: Option<u32>) {
        println!("delete characters");
    }

    fn erase_characters(&self, count: Option<u32>) {
        println!("erase characters");
    }

    fn report_device_attributes(&self, attribute: Option<u32>) {
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
