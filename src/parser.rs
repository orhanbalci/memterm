use ansi_control_codes::c0::ESC;
use ansi_control_codes::c1::{CSI, HTS, NEL, OSC, RI};
use ansi_control_codes::independent_control_functions::RIS;
use genawaiter::sync::gen;
use genawaiter::yield_;

use crate::ascii;
use crate::parser_listener::ParserListener;

pub const DECALN: &'static str = ascii!(3 / 8);
pub const IND: &'static str = ascii!(4 / 4);
pub const DECSC: &'static str = ascii!(3 / 7);
pub const DECRC: &'static str = ascii!(3 / 8);

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
        let nel_code = &NEL.to_string();
        let ri_code = &RI.to_string();
        let hts_code = &HTS.to_string();

        match escape_command {
            ec if ec == ris_code => {
                self.listener.reset();
            }
            ec if ec == ind_code => {
                self.listener.index();
            }
            ec if ec == nel_code => {
                self.listener.linefeed();
            }
            ec if ec == ri_code => {
                self.listener.reverse_index();
            }
            ec if ec == hts_code => {
                self.listener.set_tab_stop();
            }
            ec if ec == DECSC => {
                self.listener.save_cursor();
            }
            ec if ec == DECRC => {
                self.listener.restore_cursor();
            }
            _ => {
                println!("un expected escape code")
            }
        }
    }
}
