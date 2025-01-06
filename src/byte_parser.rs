use std::sync::{Arc, Mutex};

use encoding_rs::UTF_8;

use crate::parser::Parser;
use crate::parser_listener::ParserListener;

pub struct ByteParser<'a, T>
where
    T: ParserListener + Send + 'a,
{
    parser: Parser<'a, T>,
    utf8_decoder: &'static encoding_rs::Encoding,
    incomplete: Vec<u8>, // Only need to store incomplete UTF-8 sequences
}

impl<'a, T> ByteParser<'a, T>
where
    T: ParserListener + Send + 'a,
{
    pub fn new(listener: Arc<Mutex<T>>) -> Self {
        Self {
            parser: Parser::new(listener),
            utf8_decoder: UTF_8,
            incomplete: Vec::new(),
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        let use_utf8 = self.parser.parser_state.lock().unwrap().use_utf8;

        let data_str = if use_utf8 {
            // Handle incomplete UTF-8 sequences from previous feed
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&self.incomplete);
            bytes.extend_from_slice(data);

            // Decode UTF-8 with replacement characters for invalid sequences
            let (cow, _had_errors) = self.utf8_decoder.decode_with_bom_removal(&bytes);

            // Store any incomplete UTF-8 sequence for next time
            if let Some(last_valid) = cow.len().checked_sub(1) {
                self.incomplete = bytes[last_valid..].to_vec();
            } else {
                self.incomplete.clear();
            }

            cow.into_owned()
        } else {
            // Convert bytes directly to chars when not using UTF-8
            data.iter().map(|&b| b as char).collect::<String>()
        };

        self.parser.feed(data_str);
    }

    pub fn select_other_charset(&mut self, code: &str) {
        match code {
            "@" => {
                self.parser.set_use_utf8(false);
                self.incomplete.clear();
            }
            "G" | "8" => {
                self.parser.set_use_utf8(true);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use pretty_assertions::assert_eq;

    use crate::byte_parser::ByteParser;
    use crate::debug_screen::DebugScreen;
    use crate::parser_listener::ParserListener;
    use crate::screen::Screen;
    #[test]
    fn input_output() {
        // List of test cases
        let test_cases = vec!["cat-gpl3", "find-etc", "htop", "ls", "mc", "top", "vi"];

        // Get path to captured directory relative to this test file
        let captured_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("captured");

        for name in test_cases {
            // Read input file
            let input_path = captured_dir.join(format!("{}.input", name));
            let input = fs::read(&input_path)
                .unwrap_or_else(|_| panic!("Failed to read input file: {:?}", input_path));

            // Read expected output file
            let output_path = captured_dir.join(format!("{}.output", name));
            let output: Vec<String> = serde_json::from_str(
                &fs::read_to_string(&output_path)
                    .unwrap_or_else(|_| panic!("Failed to read output file: {:?}", output_path)),
            )
            .unwrap_or_else(|_| panic!("Failed to parse output JSON for {}", name));

            // Create screen and parser
            let screen = Arc::new(Mutex::new(Screen::new(80, 24)));
            let mut parser = ByteParser::new(screen.clone());

            // Feed input
            parser.feed(&input);

            //Compare display output with expected output
            dbg!(screen.lock().unwrap().display());
            dbg!(output.clone());

            assert_eq!(
                screen.lock().unwrap().display(),
                output,
                "Output mismatch for test case: {}",
                name
            );
        }
    }

    #[test]
    #[ignore = "this is a utility test to debug intermediate commands"]
    fn debug_printer() {
        // List of test cases
        let test_cases = vec!["htop"];

        // Get path to captured directory relative to this test file
        let captured_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("captured");

        for name in test_cases {
            // Read input file
            let input_path = captured_dir.join(format!("{}.input", name));
            let input = fs::read(&input_path)
                .unwrap_or_else(|_| panic!("Failed to read input file: {:?}", input_path));

            // Create debug screen and parser
            let debug_screen = Arc::new(Mutex::new(DebugScreen::new()));
            let mut parser = ByteParser::new(debug_screen.clone());

            // Feed input
            parser.feed(&input);

            // Write recorded output to mc.debug file
            let debug_output = debug_screen.lock().unwrap().output.join("\n");
            fs::write("htop.memterm", debug_output).expect("Failed to write debug output to file");
        }
    }
}
