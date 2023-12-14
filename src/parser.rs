use ansi_control_codes::c0::ESC;
use ansi_control_codes::c1::CSI;
use ansi_control_codes::c1::OSC;
use genawaiter::sync::gen;
use genawaiter::yield_;

pub struct Parser {}

impl Parser {
    pub fn start() {
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
                        // sharp ispatch
                    }
                }
                // println!("{}", char);
            }
        });

        printer.resume_with("h");
        printer.resume_with("w");
    }
}
