pub mod charset;
pub mod control;
pub mod parser;

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
