use crate::ascii;
use crate::parser_listener::ParserListener;
use ansi_control_codes::c0::ESC;
use ansi_control_codes::c1::CSI;
use ansi_control_codes::c1::OSC;
use genawaiter::sync::gen;
use genawaiter::yield_;

pub const DECALN: &'static str = ascii!(3 / 8);

pub struct Parser<T: ParserListener> {
    listener: T,
    use_utf8: bool,
}

impl<T: ParserListener> Parser<T> {
    pub fn new(listener: T, use_utf8: bool) -> Self {
        Parser { listener, use_utf8 }
    }

    pub fn start(&self) {
        let mut printer = gen!({
            loop {
                let mut char: &str = yield_!(Some(true));
                if ESC.to_string() == char {
                    char = yield_!(None);
                    if char == "[" {
                        char = &CSI.to_string();
                    } else if char == "]" {
                        char = &OSC.to_string();
                    }
                } else {
                    if char == "#" {
                        // sharp dispatch
                        if yield_!(None) == DECALN.to_string() {
                            self.listener.alignment_display();
                        } else {
                            println!("unexpected escape character");
                        }
                    } else if char == "%" {
                        self.select_other_charset(yield_!(None));
                    } else if "()".contains(char) {
                        let code = yield_!(None);
                        if self.use_utf8 {
                            continue;
                        } else {
                            self.listener.define_charset(code, char);
                        }
                    }
                }
                // println!("{}", char);
            }
        });

        printer.resume_with("h");
        printer.resume_with("w");
    }

    fn select_other_charset(&self, input: &str) {}
}
