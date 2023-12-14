pub trait ParserListener {
    fn alignment_display(&self);
    fn define_charset(&self, code: &str, mode: &str);
}
