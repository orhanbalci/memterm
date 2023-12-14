use crate::ascii;
use crate::parser_listener::ParserListener;
use ansi_control_codes::c0::ESC;
use ansi_control_codes::c1::CSI;
use ansi_control_codes::c1::OSC;
use ansi_control_codes::independent_control_functions::RIS;
use genawaiter::sync::gen;
use genawaiter::yield_;

pub const DECALN: &'static str = ascii!(3 / 8);
pub const IND: &'static str = ascii!(4 / 4);

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
                        if yield_!(None) == DECALN {
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
                    } else {
                        //escape dispatch
                        self.escape_dispatch(char);
                    }
                    continue;
                }
                // println!("{}", char);
            }
        });

        printer.resume_with("h");
        printer.resume_with("w");
    }

    fn select_other_charset(&self, input: &str) {}

    fn escape_dispatch(&self, escape_command: &str) {
        let ris_code = &RIS.to_string();
        let ind_code = &IND.to_string();
        match escape_command {
            ec if ec == ris_code => {
                self.listener.reset();
            }
            ec if ec == ind_code => {
                self.listener.index();
            }
            _ => {
                println!("un expected escape code")
            }
        }
    }
}
