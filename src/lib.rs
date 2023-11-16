pub mod charset;

#[cfg(test)]
mod test {
    use crate::charset;

    #[test]
    fn write_vt_100_chars() {
        print!("{:?}", charset::VT_100_MAP);
    }
}
