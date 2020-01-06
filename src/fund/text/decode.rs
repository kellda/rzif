use super::*;

impl Text {
    fn zscii(&self, mem: &Mem, mut addr: usize, abbr: bool) -> Result<(Vec<u8>, usize), Error> {
        let mut data = Vec::new();
        loop {
            let high = mem.get(addr).ok_or_else(|| err(Cause::StrOut, (0, 0)))?;
            let low = mem
                .get(addr + 1)
                .ok_or_else(|| err(Cause::StrOut, (0, 0)))?;
            data.push((high & 0x7c) >> 2);
            data.push((high & 0x03) << 3 | (low & 0xe0) >> 5);
            data.push(low & 0x1f);
            addr += 2;
            if (high & 0x80) != 0 {
                break;
            }
        }

        let mut zscii = Vec::new();
        let mut state = (0, 0, 0);
        for (i, &char) in data.iter().enumerate() {
            match state.0 {
                // normal
                0 => {
                    match char {
                        2 | 3 if self.v < 3 => state.1 = (state.1 + char - 1) % 3,
                        4 | 5 if self.v < 3 => state.2 = (state.1 + char - 3) % 3,
                        4 | 5 => state.1 = (char - 3) % 3,
                        0 => zscii.push(b' '),
                        1 if self.v == 1 => zscii.push(b'\r'),
                        1..=3 => {
                            if abbr {
                                return error(Cause::NestedAbbr, (0, 0));
                            }
                            state.0 = 1;
                        }
                        6 if state.1 == 2 => state.0 = 2,
                        _ => zscii.push(match self.table {
                            Some(table) => {
                                if char == 7 && state.1 == 2 {
                                    b'\r'
                                } else {
                                    let index = 26 * state.1 + char - 6;
                                    mem[table + index as usize]
                                }
                            }
                            None => {
                                let alphabet = match state.1 {
                                    0 => A0,
                                    1 => A1,
                                    2 if self.v == 1 => A2V1,
                                    2 => A2,
                                    _ => unreachable!(),
                                };
                                alphabet[char as usize - 6]
                            }
                        }),
                    }
                    match char {
                        2 | 3 if self.v < 3 => {}
                        4 | 5 if self.v >= 3 => {}
                        _ => state.1 = state.2,
                    }
                }
                // abbreviation
                1 => {
                    let abbr = 32 * (data[i - 1] - 1) + char;
                    let addr = mem.word(mem.loadw(self.abbr + 2 * u16::from(abbr))?);
                    zscii.append(&mut trace(self.zscii(mem, addr, true), Trace::Abbr(addr))?.0);
                    state.0 = 0;
                }
                // ZSCII
                2 => state.0 = 3,
                3 => {
                    let high = data[i - 1];
                    if high & 0xe0 != 0 {
                        return error(
                            Cause::BadZSCIIChar,
                            (u16::from(high) << 5 | u16::from(char), 0),
                        );
                    }
                    zscii.push(high << 5 | char);
                    state.0 = 0;
                }
                _ => unreachable!(),
            }
        }
        if abbr && state.0 != 0 {
            return error(Cause::AbbrIncompleteZSCII, (0, 0));
        }
        Ok((zscii, addr))
    }

    pub fn decode(&self, mem: &Mem, addr: usize) -> Result<(String, usize), Error> {
        let mut result = String::new();
        let (zscii, end) = self.zscii(&mem, addr, false)?;
        for &char in zscii.iter() {
            if let Some(char) = self.decode_char(mem, char)? {
                result.push(char);
            }
        }
        Ok((result, end))
    }

    pub fn decode_char(&self, mem: &Mem, char: u8) -> Result<Option<char>, Error> {
        use std::char;

        match char {
            0 => Ok(None),
            13 => Ok(Some('\n')),
            32..=126 => Ok(Some(char as char)),
            155..=251 => Ok(Some(match self.unicode {
                Some(addr) => {
                    let char = char - 155;
                    if char
                        >= *mem
                            .get(addr)
                            .ok_or_else(|| err(Cause::UnicodeOut, (addr as u16, 0)))?
                    {
                        return error(Cause::BadZSCIIChar, (char.into(), 0));
                    }
                    let char = mem
                        .getw(addr + 1 + char as usize * 2)
                        .ok_or_else(|| err(Cause::UnicodeOut, (addr as u16, 0)))?;
                    char::from_u32(char.into())
                        .ok_or_else(|| err(Cause::BadUnicodeChar, (char, 0)))?
                }
                None => UNICODE[char as usize - 155],
            })),
            _ => error(Cause::BadZSCIIChar, (char.into(), 0)),
        }
    }
}

#[test]
fn test_zscii_v1() {
    let mut data = mem::default();
    let mut str = vec![0x14, 0xe2, 0x21, 0x23, 0x29, 0x61];
    data.extend(&str);
    str[0] = 0x10;
    data.extend(&str);
    str[4] = 0xa9;
    data.extend(&str);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(
        text.zscii(&mem, 0x40, false).unwrap(),
        (b"0c2E4\rbCd3f\rB1DeF\r".to_vec(), 0x52)
    );
}

#[test]
fn test_zscii_alphabet() {
    let mut data = mem::default();
    data[0x00] = 5;
    data[0x35] = 0x40;
    data.extend(vec!['1' as u8, '2' as u8]);
    data.extend(vec![0; 24]);
    data.extend(vec!['3' as u8, '4' as u8]);
    data.extend(vec![0; 24]);
    data.extend(vec!['5' as u8, '6' as u8]);
    data.extend(vec![0; 24]);
    data.extend(vec![
        0x18, 0xe0, 0x10, 0xc4, 0x1c, 0xa7, 0x14, 0xc1, 0xd4, 0xa5,
    ]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(
        text.zscii(&mem, 0x8e, false).unwrap(),
        (b"12 34\r5".to_vec(), 0x98)
    );
}

#[test]
fn test_zscii_abbr() {
    let mut data = mem::default();
    data.extend(vec![0; 0x44]);
    data[0x00] = 3;
    data[0x19] = 0x40;
    data[0x41] = 0x42;
    data[0x83] = 0x44;
    data.extend(vec![
        0x11, 0xaa, 0xc6, 0x34, 0x13, 0x94, 0xde, 0x29, 0x04, 0x00, 0x88, 0x25,
    ]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(
        text.zscii(&mem, 0x8c, false).unwrap(),
        (b"Hello World".to_vec(), 0x90)
    );
}

#[test]
fn test_decode() {
    let mut data = mem::default();
    data[0x00] = 5;
    data.extend(vec![
        0x18, 0x07, 0x14, 0xe8, 0x01, 0x25, 0x1d, 0x4b, 0x14, 0xc4, 0x6c, 0xa6, 0x9b, 0xe5,
    ]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(
        text.decode(&mem, 0x40).unwrap(),
        ("a b\nc d\nefä¿".to_string(), 0x4e)
    );
}

#[test]
fn test_unicode() {
    let mut data = mem::default();
    data[0x00] = 5;
    data[0x37] = 0x40;
    data.extend(vec![0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x48]);
    data.extend(vec![
        0x02, 0x21, 0xaf, 0x21, 0xbb, 0x14, 0xc4, 0x6c, 0xa6, 0x93, 0x85,
    ]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(
        text.decode(&mem, 0x4d).unwrap(),
        ("↯↻".to_string(), 0x53)
    );
}
