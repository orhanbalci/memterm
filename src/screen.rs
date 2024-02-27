use std::collections::HashSet;

use lazy_static::lazy_static;

use crate::modes::{DECAWM, DECTCEM};

pub struct CharOpts {
    pub data: String,
    pub fg: String,
    pub bg: String,
    pub bold: bool,
    pub italics: bool,
    pub underscore: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub blink: bool,
}

impl Default for CharOpts {
    fn default() -> Self {
        Self {
            data: "".to_owned(),
            fg: "default".to_owned(),
            bg: "default".to_owned(),
            bold: false,
            italics: false,
            underscore: false,
            strikethrough: false,
            reverse: false,
            blink: false,
        }
    }
}

pub struct Cursor {
    pub x: u32,
    pub y: u32,
    pub attr: CharOpts,
    pub hidden: bool,
}

lazy_static! {
    static ref _DEFAULT_MODE: HashSet<u32> = {
        let mut m = HashSet::new();
        m.insert(DECAWM);
        m.insert(DECTCEM);
        m
    };
}
