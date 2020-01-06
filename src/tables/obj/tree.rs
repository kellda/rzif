use super::*;

impl Object {
    pub fn get_parent(&self, mem: &Mem, obj: u16) -> Result<u16, Error> {
        self.check(obj)?;
        if self.v123 {
            mem.loadb(self.addr + 57 + 9 * obj)
        } else {
            mem.loadw(self.addr + 118 + 14 * obj)
        }
    }

    pub fn get_sibling(&self, mem: &Mem, obj: u16) -> Result<u16, Error> {
        self.check(obj)?;
        if self.v123 {
            mem.loadb(self.addr + 58 + 9 * obj)
        } else {
            mem.loadw(self.addr + 120 + 14 * obj)
        }
    }

    pub fn get_child(&self, mem: &Mem, obj: u16) -> Result<u16, Error> {
        self.check(obj)?;
        if self.v123 {
            mem.loadb(self.addr + 59 + 9 * obj)
        } else {
            mem.loadw(self.addr + 122 + 14 * obj)
        }
    }

    pub fn insert(&self, mem: &mut Mem, obj: u16, dest: u16) -> Result<(), Error> {
        self.check(obj)?;
        self.check(dest)?;
        if self.v123 {
            let base = self.addr + 58;
            let addr = base + 9 * obj;
            let parent = mem.loadb(addr - 1)?;
            let next = mem.loadb(addr)?;

            if parent != 0 {
                let parent_addr = base + 1 + 9 * parent;
                let mut prev = mem.loadb(parent_addr)?;
                if prev == obj {
                    mem.storeb(parent_addr, next)?;
                } else {
                    let mut prev_addr = 0;
                    let mut count = 0;
                    while prev != obj {
                        if prev == 0 || count > 255 {
                            return error(Cause::NotChildOfParent, (obj, parent));
                        }
                        prev_addr = base + 9 * prev;
                        prev = mem.loadb(prev_addr)?;
                        count += 1;
                    }
                    mem.storeb(prev_addr, next)?;
                }
            }

            let dest_addr = base + 1 + 9 * dest;
            let child = mem.loadb(dest_addr)?;
            mem.storeb(addr - 1, dest)?;
            mem.storeb(addr, child)?;
            mem.storeb(dest_addr, obj)?;
        } else {
            let base = self.addr + 120;
            let addr = base + 14 * obj;
            let parent = mem.loadw(addr - 2)?;
            let next = mem.loadw(addr)?;

            if parent != 0 {
                let parent_addr = base + 2 + 14 * parent; // ERROR multiply with overflow
                let mut prev = mem.loadw(parent_addr)?;
                if prev == obj {
                    mem.storew(parent_addr, next)?;
                } else {
                    let mut prev_addr = 0;
                    let mut count = 0;
                    while prev != obj {
                        if prev == 0 || count > 65535 {
                            return error(Cause::NotChildOfParent, (obj, parent));
                        }
                        prev_addr = base + 14 * prev;
                        prev = mem.loadw(prev_addr)?;
                        count += 1;
                    }
                    mem.storew(prev_addr, next)?;
                }
            }

            let dest_addr = base + 2 + 14 * dest;
            let child = mem.loadw(dest_addr)?;
            mem.storew(addr - 2, dest)?;
            mem.storew(addr, child)?;
            mem.storew(dest_addr, obj)?;
        }
        Ok(())
    }

    pub fn remove(&self, mem: &mut Mem, obj: u16) -> Result<(), Error> {
        self.check(obj)?;
        if self.v123 {
            let base = self.addr + 58;
            let mut addr = base - 1 + 9 * obj;
            let parent = mem.loadb(addr)?;
            if parent == 0 {
                return Ok(());
            }
            mem.storeb(addr, 0)?;
            let parent_addr = base + 1 + 9 * parent;
            addr += 1;
            let next = mem.loadb(addr)?;
            mem.storeb(addr, 0)?;
            let mut prev = mem.loadb(parent_addr)?;
            if prev == obj {
                mem.storeb(parent_addr, next)?;
            } else {
                let mut prev_addr = 0;
                let mut count = 0;
                while prev != obj {
                    if prev == 0 || count > 255 {
                        return error(Cause::NotChildOfParent, (obj, parent));
                    }
                    prev_addr = base + 9 * prev;
                    prev = mem.loadb(prev_addr)?;
                    count += 1;
                }
                mem.storeb(prev_addr, next)?;
            }
        } else {
            let base = self.addr + 120;
            let mut addr = base - 2 + 14 * obj;
            let parent = mem.loadw(addr)?;
            if parent == 0 {
                return Ok(());
            }
            mem.storew(addr, 0)?;
            let parent_addr = base + 2 + 14 * parent;
            addr += 2;
            let next = mem.loadw(addr)?;
            mem.storew(addr, 0)?;
            let mut prev = mem.loadw(parent_addr)?;
            if prev == obj {
                mem.storew(parent_addr, next)?;
            } else {
                let mut prev_addr = 0;
                let mut count = 0;
                while prev != obj {
                    if prev == 0 || count > 65535 {
                        return error(Cause::NotChildOfParent, (obj, parent));
                    }
                    prev_addr = base + 14 * prev;
                    prev = mem.loadw(prev_addr)?;
                    count += 1;
                }
                mem.storew(prev_addr, next)?;
            }
        }
        Ok(())
    }

