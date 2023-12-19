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
}
