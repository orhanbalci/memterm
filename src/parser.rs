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
        let mut a = Self {
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
                        println!("basic dispatch");
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
                                        dbg!("csi dispatch");
                                        dbg!(&char, &params[..], false);
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

        a.parser_fsm.send("".to_owned());
        a
    }

    pub fn feed(&mut self, data: String) {
        for c in data.chars() {
            let char_str = c.to_string();

            // If we're in plain text mode and this is a special character
            if self.taking_plain_text && SPECIAL.contains(&char_str.as_str()) {
                // dbg!(char_str.clone());
                self.taking_plain_text = false;
            }

            if self.taking_plain_text {
                // Feed plain text directly to listener
                self.listener.lock().unwrap().draw(&char_str);
            } else {
                // Feed to parser FSM and update taking_plain_text state
                self.taking_plain_text = self.parser_fsm.send(char_str).unwrap_or(false);
            }
        }
    }
}

// fn select_other_charset(&self, input: &str) {}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use super::{Parser, CSI_COMMANDS, DECRC, DECSC, ESC, HTS, IND, NEL, RI, RIS};
    use crate::counter::Counter;
    use crate::debug_screen::DebugScreen;
    use crate::parser::{CSI, FF, LF, VT};

    #[test]
    fn first_step() {
        let listener = Arc::new(Mutex::new(DebugScreen {}));
        let mut parser = Parser::new(listener.clone());
        parser.feed(String::default());
        parser.feed(ESC.to_owned());
        parser.feed(RIS.to_owned());
    }

    #[test]
    fn basic_sequences() {
        // Map of escape sequences to their handler names
        let escape_map = vec![
            (RIS, "reset"),
            (IND, "index"),
            (NEL, "linefeed"),
            (RI, "reverse_index"),
            (HTS, "set_tab_stop"),
            (DECSC, "save_cursor"),
            (DECRC, "restore_cursor"),
        ];

        for (cmd, event) in escape_map {
            let counter = Arc::new(Mutex::new(Counter::new()));
            let mut parser = Parser::new(counter.clone());

            // First feed ESC
            parser.feed(ESC.to_string());
            assert_eq!(
                counter.lock().unwrap().get_count(event),
                0,
                "Handler {} was called before command",
                event
            );

            // Then feed the command
            parser.feed(cmd.to_string());
            assert_eq!(
                counter.lock().unwrap().get_count(event),
                1,
                "Handler {} was not called exactly once",
                event
            );

            // Verify no other handlers were called
            for (name, count) in counter.lock().unwrap().counts.iter() {
                if name != &event {
                    assert_eq!(
                        *count, 0,
                        "Unexpected handler {} was called {} times",
                        name, count
                    );
                }
            }
        }
    }

    #[test]
    fn linefeed() {
        // Create a counter to track linefeed calls
        let counter = Arc::new(Mutex::new(Counter::new()));
        let mut parser = Parser::new(counter.clone());

        // Feed LF (Line Feed), VT (Vertical Tab), and FF (Form Feed)
        parser.feed(format!("{}{}{}", LF, VT, FF));

        // Check that linefeed was called exactly 3 times
        assert_eq!(
            counter.lock().unwrap().get_count("linefeed"),
            3,
            "Linefeed should have been called exactly 3 times"
        );
    }

    #[test]
    fn non_csi_sequences() {
        for (cmd, event) in CSI_COMMANDS.iter() {
            // a) Test single parameter
            let counter = Arc::new(Mutex::new(Counter::new()));
            let mut parser = Parser::new(counter.clone());

            // Feed ESC [ 5 cmd
            parser.feed(format!("{}[5{}", ESC, cmd));
            dbg!(event);

            let counter_lock = counter.lock().unwrap();
            assert_eq!(
                counter_lock.get_count(event),
                1,
                "Handler for {} should be called exactly once",
                event
            );

            if let Some(params) = counter_lock.get_last_params(event) {
                assert_eq!(
                    params,
                    &vec![5],
                    "Handler for {} should receive [5] as parameters",
                    event
                );
            }

            // b) Test multiple parameters with CSI
            let counter = Arc::new(Mutex::new(Counter::new()));
            let mut parser = Parser::new(counter.clone());

            // Feed CSI 5;12 cmd
            parser.feed(format!("{}5;12{}", CSI, cmd));

            let counter_lock = counter.lock().unwrap();
            assert_eq!(
                counter_lock.get_count(event),
                1,
                "Handler for {} should be called exactly once",
                event
            );

            // if let Some(params) = counter_lock.get_last_params(event) {
            //     assert_eq!(
            //         params,
            //         &vec![5, 12],
            //         "Handler for {} should receive [5, 12] as parameters",
            //         event
            //     );
            // }
        }
    }

    #[test]
    fn test_set_mode() {
        let counter = Arc::new(Mutex::new(Counter::new()));
        let mut parser = Parser::new(counter.clone());

        parser.feed(format!("{}[?9;2h", ESC)); // Using CSI sequence to set modes

        let counter_lock = counter.lock().unwrap();

        // Check set_mode was called with correct arguments
        assert_eq!(counter_lock.get_count("set_mode"), 1);

        // Check last parameters passed to set_mode
        if let Some(params) = counter_lock.get_last_params("set_mode") {
            assert_eq!(*params, vec![9, 2]);
            assert!(counter_lock.get_last_private().unwrap());
        }
    }

    #[test]
    fn test_reset_mode() {
        let counter = Arc::new(Mutex::new(Counter::new()));
        let mut parser = Parser::new(counter.clone());

        parser.feed(format!("{}[?9;2l", ESC)); // Using CSI sequence to reset modes

        let counter_lock = counter.lock().unwrap();

        // Check reset_mode was called with correct arguments
        assert_eq!(counter_lock.get_count("reset_mode"), 1);

        // Check parameters passed to reset_mode
        if let Some(params) = counter_lock.get_last_params("reset_mode") {
            assert_eq!(*params, vec![9, 2]);
            assert!(counter_lock.get_last_private().unwrap());
        }
    }
}
