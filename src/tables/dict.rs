use crate::{err::*, mem::Mem, text::Text};
use std::cmp::Ordering;

pub struct Dict {
    v: u8,
    addr: u16,
}

impl Dict {
    pub fn parse(
        &self,
        mem: &mut Mem,
        text: &Text,
        mut entry: u16,
        mut parse: u16,
        dict: Option<&u16>,
        flag: bool,
    ) -> Result<(), Error> {
        if parse == 0 {
            return Ok(());
        }
        let parse_size = mem.loadb(parse)? as usize;
        parse += 1;
        if parse_size == 0 {
            mem.storeb(parse, 0)?;
            return Ok(());
        }

        let mut dict = match dict {
            Some(&dict) => dict,
            None => self.addr,
        };
        let input_size = mem.loadb(dict)?;
        let mut input = Vec::with_capacity(input_size as usize);
        dict += 1;
        for i in 0..input_size {
            input.push(mem.loadb(dict + i)?);
        }
        dict += input_size;

        if self.v > 4 {
            entry += 1;
        }
        let len = mem.loadb(entry)? + 2;
        let mut end = entry;
        let mut begin = entry + 1;
        let mut words = Vec::new();
        if self.v > 4 {
            entry -= 1;
        }

        loop {
            end += 1;
            let char = mem.loadb(end)?;
            if self.v <= 4 {
                if char == 0 {
                    break;
                }
            } else if end - entry >= len {
                break;
            }
            if char == 0x20 {
                if begin != end {
                    let len = end - begin;
                    words.push((text.encode(mem, begin, len)?, begin - entry, len));
                }
                begin = end + 1;
                continue;
            }
            for &c in input.iter() {
                if char == c {
                    if begin != end {
                        let len = end - begin;
                        words.push((text.encode(mem, begin, len)?, begin - entry, len));
                    }
                    words.push((text.encode(mem, end, 1)?, end - entry, 1));
                    begin = end + 1;
                    break;
                }
            }
        }
        if begin != end {
            let len = end - begin;
            words.push((text.encode(mem, begin, len)?, begin - entry, len));
        }
        words.truncate(parse_size);

        mem.storeb(parse, words.len() as u16)?;
        parse += 1;

        let size = mem.loadb(dict)?;
        dict += 1;
        let count = mem.loadw(dict)?;
        dict += 2;
        if count & 0x8000 != 0 {
            let count = -(count as i16) as u16;
            let mut dict_buf = Vec::with_capacity(count as usize);
            if self.v <= 3 {
                for i in 0..count {
                    let i = dict + i * size;
                    dict_buf.push((vec![mem.loadw(i)?, mem.loadw(i + 2)?], i));
                }
            } else {
                for i in 0..count {
                    let i = dict + i * size;
                    dict_buf.push((vec![mem.loadw(i)?, mem.loadw(i + 2)?, mem.loadw(i + 4)?], i));
                }
            }
            for &(ref word, addr, len) in words.iter() {
                if !flag {
                    mem.storew(parse, 0)?;
                }
                for entry in dict_buf.iter() {
                    if word == &entry.0 {
                        mem.storew(parse, entry.1)?;
                        break;
                    }
                }
                parse += 2;
                mem.storeb(parse, len)?;
                parse += 1;
                mem.storeb(parse, addr)?;
                parse += 1;
            }
        } else {
            for &(ref word, addr, len) in words.iter() {
                let mut min = 0;
                let mut max = count;
                if !flag {
                    mem.storew(parse, 0)?;
                }
                while min <= max {
                    let nbr = (min + max) / 2;
                    let addr = dict + nbr * size;
                    let entry = if self.v <= 3 {
                        vec![mem.loadw(addr)?, mem.loadw(addr + 2)?]
                    } else {
                        vec![mem.loadw(addr)?, mem.loadw(addr + 2)?, mem.loadw(addr + 4)?]
                    };
                    match word.cmp(&entry) {
                        Ordering::Less => {
                            if nbr == 0 {
                                break;
                            } else {
                                max = nbr - 1;
                            }
                        }
                        Ordering::Equal => {
                            mem.storew(parse, addr)?;
                            break;
                        }
                        Ordering::Greater => min = nbr + 1,
                    }
                }
                parse += 2;
                mem.storeb(parse, len)?;
                parse += 1;
                mem.storeb(parse, addr)?;
                parse += 1;
            }
        }
        Ok(())
    }
}

