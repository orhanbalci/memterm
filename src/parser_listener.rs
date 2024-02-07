pub trait ParserListener {
    fn alignment_display(&self);
    fn define_charset(&self, code: &str, mode: &str);
    fn reset(&self);
    fn index(&self);
    fn linefeed(&self);
    fn reverse_index(&self);
    fn set_tab_stop(&self);
    fn save_cursor(&self);
    fn restore_cursor(&self);

    // basic esvape code actions
    fn bell(&self);
    fn backspace(&self);
    fn tab(&self);
    fn cariage_return(&self);

    fn draw(&self, input: &str);

    //csi commands
    fn insert_characters(&self, count: Option<u32>);
    fn cursor_up(&self, count: Option<u32>);
    fn cursor_down(&self, count: Option<u32>);
    fn cursor_forward(&self, count: Option<u32>);
    fn cursor_back(&self, count: Option<u32>);
    fn cursor_down1(&self, count: Option<u32>);
    fn cursor_up1(&self, count: Option<u32>);
    fn cursor_to_column(&self, character: Option<u32>);
    fn cursor_position(&self, line: Option<u32>, character: Option<u32>);
    fn erase_in_display(&self, erase_page: Option<u32>);
    fn erase_in_line(&self, erase_line: Option<u32>);
    fn insert_lines(&self, count: Option<u32>);
    fn delete_lines(&self, count: Option<u32>);
    fn delete_characters(&self, count: Option<u32>);
    fn erase_characters(&self, count: Option<u32>);
    fn report_device_attributes(&self, attribute: Option<u32>);
    fn cursor_to_line(&self, count: Option<u32>);
    fn clear_tab_stop(&self, option: Option<u32>);
    fn set_mode(&self, modes: &[u32]);
    fn reset_mode(&self, modes: &[u32]);
    fn select_graphic_rendition(&self, modes: &[u32]);
}
