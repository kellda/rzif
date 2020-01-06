use std::{collections::VecDeque, io::*, str::Chars};

pub struct Getch {
    old: Termios,
    buffer: VecDeque<char>,
    stdin: Stdin,
}

pub fn init() -> Getch {
    let mut termios = Termios {
        i: 0,
        o: 0,
        c: 0,
        l: 0,
        cc: [0; 32],
    };
    unsafe { tcgetattr(0, &mut termios) };
    let old = termios.clone();
    termios.l &= !(0xa);
    termios.cc[5] = 0;
    termios.cc[7] = 0;
    unsafe { tcsetattr(0, 0, &termios) };
    Getch {
        old,
        buffer: VecDeque::new(),
        stdin: stdin(),
    }
}

impl Getch {
    pub fn getch(&mut self) -> Option<char> {
        stdout().flush().unwrap();
        if self.buffer.is_empty() {
            self.fill_buffer();
        }
        self.buffer.pop_front()
    }

    pub fn flush(&mut self) {
        self.fill_buffer();
    }

    fn fill_buffer(&mut self) {
        let mut str = String::new();
        self.stdin.read_to_string(&mut str).unwrap();
        let mut chars = str.chars();

        while let Some(mut char) = chars.next() {
            if char == '\x1b' {
                if let Some(result) = decode_esc(chars.clone()) {
                    char = result.0;
                    chars = result.1;
                }
            } else if char == '\x7f' {
                char = '\x08';
            }
            self.buffer.push_back(char);
        }
    }
}

fn decode_esc(mut chars: Chars) -> Option<(char, Chars)> {
    if chars.next()? != '[' {
        return None;
    }
    let char: u8 = match chars.next()? {
        'A' => 129,
        'B' => 130,
        'C' => 132,
        'D' => 131,
        mut char => {
            let mut num = String::new();
            while char != '~' {
                num.push(char);
                char = chars.next()?;
            }
            let num = num.parse().ok()?;
            match num {
                1..=6 => num,
                11..=15 => num + 122,
                17..=21 => num + 121,
                23..=24 => num + 120,
                _ => {
                    print!("\x1b7\nUnknown escape \\e[{}~\x1b8", num);
                    0x1b
                }
            }
        }
    };
    Some((char as char, chars))
}

impl Drop for Getch {
    fn drop(&mut self) {
        unsafe { tcsetattr(0, 0, &self.old) };
    }
}

#[derive(Clone)]
#[repr(C)]
struct Termios {
    i: u32,
    o: u32,
    c: u32,
    l: u32,
    cc: [u8; 32],
}

extern "C" {
    fn tcgetattr(fd: i32, termios: *mut Termios) -> i32;

    fn tcsetattr(fd: i32, actions: i32, termios: *const Termios) -> i32;
}
