use std::collections::HashMap;

/// This module defines graphic-related constants, mostly taken from
/// :manpage:`console_codes(4)` and
/// http://pueblo.sourceforge.net/doc/manual/ansi_color_codes.html.
use lazy_static::lazy_static;

lazy_static! {
    /// A mapping of ANSI text style codes to style names, "+" means the:
    /// attribute is set, "-" -- reset; example:
    ///
    /// ```
    /// assert_eq!(TEXT.get(&1), Some(&"+bold".to_string()));
    /// assert_eq!(TEXT.get(&9), Some(&"+strikethrough".to_string()));
    /// ```
    pub static ref TEXT: HashMap<u32, String> = {
        let mut m = HashMap::new();
        m.insert(1, "+bold".to_string());
        m.insert(3, "+italics".to_string());
        m.insert(4, "+underscore".to_string());
        m.insert(5, "+blink".to_string());
        m.insert(7, "+reverse".to_string());
        m.insert(9, "+strikethrough".to_string());
        m.insert(22, "-bold".to_string());
        m.insert(23, "-italics".to_string());
        m.insert(24, "-underscore".to_string());
        m.insert(25, "-blink".to_string());
        m.insert(27, "-reverse".to_string());
        m.insert(29, "-strikethrough".to_string());
        m
    };
}

lazy_static! {
    /// A mapping of ANSI foreground color codes to color names.
    ///
    /// ```
    /// assert_eq!(FG_ANSI.get(&30), Some(&"black".to_string()));
    /// assert_eq!(FG_ANSI.get(&38), Some(&"default".to_string()));
    /// ```
    pub static ref FG_ANSI: HashMap<u32, String> = {
        let mut m = HashMap::new();
        m.insert(30, "black".to_string());
        m.insert(31, "red".to_string());
        m.insert(32, "green".to_string());
        m.insert(33, "brown".to_string());
        m.insert(34, "blue".to_string());
        m.insert(35, "magenta".to_string());
        m.insert(36, "cyan".to_string());
        m.insert(37, "white".to_string());
        m.insert(39, "default".to_string()); // white.
        m
    };
    /// An alias to `FG_ANSI` for compatibility.
    pub static ref FG: &'static HashMap<u32, String> = &FG_ANSI;
}

lazy_static! {
    /// A mapping of non-standard `aixterm` foreground color codes to
    /// color names. These are high intensity colors.
    pub static ref FG_AIXTERM: HashMap<u32, String> = {
        let mut m = HashMap::new();
        m.insert(90, "brightblack".to_string());
        m.insert(91, "brightred".to_string());
        m.insert(92, "brightgreen".to_string());
        m.insert(93, "brightbrown".to_string());
        m.insert(94, "brightblue".to_string());
        m.insert(95, "brightmagenta".to_string());
        m.insert(96, "brightcyan".to_string());
        m.insert(97, "brightwhite".to_string());
        m
    };
}

lazy_static! {
    pub static ref BG_ANSI: HashMap<u32, String> = {
        let mut m = HashMap::new();
        m.insert(40, "black".to_string());
        m.insert(41, "red".to_string());
        m.insert(42, "green".to_string());
        m.insert(43, "brown".to_string());
        m.insert(44, "blue".to_string());
        m.insert(45, "magenta".to_string());
        m.insert(46, "cyan".to_string());
        m.insert(47, "white".to_string());
        m.insert(49, "default".to_string()); // black.
        m
    };

    pub static ref BG: &'static HashMap<u32, String> = &BG_ANSI;
}

lazy_static! {
    pub static ref BG_AIXTERM: HashMap<u32, String> = {
        let mut m = HashMap::new();
        m.insert(100, "brightblack".to_string());
        m.insert(101, "brightred".to_string());
        m.insert(102, "brightgreen".to_string());
        m.insert(103, "brightbrown".to_string());
        m.insert(104, "brightblue".to_string());
        m.insert(105, "brightmagenta".to_string());
        m.insert(106, "brightcyan".to_string());
        m.insert(107, "brightwhite".to_string());
        m
    };
}

/// SGR code for foreground in 256 or True color mode.
pub const FG_256: u32 = 38;

/// SGR code for background in 256 or True color mode.
pub const BG_256: u32 = 48;

lazy_static! {
    pub static ref FG_BG_256: Vec<String> = {
        let mut fg_bg_256 = vec![
            (0x00, 0x00, 0x00),  // 0
            (0xcd, 0x00, 0x00),  // 1
            (0x00, 0xcd, 0x00),  // 2
            (0xcd, 0xcd, 0x00),  // 3
            (0x00, 0x00, 0xee),  // 4
            (0xcd, 0x00, 0xcd),  // 5
            (0x00, 0xcd, 0xcd),  // 6
            (0xe5, 0xe5, 0xe5),  // 7
            (0x7f, 0x7f, 0x7f),  // 8
            (0xff, 0x00, 0x00),  // 9
            (0x00, 0xff, 0x00),  // 10
            (0xff, 0xff, 0x00),  // 11
            (0x5c, 0x5c, 0xff),  // 12
            (0xff, 0x00, 0xff),  // 13
            (0x00, 0xff, 0xff),  // 14
            (0xff, 0xff, 0xff),  // 15
        ];

        // colors 16..231: the 6x6x6 color cube
        let valuerange = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];
        for i in 0..216 {
            let r = valuerange[(i / 36) % 6];
            let g = valuerange[(i / 6) % 6];
            let b = valuerange[i % 6];
            fg_bg_256.push((r, g, b));
        }

        // colors 232..255: grayscale
        for i in 0..24 {
            let v = 8 + i * 10;
            fg_bg_256.push((v, v, v));
        }

        fg_bg_256.iter()
            .map(|&(r, g, b)| format!("{:02x}{:02x}{:02x}", r, g, b))
            .collect()
    };
}
