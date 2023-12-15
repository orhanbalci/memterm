use ansi_control_codes::c0::{BEL, BS, CR, ESC, FF, HT, LF, SI, SO, VT};
use ansi_control_codes::c1::{CSI, HTS, NEL, OSC, RI};
use ansi_control_codes::independent_control_functions::RIS;
use ansi_control_codes::ControlFunction;
use genawaiter::sync::gen;
use genawaiter::yield_;

use crate::ascii;
use crate::parser_listener::ParserListener;

pub const DECALN: &'static str = ascii!(3 / 8);
pub const IND: &'static str = ascii!(4 / 4);
pub const DECSC: &'static str = ascii!(3 / 7);
pub const DECRC: &'static str = ascii!(3 / 8);

pub const BASIC: &'static [ControlFunction; 9] = &[BEL, BS, HT, LF, VT, FF, CR, SO, SI];

pub struct Parser<T: ParserListener> {
    listener: T,
    use_utf8: bool,
}

impl<T: ParserListener> Parser<T> {
    pub fn new(listener: T, use_utf8: bool) -> Self {
        Parser { listener, use_utf8 }
    }

    pub fn start(&self) {
        let csi_code = &CSI.to_string();
        let osc_code = &OSC.to_string();
        let mut printer = ::genawaiter::sync::Gen::new(::genawaiter::sync_producer!({
            loop {
                let mut char: &str = yield_!(Some(true));
                if ESC.to_string() == char {
                    char = yield_!(None);
                    if char == "[" {
                        char = csi_code;
                    } else if char == "]" {
                        char = osc_code;
                    } else {
                        if char == "#" {
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
                            self.escape_dispatch(char);
                        }
                        continue;
                    }
                }
                if BASIC.iter().any(|cf| cf.to_string() == char) {
                    if char == SI.to_string() || char == SO.to_string() {
                        continue;
                    } else {
                        self.basic_dispatch(char);
                    }
                }
            }
        }));

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

    fn basic_dispatch(&self, basic_command: &str) {
        let bel_code = &BEL.to_string();
        let bs_code = &BS.to_string();
        let ht_code = &HT.to_string();
        let lf_code = &LF.to_string();
        let vt_code = &VT.to_string();
        let ff_code = &FF.to_string();
        let cr_code = &CR.to_string();

        match basic_command {
            ec if ec == bel_code => {
                self.listener.bell();
            }
            ec if ec == bs_code => {
                self.listener.backspace();
            }
            ec if ec == ht_code => {
                self.listener.tab();
            }
            ec if (ec == lf_code || ec == vt_code || ec == ff_code) => {
                self.listener.linefeed();
            }
            ec if ec == cr_code => {
                self.listener.cariage_return();
            }
            _ => {
                println!("un expected escape code")
            }
        }
    }
}
