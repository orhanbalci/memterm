macro_rules! ascii {
    ($($xx:literal/$yy:literal), *) => {
        unsafe { std::str::from_utf8_unchecked(&[$(($xx << 4) + $yy),*]) }
    };
}

pub(crate) use ascii;
pub mod charset;
pub mod control;
pub mod parser;
pub mod parser_listener;

#[cfg(test)]
mod test {
    use crate::charset;
    use ansi_control_codes::c0::ESC;

    #[test]
    fn write_vt_100_chars() {
        let code = ESC.to_string().into_bytes();
        assert!(code.len() == 1);
        println!("{:#010b}", code.get(0).unwrap());
        println!("{:?}", ESC.to_string());
        println!("{:#010b}", "".as_bytes()[0]);
    }
}
