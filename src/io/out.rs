use crate::{err::*, interface::Interface, mem::Mem, text::Text};

pub struct Output {
    s1: bool,
    s3: Vec<(u16, u16)>,
    s4: bool,
}

impl Output {
    pub fn write<I: Interface>(
        &mut self,
        mem: &mut Mem,
        encode: &Text,
        output: &mut I,
        text: &str,
        type_: u8,
    ) -> Result<(), Error> {
        match type_ {
            0 => {
                if self.s3.is_empty() {
                    // @print_*
                    if self.s1 {
                        output.write_screen(text, mem[0x11] & 0x02 != 0);
                    }
                    if mem[0x11] & 0x01 != 0 {
                        output.write_transcript(text);
                    }
                } else {
                    let addr = self.s3.last_mut().unwrap();
                    addr.1 = encode.to_zscii(mem, addr.1, text)?;
                }
            }
            1 => {
                // @read
                if mem[0x11] & 0x01 != 0 {
                    output.write_transcript(text);
                }
                if self.s4 {
                    output.write_command(&format!("{}\n", text));
                }
            }
            2 => {
                if self.s4 {
                    // @read_char
                    output.write_command(&format!("{}\n", text));
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn select(&mut self, mem: &mut Mem, stream: u16, table: Option<&u16>) -> Result<(), Error> {
        if stream & 0x8000 == 0 {
            match stream {
                1 => self.s1 = true,
                2 => {
                    let old = mem.loadb(0x11).unwrap();
                    mem.storeb(0x11, old | 0x01).unwrap();
                }
                3 => {
                    if self.s3.len() >= 16 {
                        return error(Cause::OutputS3Overflow, (0, 0));
                    }
                    let &table = table.ok_or_else(|| err(Cause::MissingOperand, (2, 1)))?;
                    self.s3.push((table, table + 2));
                }
                4 => self.s4 = true,
                _ => return error(Cause::BadOutputStream, (stream, 0)),
            }
        } else {
            match !stream + 1 {
                1 => self.s1 = false,
                2 => {
                    let old = mem.loadb(0x11).unwrap();
                    mem.storeb(0x11, old & 0xfe).unwrap();
                }
                3 => {
                    let (begin, end) = self
                        .s3
                        .pop()
                        .ok_or_else(|| err(Cause::NoOutputS3, (0, 0)))?;
                    mem.storew(begin, end - begin - 2)?;
                }
                4 => self.s4 = false,
                _ => return error(Cause::BadOutputStream, (stream, 0)),
            }
        }
        Ok(())
    }
}

pub fn init() -> Output {
    Output {
        s1: true,
        s3: Vec::new(),
        s4: false,
    }
}

#[cfg(test)]
struct Out(String, String, String);

#[cfg(test)]
impl Interface for Out {
    fn write_screen(&mut self, str: &str, _: bool) {
        self.0 += str;
    }
    fn write_transcript(&mut self, str: &str) {
        self.1 += str;
    }
    fn write_command(&mut self, str: &str) {
        self.2 += str;
    }
}

#[cfg(test)]
use crate::{header, mem, text};

#[test]
fn test_out() {
    let data = mem::default();
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = text::init(&mem, &header).unwrap();
    let mut out = init();
    let mut output = Out(String::new(), String::new(), String::new());

    out.write(&mut mem, &text, &mut output, "A0", 0).unwrap();
    out.write(&mut mem, &text, &mut output, "A1", 1).unwrap();
    out.write(&mut mem, &text, &mut output, "A2", 2).unwrap();

    out.select(&mut mem, 0xffff, None).unwrap();
    out.select(&mut mem, 2, None).unwrap();
    out.select(&mut mem, 4, None).unwrap();
    assert_eq!(mem.loadb(0x11).unwrap() & 1, 1);

    out.write(&mut mem, &text, &mut output, "B0", 0).unwrap();
    out.write(&mut mem, &text, &mut output, "B1", 1).unwrap();
    out.write(&mut mem, &text, &mut output, "B2", 2).unwrap();

    out.select(&mut mem, 1, None).unwrap();
    out.select(&mut mem, 0xfffe, None).unwrap();
    out.select(&mut mem, 0xfffc, None).unwrap();
    assert_eq!(mem.loadb(0x11).unwrap() & 1, 0);

    out.write(&mut mem, &text, &mut output, "C0", 0).unwrap();
    out.write(&mut mem, &text, &mut output, "C1", 1).unwrap();
    out.write(&mut mem, &text, &mut output, "C2", 2).unwrap();

    assert_eq!(&output.0, "A0C0");
    assert_eq!(&output.1, "B0B1");
    assert_eq!(&output.2, "B1\nB2\n");
}

#[test]
fn test_s3() {
    let mut data = mem::default();
    data[0x0f] = 0x47;
    data.extend(vec![0; 7]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = text::init(&mem, &header).unwrap();
    let mut out = init();
    let mut output = Out(String::new(), String::new(), String::new());

    out.write(&mut mem, &text, &mut output, "0", 0).unwrap();
    out.select(&mut mem, 3, Some(&0x40)).unwrap();
    out.write(&mut mem, &text, &mut output, "1", 0).unwrap();
    out.select(&mut mem, 3, Some(&0x44)).unwrap();
    out.write(&mut mem, &text, &mut output, "2", 0).unwrap();
    out.select(&mut mem, 0xfffd, None).unwrap();
    out.write(&mut mem, &text, &mut output, "3", 0).unwrap();
    out.select(&mut mem, 0xfffd, None).unwrap();
    out.write(&mut mem, &text, &mut output, "4", 0).unwrap();

    assert_eq!(&output.0, "04");
    assert_eq!(mem.loadw(0x40).unwrap(), 0x02);
    assert_eq!(mem.loadw(0x42).unwrap(), 0x3133);
    assert_eq!(mem.loadw(0x44).unwrap(), 0x01);
    assert_eq!(mem.loadb(0x46).unwrap(), 0x32);
}
