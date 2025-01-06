// Ensures that `pub` means published in the public API.
// This property is useful for reasoning about breaking API changes.
#![deny(unreachable_pub)]

//! ![memtermlogo](https://github.com/orhanbalci/memterm/blob/main/assets/memterm.png?raw=true)
//!
//! **memterm** is a Rust virtual terminal emulator, offering a lightweight and efficient implementation for handling ANSI escape sequences and emulating terminal behavior. Inspired by the Python library [pyte](https://github.com/selectel/pyte), it provides a robust and customizable terminal interface for your Rust applications.
//!
//! ## Features
//!
//! - **ANSI Escape Sequence Support:** Handles a wide range of ANSI escape codes for terminal emulation.
//! - **Screen Buffer Abstraction:** Maintains an emulated terminal screen for rendering and manipulation.
//! - **Customizable Dimensions:** Flexible terminal width and height configuration.
//! - **Performance Focused:** Designed for high efficiency in terminal operations.
//! - **Easy-to-Use API:** Clean and idiomatic interface for seamless integration.
//!
//! ## Getting Started
//!
//! ### 📦 Installation
//!
//! Add **memterm** to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! memterm = "0.1"
//! ```
//!
//! Then, run:
//!
//! ```sh
//! cargo build
//! ```
//!
//! ### 🔭Example Usage
//!
//! ```rust ignore
//! #[test]
//! fn draw() {
//!     // DECAWM on (default)
//!     let mut screen = Screen::new(3, 3);
//!     screen.set_mode(&[LNM], false);
//!     assert!(screen.mode.contains(&DECAWM));
//!
//!     for ch in "abc".chars() {
//!         screen.draw(&ch.to_string());
//!     }
//!
//!     assert_eq!(
//!         screen.display(),
//!         vec!["abc".to_string(), "   ".to_string(), "   ".to_string()]
//!     );
//!     assert_eq!((screen.cursor.y, screen.cursor.x), (0, 3));
//!
//!     // One more character -- now we got a linefeed!
//!     screen.draw("a");
//!     assert_eq!((screen.cursor.y, screen.cursor.x), (1, 1));
//!
//!     // DECAWM is off
//!     let mut screen = Screen::new(3, 3);
//!     screen.reset_mode(&[DECAWM], false);
//!
//!     for ch in "abc".chars() {
//!         screen.draw(&ch.to_string());
//!     }
//!
//!     assert_eq!(
//!         screen.display(),
//!         vec!["abc".to_string(), "   ".to_string(), "   ".to_string()]
//!     );
//!     assert_eq!((screen.cursor.y, screen.cursor.x), (0, 3));
//!
//!     // No linefeed is issued on the end of the line ...
//!     screen.draw("a");
//!     assert_eq!(
//!         screen.display(),
//!         vec!["aba".to_string(), "   ".to_string(), "   ".to_string()]
//!     );
//!     assert_eq!((screen.cursor.y, screen.cursor.x), (0, 3));
//!
//!     // IRM mode is on, expecting new characters to move the old ones
//!     // instead of replacing them
//!     screen.set_mode(&[IRM], false);
//!     screen.cursor_position(None, None);
//!     screen.draw("x");
//!     assert_eq!(
//!         screen.display(),
//!         vec!["xab".to_string(), "   ".to_string(), "   ".to_string()]
//!     );
//!
//!     screen.cursor_position(None, None);
//!     screen.draw("y");
//!     assert_eq!(
//!         screen.display(),
//!         vec!["yxa".to_string(), "   ".to_string(), "   ".to_string()]
//!     );
//! }
//! ```
//!
//! ### 🧩 Core Features
//!
//! 1. **Escape Sequence Parsing**
//!    Automatically interprets and applies ANSI escape sequences for text formatting, cursor movements, and more.
//!
//! 2. **Screen Buffer Access**
//!    Provides direct access to the virtual screen buffer for introspection or manipulation.
//!
//! 3. **Terminal State Management**
//!    Offers APIs to adjust dimensions and reset or inspect the terminal state.

macro_rules! ascii {
    ($($xx:literal/$yy:literal), *) => {
        unsafe { std::str::from_utf8_unchecked(&[$(($xx << 4) + $yy),*]) }
    };
}

pub(crate) use ascii;
pub mod byte_parser;
pub mod charset;
pub mod control;
pub mod counter;
pub mod debug_screen;
pub mod graphics;
pub mod modes;
pub mod parser;
pub mod parser_listener;
pub mod screen;
