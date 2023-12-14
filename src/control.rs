// *Space*: Not surprisingly -- ``" "``.
const SP: &str = " ";

// *Null*: Does nothing.
const NUL: &str = "\x00";

// *Bell*: Beeps.
const BEL: &str = "\x07";

// *Backspace*: Backspace one column, but not past the beginning of the
// line.
const BS: &str = "\x08";

// *Horizontal tab*: Move cursor to the next tab stop, or to the end
// of the line if there is no earlier tab stop.
const HT: &str = "\x09";

// *Linefeed*: Give a line feed, and, if :data:`pyte.modes.LNM` (new
// line mode) is set also a carriage return.
const LF: &str = "\n";

// *Vertical tab*: Same as :data:`LF`.
const VT: &str = "\x0b";

// #: *Form feed*: Same as :data:`LF`.
const FF: &str = "\x0c";

// #: *Carriage return*: Move cursor to left margin on current line.
const CR: &str = "\r";

// #: *Shift out*: Activate G1 character set.
const SO: &str = "\x0e;";

// #: *Shift in*: Activate G0 character set.
const SI: &str = "\x0f";

// #: *Cancel*: Interrupt escape sequence. If received during an escape or
// #: control sequence, cancels the sequence and displays substitution
// #: character.
const CAN: &str = "\x18";
// #: *Substitute*: Same as :data:`CAN`.
const SUB: &str = "\x1a";

// #: *Escape*: Starts an escape sequence.
const ESC: &str = "\x1b";

// #: *Delete*: Is ignored.
const DEL: &str = "\x7f";

// #: *Control sequence introducer*.
// const CSI_C0: &str = &format!("{}{}", ESC, "[");
// const CSI_C1 : &str = "\x9b";
// const CSI: &str = CSI_C0;

// #: *String terminator*.
// const ST_C0 : &str = ESC + "\\";
// const ST_C1 : &str = "\x9c";
// const ST: &str = ST_C0;

// #: *Operating system command*.
// const OSC_C0 : &str = ESC + "]";
// const OSC_C1 : &'static str = "\x9d";
// const OSC = OSC_C0;

#[cfg(test)]
mod test {

    #[test]
    fn ris() {
        use ansi_control_codes::independent_control_functions::RIS;
        println!("RIS: {}", RIS);
    }
}
