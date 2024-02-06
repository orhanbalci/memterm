#![allow(clippy::cmp_owned)]

use genawaiter::yield_;

use crate::control::*;
use crate::parser_listener::ParserListener;

pub struct Parser<T: ParserListener> {
    listener: T,
    use_utf8: bool,
}

impl<T: ParserListener> Parser<T> {
    pub fn new(listener: T, use_utf8: bool) -> Self {
        Parser { listener, use_utf8 }
    }

    pub fn start(&self) {
        let mut printer = genawaiter::sync::Gen::new(genawaiter::sync_producer!({
            loop {
                let mut char: &str = yield_!(Some(true));
                if ESC == char {
                    char = yield_!(None);
                    if char == "[" {
                        char = CSI;
                    } else if char == "]" {
                        char = OSC;
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
                    if char == SI || char == SO {
                        continue;
                    } else {
                        self.basic_dispatch(char);
                    }
                } else if char == CSI {
                    let mut params: Vec<u32> = vec![];
                    let mut private: bool = false;
                    let mut current: String = "".to_owned();
                    loop {
                        char = yield_!(None);
                        if char == "?" {
                            private = true;
                        } else if ALLOWED_IN_CSI.iter().any(|cf| *cf == char) {
                            self.basic_dispatch(char);
                        } else if char == SP || char == GREATER {
                        } else if char == CAN || char == SUB {
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
                } else if char == OSC {
                    let code = yield_!(None);
                    if code == "R" {
                        continue; // reset palette not implemented
                    } else if code == "p" {
                        continue; // set palette not implemented
                    }

                    let mut param = "".to_owned();

                    loop {
                        let mut accu = String::from(yield_!(None));
                        if accu == ESC {
                            accu.push_str(yield_!(None));
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
        match escape_command {
            ec if ec == RIS => {
                self.listener.reset();
            }
            ec if ec == IND => {
                self.listener.index();
            }
            ec if ec == NEL => {
                self.listener.linefeed();
            }
            ec if ec == RI => {
                self.listener.reverse_index();
            }
            ec if ec == HTS => {
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
        match basic_command {
            ec if ec == BEL => {
                self.listener.bell();
            }
            ec if ec == BS => {
                self.listener.backspace();
            }
            ec if ec == HT => {
                self.listener.tab();
            }
            ec if (ec == LF || ec == VT || ec == FF) => {
                self.listener.linefeed();
            }
            ec if ec == CR => {
                self.listener.cariage_return();
            }
            _ => {
                println!("un expected escape code")
            }
        }
    }

    fn csi_dispatch(&self, csi_command: &str, params: &[u32], is_private: bool) {
        match csi_command {
            ec if ec == ICH => self.listener.insert_characters(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUD => self.listener.cursor_up(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUU => self.listener.cursor_down(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUF => self.listener.cursor_forward(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUB => self.listener.cursor_back(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CNL => self.listener.cursor_down1(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CPL => self.listener.cursor_up1(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CHA => self.listener.cursor_to_column(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUP => {
                if !params.is_empty() {
                    self.listener
                        .cursor_position(Some(params[0]), Some(params[1]));
                } else {
                    self.listener.cursor_position(None, None)
                }
            }
            ec if ec == ED => self.listener.erase_in_display(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == EL => self.listener.erase_in_line(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == IL => self.listener.insert_lines(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == DL => self.listener.delete_lines(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == DCH => self
                .listener
                .delete_characters(params.iter().cloned().next()),
            ec if ec == ECH => self
                .listener
                .erase_characters(params.iter().cloned().next()),
            ec if ec == HPR => self.listener.cursor_forward(params.iter().cloned().next()),
            ec if ec == DA => self
                .listener
                .report_device_attributes(params.iter().cloned().next()),
            ec if ec == VPA => self.listener.cursor_to_line(params.iter().cloned().next()),
            ec if ec == VPR => self.listener.cursor_down(params.iter().cloned().next()),
            ec if ec == HVP => self
                .listener
                .cursor_position(params.iter().cloned().nth(0), params.iter().cloned().nth(1)),
            _ => {
                println!("unexpected csi escape code");
            }
        }
    }
}
