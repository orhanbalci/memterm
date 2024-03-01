use crate::control::{
    BEL,
    BS,
    CHA,
    CNL,
    CPL,
    CR,
    CUB,
    CUD,
    CUF,
    CUP,
    CUU,
    DA,
    DCH,
    DECRC,
    DECSC,
    DL,
    ECH,
    ED,
    EL,
    FF,
    HPR,
    HT,
    HTS,
    HVP,
    ICH,
    IL,
    IND,
    LF,
    NEL,
    RI,
    RIS,
    RM,
    SGR,
    SI,
    SM,
    SO,
    TBC,
    VPA,
    VPR,
    VT,
};

pub trait ParserListener {
    fn alignment_display(&self);
    fn define_charset(&mut self, code: &str, mode: &str);
    fn reset(&mut self);
    fn index(&self);
    fn linefeed(&self);
    fn reverse_index(&self);
    fn set_tab_stop(&self);
    fn save_cursor(&self);
    fn restore_cursor(&self);
    fn shift_out(&self);
    fn shift_in(&self);

    // basic esvape code actions
    fn bell(&self);
    fn backspace(&self);
    fn tab(&self);
    fn cariage_return(&self);

    fn draw(&self, input: &str);

    //csi commands
    fn insert_characters(&self, count: Option<u32>);
    fn cursor_up(&self, count: Option<u32>);
    fn cursor_down(&self, count: Option<u32>);
    fn cursor_forward(&self, count: Option<u32>);
    fn cursor_back(&self, count: Option<u32>);
    fn cursor_down1(&self, count: Option<u32>);
    fn cursor_up1(&self, count: Option<u32>);
    fn cursor_to_column(&self, character: Option<u32>);
    fn cursor_position(&self, line: Option<u32>, character: Option<u32>);
    fn erase_in_display(&self, erase_page: Option<u32>);
    fn erase_in_line(&self, erase_line: Option<u32>);
    fn insert_lines(&self, count: Option<u32>);
    fn delete_lines(&self, count: Option<u32>);
    fn delete_characters(&self, count: Option<u32>);
    fn erase_characters(&self, count: Option<u32>);
    fn report_device_attributes(&self, attribute: Option<u32>);
    fn cursor_to_line(&self, count: Option<u32>);
    fn clear_tab_stop(&self, option: Option<u32>);
    fn set_mode(&mut self, modes: &[u32], is_private: bool);
    fn reset_mode(&mut self, modes: &[u32], is_private: bool);
    fn select_graphic_rendition(&self, modes: &[u32]);

    fn escape_dispatch(&mut self, escape_command: &str) {
        match escape_command {
            ec if ec == RIS => {
                self.reset();
            }
            ec if ec == IND => {
                self.index();
            }
            ec if ec == NEL => {
                self.linefeed();
            }
            ec if ec == RI => {
                self.reverse_index();
            }
            ec if ec == HTS => {
                self.set_tab_stop();
            }
            ec if ec == DECSC => {
                self.save_cursor();
            }
            ec if ec == DECRC => {
                self.restore_cursor();
            }
            _ => {
                println!("un expected escape code")
            }
        }
    }

    fn basic_dispatch(&self, basic_command: &str) {
        match basic_command {
            ec if ec == BEL => {
                self.bell();
            }
            ec if ec == BS => {
                self.backspace();
            }
            ec if ec == HT => {
                self.tab();
            }
            ec if (ec == LF || ec == VT || ec == FF) => {
                self.linefeed();
            }
            ec if ec == CR => {
                self.cariage_return();
            }
            ec if ec == SO => {
                self.shift_out();
            }
            ec if ec == SI => {
                self.shift_in();
            }
            _ => {
                println!("un expected escape code")
            }
        }
    }

    fn csi_dispatch(&mut self, csi_command: &str, params: &[u32], is_private: bool) {
        match csi_command {
            ec if ec == ICH => self.insert_characters(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUD => self.cursor_up(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUU => self.cursor_down(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUF => self.cursor_forward(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUB => self.cursor_back(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CNL => self.cursor_down1(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CPL => self.cursor_up1(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CHA => self.cursor_to_column(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUP => {
                if !params.is_empty() {
                    self.cursor_position(Some(params[0]), Some(params[1]));
                } else {
                    self.cursor_position(None, None)
                }
            }
            ec if ec == ED => self.erase_in_display(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == EL => self.erase_in_line(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == IL => self.insert_lines(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == DL => self.delete_lines(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == DCH => self.delete_characters(params.iter().cloned().next()),
            ec if ec == ECH => self.erase_characters(params.iter().cloned().next()),
            ec if ec == HPR => self.cursor_forward(params.iter().cloned().next()),
            ec if ec == DA => self.report_device_attributes(params.iter().cloned().next()),
            ec if ec == VPA => self.cursor_to_line(params.iter().cloned().next()),
            ec if ec == VPR => self.cursor_down(params.iter().cloned().next()),
            ec if ec == HVP => {
                self.cursor_position(params.iter().cloned().nth(0), params.iter().cloned().nth(1))
            }
            ec if ec == TBC => self.clear_tab_stop(params.iter().cloned().next()),
            ec if ec == SM => self.set_mode(params, is_private),
            ec if ec == RM => self.reset_mode(params, is_private),
            ec if ec == SGR => self.select_graphic_rendition(params),
            _ => {
                println!("unexpected csi escape code");
            }
        }
    }
}