    fn check(&self, obj: u16) -> Result<(), Error> {
        if obj == 0 {
            return error(Cause::BadObj, (obj, 0));
        }
        if self.v123 && obj > 255 {
            return error(Cause::BadObj, (obj, 0));
        }
        Ok(())
    }
}

#[test]
fn test_get_old() {
    let mut data = mem::default();
    data[0x0b] = 0x40;
    data.extend(vec![0; 0x42]);
    data.extend(vec![0x12, 0x34, 0x56, 0x00, 0x00]);
    let mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.get_parent(&mem, 1).unwrap(), 0x12);
    assert_eq!(obj.get_sibling(&mem, 1).unwrap(), 0x34);
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 0x56);
}

#[test]
fn test_get_new() {
    let mut data = mem::default();
    data[0x00] = 0x04;
    data[0x0b] = 0x40;
    data.extend(vec![0; 0x84]);
    data.extend(vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0x00, 0x00]);
    let mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.get_parent(&mem, 1).unwrap(), 0x1234);
    assert_eq!(obj.get_sibling(&mem, 1).unwrap(), 0x5678);
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 0x9abc);
}

#[test]
fn test_edit_old() {
    let mut data = mem::default();
    data.extend(vec![0; 0x62]);
    data[0x0b] = 0x40;
    data[0x0f] = 0xa2;
    data[0x84] = 0x02;
    data[0x8b] = 0x01;
    data[0x8c] = 0x03;
    data[0x94] = 0x01;
    data[0x95] = 0x04;
    data[0x9d] = 0x01;
    let mut mem = mem::new(data).unwrap();
    let obj = init(&mem);

    obj.remove(&mut mem, 3).unwrap();
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 2);
    assert_eq!(obj.get_sibling(&mem, 2).unwrap(), 4);
    assert_eq!(obj.get_parent(&mem, 3).unwrap(), 0);
    assert_eq!(obj.get_sibling(&mem, 3).unwrap(), 0);
    obj.remove(&mut mem, 2).unwrap();
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 4);
    assert_eq!(obj.get_parent(&mem, 2).unwrap(), 0);
    assert_eq!(obj.get_sibling(&mem, 2).unwrap(), 0);
    obj.remove(&mut mem, 1).unwrap();

    obj.insert(&mut mem, 3, 1).unwrap();
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 3);
    assert_eq!(obj.get_parent(&mem, 3).unwrap(), 1);
    assert_eq!(obj.get_sibling(&mem, 3).unwrap(), 4);
    obj.insert(&mut mem, 2, 1).unwrap();
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 2);
    assert_eq!(obj.get_parent(&mem, 2).unwrap(), 1);
    assert_eq!(obj.get_sibling(&mem, 2).unwrap(), 3);
}

#[test]
fn test_edit_new() {
    let mut data = mem::default();
    data.extend(vec![0; 0xb6]);
    data[0x00] = 0x04;
    data[0x0b] = 0x40;
    data[0x0f] = 0xf6;
    data[0xc9] = 0x02;
    data[0xd3] = 0x01;
    data[0xd5] = 0x03;
    data[0xe1] = 0x01;
    data[0xe3] = 0x04;
    data[0xef] = 0x01;
    let mut mem = mem::new(data).unwrap();
    let obj = init(&mem);

    obj.remove(&mut mem, 3).unwrap();
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 2);
    assert_eq!(obj.get_sibling(&mem, 2).unwrap(), 4);
    assert_eq!(obj.get_parent(&mem, 3).unwrap(), 0);
    assert_eq!(obj.get_sibling(&mem, 3).unwrap(), 0);
    obj.remove(&mut mem, 2).unwrap();
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 4);
    assert_eq!(obj.get_parent(&mem, 2).unwrap(), 0);
    assert_eq!(obj.get_sibling(&mem, 2).unwrap(), 0);
    obj.remove(&mut mem, 1).unwrap();

    obj.insert(&mut mem, 3, 1).unwrap();
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 3);
    assert_eq!(obj.get_parent(&mem, 3).unwrap(), 1);
    assert_eq!(obj.get_sibling(&mem, 3).unwrap(), 4);
    obj.insert(&mut mem, 2, 1).unwrap();
    assert_eq!(obj.get_child(&mem, 1).unwrap(), 2);
    assert_eq!(obj.get_parent(&mem, 2).unwrap(), 1);
    assert_eq!(obj.get_sibling(&mem, 2).unwrap(), 3);
}
