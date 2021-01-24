use crate::{err::*, *};

pub fn exec<I: Interface>(
    instr: &instr::Instr,
    mem: &mut mem::Mem,
    rand: &mut alu::Random,
    text: &text::Text,
    state: &mut state::State,
    out: &mut out::Output,
    screen: &mut screen::Screen,
    input: &mut input::Input,
    header: &header::Header,
    obj: &obj::Object,
    dict: &dict::Dict,
    interface: &mut I,
    restart: &state::Saved,
    undo: &mut [state::Saved],
    config: &Config,
    err_said: &mut [bool],
) -> Result<(), Error> {
    let &instr::Instr {
        count,
        opcode,
        ref operands,
        end,
    } = instr;
    let v = mem[0];

    match count {
        0 => match opcode {
            0x00 => {
                state.ret(mem, 1)?;
            }
            0x01 => {
                state.ret(mem, 0)?;
            }
            0x02 => {
                let string = trace(fatal(text.decode(mem, end)), Trace::String(end))?;
                state.pc = string.1;
                out.write(mem, text, interface, &string.0, 0)?;
            }
            0x03 => {
                let ret = state.ret(mem, 1);
                let mut string = trace(text.decode(mem, end), Trace::String(end))?;
                string.0.push('\n');
                out.write(mem, text, interface, &string.0, 0)?;
                ret?;
            }
            0x04 => state.pc = end,
            0x05 if v < 5 => {
                let save = state.save(mem)?;
                let result = interface.save(&quetzal::save(mem, restart, &save, header.checksum));
                if v < 4 {
                    instr.branch(mem, state, result, false)?;
                } else {
                    state.pc = end + 1;
                    instr.store(mem, state, if result { 1 } else { 0 })?;
                }
            }
            0x06 if v < 5 => {
                let save = quetzal::restore(restart, &interface.restore(), header.checksum);
                match save {
                    Ok(save) => {
                        state.restore(mem, &save);
                        let instr = instr::Instr {
                            count: 0,
                            opcode: 0,
                            operands: Vec::new(),
                            end: state.pc,
                        };
                        if v < 4 {
                            instr.branch(mem, state, true, false)?;
                        } else {
                            state.pc += 1;
                            instr.store(mem, state, 2)?;
                        }
                        header::init(mem, config)?;
                    }
                    Err(cause) => {
                        interface.restore_failed(cause);
                        if v < 4 {
                            instr.branch(mem, state, false, false)?;
                        } else {
                            state.pc = end + 1;
                            instr.store(mem, state, 0)?;
                        }
                    }
                }
            }
            0x07 => state.restore(mem, restart),
            0x08 => {
                let val = handle(state.get_var(mem, 0), || state.ret(mem, 0))?;
                state.ret(mem, val)?;
            }
            0x09 => {
                if v < 5 {
                    state.pc = end;
                    state.get_var(mem, 0)?;
                } else {
                    state.pc = end + 1;
                    let frame = state.catch();
                    instr.store(mem, state, frame)?;
                }
            }
            0x0a => return fatal(error(Cause::Quit, (0, 0))),
            0x0b => {
                state.pc = end;
                out.write(mem, text, interface, "\n", 0)?;
            }
            0x0c if v == 3 => {
                state.pc = end;
                screen.status(mem, text, state, obj, interface)?;
            }
            0x0d => {
                let checksum = mem.loadw(0x1c).unwrap();
                let eq = header.checksum == checksum;
                instr.branch(mem, state, eq, false)?;
            }
            0x0f => instr.branch(mem, state, true, false)?,
            _ => return fatal(error(Cause::BadOpcode, (count.into(), opcode.into()))),
        },
        1 => match opcode {
            0x00 => {
                instr.branch(mem, state, operands[0] == 0, false)?;
            }
            0x01 => {
                let obj = handle(obj.get_sibling(mem, operands[0]), || {
                    instr.branch(mem, state, false, true)
                })?;
                instr.branch(mem, state, obj != 0, true)?;
                instr.store(mem, state, obj)?;
            }
            0x02 => {
                let obj = handle(obj.get_child(mem, operands[0]), || {
                    instr.branch(mem, state, false, true)
                })?;
                instr.branch(mem, state, obj != 0, true)?;
                instr.store(mem, state, obj)?;
            }
            0x03 => {
                state.pc = end + 1;
                let obj = obj.get_parent(mem, operands[0])?;
                instr.store(mem, state, obj)?;
            }
            0x04 => {
                state.pc = end + 1;
                let len = obj.prop_len(mem, operands[0])?;
                instr.store(mem, state, len)?;
            }
            0x05 => {
                state.pc = end;
                let mut val = state.get_var(mem, operands[0])?;
                val = alu::add(val, 1);
                state.set_var(mem, operands[0], val)?;
            }
            0x06 => {
                state.pc = end;
                let mut val = state.get_var(mem, operands[0])?;
                val = alu::sub(val, 1);
                state.set_var(mem, operands[0], val)?;
            }
            0x07 => {
                state.pc = end;
                let addr = mem.byte(operands[0]);
                let string = trace(text.decode(mem, addr), Trace::String(addr))?;
                out.write(mem, text, interface, &string.0, 0)?;
            }
            0x08 if v >= 4 => {
                let addr = mem.packed(operands[0], true);
                state.pc = end + 1;
                state.call(mem, addr, Vec::new(), true)?;
            }
            0x09 => {
                state.pc = end;
                obj.remove(mem, operands[0])?;
            }
            0x0a => {
                state.pc = end;
                let string = obj.name(mem, text, operands[0])?;
                out.write(mem, text, interface, &string, 0)?;
            }
            0x0b => state.ret(mem, operands[0])?,
            0x0c => {
                if operands[0] & 0x8000 == 0 {
                    state.pc = end + operands[0] as usize - 2;
                } else {
                    state.pc = end - !operands[0] as usize - 3;
                }
            }
            0x0d => {
                state.pc = end;
                let addr = mem.packed(operands[0], false);
                let string = trace(text.decode(mem, addr), Trace::String(addr))?;
                out.write(mem, text, interface, &string.0, 0)?;
            }
            0x0e => {
                state.pc = end + 1;
                let val = if operands[0] == 0 {
                    state.get_inplace()?
                } else {
                    state.get_var(mem, operands[0])?
                };
                instr.store(mem, state, val)?;
            }
            0x0f if v < 5 => {
                state.pc = end + 1;
                instr.store(mem, state, !operands[0])?;
            }
            0x0f => {
                let addr = mem.packed(operands[0], true);
                state.pc = end;
                state.call(mem, addr, Vec::new(), false)?;
            }
            _ => return fatal(error(Cause::BadOpcode, (count.into(), opcode.into()))),
        },
        2 => match opcode {
            0x01 => {
                let mut eq = false;
                for i in 1..operands.len() {
                    eq |= alu::eq(operands[0], operands[i]);
                }
                instr.branch(mem, state, eq, false)?;
            }
            0x02 => {
                let lt = alu::lt(operands[0], operands[1]);
                instr.branch(mem, state, lt, false)?;
            }
            0x03 => {
                let gt = alu::gt(operands[0], operands[1]);
                instr.branch(mem, state, gt, false)?;
            }
            0x04 => {
                let mut val = handle(state.get_var(mem, operands[0]), || {
                    instr.branch(mem, state, false, false)
                })?;
                val = alu::sub(val, 1);
                instr.branch(mem, state, alu::lt(val, operands[1]), false)?;
                state.set_var(mem, operands[0], val)?;
            }
            0x05 => {
                let mut val = handle(state.get_var(mem, operands[0]), || {
                    instr.branch(mem, state, false, false)
                })?;
                val = alu::add(val, 1);
                instr.branch(mem, state, alu::gt(val, operands[1]), false)?;
                state.set_var(mem, operands[0], val)?;
            }
            0x06 => {
                let parent = handle(obj.get_parent(mem, operands[0]), || {
                    instr.branch(mem, state, 0 == operands[1], false)
                })?;
                instr.branch(mem, state, parent == operands[1], false)?;
            }
            0x07 => {
                let flags = operands[1];
                instr.branch(mem, state, operands[0] & flags == flags, false)?;
            }
            0x08 => {
                state.pc = instr.end + 1;
                instr.store(mem, state, operands[0] | operands[1])?;
            }
            0x09 => {
                state.pc = end + 1;
                instr.store(mem, state, operands[0] & operands[1])?;
            }
            0x0a => {
                let attr = handle(obj.test_attr(mem, operands[0], operands[1]), || {
                    instr.branch(mem, state, false, false)
                })?;
                instr.branch(mem, state, attr, false)?;
            }
            0x0b => {
                state.pc = end;
                obj.set_attr(mem, operands[0], operands[1])?;
            }
            0x0c => {
                state.pc = end;
                obj.clear_attr(mem, operands[0], operands[1])?;
            }
            0x0d => {
                state.pc = end;
                if operands[0] == 0 {
                    state.set_inplace(operands[1])?;
                } else {
                    state.set_var(mem, operands[0], operands[1])?;
                }
            }
            0x0e => {
                state.pc = end;
                obj.insert(mem, operands[0], operands[1])?;
            }
            0x0f => {
                state.pc = end + 1;
                let val = mem.loadw(operands[0] + 2 * operands[1])?;
                instr.store(mem, state, val)?;
            }
            0x10 => {
                state.pc = end + 1;
                let val = mem.loadb(operands[0] + operands[1])?;
                instr.store(mem, state, val)?;
            }
            0x11 => {
                state.pc = end + 1;
                let prop = obj.get_prop(mem, operands[0], operands[1])?;
                instr.store(mem, state, prop)?;
            }
            0x12 => {
                state.pc = end + 1;
                let addr = obj.prop_addr(mem, operands[0], operands[1])?;
                instr.store(mem, state, addr)?;
            }
            0x13 => {
                state.pc = end + 1;
                let prop = obj.next_prop(mem, operands[0], operands[1])?;
                instr.store(mem, state, prop)?;
            }
            0x14 => {
                state.pc = end + 1;
                let result = alu::add(operands[0], operands[1]);
                instr.store(mem, state, result)?;
            }
            0x15 => {
                state.pc = end + 1;
                let result = alu::sub(operands[0], operands[1]);
                instr.store(mem, state, result)?;
            }
            0x16 => {
                state.pc = end + 1;
                let result = alu::mul(operands[0], operands[1]);
                instr.store(mem, state, result)?;
            }
            0x17 => {
                state.pc = end + 1;
                let result = alu::div(operands[0], operands[1])?;
                instr.store(mem, state, result)?;
            }
            0x18 => {
                state.pc = end + 1;
                let result = alu::rem(operands[0], operands[1])?;
                instr.store(mem, state, result)?;
            }
            0x19 if v >= 4 => {
                state.pc = end + 1;
                let addr = mem.packed(operands[0], true);
                let args = operands[1..].to_vec();
                state.call(mem, addr, args, true)?;
            }
            0x1a if v >= 5 => {
                state.pc = end;
                let addr = mem.packed(operands[0], true);
                let args = operands[1..].to_vec();
                state.call(mem, addr, args, false)?;
            }
            0x1b if v >= 5 => {
                state.pc = end;
                screen.color(interface, operands[0], operands[1])?;
            }
            0x1c if v >= 5 => {
                state.throw(mem, operands[1], operands[0])?;
            }
            _ => return fatal(error(Cause::BadOpcode, (count.into(), opcode.into()))),
        },
        3 => match opcode {
            0x00 => {
                state.pc = end + 1;
                let addr = mem.packed(*get(operands, 0)?, true);
                let args = operands[1..].to_vec();
                state.call(mem, addr, args, true)?;
            }
            0x01 => {
                state.pc = end;
                check(operands, 3)?;
                let addr = operands[0] + 2 * operands[1];
                header.checked_storew(mem, addr, operands[2])?;
            }
            0x02 => {
                state.pc = end;
                check(operands, 3)?;
                let addr = operands[0] + operands[1];
                header.checked_storeb(mem, addr, operands[2])?;
            }
            0x03 => {
                state.pc = end;
                check(operands, 3)?;
                obj.put_prop(mem, operands[0], operands[1], operands[2])?;
            }
            0x04 => {
                if v < 5 {
                    state.pc = end;
                    if v <= 3 {
                        screen.status(mem, text, state, obj, interface)?;
                    }
                } else {
                    state.pc = end + 1;
                }

                let (mut str, char) = input.read(
                    *get(operands, 0)?,
                    *operands.get(2).unwrap_or(&0),
                    *operands.get(3).unwrap_or(&0),
                    mem,
                    rand,
                    text,
                    state,
                    out,
                    screen,
                    header,
                    obj,
                    dict,
                    interface,
                    restart,
                    undo,
                    config,
                    err_said,
                )?;
                if let Some(&addr) = operands.get(1) {
                    dict.parse(mem, text, operands[0], addr, None, false)?;
                }
                str.push(char);
                out.write(mem, text, interface, &str, 1)?;

                if v >= 5 {
                    let char = text.to_zscii_char(mem, char)?;
                    instr.store(mem, state, char)?;
                }
            }
            0x05 => {
                state.pc = end;
                if let Some(char) = text.decode_char(mem, *get(operands, 0)? as u8)? {
                    out.write(mem, text, interface, &char.to_string(), 0)?;
                }
            }
            0x06 => {
                state.pc = end;
                out.write(
                    mem,
                    text,
                    interface,
                    &(*get(operands, 0)? as i16).to_string(),
                    0,
                )?;
            }
            0x07 => {
                state.pc = end + 1;
                instr.store(mem, state, rand.rand(*get(operands, 0)?))?;
            }
            0x08 => {
                state.pc = end;
                state.set_var(mem, 0, *get(operands, 0)?)?;
            }
            0x09 => {
                state.pc = end;
                let val = state.get_var(mem, 0)?;
                if *get(operands, 0)? == 0 {
                    state.set_inplace(val)?;
                } else {
                    state.set_var(mem, operands[0], val)?;
                }
            }
            0x0a if v >= 3 => {
                state.pc = end;
                screen.split_window(interface, *get(operands, 0)?);
            }
            0x0b if v >= 3 => {
                state.pc = end;
                screen.window(interface, *get(operands, 0)?);
            }
            0x0c if v >= 4 => {
                state.pc = end + 1;
                let addr = mem.packed(*get(operands, 0)?, true);
                let args = operands[1..].to_vec();
                state.call(mem, addr, args, true)?;
            }
            0x0d if v >= 4 => {
                state.pc = end;
                screen.erase_window(interface, *get(operands, 0)?);
            }
            0x0e if v >= 4 => {
                state.pc = end;
                if *get(operands, 0)? == 1 {
                    screen.erase_line(interface);
                }
            }
            0x0f if v >= 4 => {
                state.pc = end;
                check(operands, 2)?;
                screen.set_cursor(interface, operands[0], operands[1]);
            }
            0x10 if v >= 4 => {
                state.pc = end;
                let (row, col) = screen.get_cursor(interface);
                header.checked_storew(mem, *get(operands, 0)?, row)?;
                header.checked_storew(mem, operands[0] + 2, col)?;
            }
            0x11 if v >= 4 => {
                state.pc = end;
                screen.style(interface, *get(operands, 0)?);
            }
            0x12 if v >= 4 => {
                state.pc = end;
                screen.buffer(interface, *get(operands, 0)?);
            }
            0x13 if v >= 3 => {
                state.pc = end;
                out.select(mem, *get(operands, 0)?, operands.get(1))?;
            }
            0x14 if v >= 3 => {
                state.pc = end;
                input.set_stream(*get(operands, 0)?)?;
            }
            0x15 if v >= 5 => {
                state.pc = end;
                sound::bleep(interface, *get(operands, 0)?);
            }
            0x16 if v >= 4 => {
                state.pc = end + 1;
                let char = input.read_char(
                    *operands.get(1).unwrap_or(&0),
                    *operands.get(2).unwrap_or(&0),
                    mem,
                    rand,
                    text,
                    state,
                    out,
                    screen,
                    header,
                    obj,
                    dict,
                    interface,
                    restart,
                    undo,
                    config,
                    err_said,
                );
                out.write(mem, text, interface, &char.to_string(), 2)?;
                let char = text.to_zscii_char(mem, char)?;
                instr.store(mem, state, char)?;
            }
            0x17 if v >= 4 => {
                instr.branch(mem, state, false, true)?;
                check(operands, 3)?;
                let form = *operands.get(3).unwrap_or(&0x82);
                let size = form & 0x7f;
                let table = operands[1];
                if form & 0x80 != 0 {
                    for i in 0..operands[2] {
                        if mem.loadw(table + i * size)? == operands[0] {
                            instr.store(mem, state, table + i * size)?;
                            instr.branch(mem, state, true, true)?;
                            break;
                        }
                    }
                } else {
                    for i in 0..operands[2] {
                        if mem.loadb(table + i * size)? == operands[0] {
                            instr.store(mem, state, table + i * size)?;
                            instr.branch(mem, state, true, true)?;
                            break;
                        }
                    }
                }
            }
            0x18 if v >= 5 => {
                state.pc = end + 1;
                instr.store(mem, state, !*get(operands, 0)?)?;
            }
            0x19 if v >= 5 => {
                state.pc = end;
                let addr = mem.packed(*get(operands, 0)?, true);
                let args = operands[1..].to_vec();
                state.call(mem, addr, args, false)?;
            }
            0x1a if v >= 5 => {
                state.pc = end;
                let addr = mem.packed(*get(operands, 0)?, true);
                let args = operands[1..].to_vec();
                state.call(mem, addr, args, false)?;
            }
            0x1b if v >= 5 => {
                state.pc = end;
                check(operands, 2)?;
                let flag = operands.get(3).unwrap_or(&0) != &0;
                dict.parse(mem, text, operands[0], operands[1], operands.get(2), flag)?;
            }
            0x1c if v >= 5 => {
                state.pc = end;
                check(operands, 4)?;
                let result = text.encode(&mem, operands[0] + operands[2], operands[1])?;
                for (i, &res) in result.iter().enumerate() {
                    mem.storew(operands[3] + 2 * i as u16, res)?;
                }
            }
            0x1d if v >= 5 => {
                state.pc = end;
                check(operands, 3)?;
                let first = operands[0];
                let second = operands[1];
                let size = operands[2];
                if second == 0 {
                    for i in 0..size {
                        mem.storeb(first + i, 0)?;
                    }
                } else if size & 0x8000 == 0 {
                    if first > second {
                        for i in 0..size {
                            let val = mem.loadb(first + i)?;
                            mem.storeb(second + i, val)?;
                        }
                    } else {
                        for i in (0..size).rev() {
                            let val = mem.loadb(first + i)?;
                            mem.storeb(second + i, val)?;
                        }
                    }
                } else {
                    for i in 0..=!size {
                        let val = mem.loadb(first + i)?;
                        mem.storeb(second + i, val)?;
                    }
                }
            }
            0x1e if v >= 5 => {
                state.pc = end;
                check(operands, 2)?;
                let mut addr = operands[0];
                let width = operands[1];
                let height = *operands.get(2).unwrap_or(&1);
                let skip = *operands.get(3).unwrap_or(&0);
                for i in 0..height {
                    if i != 0 {
                        out.write(mem, text, interface, "\n", 0)?;
                    }
                    for _ in 0..width {
                        let char = mem.loadb(addr)? as u8;
                        if let Some(char) = text.decode_char(mem, char)? {
                            out.write(mem, text, interface, &format!("{}", char), 0)?;
                        }
                        addr += 1;
                    }
                    // let y = screen.get_cursor(interface).1;
                    // screen.set_cursor(interface, x, y + 1);
                    addr += skip;
                }
            }
            0x1f if v >= 5 => {
                let args = state.arg_count()?;
                instr.branch(mem, state, args >= *get(operands, 0)?, false)?;
            }
            _ => return fatal(error(Cause::BadOpcode, (count.into(), opcode.into()))),
        },
        4 if v >= 5 => match opcode {
            0x00 => {
                let result = if operands.is_empty() {
                    state.pc = end;
                    let save = state.save(mem)?;
                    interface.save(&quetzal::save(mem, restart, &save, header.checksum))
                } else {
                    false
                };
                state.pc = end + 1;
                instr.store(mem, state, if result { 1 } else { 0 })?;
            }
            0x01 => {
                let save = quetzal::restore(restart, &interface.restore(), header.checksum);
                match save {
                    Ok(save) => {
                        state.restore(mem, &save);
                        state.pc += 1;
                        header::init(mem, config)?;
                        let var = mem[state.pc];
                        state.set_var(mem, var.into(), 2)?;
                    }
                    Err(cause) => {
                        interface.restore_failed(cause);
                        state.pc = end + 1;
                        instr.store(mem, state, 0)?;
                    }
                }
            }
            0x02 => {
                state.pc = end + 1;
                let val = alu::log(*get(operands, 0)?, *get(operands, 1)?);
                instr.store(mem, state, val)?;
            }
            0x03 => {
                state.pc = end + 1;
                let val = alu::art(*get(operands, 0)?, *get(operands, 1)?);
                instr.store(mem, state, val)?;
            }
            0x04 => {
                state.pc = end + 1;
                let font = screen.font(interface, *get(operands, 0)?);
                instr.store(mem, state, font)?;
            }
            0x09 => {
                state.pc = end + 1;
                undo[0] = state.save(mem)?;
                instr.store(mem, state, 1)?;
            }
            0x0a => {
                state.restore(mem, &undo[0]);
                let var = *fatal(
                    mem.get(state.pc - 1)
                        .ok_or_else(|| err(Cause::PcOut, (0, 0))),
                )?;
                state.set_var(mem, var.into(), 2)?;
            }
            0x0b => {
                use std::char;
                state.pc = end;
                let char = *get(operands, 0)?;
                let char = char::from_u32(char.into())
                    .ok_or_else(|| err(Cause::BadUnicodeChar, (char, 0)))?;
                out.write(mem, text, interface, &char.to_string(), 0)?;
            }
            0x0c => {
                use std::char;
                state.pc = end + 1;
                let check = if let Some(char) = char::from_u32(u32::from(*get(operands, 0)?)) {
                    if char == '?' || text.to_zscii_char(mem, char)? != 0x3f {
                        3
                    } else {
                        1
                    }
                } else {
                    0
                };
                instr.store(mem, state, check)?;
            }
            0x0d => {
                state.pc = end;
                screen.true_color(interface, *get(operands, 0)?, *get(operands, 1)?);
            }
            _ => {
                state.pc = end;
                return error(Cause::BadOpcode, (count.into(), opcode.into()));
            }
        },
        _ => unreachable!(),
    }
    Ok(())
}

fn get(operands: &[u16], i: usize) -> Result<&u16, Error> {
    operands
        .get(i)
        .ok_or_else(|| err(Cause::MissingOperand, (i as u16 + 1, operands.len() as u16)))
}

fn check(operands: &[u16], min: usize) -> Result<(), Error> {
    if operands.len() < min {
        error(Cause::MissingOperand, (min as u16, operands.len() as u16))
    } else {
        Ok(())
    }
}

fn handle<T, F>(result: Result<T, Error>, fallback: F) -> Result<T, Error>
where
    F: FnOnce() -> Result<(), Error>,
{
    if result.is_err() {
        fallback().or_else(|err| if err.fatal { Err(err) } else { Ok(()) })?;
    }
    result
}
