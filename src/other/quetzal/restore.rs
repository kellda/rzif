use crate::{
    err::SaveError::{self, *},
    state::{Frame, Saved},
};

pub fn restore(init: &Saved, saved: &[u8], checksum: u16) -> Result<Saved, SaveError> {
    if saved.len() < 12 || &saved[0..4] != b"FORM" || &saved[8..12] != b"IFZS" {
        return Err(BadSave);
    }
    let len = from_bytes(&saved[4..8]);

    let mut mem = Err(MissingMem);
    let mut stack = Err(MissingStks);
    let mut pc = None;
    let mut call = None;

    let mut i = 12;
    while i < len {
        let len = from_bytes(&saved.get(i + 4..i + 8).ok_or(BadSave)?);
        let chunk = &saved.get(i..i + len + 8).ok_or(BadSave)?;
        i += len + 8;
        if len & 1 == 1 {
            i += 1;
        }

        match &chunk[0..4] {
            b"IFhd" => {
                if pc.is_some() {
                    return Err(TwoIFhd);
                }
                if len < 13 {
                    return Err(BadSave);
                }
                if chunk[8..10] != init.mem[0x02..0x04]
                    || chunk[10..16] != init.mem[0x12..0x18]
                    || from_bytes(&chunk[16..18]) != checksum as usize
                {
                    return Err(GamesDiffer);
                }
                pc = Some(from_bytes(&chunk[18..21]));
            }
            b"CMem" => {
                if mem.is_ok() {
                    return Err(TwoMem);
                }
                mem = Ok(uncompress(&init.mem, &chunk[8..])?);
            }
            b"UMem" => {
                if mem.is_ok() {
                    return Err(TwoMem);
                }
                if len != init.mem.len() {
                    return Err(UMemBadSize(len, init.mem.len()));
                }
                mem = Ok(chunk[8..].to_vec());
            }
            b"Stks" => {
                if stack.is_ok() {
                    return Err(TwoStks);
                }
                let mut stack_ = Vec::new();
                let mut call_ = Vec::new();

                let stk_len = from_bytes(&chunk.get(14..16).ok_or(BadSave)?) * 2;
                stack_.push(to_u16(&chunk.get(16..stk_len + 16).ok_or(BadSave)?));
                let mut i = stk_len + 16;

                while i < len + 8 {
                    let var_len = (chunk[i + 3] as usize & 0x0f) * 2;
                    let stk_len = from_bytes(&chunk[i + 6..i + 8]) * 2;
                    call_.push(Frame {
                        ret: from_bytes(&chunk[i..i + 3]),
                        local: to_u16(&chunk.get(i + 8..i + 8 + var_len).ok_or(BadSave)?),
                        args: chunk[i + 5].count_ones() as u16,
                        store: chunk[i + 3] & 0x10 == 0,
                    });
                    i += 8 + var_len;
                    stack_.push(to_u16(&chunk.get(i..i + stk_len).ok_or(BadSave)?));
                    i += stk_len;
                }
                stack = Ok(stack_);
                call = Some(call_);
            }
            _ => {}
        }
    }
    Ok(Saved {
        mem: mem?,
        stack: stack?,
        pc: pc.unwrap(),
        call: call.unwrap(),
    })
}

fn from_bytes(data: &[u8]) -> usize {
    let mut result = 0;
    for i in 0..data.len() {
        result += (data[i] as usize) << (8 * (data.len() - i - 1));
    }
    result
}

fn to_u16(data: &[u8]) -> Vec<u16> {
    let mut result = Vec::new();
    for i in 0..data.len() / 2 {
        result.push(u16::from(data[2 * i]) << 8 | u16::from(data[2 * i + 1]));
    }
    result
}

fn uncompress(init: &[u8], comp: &[u8]) -> Result<Vec<u8>, SaveError> {
    let mut result = Vec::new();
    let mut state = false;
    for &byte in comp.iter() {
        if state {
            result.extend(vec![0; byte as usize + 1]);
            state = false;
        } else if byte == 0 {
            state = true;
        } else {
            result.push(byte);
        }
    }
    if state {
        return Err(CMemIncomplete);
    }
    if result.len() > init.len() {
        return Err(CMemLonger(result.len(), init.len()));
    }
    result.resize(init.len(), 0);
    Ok(result.iter().zip(init).map(|(a, b)| a ^ b).collect())
}

#[cfg(test)]
use crate::mem;

#[test]
fn test_restore() {
    let mut save = Vec::new();
    save.extend(b"FORM");
    save.extend(vec![0, 0, 0, 100]);
    save.extend(b"IFZSIFhd");
    save.extend(vec![0, 0, 0, 13]);
    save.extend(vec![0; 8]);
    save.extend(vec![0x43, 0x21, 0, 0, 10, 0]);
    save.extend(b"CMem");
    save.extend(vec![0, 0, 0, 3, 0, 0x3e, 1, 0]);
    save.extend(b"Stks");
    save.extend(vec![0, 0, 0, 54]);
    save.extend(vec![0, 0, 0, 0, 0, 0, 0, 3, 0, 1, 0, 2, 0, 3]); // frame 0
    save.extend(vec![
        0, 0, 1, 3, 1, 3, 0, 3, 0, 3, 0, 2, 0, 1, 0, 4, 0, 5, 0, 6,
    ]); // frame 1
    save.extend(vec![
        0, 0, 16, 19, 0, 3, 0, 3, 0, 6, 0, 5, 0, 4, 0, 7, 0, 8, 0, 9,
    ]); // frame 2

    let mut data = mem::default();
    let init = Saved {
        mem: data.clone(),
        stack: Vec::new(),
        pc: 10,
        call: Vec::new(),
    };
    let result = restore(&init, &save, 0x4321).unwrap();

    data[0x3f] = 1;
    assert_eq!(result.mem, data);
    assert_eq!(
        result.stack,
        vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]
    );
    assert_eq!(result.pc, 10);
    assert_eq!(result.call.len(), 2);

    let frame = &result.call[0];
    assert_eq!(frame.ret, 1);
    assert_eq!(frame.local, vec![3, 2, 1]);
    assert_eq!(frame.args, 2);
    assert_eq!(frame.store, true);

    let frame = &result.call[1];
    assert_eq!(frame.ret, 16);
    assert_eq!(frame.local, vec![6, 5, 4]);
    assert_eq!(frame.args, 2);
    assert_eq!(frame.store, false);
}

#[test]
fn test_uncompress() {
    assert_eq!(
        uncompress(&[1, 2, 3, 1, 2, 3], &[0, 2, 5, 7, 5]).unwrap(),
        [1, 2, 3, 4, 5, 6]
    );
    let init = [1; 261];
    let mut data = init.to_vec();
    data[260] = 0;
    assert_eq!(uncompress(&init, &[0, 255, 0, 3, 1]).unwrap(), data);
}
