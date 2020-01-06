use crate::{err::*, mem::Mem, state::State};

pub struct Instr {
    pub count: u8,
    pub opcode: u8,
    pub operands: Vec<u16>,
    pub end: usize,
}

impl Instr {
    pub fn store(&self, mem: &mut Mem, state: &mut State, val: u16) -> Result<(), Error> {
        let var = *fatal(mem.get(self.end).ok_or_else(|| err(Cause::PcOut, (0, 0))))?;
        state.set_var(mem, var.into(), val)?;
        Ok(())
    }

    pub fn branch(
        &self,
        mem: &mut Mem,
        state: &mut State,
        cond: bool,
        store: bool,
    ) -> Result<(), Error> {
        let mut addr = self.end + if store { 1 } else { 0 };
        let top = *fatal(mem.get(addr).ok_or_else(|| err(Cause::PcOut, (0, 0))))? as usize;
        let long = top & 0x40 == 0;
        let when = top & 0x80 != 0;

        if when == cond {
            // branch
            let offset = if long {
                (top & 0x3f) << 8
                    | *fatal(mem.get(addr + 1).ok_or_else(|| err(Cause::PcOut, (0, 0))))? as usize
            } else {
                addr -= 1;
                top & 0x3f
            };
            match offset {
                0 | 1 => state.ret(mem, offset as u16)?,
                2..=0x1fff => state.pc = addr + offset,
                _ => state.pc = addr - (!offset & 0x1fff) - 1, // ERROR substract with overflow
            }
        } else {
            // continue
            if long {
                state.pc = addr + 2;
            } else {
                state.pc = addr + 1;
            }
        }
        Ok(())
    }
}

pub fn decode(mem: &Mem, state: &mut State, mut addr: usize) -> Result<Instr, Error> {
    let top = *mem.get(addr).ok_or_else(|| err(Cause::PcOut, (0, 0)))?;
    if mem[0] >= 5 && top == 0xbe {
        // ext
        let opcode = *mem.get(addr + 1).ok_or_else(|| err(Cause::PcOut, (0, 0)))?;
        let types = *mem.get(addr + 2).ok_or_else(|| err(Cause::PcOut, (0, 0)))?;
        let mut operands = Vec::new();
        addr += 3;
        for i in (0..4).rev() {
            let type_ = (types >> (i * 2)) & 0x03;
            if type_ == 3 {
                break;
            }
            let operand = parse_op(mem, state, addr, type_)?;
            operands.push(operand.0);
            addr = operand.1
        }
        Ok(Instr {
            count: 4,
            opcode,
            operands,
            end: addr,
        })
    } else {
        match top & 0xc0 {
            0xc0 => {
                // var
                let opcode = top & 0x1f;
                addr += 1;
                let mut operands = Vec::new();
                let mut types = vec![*mem.get(addr).ok_or_else(|| err(Cause::PcOut, (0, 0)))?];
                addr += 1;
                let count = if top & 0x20 == 0 {
                    2
                } else {
                    if opcode == 0x0c || opcode == 0x1a {
                        types.push(*mem.get(addr).ok_or_else(|| err(Cause::PcOut, (0, 0)))?);
                        addr += 1;
                    }
                    3
                };

                for types in types.iter() {
                    for i in (0..4).rev() {
                        let type_ = (types >> (i * 2)) & 0x03u8;
                        if type_ == 3 {
                            break;
                        }
                        let operand = parse_op(mem, state, addr, type_)?;
                        operands.push(operand.0);
                        addr = operand.1;
                    }
                }
                if count == 2 && operands.len() < 2 {
                    return error(Cause::MissingOperand, (2, operands.len() as u16));
                }

                Ok(Instr {
                    count,
                    opcode,
                    operands,
                    end: addr,
                })
            }
            0x80 => {
                // short
                let type_ = top & 0x30;
                if type_ == 0x30 {
                    // 0OP
                    Ok(Instr {
                        count: 0,
                        opcode: top & 0x0f,
                        operands: vec![],
                        end: addr + 1,
                    })
                } else {
                    // 1OP
                    let operand = parse_op(mem, state, addr + 1, type_ >> 4)?;
                    Ok(Instr {
                        count: 1,
                        opcode: top & 0x0f,
                        operands: vec![operand.0],
                        end: operand.1,
                    })
                }
            }
            _ => {
                // long
                let mut operands = Vec::new();
                addr += 1;
                for i in 1..3 {
                    let type_ = if top & (0x80 >> i) == 0 { 1 } else { 2 };
                    let operand = parse_op(mem, state, addr, type_)?;
                    operands.push(operand.0);
                    addr = operand.1;
                }
                Ok(Instr {
                    count: 2,
                    opcode: top & 0x1f,
                    operands,
                    end: addr,
                })
            }
        }
    }
}