pub fn init(mem: &Mem) -> Dict {
    Dict {
        v: mem[0],
        addr: mem.loadw(0x08).unwrap(),
    }
}

#[cfg(test)]
use crate::{header, mem, text};

#[test]
fn test_old() {
    let mut data = mem::default();
    data[0x09] = 0x76;
    data[0x0f] = 0x5e;
    data.extend(vec![0; 0x1e]);
    data[0x40] = 0x07;
    data[0x46] = 0xa5;
    data[0x4f] = 0x5a;
    data.push(0x16);
    data.extend(b"abca. bcda ,abcd abc\0. ".to_vec());
    data.extend(vec![
        0x03, 0x2e, 0x2c, 0x27, 0x06, 0x00, 0x04, 0x18, 0xe8, 0x94, 0xa5, 0xff, 0x00, 0x18, 0xe8,
        0x98, 0xa5, 0x00, 0xff, 0x18, 0xe8, 0xa4, 0xa5, 0xaa, 0x55, 0x1d, 0x09, 0x98, 0xa5, 0x55,
        0xaa,
    ]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = text::init(&mem, &header).unwrap();
    let dict = init(&mem);
    dict.parse(&mut mem, &text, 0x5e, 0x40, None, false)
        .unwrap();

    assert_eq!(
        mem.save()[0x40..0x5e],
        [
            0x07, 0x06, 0x00, 0x83, 0x04, 0x01, 0x00, 0x00, 0x01, 0x05, 0x00, 0x8f, 0x04, 0x07,
            0x00, 0x00, 0x01, 0x0c, 0x00, 0x89, 0x04, 0x0d, 0x00, 0x7d, 0x03, 0x12, 0x00, 0x00,
            0x00, 0x00
        ]
    );
}

#[test]
fn test_new() {
    let mut data = mem::default();
    data[0x00] = 0x05;
    data[0x0f] = 0x5e;
    data.extend(vec![0; 0x1e]);
    data[0x40] = 0x06;
    data[0x46] = 0xa5;
    data[0x53] = 0x5a;
    data.extend(vec![0x16, 0x16]);
    data.extend(b"abca,bcda  abcd . abc'".to_vec());
    data.extend(vec![
        0x03, 0x2e, 0x2c, 0x27, 0x06, 0xff, 0xfc, 0x18, 0xe8, 0x18, 0xa5, 0x94, 0xa5, 0x18, 0xe8,
        0x14, 0xa5, 0x94, 0xa5, 0x1d, 0x09, 0x18, 0xa5, 0x94, 0xa5, 0x18, 0xe8, 0x24, 0xa5, 0x94,
        0xa5,
    ]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = text::init(&mem, &header).unwrap();
    let dict = init(&mem);
    dict.parse(&mut mem, &text, 0x5e, 0x40, Some(&0x76), true)
        .unwrap();

    assert_eq!(
        mem.save()[0x40..0x5e],
        [
            0x06, 0x06, 0x00, 0x7d, 0x04, 0x02, 0xa5, 0x00, 0x01, 0x06, 0x00, 0x89, 0x04, 0x07,
            0x00, 0x8f, 0x04, 0x0d, 0x00, 0x5a, 0x01, 0x12, 0x00, 0x83, 0x03, 0x14, 0x00, 0x00,
            0x00, 0x00
        ]
    );
}
