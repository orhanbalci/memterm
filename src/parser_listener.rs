pub trait ParserListener {
    fn alignment_display(&self);
    fn define_charset(&self, code: &str, mode: &str);
    fn reset(&self);
    fn index(&self);
    fn linefeed(&self);
    fn reverse_index(&self);
}
