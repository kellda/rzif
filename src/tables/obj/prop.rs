use super::*;
use crate::text::Text;

impl Object {
    pub fn prop_addr(&self, mem: &Mem, obj: u16, prop: u16) -> Result<u16, Error> {
        if obj == 0 {
            return error(Cause::BadObj, (obj, 0));
        }
        if self.v123 {
            if obj > 255 {
                return error(Cause::BadObj, (obj, 0));
            }
            self.prop_addr_old(mem, obj, prop)
        } else {
            self.prop_addr_new(mem, obj, prop)
        }
    }

    fn prop_addr_old(&self, mem: &Mem, obj: u16, prop: u16) -> Result<u16, Error> {
        if prop == 0 || prop > 31 {
            return error(Cause::BadProp, (prop, 0));
        }
        let mut table = mem.loadw(self.addr + 60 + 9 * obj)?;
        table += 2 * mem.loadb(table)? + 1;
        let mut info = mem.loadb(table)?;
        while prop <= info & 0x1f {
            if info & 0x1f == prop {
                return Ok(table + 1);
            }
            table += (info >> 5) + 2;
            info = mem.loadb(table)?;
        }
        Ok(0)
    }

    fn prop_addr_new(&self, mem: &Mem, obj: u16, prop: u16) -> Result<u16, Error> {
        if prop == 0 || prop > 63 {
            return error(Cause::BadProp, (prop, 0));
        }
        let mut table = mem.loadw(self.addr + 124 + 14 * obj)?;
        table += 2 * mem.loadb(table)? + 1;
        let mut info = mem.loadb(table)?;
        while prop <= info & 0x3f {
            if info & 0x3f == prop {
                return Ok(table + if info & 0x80 == 0 { 1 } else { 2 });
            }
            match info & 0xc0 {
                0x00 => table += 2,
                0x40 => table += 3,
                _ => {
                    let size = mem.loadb(table + 1)? & 0x3f;
                    if size == 0 {
                        table += 66;
                    } else {
                        table += size + 2;
                    }
                }
            }
            info = mem.loadb(table)?;
        }
        Ok(0)
    }

    pub fn prop_len(&self, mem: &Mem, addr: u16) -> Result<u16, Error> {
        if addr == 0 {
            return Ok(0);
        }
        if self.v123 {
            Ok((mem.loadb(addr - 1)? >> 5) + 1)
        } else {
            Ok(match mem.loadb(addr - 1)? & 0xc0 {
                0x00 => 1,
                0x40 => 2,
                _ => {
                    let size = mem.loadb(addr - 1)? & 0x3f;
                    if size == 0 {
                        64
                    } else {
                        size
                    }
                }
            })
        }
    }

    pub fn next_prop(&self, mem: &Mem, obj: u16, prop: u16) -> Result<u16, Error> {
        if obj == 0 {
            return error(Cause::BadObj, (obj, 0));
        }
        if self.v123 {
            if obj > 255 {
                return error(Cause::BadObj, (obj, 0));
            }
            let addr = if prop == 0 {
                let addr = mem.loadw(self.addr + 60 + 9 * obj)?;
                addr + 2 * mem.loadb(addr)? + 1
            } else {
                let addr = self.prop_addr_old(mem, obj, prop)?;
                if addr == 0 {
                    return error(Cause::NoProp, (obj, prop));
                }
                addr + (mem.loadb(addr - 1)? >> 5) + 1
            };
            Ok(mem.loadb(addr)? & 0x1f)
        } else {
            let addr = if prop == 0 {
                let addr = mem.loadw(self.addr + 124 + 14 * obj)?;
                addr + 2 * mem.loadb(addr)? + 1
            } else {
                let addr = self.prop_addr_new(mem, obj, prop)?;
                if addr == 0 {
                    return error(Cause::NoProp, (obj, prop));
                }
                let size = match mem.loadb(addr - 1)? & 0xc0 {
                    0x00 => 1,
                    0x40 => 2,
                    _ => {
                        let size = mem.loadb(addr - 1)? & 0x3f;
                        if size == 0 {
                            64
                        } else {
                            size
                        }
                    }
                };
                addr + size
            };
            Ok(mem.loadb(addr)? & 0x3f)
        }
    }

