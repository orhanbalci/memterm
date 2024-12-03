
![memtermlogo](https://github.com/orhanbalci/memterm/blob/main/assets/memterm.png?raw=true)

**memterm** is a Rust virtual terminal emulator, offering a lightweight and efficient implementation for handling ANSI escape sequences and emulating terminal behavior. Inspired by the Python library [pyte](https://github.com/selectel/pyte), it provides a robust and customizable terminal interface for your Rust applications.

## Features

- **ANSI Escape Sequence Support:** Handles a wide range of ANSI escape codes for terminal emulation.
- **Screen Buffer Abstraction:** Maintains an emulated terminal screen for rendering and manipulation.
- **Customizable Dimensions:** Flexible terminal width and height configuration.
- **Performance Focused:** Designed for high efficiency in terminal operations.
- **Easy-to-Use API:** Clean and idiomatic interface for seamless integration.

## Getting Started

### Installation

Add **memterm** to your `Cargo.toml` file:

```toml
[dependencies]
memterm = "0.1"
```

Then, run:

```sh
cargo build
```

### Example Usage

```rust
use std::sync::{Arc, Mutex};

use super::{Parser, ESC, RIS};
use crate::parser_printer::ParserPrinter;

#[test]
fn first_step() {
    let listener = Arc::new(Mutex::new(ParserPrinter {}));
    let mut parser = Parser::new(listener.clone());
    parser.feed(String::default());
    parser.feed(ESC.to_owned());
    parser.feed(RIS.to_owned());
}
```

### Core Features

1. **Escape Sequence Parsing**
   Automatically interprets and applies ANSI escape sequences for text formatting, cursor movements, and more.

2. **Screen Buffer Access**
   Provides direct access to the virtual screen buffer for introspection or manipulation.

3. **Terminal State Management**
   Offers APIs to adjust dimensions and reset or inspect the terminal state.

## Documentation

Detailed documentation is available on [docs.rs](https://docs.rs/memterm).
To generate local documentation:

```sh
cargo doc --open
```

## Contributing

Contributions are encouraged! You can:

- Report bugs and request features via [issues](https://github.com/orhanbalci/memterm/issues).
- Submit pull requests to enhance the library.

### Development Setup

1. Clone the repository:
   ```sh
   git clone https://github.com/orhanbalci/memterm.git
   cd memterm
   ```

2. Build the crate:
   ```sh
   cargo build
   ```

3. Run tests:
   ```sh
   cargo test
   ```

## License

**memterm** is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.

## Acknowledgments

**memterm** draws inspiration from the Python library [pyte](https://github.com/selectel/pyte) and aims to bring similar functionality to the Rust ecosystem.

---

Developed with ❤️ by [orhanbalci](https://github.com/orhanbalci)
