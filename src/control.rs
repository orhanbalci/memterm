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
pub const CSI: &str = ascii!(5 / 11);
pub const HTS: &str = ascii!(4 / 8);
pub const NEL: &str = ascii!(4 / 5);
pub const OSC: &str = ascii!(5 / 13);
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

pub const DECALN: &str = ascii!(3 / 8);
pub const IND: &str = ascii!(4 / 4);
pub const DECSC: &str = ascii!(3 / 7);
pub const DECRC: &str = ascii!(3 / 8);
pub const SP: &str = ascii!(2 / 0);
pub const GREATER: &str = ascii!(3 / 14);
pub const RIS: &str = ascii!(6 / 3);

pub const BASIC: &[&str; 9] = &[BEL, BS, HT, LF, VT, FF, CR, SO, SI];
pub const ALLOWED_IN_CSI: &[&str; 7] = &[BEL, BS, HT, LF, VT, FF, CR];
pub const OSC_TERMINATORS: &[&str; 2] = &[BEL, ST];