fn parse_op(mem: &Mem, state: &mut State, addr: usize, type_: u8) -> Result<(u16, usize), Error> {
    match type_ {
        0 => Ok((
            mem.getw(addr).ok_or_else(|| err(Cause::PcOut, (0, 0)))?,
            addr + 2,
        )),
        1 => Ok((
            u16::from(*mem.get(addr).ok_or_else(|| err(Cause::PcOut, (0, 0)))?),
            addr + 1,
        )),
        2 => {
            let var = u16::from(*mem.get(addr).ok_or_else(|| err(Cause::PcOut, (0, 0)))?);
            Ok((state.get_var(&mem, var)?, addr + 1))
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
use crate::{mem, state};

#[test]
fn test_decode() {
    let mut data = mem::default();
    data[0x00] = 5;
    data.extend(vec![
        0xbf, 0x8f, 0x0f, 0x0f, 0x5f, 0x01, 0x02, 0xdf, 0x9f, 0x01, 0x02, 0xff, 0x1b, 0xf0, 0xf0,
        0x02, 0x01, 0xbe, 0x00, 0x63, 0x02, 0x01, 0x55, 0xaa,
    ]);
    let mut mem = mem::new(data).unwrap();
    let mut state = state::init(&mem);
    state.call(&mut mem, 0x46, vec![], false).unwrap();
    state.set_var(&mut mem, 1, 0xa5).unwrap();

    let instr = decode(&mem, &mut state, 0x40).unwrap();
    assert_eq!(instr.count, 0);
    assert_eq!(instr.opcode, 0xf);
    assert_eq!(instr.operands, vec![]);
    assert_eq!(instr.end, 0x41);

    // @"1OP:15"     8f    0f0f
    let instr = decode(&mem, &mut state, 0x41).unwrap();
    assert_eq!(instr.count, 1);
    assert_eq!(instr.opcode, 0xf);
    assert_eq!(instr.operands, vec![0x0f0f]);
    assert_eq!(instr.end, 0x44);

    // @"2OP:31"     5f    01 02
    let instr = decode(&mem, &mut state, 0x44).unwrap();
    assert_eq!(instr.count, 2);
    assert_eq!(instr.opcode, 0x1f);
    assert_eq!(instr.operands, vec![0xa5, 2]);
    assert_eq!(instr.end, 0x47);

    // @"2OP:31"     df 9f 01 02
    let instr = decode(&mem, &mut state, 0x47).unwrap();
    assert_eq!(instr.count, 2);
    assert_eq!(instr.opcode, 0x1f);
    assert_eq!(instr.operands, vec![0xa5, 2]);
    assert_eq!(instr.end, 0x4b);

    // @"VAR:31"     ff 1b f0f0 02 01
    let instr = decode(&mem, &mut state, 0x4b).unwrap();
    assert_eq!(instr.count, 3);
    assert_eq!(instr.opcode, 0x1f);
    assert_eq!(instr.operands, vec![0xf0f0, 2, 0xa5]);
    assert_eq!(instr.end, 0x51);

    // @"EXT:0"      be 00 63 02 01 55aa
    let instr = decode(&mem, &mut state, 0x51).unwrap();
    assert_eq!(instr.count, 4);
    assert_eq!(instr.opcode, 0);
    assert_eq!(instr.operands, vec![2, 0xa5, 0x55aa]);
    assert_eq!(instr.end, 0x58);
}

#[test]
fn test_store() {
    let mut data = mem::default();
    data[0x0d] = 0x40;
    data[0x0f] = 0x42;
    data.extend(vec![0x10, 0x20]);
    let mut mem = mem::new(data).unwrap();
    let mut state = state::init(&mem);
    let instr = Instr {
        count: 0,
        opcode: 0,
        operands: vec![],
        end: 0x40,
    };
    instr.store(&mut mem, &mut state, 0x0102).unwrap();
    assert_eq!(mem.loadw(0x40).unwrap(), 0x0102);
}

#[test]
fn test_branch() {
    let mut data = mem::default();
    data.extend(vec![
        0x40, 0xc1, 0x50, 0x91, 0x00, 0x3f, 0xf0, 0x01, 0x01, 0x00,
    ]);
    let mut mem = mem::new(data).unwrap();
    let mut instr = Instr {
        count: 0,
        opcode: 0,
        operands: vec![],
        end: 0x40,
    };
    let mut state = state::init(&mem);
    state.call(&mut mem, 0x47, Vec::new(), false).unwrap();
    state.pc = 0x48;
    state.call(&mut mem, 0x47, Vec::new(), true).unwrap();
    state.pc = 0x49;
    state.call(&mut mem, 0x44, Vec::new(), true).unwrap();
    state.pc = 0x01;

    instr.branch(&mut mem, &mut state, false, false).unwrap();
    assert_eq!(state.pc, 0x49);
    assert_eq!(state.get_var(&mem, 0x01).unwrap(), 0x00);
    instr.branch(&mut mem, &mut state, true, false).unwrap();
    assert_eq!(state.pc, 0x41);
    instr.branch(&mut mem, &mut state, false, true).unwrap();
    assert_eq!(state.pc, 0x42);
    instr.branch(&mut mem, &mut state, true, true).unwrap();
    assert_eq!(state.pc, 0x48);
    assert_eq!(state.get_var(&mem, 0x01).unwrap(), 0x01);
    instr.end = 0x42;
    instr.branch(&mut mem, &mut state, false, false).unwrap();
    assert_eq!(state.pc, 0x51);
    instr.branch(&mut mem, &mut state, true, false).unwrap();
    assert_eq!(state.pc, 0x43);
    instr.branch(&mut mem, &mut state, false, true).unwrap();
    assert_eq!(state.pc, 0x45);
    instr.branch(&mut mem, &mut state, true, true).unwrap();
    assert_eq!(state.pc, 0x1143);
    instr.end = 0x45;
    instr.branch(&mut mem, &mut state, false, false).unwrap();
    assert_eq!(state.pc, 0x35);
    instr.branch(&mut mem, &mut state, true, false).unwrap();
    assert_eq!(state.pc, 0x47);
}
