#![allow(clippy::cmp_owned)]

#[cfg(test)]
use std::fs::OpenOptions;
#[cfg(test)]
use std::io::Write;
use std::sync::{Arc, Mutex};

use generator::{Generator, Gn};

use crate::control::*;
use crate::parser_listener::ParserListener;

pub struct ParserState {
    pub(crate) use_utf8: bool,
}
pub struct Parser<'a, T>
where
    T: ParserListener + Send + 'a,
{
    parser_fsm: Generator<'a, String, Option<bool>>,
    pub(crate) parser_state: Arc<Mutex<ParserState>>,
    pub(crate) taking_plain_text: bool,
    listener: Arc<Mutex<T>>,
    #[cfg(test)]
    log_file: Arc<Mutex<std::fs::File>>, // Add file handle
}

impl<'a, T> Parser<'a, T>
where
    T: ParserListener + Send + 'a,
{
    pub fn new(listener: Arc<Mutex<T>>) -> Self {
        // Open file in append mode, create if doesn't exist
        #[cfg(test)]
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("parser_log.txt")
            .expect("Failed to open log file");

        #[cfg(test)]
        let log_file = Arc::new(Mutex::new(file));

        let parser_state = Arc::new(Mutex::new(ParserState { use_utf8: true }));
        let parser_state_cloned = parser_state.clone();

        #[cfg(not(test))]
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
                        if (char == SI || char == SO)
                            && parser_state_cloned.lock().unwrap().use_utf8
                        {
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
                                let mut current_param = match current.parse::<u64>() {
                                    Ok(val) => val,
                                    _ => 0,
                                };
                                current_param = u64::min(current_param, 9999);
                                params.push(current_param as u32);
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
                        let mut param = "".to_owned();

                        'param_loop: loop {
                            let mut accu = co.yield_(None).unwrap_or_default();
                            if accu == ESC {
                                accu.push_str(&co.yield_(None).unwrap_or_default());
                            }

                            if OSC_TERMINATORS.contains(&accu.as_str()) {
                                break 'param_loop;
                            } else {
                                param.push(accu.chars().next().unwrap());
                            }
                        }

                        param = param.chars().skip(1).take(param.len() - 1).collect();

                        if "01".contains(&code) {
                            listener.lock().unwrap().set_icon_name(&param);
                        }
                        if "02".contains(&code) {
                            listener.lock().unwrap().set_title(&param);
                        }
                    }
                }
            }),
            parser_state,
        };

        #[cfg(test)]
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
                        if (char == SI || char == SO)
                            && parser_state_cloned.lock().unwrap().use_utf8
                        {
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
                                let mut current_param = match current.parse::<u64>() {
                                    Ok(val) => val,
                                    _ => 0,
                                };
                                current_param = u64::min(current_param, 9999);
                                params.push(current_param as u32);
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
                        let mut param = "".to_owned();

                        'param_loop: loop {
                            let mut accu = co.yield_(None).unwrap_or_default();
                            if accu == ESC {
                                accu.push_str(&co.yield_(None).unwrap_or_default());
                            }

                            if OSC_TERMINATORS.contains(&accu.as_str()) {
                                break 'param_loop;
                            } else {
                                param.push(accu.chars().next().unwrap());
                            }
                        }

                        param = param.chars().skip(1).take(param.len() - 1).collect();

                        if "01".contains(&code) {
                            listener.lock().unwrap().set_icon_name(&param);
                        }
                        if "02".contains(&code) {
                            listener.lock().unwrap().set_title(&param);
                        }
                    }
                }
            }),
            parser_state,
            log_file: log_file, // Add file handle to struct
        };

        a.parser_fsm.send("".to_owned());
        a
    }

    pub fn is_special_start(s: &str) -> bool {
        SPECIAL.iter().any(|special| s.starts_with(special))
    }

    // New method for writing to log file
    #[cfg(test)]
    fn _log_screen_state(&self, screen_state: String) {
        if let Ok(mut file) = self.log_file.lock() {
            let log_entry = format!("{}\n{}\n{}\n", "#".repeat(25), screen_state, "#".repeat(25));

            if let Err(e) = writeln!(file, "{}", log_entry) {
                eprintln!("Failed to write to log file: {}", e);
            }

            if let Err(e) = file.flush() {
                eprintln!("Failed to flush log file: {}", e);
            }
        } else {
            eprintln!("Failed to acquire lock on log file");
        }
    }

    pub fn feed(&mut self, data: String) {
        for c in data.chars() {
            let char_str = c.to_string();

            // If we're in plain text mode and this is a special character
            if self.taking_plain_text && Self::is_special_start(&char_str) {
                self.taking_plain_text = false;
            }

            if self.taking_plain_text {
                // Feed plain text directly to listener
                self.listener.lock().unwrap().draw(&char_str);
                // Log the screen state using the new method
                // if let Ok(display) = self.listener.lock().map(|mut l| l.display()) {
                //     self.log_screen_state(display.join("\n"));
                // }
            } else {
                // Feed to parser FSM and update taking_plain_text state
                self.taking_plain_text = self.parser_fsm.send(char_str).unwrap_or(false);
            }
        }
    }

    pub fn set_use_utf8(&mut self, use_utf8: bool) {
        self.parser_state.lock().unwrap().use_utf8 = use_utf8;
    }
}