    pub fn get_prop(&self, mem: &Mem, obj: u16, prop: u16) -> Result<u16, Error> {
        if obj == 0 {
            return error(Cause::BadObj, (obj, 0));
        }
        if self.v123 {
            if obj > 255 {
                return error(Cause::BadObj, (obj, 0));
            }
            let addr = self.prop_addr_old(mem, obj, prop)?;
            if addr == 0 {
                Ok(mem.loadw(self.addr + prop * 2 - 2)?)
            } else {
                match mem.loadb(addr - 1)? >> 5 {
                    0 => mem.loadb(addr),
                    1 => mem.loadw(addr),
                    _ => error(Cause::GetLongProp, (obj, prop)),
                }
            }
        } else {
            let addr = self.prop_addr_new(mem, obj, prop)?;
            if addr == 0 {
                Ok(mem.loadw(self.addr + prop * 2 - 2)?)
            } else {
                match mem.loadb(addr - 1)? & 0xc0 {
                    0x00 => mem.loadb(addr),
                    0x40 => mem.loadw(addr),
                    _ => error(Cause::GetLongProp, (obj, prop)),
                }
            }
        }
    }

    pub fn put_prop(&self, mem: &mut Mem, obj: u16, prop: u16, data: u16) -> Result<(), Error> {
        if obj == 0 {
            return error(Cause::BadObj, (obj, 0));
        }
        if self.v123 {
            if obj > 255 {
                return error(Cause::BadObj, (obj, 0));
            }
            let addr = self.prop_addr_old(mem, obj, prop)?;
            if addr == 0 {
                return error(Cause::NoProp, (obj, prop));
            }
            match mem.loadb(addr - 1)? >> 5 {
                0 => mem.storeb(addr, data),
                1 => mem.storew(addr, data),
                _ => error(Cause::PutLongProp, (obj, prop)),
            }
        } else {
            let addr = self.prop_addr_new(mem, obj, prop)?;
            if addr == 0 {
                return error(Cause::NoProp, (obj, prop));
            }
            match mem.loadb(addr - 1)? & 0xc0 {
                0x00 => mem.storeb(addr, data),
                0x40 => mem.storew(addr, data),
                _ => error(Cause::PutLongProp, (obj, prop)),
            }
        }
    }

    pub fn name(&self, mem: &Mem, text: &Text, obj: u16) -> Result<String, Error> {
        if obj == 0 {
            return error(Cause::BadObj, (obj, 0));
        }
        let addr = if self.v123 {
            if obj > 255 {
                return error(Cause::BadObj, (obj, 0));
            }
            mem.loadw(self.addr + 60 + 9 * obj)? as usize + 1
        } else {
            mem.loadw(self.addr + 124 + 14 * obj)? as usize + 1
        };
        Ok(trace(text.decode(mem, addr), Trace::String(addr))?.0)
    }
}

#[cfg(test)]
use crate::{header, text};

#[test]
fn test_addr_old() {
    let mut data = mem::default();
    data.extend(vec![0; 0x56]);
    data[0x0b] = 0x40;
    data[0x86] = 0x87;
    data[0x87] = 0x01;
    data[0x8a] = 0xe3;
    data[0x93] = 0x02;

    let mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.prop_addr(&mem, 1, 3).unwrap(), 0x8b);
    assert_eq!(obj.prop_addr(&mem, 1, 2).unwrap(), 0x94);
    assert_eq!(obj.prop_addr(&mem, 1, 1).unwrap(), 0x00);
    assert_eq!(obj.next_prop(&mem, 1, 3).unwrap(), 2);
    assert_eq!(obj.next_prop(&mem, 1, 2).unwrap(), 0);
    assert_eq!(obj.next_prop(&mem, 1, 0).unwrap(), 3);
}

#[test]
fn test_addr_new() {
    let mut data = mem::default();
    data.extend(vec![0; 0x9c]);
    data[0x00] = 0x04;
    data[0x0b] = 0x40;
    data[0xcb] = 0xcc;
    data[0xcc] = 0x01;
    data[0xcf] = 0x83;
    data[0xd0] = 0xc8;
    data[0xd9] = 0x02;

    let mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.prop_addr(&mem, 1, 3).unwrap(), 0xd1);
    assert_eq!(obj.prop_addr(&mem, 1, 2).unwrap(), 0xda);
    assert_eq!(obj.prop_addr(&mem, 1, 1).unwrap(), 0x00);
    assert_eq!(obj.next_prop(&mem, 1, 3).unwrap(), 2);
    assert_eq!(obj.next_prop(&mem, 1, 2).unwrap(), 0);
    assert_eq!(obj.next_prop(&mem, 1, 0).unwrap(), 3);
}

