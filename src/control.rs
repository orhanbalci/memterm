use std::collections::{BTreeMap, HashSet};

use lazy_static::lazy_static;

use crate::ascii;

//C0 codes
pub const BEL: &str = ascii!(0 / 7);
pub const BS: &str = ascii!(0 / 8);
pub const CAN: &str = ascii!(1 / 8);
pub const CR: &str = ascii!(0 / 13);
pub const ESC: &str = ascii!(1 / 11);
pub const FF: &str = ascii!(0 / 12);
pub const HT: &str = ascii!(0 / 9);
pub const LF: &str = ascii!(0 / 10);
pub const SI: &str = ascii!(0 / 15);
pub const SO: &str = ascii!(0 / 14);
pub const SUB: &str = ascii!(1 / 10);
pub const VT: &str = ascii!(0 / 11);

//C1 codes
pub const CSI: &str = "\u{009B}";
pub const HTS: &str = ascii!(4 / 8);
pub const NEL: &str = ascii!(4 / 5);
pub const OSC: &str = "\u{009D}";
pub const RI: &str = ascii!(4 / 13);
pub const ST: &str = ascii!(5 / 12);

// CSI escape sequences
pub const ICH: &str = ascii!(4 / 0);
pub const CUU: &str = ascii!(4 / 1);
pub const CUD: &str = ascii!(4 / 2);
pub const CUF: &str = ascii!(4 / 3);
pub const CUB: &str = ascii!(4 / 4);
pub const CNL: &str = ascii!(4 / 5);
pub const CPL: &str = ascii!(4 / 6);
pub const CHA: &str = ascii!(4 / 7);
pub const CUP: &str = ascii!(4 / 8);
pub const ED: &str = ascii!(4 / 10);
pub const EL: &str = ascii!(4 / 11);
pub const IL: &str = ascii!(4 / 12);
pub const DL: &str = ascii!(4 / 13);
pub const DCH: &str = ascii!(5 / 0);
pub const ECH: &str = ascii!(5 / 8);
pub const HPR: &str = ascii!(6 / 1);
pub const DA: &str = ascii!(6 / 3);
pub const VPA: &str = ascii!(6 / 4);
pub const VPR: &str = ascii!(6 / 5);
pub const HVP: &str = ascii!(6 / 6);
pub const TBC: &str = ascii!(6 / 7);
pub const SM: &str = ascii!(6 / 8);
pub const RM: &str = ascii!(6 / 12);
pub const SGR: &str = ascii!(6 / 13);
pub const DECSTBM: &str = ascii!(7 / 2);

pub const DECALN: &str = ascii!(3 / 8);
pub const IND: &str = ascii!(4 / 4);
pub const DECSC: &str = ascii!(3 / 7);
pub const DECRC: &str = ascii!(3 / 8);
pub const SP: &str = ascii!(2 / 0);
pub const GREATER: &str = ascii!(3 / 14);
pub const RIS: &str = ascii!(6 / 3);

pub const BASIC: &[&str; 9] = &[BEL, BS, HT, LF, VT, FF, CR, SO, SI];
pub const ALLOWED_IN_CSI: &[&str; 7] = &[BEL, BS, HT, LF, VT, FF, CR];
pub const ST_C0: &str = "\u{001B}\u{009C}";
pub const ST_C1: &str = ST;
pub const OSC_TERMINATORS: &[&str; 3] = &[BEL, ST_C0, ST_C1];

lazy_static! {
// Special characters set
    pub static ref SPECIAL: HashSet<&'static str> = {
        let mut special = HashSet::new();
        special.insert(ESC);
        special.insert(CSI);
        // Add NUL and DEL if you have them defined
        special.insert(OSC);

        // Add all basic control characters
        for &key in BASIC {
            special.insert(key);
        }
        special
    };

}

// Define the CSI command mapping
lazy_static! {
    pub static ref CSI_COMMANDS: BTreeMap<&'static str, &'static str> = {
        let mut m = BTreeMap::new();
        m.insert(ICH, "insert_characters");
        m.insert(CUU, "cursor_up");
        m.insert(CUD, "cursor_down");
        m.insert(CUF, "cursor_forward");
        m.insert(CUB, "cursor_back");
        m.insert(CNL, "cursor_down1");
        m.insert(CPL, "cursor_up1");
        m.insert(CHA, "cursor_to_column");
        m.insert(CUP, "cursor_position");
        m.insert(ED, "erase_in_display");
        m.insert(EL, "erase_in_line");
        m.insert(IL, "insert_lines");
        m.insert(DL, "delete_lines");
        m.insert(DCH, "delete_characters");
        m.insert(ECH, "erase_characters");
        m.insert(HPR, "cursor_forward");
        m.insert(DA, "report_device_attributes");
        m.insert(VPA, "cursor_to_line");
        m.insert(VPR, "cursor_down");
        m.insert(HVP, "cursor_position");
        m.insert(TBC, "clear_tab_stop");
        m.insert(SM, "set_mode");
        m.insert(RM, "reset_mode");
        m.insert(SGR, "select_graphic_rendition");
        m
    };
}
