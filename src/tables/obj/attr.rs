use super::*;

impl Object {
    pub fn test_attr(&self, mem: &Mem, obj: u16, attr: u16) -> Result<bool, Error> {
        let addr = self.addr(obj, attr)?;
        Ok(mem.loadb(addr)? & 0x80 >> (attr & 0x07) != 0)
    }

    pub fn set_attr(&self, mem: &mut Mem, obj: u16, attr: u16) -> Result<(), Error> {
        let addr = self.addr(obj, attr)?;
        let byte = mem.loadb(addr)? | 0x80 >> (attr & 0x07);
        mem.storeb(addr, byte)?;
        Ok(())
    }

    pub fn clear_attr(&self, mem: &mut Mem, obj: u16, attr: u16) -> Result<(), Error> {
        let addr = self.addr(obj, attr)?;
        let byte = mem.loadb(addr)? & !(0x80 >> (attr & 0x07));
        mem.storeb(addr, byte)?;
        Ok(())
    }

    fn addr(&self, obj: u16, attr: u16) -> Result<u16, Error> {
        let base;
        if obj == 0 {
            return error(Cause::BadObj, (obj, 0));
        }
        if self.v123 {
            if obj > 255 {
                return error(Cause::BadObj, (obj, 0));
            }
            if attr >= 32 {
                return error(Cause::BadAttr, (attr, 0));
            }
            base = 53 + 9 * obj;
        } else {
            if attr >= 48 {
                return error(Cause::BadAttr, (attr, 0));
            }
            base = 112 + 14 * obj;
        }
        Ok(base + self.addr + (attr >> 3))
    }
}

#[test]
fn test_old() {
    let mut data = mem::default();
    data[0x0b] = 0x40;
    data[0x0f] = 0x82;
    data.extend(vec![0; 0x3e]);
    data.extend(vec![0xfe, 0x02, 0xbf, 0x80]);
    let mut mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert!(!obj.test_attr(&mem, 1, 0x07).unwrap());
    assert!(obj.test_attr(&mem, 1, 0x0e).unwrap());
    assert!(!obj.test_attr(&mem, 1, 0x11).unwrap());
    assert!(obj.test_attr(&mem, 1, 0x18).unwrap());
    obj.clear_attr(&mut mem, 1, 0x00).unwrap();
    obj.set_attr(&mut mem, 1, 0x09).unwrap();
    obj.clear_attr(&mut mem, 1, 0x16).unwrap();
    obj.set_attr(&mut mem, 1, 0x1f).unwrap();
    assert_eq!(mem.loadw(0x7e).unwrap(), 0x7e42);
    assert_eq!(mem.loadw(0x80).unwrap(), 0xbd81);
}

#[test]
fn test_new() {
    let mut data = mem::default();
    data.extend(vec![0; 0x7e]);
    data[0x00] = 0x04;
    data[0x0b] = 0x40;
    data[0x0f] = 0xc4;
    data.extend(vec![0xfe, 0xff, 0x02, 0xbf, 0x00, 0x80]);
    let mut mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert!(!obj.test_attr(&mem, 1, 0x07).unwrap());
    assert!(obj.test_attr(&mem, 1, 0x16).unwrap());
    assert!(!obj.test_attr(&mem, 1, 0x19).unwrap());
    assert!(obj.test_attr(&mem, 1, 0x28).unwrap());
    obj.clear_attr(&mut mem, 1, 0x00).unwrap();
    obj.set_attr(&mut mem, 1, 0x11).unwrap();
    obj.clear_attr(&mut mem, 1, 0x1e).unwrap();
    obj.set_attr(&mut mem, 1, 0x2f).unwrap();
    assert_eq!(mem.loadw(0xbe).unwrap(), 0x7eff);
    assert_eq!(mem.loadw(0xc0).unwrap(), 0x42bd);
    assert_eq!(mem.loadw(0xc2).unwrap(), 0x0081);
}
