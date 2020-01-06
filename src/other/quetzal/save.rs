use crate::{mem::Mem, state::Saved};

pub fn save(mem: &Mem, init: &Saved, saved: &Saved, checksum: u16) -> Vec<u8> {
    let mut save = b"FORM\0\0\0\0IFZSIFhd\0\0\0\x0d".to_vec();
    save.extend_from_slice(&saved.mem[0x02..0x04]);
    save.extend_from_slice(&saved.mem[0x12..0x18]);
    save.extend_from_slice(&bytes(checksum as usize)[2..4]);
    save.extend_from_slice(&bytes(saved.pc)[1..4]);
    save.push(0);

    let data = compress(&init.mem, &saved.mem);
    let len = data.len();
    save.extend(b"CMem");
    save.extend_from_slice(&bytes(len));
    save.extend(data);
    if len & 1 != 0 {
        save.push(0);
    }

    let mut stacks = vec![0; 6];
    stacks.extend_from_slice(&bytes(saved.stack[0].len())[2..4]);
    extend_u16(&saved.stack[0], &mut stacks);
    for frame in saved.stack.iter().skip(1).zip(&saved.call) {
        stacks.extend_from_slice(&bytes(frame.1.ret)[1..4]);
        let mut flags = frame.1.local.len() as u8;
        let save = if frame.1.store {
            mem[frame.1.ret - 1]
        } else {
            flags += 16;
            0
        };
        stacks.push(flags);
        stacks.push(save);
        stacks.push(0x7f >> (7 - frame.1.args));
        stacks.extend_from_slice(&bytes(frame.0.len())[2..4]);
        extend_u16(&frame.1.local, &mut stacks);
        extend_u16(&frame.0, &mut stacks);
    }

    let len = stacks.len();
    save.extend(b"Stks");
    save.extend_from_slice(&bytes(len));
    save.extend(stacks);
    if len & 1 != 0 {
        save.push(0);
    }

    let len = save.len() - 8;
    save[4..8].copy_from_slice(&bytes(len));
    save
}

fn bytes(data: usize) -> [u8; 4] {
    let mut result = [0; 4];
    for (i, res) in result.iter_mut().enumerate() {
        *res = (data >> (24 - 8 * i)) as u8;
    }
    result
}

fn extend_u16(data: &[u16], dest: &mut Vec<u8>) {
    for &byte in data {
        dest.extend_from_slice(&[(byte >> 8) as u8, byte as u8]);
    }
}

fn compress(init: &[u8], mem: &[u8]) -> Vec<u8> {
    let xor = mem.iter().zip(init).map(|(a, b)| a ^ b);
    let mut result = Vec::new();
    let mut count = 0;
    for byte in xor {
        if byte == 0 {
            if count == 0xff {
                result.push(0);
                result.push(0xff);
                count = 0;
            } else {
                count += 1;
            }
        } else {
            if count != 0 {
                result.push(0);
                result.push(count - 1);
                count = 0;
            }
            result.push(byte);
        }
    }
    result
}

#[cfg(test)]
use crate::{mem, state::Frame};

#[test]
fn test_save() {
    let data = mem::default();
    let mut mem = mem::new(data).unwrap();
    let init = Saved {
        mem: mem.save(),
        stack: Vec::new(),
        pc: 10,
        call: Vec::new(),
    };
    mem.storeb(0x3f, 1).unwrap();
    let mut saved = Saved {
        mem: mem.save(),
        stack: Vec::new(),
        pc: 10,
        call: Vec::new(),
    };
    saved.stack.push(vec![1, 2, 3]);
    saved.stack.push(vec![4, 5, 6]);
    saved.stack.push(vec![7, 8, 9]);
    saved.call.push(Frame {
        ret: 1,
        local: vec![3, 2, 1],
        args: 2,
        store: true,
    });
    saved.call.push(Frame {
        ret: 16,
        local: vec![6, 5, 4],
        args: 2,
        store: false,
    });

    let mut exepted = Vec::new();
    exepted.extend(b"FORM");
    exepted.extend(vec![0, 0, 0, 100]);
    exepted.extend(b"IFZSIFhd");
    exepted.extend(vec![0, 0, 0, 13]);
    exepted.extend(vec![0; 8]);
    exepted.extend(vec![0x43, 0x21, 0, 0, 10, 0]);
    exepted.extend(b"CMem");
    exepted.extend(vec![0, 0, 0, 3, 0, 0x3e, 1, 0]);
    exepted.extend(b"Stks");
    exepted.extend(vec![0, 0, 0, 54]);
    exepted.extend(vec![0, 0, 0, 0, 0, 0, 0, 3, 0, 1, 0, 2, 0, 3]); // frame 0
    exepted.extend(vec![
        0, 0, 1, 3, 1, 3, 0, 3, 0, 3, 0, 2, 0, 1, 0, 4, 0, 5, 0, 6,
    ]); // frame 1
    exepted.extend(vec![
        0, 0, 16, 19, 0, 3, 0, 3, 0, 6, 0, 5, 0, 4, 0, 7, 0, 8, 0, 9,
    ]); // frame 2
    assert_eq!(save(&mem, &init, &saved, 0x4321), exepted);
}

#[test]
fn test_compress() {
    assert_eq!(
        compress(&[1, 2, 3, 1, 2, 3], &[1, 2, 3, 4, 5, 6]),
        vec![0, 2, 5, 7, 5]
    );
    let init = [1; 261];
    let mut data = init;
    data[260] = 0;
    assert_eq!(compress(&init, &data), vec![0, 255, 0, 3, 1]);
}
