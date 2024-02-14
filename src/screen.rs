use crate::parser_listener::ParserListener;

pub struct Screen {}

impl ParserListener for Screen {
    fn alignment_display(&self) {
        println!("alignment display");
    }

    fn define_charset(&self, code: &str, mode: &str) {
        println!("defining charset");
    }

    fn reset(&self) {
        println!("reset");
    }

    fn index(&self) {
        println!("index");
    }

    fn linefeed(&self) {
        println!("linefeed");
    }

    fn reverse_index(&self) {
        println!("reverse_index");
    }

    fn set_tab_stop(&self) {
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

    fn cariage_return(&self) {
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

    fn set_mode(&self, modes: &[u32]) {
        println!("set mode");
    }

    fn reset_mode(&self, modes: &[u32]) {
        println!("reset mode");
    }

    fn select_graphic_rendition(&self, modes: &[u32]) {
        println!("select graphic rendition");
    }
}