#[test]
fn test_len_old() {
    let mut data = mem::default();
    data.extend(vec![0x00, 0x1f, 0xff]);
    let mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.prop_len(&mem, 0x00).unwrap(), 0);
    assert_eq!(obj.prop_len(&mem, 0x41).unwrap(), 1);
    assert_eq!(obj.prop_len(&mem, 0x42).unwrap(), 1);
    assert_eq!(obj.prop_len(&mem, 0x43).unwrap(), 8);
}

#[test]
fn test_len_new() {
    let mut data = mem::default();
    data[0x00] = 4;
    data.extend(vec![0x00, 0x2f, 0x40, 0x80, 0xc0, 0xbf, 0xff]);
    let mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.prop_len(&mem, 0x00).unwrap(), 0);
    assert_eq!(obj.prop_len(&mem, 0x41).unwrap(), 1);
    assert_eq!(obj.prop_len(&mem, 0x42).unwrap(), 1);
    assert_eq!(obj.prop_len(&mem, 0x43).unwrap(), 2);
    assert_eq!(obj.prop_len(&mem, 0x45).unwrap(), 64);
    assert_eq!(obj.prop_len(&mem, 0x47).unwrap(), 63);
}

#[test]
fn test_get_put_old() {
    let mut data = mem::default();
    data.extend(vec![0; 0x47]);
    data[0x0b] = 0x40;
    data[0x0f] = 0x8e;
    data[0x44] = 0x12;
    data[0x45] = 0x34;
    data[0x86] = 0x87;
    data.extend(vec![0x00, 0x22, 0x56, 0x78, 0x01, 0x9a, 0x00]);
    let mut mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.get_prop(&mem, 1, 3).unwrap(), 0x1234);
    assert_eq!(obj.get_prop(&mem, 1, 2).unwrap(), 0x5678);
    assert_eq!(obj.get_prop(&mem, 1, 1).unwrap(), 0x9a);
    obj.put_prop(&mut mem, 1, 1, 0xcba9).unwrap();
    obj.put_prop(&mut mem, 1, 2, 0x8765).unwrap();
    assert_eq!(mem.loadw(0x8b).unwrap(), 0x01a9);
    assert_eq!(mem.loadw(0x89).unwrap(), 0x8765);
    assert_eq!(obj.get_prop(&mem, 1, 3).unwrap(), 0x1234);
}

#[test]
fn test_get_put_new() {
    let mut data = mem::default();
    data.extend(vec![0; 0x8c]);
    data[0x00] = 0x04;
    data[0x0b] = 0x40;
    data[0x0f] = 0xd3;
    data[0x44] = 0x12;
    data[0x45] = 0x34;
    data[0xcb] = 0xcc;
    data.extend(vec![0x00, 0x42, 0x56, 0x78, 0x01, 0x9a, 0x00]);
    let mut mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.get_prop(&mem, 1, 3).unwrap(), 0x1234);
    assert_eq!(obj.get_prop(&mem, 1, 2).unwrap(), 0x5678);
    assert_eq!(obj.get_prop(&mem, 1, 1).unwrap(), 0x9a);
    obj.put_prop(&mut mem, 1, 1, 0xcba9).unwrap();
    obj.put_prop(&mut mem, 1, 2, 0x8765).unwrap();
    assert_eq!(mem.loadw(0xd0).unwrap(), 0x01a9);
    assert_eq!(mem.loadw(0xce).unwrap(), 0x8765);
    assert_eq!(obj.get_prop(&mem, 1, 3).unwrap(), 0x1234);
}

#[test]
fn test_name_old() {
    let mut data = mem::default();
    data[0x0b] = 0x40;
    data.extend(vec![0; 0x46]);
    data.extend(vec![0x87, 0x02, 0x0b, 0x2a, 0xe3, 0x25]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = text::init(&mem, &header).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.name(&mem, &text, 1).unwrap(), "Test");
}

#[test]
fn test_name_new() {
    let mut data = mem::default();
    data[0x00] = 0x04;
    data[0x0b] = 0x40;
    data.extend(vec![0; 0x8b]);
    data.extend(vec![0xcc, 0x02, 0x13, 0x2a, 0xe3, 0x25]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = text::init(&mem, &header).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.name(&mem, &text, 1).unwrap(), "Test");
}