// fn select_other_charset(&self, input: &str) {}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use super::{Parser, CSI_COMMANDS, DECRC, DECSC, ESC, HTS, IND, NEL, OSC, RI, RIS, ST, ST_C0};
    use crate::counter::Counter;
    use crate::debug_screen::DebugScreen;
    use crate::parser::{CSI, FF, HVP, LF, SI, SO, VT};
    use crate::parser_listener::ParserListener;
    use crate::screen::Screen;

    #[test]
    fn first_step() {
        let listener = Arc::new(Mutex::new(DebugScreen::new()));
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

            assert_eq!(
                counter.lock().unwrap().get_count(event),
                1,
                "Handler for {} should be called exactly once",
                event
            );

            {
                if let Some(params) = counter.lock().unwrap().get_last_params(event) {
                    assert_eq!(
                        params,
                        &vec![5],
                        "Handler for {} should receive [5] as parameters",
                        event
                    );
                }
            }

            // b) Test multiple parameters with CSI
            let counter = Arc::new(Mutex::new(Counter::new()));
            let mut parser = Parser::new(counter.clone());

            // Feed CSI 5;12 cmd
            parser.feed(format!("{}5;12{}", CSI, cmd));

            assert_eq!(
                counter.lock().unwrap().get_count(event),
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
    fn set_mode() {
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
    fn reset_mode() {
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

    #[test]
    fn missing_params() {
        let counter = Arc::new(Mutex::new(Counter::new()));
        let mut parser = Parser::new(counter.clone());

        // Feed CSI sequence with missing parameter
        parser.feed(format!("{}[;{}", ESC, HVP)); // H is the HVP (Horizontal Vertical Position) command

        let counter_lock = counter.lock().unwrap();

        // Check cursor_position was called once
        assert_eq!(counter_lock.get_count("cursor_position"), 1);

        // Check parameters - should default to (0, 0) when missing
        if let Some(params) = counter_lock.get_last_params("cursor_position") {
            assert_eq!(*params, vec![0, 0]);
        }
    }

    #[test]
    fn overflow() {
        let counter = Arc::new(Mutex::new(Counter::new()));
        let mut parser = Parser::new(counter.clone());

        // Feed CSI sequence with very large numbers
        parser.feed(format!("{}[999999999999999;99999999999999{}", ESC, HVP));

        let counter_lock = counter.lock().unwrap();

        // Check cursor_position was called once
        assert_eq!(counter_lock.get_count("cursor_position"), 1);

        // Check parameters - should be clamped to 9999
        if let Some(params) = counter_lock.get_last_params("cursor_position") {
            assert_eq!(*params, vec![9999, 9999]);
        }
    }

    #[test]
    fn control_characters() {
        let handler = Arc::new(Mutex::new(Counter::new()));

        let mut parser = Parser::new(handler.clone());

        parser.feed(format!("{}10;\t\t\n\r\n10{}", CSI, HVP));

        assert_eq!(handler.lock().unwrap().get_count("cursor_position"), 1);
        assert_eq!(
            handler.lock().unwrap().get_last_params("cursor_position"),
            Some(&vec![10, 10])
        );
    }

    #[test]
    fn set_title_icon_name() {
        let test_cases = vec![
            (format!("{}{}", ESC, "]"), ST_C0.to_owned()),
            (format!("{}{}", ESC, "]"), ST.to_owned()),
            (OSC.to_owned(), ST_C0.to_owned()),
            (OSC.to_owned(), ST.to_owned()),
        ];

        for (osc, st) in test_cases {
            let screen = Arc::new(Mutex::new(Screen::new(80, 24)));
            let mut parser = Parser::new(screen.clone());

            // // a) set only icon name
            parser.feed(format!("{}1;foo{}", osc, st));
            assert_eq!(screen.lock().unwrap().icon_name, "foo");

            // // b) set only title
            parser.feed(format!("{}2;foo{}", osc, st));
            assert_eq!(screen.lock().unwrap().title, "foo");

            // // c) set both icon name and title
            parser.feed(format!("{}0;bar{}", osc, st));
            assert_eq!(screen.lock().unwrap().title, "bar");
            assert_eq!(screen.lock().unwrap().icon_name, "bar");

            //d) set both icon name and title then terminate with BEL
            parser.feed(format!("{}0;bar{}", osc, st));
            assert_eq!(screen.lock().unwrap().title, "bar");
            assert_eq!(screen.lock().unwrap().icon_name, "bar");

            // e) test ➜ ('\xe2\x9e\x9c') symbol, that contains string terminator \x9c
            parser.feed("➜".to_string());
            assert_eq!(screen.lock().unwrap().buffer[&0][&0].data, "➜");
        }
    }

    #[test]
    fn define_charset() {
        // Should be a noop. All input is UTF8.
        let screen = Arc::new(Mutex::new(Screen::new(3, 3)));
        let mut parser = Parser::new(screen.clone());

        parser.feed(format!("{}(B", ESC)); // ESC ( B sequence

        assert_eq!(screen.lock().unwrap().display()[0], "   ".to_string());
    }

    #[test]
    fn test_non_utf8_shifts() {
        let counter = Arc::new(Mutex::new(Counter::new()));

        // Create parser with screen
        let mut parser = Parser::new(counter.clone());
        parser.set_use_utf8(false);

        // Feed SI (Shift In) and SO (Shift Out) control characters
        parser.feed(SI.to_string()); // SI = "\x0F"
        parser.feed(SO.to_string()); // SO = "\x0E"

        // Get count of shift_in and shift_out calls
        assert_eq!(counter.lock().unwrap().get_count("shift_in"), 1);
        assert_eq!(counter.lock().unwrap().get_count("shift_out"), 1);
    }

    #[test]
    fn test_dollar_skip() {
        let counter = Arc::new(Mutex::new(Counter::new()));
        let mut parser = Parser::new(counter.clone());

        // Feed CSI sequence with dollar commands
        parser.feed(format!("{}12$p", CSI)); // CSI 12 $ p sequence

        // Check that draw wasn't called
        assert_eq!(counter.lock().unwrap().get_count("draw"), 0);

        // Feed another CSI sequence with dollar command
        parser.feed(format!("{}1;2;3;4$x", CSI)); // CSI 1;2;3;4 $ x sequence

        // Check that draw still wasn't called
        assert_eq!(counter.lock().unwrap().get_count("draw"), 0);
    }

    #[test]
    fn escape_like() {
        let counter = Arc::new(Mutex::new(Counter::new()));
        let mut parser = Parser::new(counter.clone());

        // Feed CSI sequence with dollar commands
        parser.feed("[^]".to_owned()); // CSI 12 $ p sequence

        // Check that draw wasn't called
        assert_eq!(counter.lock().unwrap().get_count("draw"), 3);
    }
}
