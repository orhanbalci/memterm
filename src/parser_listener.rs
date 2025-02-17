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
    DECSTBM,
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
    fn alignment_display(&mut self);
    fn define_charset(&mut self, code: &str, mode: &str);
    fn reset(&mut self);
    fn index(&mut self);
    fn linefeed(&mut self);
    fn reverse_index(&mut self);
    fn set_tab_stop(&mut self);
    fn save_cursor(&mut self);
    fn restore_cursor(&mut self);
    fn shift_out(&mut self);
    fn shift_in(&mut self);

    // basic escape code actions
    fn bell(&mut self);
    fn backspace(&mut self);
    fn tab(&mut self);
    fn cariage_return(&mut self);

    fn draw(&mut self, input: &str);

    //csi commands
    fn insert_characters(&mut self, count: Option<u32>);
    fn cursor_up(&mut self, count: Option<u32>);
    fn cursor_down(&mut self, count: Option<u32>);
    fn cursor_forward(&mut self, count: Option<u32>);
    fn cursor_back(&mut self, count: Option<u32>);
    fn cursor_down1(&mut self, count: Option<u32>);
    fn cursor_up1(&mut self, count: Option<u32>);
    fn cursor_to_column(&mut self, character: Option<u32>);
    fn cursor_position(&mut self, line: Option<u32>, character: Option<u32>);
    fn erase_in_display(&mut self, how: Option<u32>, private: Option<bool>);
    fn erase_in_line(&mut self, how: Option<u32>, private: Option<bool>);
    fn insert_lines(&mut self, count: Option<u32>);
    fn delete_lines(&mut self, count: Option<u32>);
    fn delete_characters(&mut self, count: Option<u32>);
    fn erase_characters(&mut self, count: Option<u32>);
    fn report_device_attributes(&mut self, mode: Option<u32>, private: Option<bool>);
    fn cursor_to_line(&mut self, line: Option<u32>);
    fn clear_tab_stop(&mut self, how: Option<u32>);
    fn set_mode(&mut self, modes: &[u32], is_private: bool);
    fn reset_mode(&mut self, modes: &[u32], is_private: bool);
    fn select_graphic_rendition(&mut self, modes: &[u32]);
    fn set_title(&mut self, title: &str);
    fn set_icon_name(&mut self, icon_name: &str);
    fn set_margins(&mut self, top: Option<u32>, bottom: Option<u32>);

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

    fn basic_dispatch(&mut self, basic_command: &str) {
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
            ec if ec == CUD => self.cursor_down(if !params.is_empty() {
                Some(params[0])
            } else {
                None
            }),
            ec if ec == CUU => self.cursor_up(if !params.is_empty() {
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
                    if params.len() == 1 {
                        self.cursor_position(Some(params[0]), None);
                    } else {
                        self.cursor_position(Some(params[0]), Some(params[1]));
                    }
                } else {
                    self.cursor_position(None, None)
                }
            }
            ec if ec == ED => self.erase_in_display(
                if !params.is_empty() {
                    Some(params[0])
                } else {
                    None
                },
                None,
            ),
            ec if ec == EL => self.erase_in_line(
                if !params.is_empty() {
                    Some(params[0])
                } else {
                    None
                },
                None,
            ),
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
            ec if ec == DA => self.report_device_attributes(params.iter().cloned().next(), None), // TODO handle second parameter
            ec if ec == VPA => self.cursor_to_line(params.iter().cloned().next()),
            ec if ec == VPR => self.cursor_down(params.iter().cloned().next()),
            ec if ec == HVP => {
                self.cursor_position(params.iter().cloned().nth(0), params.iter().cloned().nth(1))
            }
            ec if ec == TBC => self.clear_tab_stop(params.iter().cloned().next()),
            ec if ec == SM => self.set_mode(params, is_private),
            ec if ec == RM => self.reset_mode(params, is_private),
            ec if ec == SGR => self.select_graphic_rendition(params),
            ec if ec == DECSTBM => {
                self.set_margins(params.iter().cloned().nth(0), params.iter().cloned().nth(1))
            }
            ec => {
                println!("unexpected csi escape code {}", ec);
            }
        }
    }

    fn display(&mut self) -> Vec<String>;
}
