use crate::{err::*, mem::Mem};

pub struct Object {
    v123: bool,
    addr: u16,
}

mod attr;
mod prop;
mod tree;

pub fn init(mem: &Mem) -> Object {
    Object {
        v123: mem[0] < 4,
        addr: mem.loadw(0x0a).unwrap(),
    }
}

#[cfg(test)]
use crate::mem;

#[test]
fn test_init() {
    let mut data = mem::default();
    data[0x0a] = 1;
    let mem = mem::new(data.clone()).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.v123, true);
    assert_eq!(obj.addr, 0x100);
    data[0x00] = 4;
    let mem = mem::new(data).unwrap();
    let obj = init(&mem);
    assert_eq!(obj.v123, false);
    assert_eq!(obj.addr, 0x100);
}
