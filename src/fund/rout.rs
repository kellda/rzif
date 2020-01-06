use crate::{err::*, mem::Mem};

pub fn info(mem: &Mem, mut addr: usize) -> Result<(Vec<u16>, usize), Error> {
    let nbr = *mem.get(addr).ok_or_else(|| err(Cause::PcOut, (0, 0)))? as usize;
    if nbr > 15 {
        return error(Cause::TooManyLocals, (nbr as u16, 0));
    }
    let mut locals = vec![0; nbr];
    addr += 1;
    if mem[0x00] < 5 {
        for local in locals.iter_mut() {
            *local = mem.getw(addr).ok_or_else(|| err(Cause::PcOut, (0, 0)))?;
            addr += 2;
        }
    }
    Ok((locals, addr))
}

#[test]
fn test_info() {
    use crate::mem;

    let mut data = mem::default();
    data[0x00] = 4;
    data.extend(vec![0x02, 0xf0, 0x0f, 0xaa, 0x55]);
    let mem = mem::new(data.clone()).unwrap();
    assert_eq!(info(&mem, 0x40).unwrap(), (vec![0xf00f, 0xaa55], 0x45));
    data[0x00] = 5;
    let mem = mem::new(data).unwrap();
    assert_eq!(info(&mem, 0x40).unwrap(), (vec![0, 0], 0x41));
}
