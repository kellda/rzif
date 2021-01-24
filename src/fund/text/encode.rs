use super::*;

impl Text {
    pub fn to_zscii(&self, mem: &mut Mem, mut addr: u16, text: &str) -> Result<u16, Error> {
        for char in text.chars() {
            let char = self.to_zscii_char(mem, char)?;
            mem.storeb(addr, char)?;
            addr += 1;
        }
        Ok(addr)
    }

    pub fn to_zscii_char(&self, mem: &Mem, char: char) -> Result<u16, Error> {
        match char {
            '\n' => Ok(13),
            '\x08' | '\x1b' | ' '..='~' | '\u{81}'..='\u{9a}' => Ok(char as u16),
            _ => {
                match self.unicode {
                    Some(table) => {
                        let char = char as u16;
                        let len = *mem
                            .get(table)
                            .ok_or_else(|| err(Cause::UnicodeOut, (table as u16, 0)))?
                            as usize;
                        if table + len > mem.len() {
                            return error(Cause::UnicodeOut, (table as u16, 0));
                        }
                        for i in 0..len {
                            if char
                                == mem
                                    .getw(table + 1 + 2 * i)
                                    .ok_or_else(|| err(Cause::UnicodeOut, (table as u16, 0)))?
                            {
                                return Ok(i as u16 + 155);
                            }
                        }
                    }
                    None => {
                        for (i, &c) in UNICODE.iter().enumerate() {
                            if c == char {
                                return Ok(i as u16 + 155);
                            }
                        }
                    }
                }
                Ok(0x3f)
            }
        }
    }

    pub fn encode(&self, mem: &Mem, addr: u16, len: u16) -> Result<Vec<u16>, Error> {
        let mut bytes = Vec::new();
        if self.v < 3 {
            let mut state = (true, (0, 0), 0);
            for i in addr..addr + len {
                let prev = state.1;
                let char = mem.loadb(i)? as u8;
                state.1 = self.char(mem, char);
                if state.0 {
                    state.0 = false;
                    continue;
                }
                let prev_alph = match prev.0 {
                    0..=2 => prev.0,
                    3 => 2,
                    4 => state.2,
                    _ => unreachable!(),
                };
                if prev_alph == state.2 {
                    bytes.push(prev.1);
                    continue;
                }
                let cur_alph = match (state.1).0 {
                    0..=2 => (state.1).0,
                    3 => 2,
                    4 => state.2,
                    _ => unreachable!(),
                };
                let shift = (prev_alph + 3 - state.2) % 3;
                if prev_alph == cur_alph {
                    bytes.push(shift + 3);
                    state.2 = prev_alph;
                } else {
                    bytes.push(shift + 1);
                }
                if prev.0 == 4 {
                    bytes.push(6);
                    bytes.push(prev.1 >> 5);
                    bytes.push(prev.1 & 0x1f);
                } else {
                    bytes.push(prev.1);
                }
            }
            let prev = state.1;
            let prev_alph = match prev.0 {
                0..=2 => prev.0,
                3 => 2,
                4 => state.2,
                _ => unreachable!(),
            };
            if prev_alph == state.2 {
                bytes.push(prev.1);
            } else {
                let shift = (prev_alph + 3 - state.2) % 3;
                bytes.push(shift + 1);
                if prev.0 == 4 {
                    bytes.push(6);
                    bytes.push(prev.1 >> 5);
                    bytes.push(prev.1 & 0x1f);
                } else {
                    bytes.push(prev.1);
                }
            }
        } else {
            for i in addr..addr + len {
                let char = mem.loadb(i)? as u8;
                match self.char(mem, char) {
                    (0, char) | (4, char) => bytes.push(char),
                    (1, char) => {
                        bytes.push(4);
                        bytes.push(char);
                    }
                    (2, char) => {
                        bytes.push(5);
                        bytes.push(char);
                    }
                    (3, char) => {
                        bytes.push(5);
                        bytes.push(6);
                        bytes.push(char >> 5);
                        bytes.push(char & 0x1f);
                    }
                    _ => unreachable!(),
                }
            }
        }

        let len = if self.v <= 3 { 2 } else { 3 };
        bytes.resize(len * 3, 5);
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            result.push(
                u16::from(bytes[3 * i]) << 10
                    | u16::from(bytes[3 * i + 1]) << 5
                    | u16::from(bytes[3 * i + 2]),
            );
        }
        result[len - 1] |= 0x8000;
        Ok(result)
    }

    fn char(&self, mem: &Mem, char: u8) -> (u8, u8) {
        if char == 0x0d {
            if self.v == 1 {
                return (4, 1);
            } else {
                return (2, 7);
            }
        }
        if char == 0x20 {
            return (4, 0);
        };

        match self.table {
            Some(table) => {
                for i in 0..78 {
                    if mem[table + i] == char {
                        let i = i as u8;
                        return (i / 26, i % 26 + 6);
                    }
                }
            }
            None => {
                for (id, alphabet) in [A0, A1, if self.v == 1 { A2V1 } else { A2 }]
                    .iter()
                    .enumerate()
                {
                    for (i, &c) in alphabet.iter().enumerate() {
                        if c == char {
                            return (id as u8, i as u8 + 6);
                        }
                    }
                }
            }
        }
        (3, char)
    }
}

#[test]
fn test_to_zscii() {
    let mut data = mem::default();
    data[0x00] = 5;
    data[0x0f] = 0x4c;
    data.extend(vec![0; 0x0c]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(
        text.to_zscii(&mut mem, 0x40, "a b\nc d\nefä¿").unwrap(),
        0x4c
    );
    assert_eq!(mem.loadw(0x40).unwrap(), 0x6120);
    assert_eq!(mem.loadw(0x42).unwrap(), 0x620d);
    assert_eq!(mem.loadw(0x44).unwrap(), 0x6320);
    assert_eq!(mem.loadw(0x46).unwrap(), 0x640d);
    assert_eq!(mem.loadw(0x48).unwrap(), 0x6566);
    assert_eq!(mem.loadw(0x4a).unwrap(), 0x9bdf);
}

#[test]
fn test_char() {
    let mut data = mem::default();
    data[0] = 3;
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(text.char(&mem, 0x20), (4, 0));
    assert_eq!(text.char(&mem, 0x0D), (2, 7));
    assert_eq!(text.char(&mem, 0x61), (0, 6));
    assert_eq!(text.char(&mem, 0x30), (2, 8));
    assert_eq!(text.char(&mem, 0x9b), (3, 0x9b));
}

#[test]
fn test_encode_old() {
    let mut data = mem::default();
    data.extend(b"a0b".to_vec());
    data.extend(b"abc012".to_vec());
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(text.encode(&mem, 0x40, 3).unwrap(), vec![0x1867, 0x9ca5]);
    assert_eq!(text.encode(&mem, 0x43, 6).unwrap(), vec![0x18e8, 0x94e8]);
}

#[test]
fn test_encode_new() {
    let mut data = mem::default();
    data[0x00] = 4;
    data.extend(b"a;1".to_vec());
    data.extend(b"abc12;".to_vec());
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(
        text.encode(&mem, 0x40, 3).unwrap(),
        vec![0x18a6, 0x0765, 0xa4a5]
    );
    assert_eq!(
        text.encode(&mem, 0x43, 6).unwrap(),
        vec![0x18e8, 0x1525, 0xa8a6]
    );
}
