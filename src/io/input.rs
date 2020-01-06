use crate::{err::*, interface::Interface, mem::Mem, text::Text, *};

pub struct Input {
    v: u8,
    current: u16,
    terminating: Option<usize>,
}

impl Input {
    pub fn set_stream(&mut self, stream: u16) -> Result<(), Error> {
        if stream > 1 {
            return error(Cause::BadInputStream, (stream, 0));
        }
        self.current = stream;
        Ok(())
    }

    pub fn read<I: Interface>(
        &mut self,
        mut addr: u16,
        mut time: u16,
        routine: u16,
        mem: &mut Mem,
        rand: &mut alu::Random,
        text: &Text,
        state: &mut state::State,
        out: &mut out::Output,
        screen: &mut screen::Screen,
        header: &header::Header,
        obj: &obj::Object,
        dict: &dict::Dict,
        interface: &mut I,
        restart: &state::Saved,
        undo: &mut [state::Saved],
        config: &Config,
        err_said: &mut [bool],
    ) -> Result<(String, char), Error> {
        let mut terminating = Vec::new();
        if let Some(mut addr) = self.terminating {
            let mut zscii = *mem
                .get(addr)
                .ok_or_else(|| err(Cause::TerminatingOut, (addr as u16, 0)))?;
            while zscii != 0 {
                match zscii {
                    129..=154 => terminating.push(zscii as char),
                    255 => {
                        terminating = (129u8..155u8).map(|n| n as char).collect::<Vec<_>>();
                        break;
                    }
                    _ => {}
                }
                addr += 1;
                zscii = *mem
                    .get(addr)
                    .ok_or_else(|| err(Cause::TerminatingOut, (addr as u16, 0)))?;
            }
        }

        let max = mem.loadb(addr)?;
        let nbr = if self.v < 5 {
            0
        } else {
            addr += 1;
            mem.loadb(addr)? as usize
        };
        addr += 1;

        let (mut str, char) = match self.current {
            0 => {
                let mut left = String::new();
                let addr = addr as usize;
                if addr + nbr > mem.len() {
                    return error(Cause::TextBufferOut, (addr as u16, 0));
                }
                for i in 0..nbr {
                    if let Some(char) = text.decode_char(mem, mem[addr + i])? {
                        left.push(char);
                    }
                }
                let routine = if self.v < 4 || time == 0 || routine == 0 {
                    time = 0;
                    0
                } else {
                    mem.packed(routine, true)
                };
                interface.read(terminating, left, max, time, |interface| {
                    state
                        .interrupt(
                            routine, mem, rand, text, out, screen, self, header, obj, dict,
                            interface, restart, undo, config, err_said,
                        )
                        .unwrap_or(0)
                        != 0
                })
            }
            1 => (interface.read_file(), '\n'),
            _ => unreachable!(),
        };
        str.truncate(max as usize);

        let end = text.to_zscii(mem, addr, &str)?;
        if self.v < 5 {
            mem.storeb(end, 0)?;
        } else {
            mem.storeb(addr - 1, end - addr)?;
        }
        Ok((str, char))
    }

    pub fn read_char<I: Interface>(
        &mut self,
        time: u16,
        routine: u16,
        mem: &mut Mem,
        rand: &mut alu::Random,
        text: &Text,
        state: &mut state::State,
        out: &mut out::Output,
        screen: &mut screen::Screen,
        header: &header::Header,
        obj: &obj::Object,
        dict: &dict::Dict,
        interface: &mut I,
        restart: &state::Saved,
        undo: &mut [state::Saved],
        config: &Config,
        err_said: &mut [bool],
    ) -> char {
        match self.current {
            0 => {
                let routine = mem.packed(routine, true);
                interface.read_char(time, |interface| {
                    state
                        .interrupt(
                            routine, mem, rand, text, out, screen, self, header, obj, dict,
                            interface, restart, undo, config, err_said,
                        )
                        .unwrap_or(0)
                        != 0
                })
            }
            1 => interface.read_file().chars().next().unwrap_or('\n'),
            _ => unreachable!(),
        }
    }
}

pub fn init(mem: &Mem) -> Input {
    let v = mem[0];
    let terminating = if v < 5 {
        None
    } else {
        let terminating = mem.loadw(0x2e).unwrap();
        if terminating == 0 {
            None
        } else {
            Some(mem.byte(terminating))
        }
    };
    Input {
        v,
        current: 0,
        terminating,
    }
}
