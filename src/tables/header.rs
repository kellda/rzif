use crate::{err::*, interface::Config, mem::Mem};

pub struct Header {
    pub checksum: u16,
    extension: (u16, u16),
}

impl Header {
    pub fn checked_storeb(&self, mem: &mut Mem, addr: u16, data: u16) -> Result<(), Error> {
        if addr > 0x40 {
            mem.storeb(addr, data)?;
        } else if addr == 0x11 {
            let flags = mem.loadb(0x11).unwrap() & 0xf8;
            mem.storeb(0x11, flags ^ (data & 0x07))?
        }
        Ok(())
    }

    pub fn checked_storew(&self, mem: &mut Mem, addr: u16, data: u16) -> Result<(), Error> {
        if addr > 0x40 {
            mem.storew(addr, data)?;
        } else {
            match addr {
                0x10 => {
                    let flags = mem.loadb(0x11).unwrap() & 0xf8;
                    mem.storeb(0x11, flags ^ (data & 0x07))?;
                }
                0x11 => {
                    let flags = mem.loadb(0x11).unwrap() & 0xf8;
                    mem.storeb(0x11, flags ^ (data >> 8 & 0x07)).unwrap();
                }
                0x40 => mem.storeb(0x41, data)?,
                _ => {}
            }
        }
        Ok(())
    }

    pub fn get_extension(&self, mem: &Mem, word: u16) -> Result<u16, Error> {
        if word <= self.extension.1 && self.extension.0 != 0 {
            mem.loadw(self.extension.0 + word * 2)
        } else {
            Ok(0)
        }
    }

    pub fn set_extension(&self, mem: &mut Mem, word: u16, data: u16) -> Result<(), Error> {
        if word <= self.extension.1 && self.extension.0 != 0 {
            mem.storew(self.extension.0 + word * 2, data)
        } else {
            Ok(())
        }
    }
}

pub fn init(mem: &mut Mem, config: &Config) -> Result<Header, Error> {
    let v = mem[0];
    let mut checksum = 0u16;
    let len = match v {
        1..=3 => 2,
        4..=5 => 4,
        7..=8 => 8,
        _ => unreachable!(),
    } * mem.loadw(0x1a).unwrap() as usize;
    for &m in mem.iter().take(len).skip(0x40) {
        checksum = checksum.wrapping_add(u16::from(m));
    }
    let ext_addr = mem.loadw(0x36).unwrap();
    let ext_len = mem.loadw(ext_addr)?;

    if v <= 3 {
        let mut flags = mem.loadb(0x01).unwrap() & 0x8f;
        if !config.status {
            flags |= 0x10;
        }
        if config.split {
            flags |= 0x20;
        }
        if config.fixed_default {
            flags |= 0x40;
        }
        mem.storeb(0x01, flags)?;
    } else {
        let mut flags = mem.loadb(0x01).unwrap() & 0x60;
        if config.color {
            flags |= 0x01;
        }
        if config.bold {
            flags |= 0x04;
        }
        if config.italic {
            flags |= 0x08;
        }
        if config.fixed {
            flags |= 0x10;
        }
        if config.timed {
            flags |= 0x80;
        }
        mem.storeb(0x01, flags)?;
    }

    let mut flags = mem.loadb(0x11).unwrap();
    flags &= if v >= 5 && !config.picture {
        0x57
    } else {
        0x5f
    };
    mem.storeb(0x11, flags)?;
    mem.storeb(0x1e, 0x01)?;
    if v >= 4 {
        mem.storeb(0x20, config.screen.1)?;
        mem.storeb(0x21, config.screen.0)?;
    }
    if v >= 5 {
        mem.storew(0x22, config.screen.0)?;
        mem.storew(0x24, config.screen.1)?;
        mem.storeb(0x26, 0x01)?;
        mem.storeb(0x27, 0x01)?;
    }
    mem.storeb(0x2c, config.default_color.1.into())?;
    mem.storeb(0x2d, config.default_color.0.into())?;
    mem.storew(0x32, 0x0101)?;

    let header = Header {
        checksum,
        extension: (ext_addr, ext_len),
    };
    header.set_extension(mem, 4, 0)?;
    header.set_extension(mem, 5, config.true_color.0)?;
    header.set_extension(mem, 6, config.true_color.1)?;
    Ok(header)
}

#[cfg(test)]
pub fn init_test(mem: &mut Mem) -> Header {
    init(mem, &interface::DEFAULT).unwrap()
}

#[cfg(test)]
use crate::{interface, mem};

#[test]
fn test_checked() {
    let mut data = mem::default();
    data[0x11] = 0x52;
    let mut mem = mem::new(data).unwrap();
    let header = init(&mut mem, &interface::DEFAULT).unwrap();
    assert_eq!(mem[0x11], 0x52);
    header.checked_storeb(&mut mem, 0x11, 0xff).unwrap();
    assert_eq!(mem[0x11], 0x57);
    header.checked_storew(&mut mem, 0x11, 0xff).unwrap();
    assert_eq!(mem[0x11], 0x50);
    header.checked_storew(&mut mem, 0x10, 0xa5).unwrap();
    assert_eq!(mem[0x11], 0x55);
}

#[test]
fn test_extension() {
    let mut data = mem::default();
    data[0x0f] = 0x46;
    data[0x37] = 0x40;
    data.extend(vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
    let mut mem = mem::new(data).unwrap();
    let header = init(&mut mem, &interface::DEFAULT).unwrap();
    assert_eq!(header.get_extension(&mem, 1).unwrap(), 0x0203);
    assert_eq!(header.get_extension(&mem, 2).unwrap(), 0x0000);
    header.set_extension(&mut mem, 1, 0x0607).unwrap();
    assert_eq!(mem.loadw(0x42).unwrap(), 0x0607);
    header.set_extension(&mut mem, 2, 0x0809).unwrap();
    assert_eq!(mem.loadw(0x44).unwrap(), 0x0405);
}
