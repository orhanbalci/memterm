// *Line Feed/New Line Mode*: When enabled, causes a received
// :data:`~pyte.control.LF`, :data:`pyte.control.FF`, or
// :data:`~pyte.control.VT` to move the cursor to the first column of
// the next line.
pub const LNM: u32 = 20;

// *Insert/Replace Mode*: When enabled, new display characters move
// old display characters to the right. Characters moved past the
// right margin are lost. Otherwise, new display characters replace
// old display characters at the cursor position.
pub const IRM: u32 = 4;

//Private modes.
//..............
// *Text Cursor Enable Mode*: determines if the text cursor is
// visible.
pub const DECTCEM: u32 = 25 << 5;

// *Screen Mode*: toggles screen-wide reverse-video mode.
pub const DECSCNM: u32 = 5 << 5;

// *Origin Mode*: allows cursor addressing relative to a user-defined
// origin. This mode resets when the terminal is powered up or reset.
// It does not affect the erase in display (ED) function.
pub const DECOM: u32 = 6 << 5;

// *Auto Wrap Mode*: selects where received graphic characters appear
// when the cursor is at the right margin.
pub const DECAWM: u32 = 7 << 5;

// *Column Mode*: selects the number of columns per line (80 or 132)
// on the screen.
pub const DECCOLM: u32 = 3 << 5;
