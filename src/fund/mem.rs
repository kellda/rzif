use crate::err::*;
use std::ops::Deref;

pub struct Mem {
    mem: Vec<u8>,
    stat: u16,
    v: u8,
}

impl Mem {
    pub fn byte(&self, addr: u16) -> usize {
        addr as usize
    }

    pub fn word(&self, addr: u16) -> usize {
        addr as usize * 2
    }

    pub fn packed(&self, addr: u16, rout: bool) -> usize {
        match self.v {
            1..=3 => addr as usize * 2,
            4..=5 => addr as usize * 4,
            7 => {
                let offset = if rout {
                    get(&self.mem, 0x28)
                } else {
                    get(&self.mem, 0x2a)
                } as usize
                    * 8;
                addr as usize * 4 + offset
            }
            8 => addr as usize * 8,
            _ => unreachable!(),
        }
    }

    pub fn storeb(&mut self, addr: u16, data: u16) -> Result<(), Error> {
        if addr >= self.stat {
            return error(Cause::WriteOut, (addr, 0));
        }
        self.mem[addr as usize] = data as u8;
        Ok(())
    }

    pub fn storew(&mut self, addr: u16, data: u16) -> Result<(), Error> {
        if addr + 1 >= self.stat {
            return error(Cause::WriteOut, (addr, 0));
        }
        set(&mut self.mem, addr as usize, data);
        Ok(())
    }

    pub fn loadb(&self, addr: u16) -> Result<u16, Error> {
        if addr as usize >= self.mem.len() {
            return error(Cause::ReadOut, (addr, 0));
        }
        Ok(u16::from(self.mem[addr as usize]))
    }

    pub fn loadw(&self, addr: u16) -> Result<u16, Error> {
        if addr as usize + 1 >= self.mem.len() {
            return error(Cause::ReadOut, (addr, 0));
        }
        Ok(get(&self.mem, addr as usize))
    }

    pub fn getw(&self, addr: usize) -> Option<u16> {
        if addr + 1 < self.mem.len() {
            Some(get(&self.mem, addr))
        } else {
            None
        }
    }

    pub fn save(&self) -> Vec<u8> {
        self.mem[..self.stat as usize].to_vec()
    }

    pub fn restore(&mut self, data: &[u8]) {
        let flags = self.mem[0x11];
        self.mem[..self.stat as usize].copy_from_slice(data);
        self.mem[0x11] = flags;
    }
}

impl Deref for Mem {
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8> {
        &self.mem
    }
}

fn get(data: &[u8], addr: usize) -> u16 {
    u16::from(data[addr]) << 8 | u16::from(data[addr + 1])
}

fn set(data: &mut Vec<u8>, addr: usize, value: u16) {
    data[addr] = (value >> 8) as u8;
    data[addr + 1] = value as u8;
}

pub fn new(data: Vec<u8>) -> Result<Mem, Error> {
    if data.len() < 0x40 {
        return error(Cause::TooShort, (data.len() as u16, 0));
    }
    let v = data[0x00];
    if v == 0 || v == 6 || v > 8 {
        return error(Cause::BadVer, (v.into(), 0));
    }
    let stat = get(&data, 0x0e);
    if stat as usize > data.len() {
        return error(Cause::StaticOut, (stat, 0));
    }
    Ok(Mem { mem: data, stat, v })
}

#[cfg(test)]
pub fn default() -> Vec<u8> {
    let mut data = vec![0; 0x40];
    data[0x00] = 1;
    data[0x0f] = 0x40;
    data[0x1e] = 1;
    data[0x26] = 1;
    data[0x27] = 1;
    data
}

#[test]
fn test_get_set() {
    let mut data = vec![1, 2, 3];
    assert_eq!(get(&data, 0), 0x102);
    assert_eq!(get(&data, 1), 0x203);
    set(&mut data, 0, 0x403);
    assert_eq!(data, vec![4, 3, 3]);
    set(&mut data, 1, 0x201);
    assert_eq!(data, vec![4, 2, 1]);
}

#[test]
fn test_addr() {
    let mut data = vec![0; 0x40];
    data[0x29] = 20;
    data[0x2b] = 30;
    let mut mem = Mem {
        mem: data,
        stat: 0,
        v: 1,
    };
    assert_eq!(mem.byte(10), 10);
    assert_eq!(mem.word(10), 20);
    assert_eq!(mem.packed(10, false), 20);
    mem.v = 2;
    assert_eq!(mem.packed(10, true), 20);
    mem.v = 4;
    assert_eq!(mem.packed(10, false), 40);
    mem.v = 7;
    assert_eq!(mem.packed(10, true), 200);
    assert_eq!(mem.packed(10, false), 280);
    mem.v = 8;
    assert_eq!(mem.packed(10, true), 80);
}

#[test]
fn test_mem() {
    let mut data = default();
    data[0x0f] = 0x45;
    data.extend(vec![0; 0x10]);
    let mut mem = new(data.clone()).unwrap();
    assert_eq!(mem.mem, data);
    assert_eq!(mem.stat, 0x45);
    assert_eq!(mem.v, 1);
    mem.storeb(0x42, 0x10).unwrap();
    mem.storew(0x43, 0x2030).unwrap();
    assert_eq!(mem.loadb(0x44).unwrap(), 0x30);
    assert_eq!(mem.loadw(0x42).unwrap(), 0x1020);
    assert_eq!(mem[0x42], 0x10);
    assert_eq!(mem[0x43], 0x20);
    assert_eq!(mem[0x44], 0x30);
}

#[test]
fn test_save_restore() {
    let mut data = default();
    data[0x0f] = 0x43;
    data.extend(vec![1, 2, 3, 4, 5, 6]);
    let mut mem = new(data.clone()).unwrap();
    assert_eq!(mem.save()[..], data[..0x43]);
    data[0x40] = 7;
    data[0x41] = 8;
    data[0x42] = 9;
    mem.restore(&data[..0x43]);
    assert_eq!(mem.mem, data);
}
