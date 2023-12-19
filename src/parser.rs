use ansi_control_codes::c0::{BEL, BS, CAN, CR, ESC, FF, HT, LF, SI, SO, SUB, VT};
use ansi_control_codes::c1::{CSI, HTS, NEL, OSC, RI}; // TODO direct string comparison for this codes does not work. You should match whole code viwth ESC
use ansi_control_codes::control_sequences::ICH; // TODO direct string comparison does not work
use ansi_control_codes::independent_control_functions::RIS; //TODO direct string comparison does not work
use ansi_control_codes::ControlFunction;
use genawaiter::yield_;

use crate::ascii;
use crate::parser_listener::ParserListener;

pub const DECALN: &str = ascii!(3 / 8);
pub const IND: &str = ascii!(4 / 4);
pub const DECSC: &str = ascii!(3 / 7);
pub const DECRC: &str = ascii!(3 / 8);
pub const SP: &str = ascii!(2 / 0);
pub const GREATER: &str = ascii!(3 / 14);

pub const BASIC: &[ControlFunction; 9] = &[BEL, BS, HT, LF, VT, FF, CR, SO, SI];
pub const ALLOWED_IN_CSI: &[ControlFunction; 7] = &[BEL, BS, HT, LF, VT, FF, CR];

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
                if BASIC.iter().any(|cf| *cf == char) {
                    if char == SI.to_string() || char == SO.to_string() {
                        continue;
                    } else {
                        self.basic_dispatch(char);
                    }
                } else if char == CSI.to_string() {
                    let mut params: Vec<u32> = vec![];
                    let mut private: bool = false;
                    let mut current: String = "".to_owned();
                    loop {
                        char = yield_!(None);
                        if char == "?" {
                            private = true;
                        } else if ALLOWED_IN_CSI.iter().any(|cf| *cf == char) {
                            self.basic_dispatch(char);
                        } else if char == SP.to_string() || char == GREATER.to_string() {
                        } else if char == CAN.to_string() || char == SUB.to_string() {
                            self.listener.draw(char);
                            break;
                        } else if char.chars().nth(0).unwrap().is_digit(10) {
                            current.push(char.chars().nth(0).unwrap());
                        } else if (char == "$") {
                            yield_!(None);
                            break;
                        } else {
                            let mut current_param = match current.parse::<u32>() {
                                Ok(val) => val,
                                _ => 0,
                            };
                            current_param = u32::min(current_param, 9999);
                            params.push(current_param);
                            if char == ";" {
                                current = "".to_owned();
                            } else {
                                if private {
                                    self.csi_dispatch(char, &params[..], true);
                                } else {
                                    self.csi_dispatch(char, &params[..], false);
                                }
                                break;
                            }
                        }
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

    fn csi_dispatch(&self, csi_command: &str, params: &[u32], is_private: bool) {
        let ich_code = &ICH(None).to_string(); // TODO fix this code
        match csi_command {
            ec if ec == ich_code => {
                self.listener.insert_characters(if !params.is_empty() {
                    Some(params[0])
                } else {
                    None
                });
            }
            _ => {
                println!("unexpected csi escape code");
            }
        }
    }
}
