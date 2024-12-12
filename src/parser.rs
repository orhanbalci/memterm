#![allow(clippy::cmp_owned)]

use std::sync::{Arc, Mutex};

use generator::{Generator, Gn};

use crate::control::*;
use crate::parser_listener::ParserListener;

pub struct ParserState {
    use_utf8: bool,
}
pub struct Parser<'a, T>
where
    T: ParserListener + Send + 'a,
{
    parser_fsm: Generator<'a, String, Option<bool>>,
    _parser_state: Arc<Mutex<ParserState>>,
    taking_plain_text: bool,
    listener: Arc<Mutex<T>>,
}

impl<'a, T> Parser<'a, T>
where
    T: ParserListener + Send + 'a,
{
    pub fn new(listener: Arc<Mutex<T>>) -> Self {
        let parser_state = Arc::new(Mutex::new(ParserState { use_utf8: true }));
        let parser_state_cloned = parser_state.clone();
        return Self {
            listener: listener.clone(),
            taking_plain_text: true,
            parser_fsm: Gn::<String>::new_scoped(move |mut co| {
                loop {
                    let mut char = co.yield_(Some(true)).unwrap_or_default();
                    if ESC == char {
                        char = co.yield_(None).unwrap_or_default();
                        if char == "[" {
                            char = CSI.to_owned();
                        } else if char == "]" {
                            char = OSC.to_owned();
                        } else {
                            if char == "#" {
                                if co.yield_(None).unwrap_or_default() == DECALN {
                                    listener.lock().unwrap().alignment_display();
                                } else {
                                    println!("unexpected escape character");
                                }
                            } else if char == "%" {
                                // self.select_other_charset(yield_!(None));
                            } else if "()".contains(&char) {
                                let _code = co.yield_(None);
                                if parser_state_cloned.lock().unwrap().use_utf8 {
                                    continue;
                                } else {
                                    // listener.lock().unwrap().define_charset(code, char);
                                }
                            } else {
                                listener.lock().unwrap().escape_dispatch(&char);
                            }
                            continue;
                        }
                    }
                    if BASIC.iter().any(|cf| *cf == char) {
                        if char == SI || char == SO {
                            continue;
                        } else {
                            listener.lock().unwrap().basic_dispatch(&char);
                        }
                    } else if char == CSI {
                        let mut params: Vec<u32> = vec![];
                        let mut private: bool = false;
                        let mut current: String = "".to_owned();
                        loop {
                            char = co.yield_(None).unwrap_or_default();
                            if char == "?" {
                                private = true;
                            } else if ALLOWED_IN_CSI.iter().any(|cf| *cf == char) {
                                listener.lock().unwrap().basic_dispatch(&char);
                            } else if char == SP || char == GREATER {
                            } else if char == CAN || char == SUB {
                                listener.lock().unwrap().draw(&char);
                                break;
                            } else if char.chars().next().unwrap().is_ascii_digit() {
                                current.push(char.chars().next().unwrap());
                            } else if char == "$" {
                                co.yield_(None);
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
                                        listener.lock().unwrap().csi_dispatch(
                                            &char,
                                            &params[..],
                                            true,
                                        );
                                    } else {
                                        listener.lock().unwrap().csi_dispatch(
                                            &char,
                                            &params[..],
                                            false,
                                        );
                                    }
                                    break;
                                }
                            }
                        }
                    } else if char == OSC {
                        let code = co.yield_(None).unwrap_or_default();
                        if code == "R" || code == "p" {
                            continue; // reset palette not implemented
                        }
                        let _param = "".to_owned();

                        loop {
                            let mut accu = co.yield_(None).unwrap_or_default();
                            if accu == ESC {
                                accu.push_str(&co.yield_(None).unwrap_or_default());
                            }
                        }
                    }
                }
            }),
            _parser_state: parser_state,
        };
    }

    // pub fn feed(&mut self, input: String) {
    //     input.chars().for_each(|c| {
    //         self.parser_fsm.send(c.to_string());
    //     });
    // }

    pub fn feed(&mut self, data: String) {
        let length = data.len();
        let mut offset = 0;

        while offset < length {
            if self.taking_plain_text {
                if let Some(mat) = TEXT_PATTERN.find_at(&data, offset) {
                    let (start, end) = (mat.start(), mat.end());
                    let text = &data[start..end];
                    text.chars().for_each(|c| {
                        self.listener.lock().unwrap().draw(&c.to_string());
                    });
                    offset = end;
                } else {
                    self.taking_plain_text = false;
                }
            } else {
                let next_char = &data[offset..offset + 1];
                self.taking_plain_text =
                    self.parser_fsm.send(next_char.to_string()).unwrap_or(false);
                offset += 1;
            }
        }
    }
}

// fn select_other_charset(&self, input: &str) {}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use super::{Parser, ESC, RIS};
    use crate::debug_screen::DebugScreen;

    #[test]
    fn first_step() {
        let listener = Arc::new(Mutex::new(DebugScreen {}));
        let mut parser = Parser::new(listener.clone());
        parser.feed(String::default());
        parser.feed(ESC.to_owned());
        parser.feed(RIS.to_owned());
    }
}
