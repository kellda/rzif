use crate::err::*;
use std::time::UNIX_EPOCH;

pub fn eq(a: u16, b: u16) -> bool {
    a == b
}
pub fn lt(a: u16, b: u16) -> bool {
    (a as i16) < (b as i16)
}
pub fn gt(a: u16, b: u16) -> bool {
    (a as i16) > (b as i16)
}

pub fn add(a: u16, b: u16) -> u16 {
    a.wrapping_add(b)
}
pub fn sub(a: u16, b: u16) -> u16 {
    a.wrapping_sub(b)
}
pub fn mul(a: u16, b: u16) -> u16 {
    (a as i16).wrapping_mul(b as i16) as u16
}
pub fn div(a: u16, b: u16) -> Result<u16, Error> {
    if b == 0 {
        return error(Cause::DivByZero, (0, 0));
    }
    Ok((a as i16).wrapping_div(b as i16) as u16)
}
pub fn rem(a: u16, b: u16) -> Result<u16, Error> {
    if b == 0 {
        return error(Cause::DivByZero, (0, 0));
    }
    Ok((a as i16).wrapping_rem(b as i16) as u16)
}
pub fn log(a: u16, b: u16) -> u16 {
    if b & 0x8000 == 0 {
        a << b
    } else {
        a >> (!b + 1)
    }
}
pub fn art(a: u16, b: u16) -> u16 {
    if b & 0x8000 == 0 {
        ((a as i16) << b) as u16
    } else {
        ((a as i16) >> (!b + 1)) as u16
    }
}

pub struct Random(u64, u16);

impl Random {
    pub fn rand(&mut self, range: u16) -> u16 {
        match range {
            0 => {
                self.0 = random().into();
                self.1 = 0;
            }
            1..=0x7fff => {
                return match *self {
                    Random(0, 0) => random(),
                    Random(_, 0) => {
                        self.0 = self.0.wrapping_mul(0x015a_4e35).wrapping_add(1);
                        (self.0 >> 16) as u16
                    }
                    Random(_, _) => {
                        self.0 += 1;
                        if self.0 as u16 == self.1 {
                            self.0 = 0;
                            self.1
                        } else {
                            self.0 as u16 - 1
                        }
                    }
                } % range
                    + 1
            }
            _ => {
                let seed = !range + 1;
                if seed < 1000 {
                    self.0 = 0;
                    self.1 = seed;
                } else {
                    self.0 = seed.into();
                    self.1 = 0;
                };
            }
        }
        0
    }
}

pub fn init() -> Random {
    Random(0, 0)
}

fn random() -> u16 {
    UNIX_EPOCH
        .elapsed()
        .unwrap_or_else(|err| err.duration())
        .subsec_nanos() as u16
}

#[test]
fn test_comp() {
    assert!(eq(1, 1));
    assert!(!eq(1, 2));
    assert!(lt(0xa000, 0x1000));
    assert!(!lt(0x7000, 0x1000));
    assert!(gt(0x1000, 0xa000));
    assert!(!gt(0x1000, 0x7000));
}

#[test]
fn test_art() {
    assert_eq!(add(0xfff0, 0x0020), 0x10);
    assert_eq!(sub(0x0020, 0x0040), 0xffe0);
    let m11 = sub(0, 11);
    let m13 = sub(0, 13);
    assert_eq!(mul(m11, 2) as i16, -22);
    assert_eq!(mul(m13, -5i16 as u16), 65);
    assert_eq!(div(m11, 2).unwrap() as i16, -5);
    assert_eq!(div(m13, (-5i16) as u16).unwrap(), 2);
    assert_eq!(rem(m11, 2).unwrap() as i16, -1);
    assert_eq!(rem(m13, (-5i16) as u16).unwrap() as i16, -3);
}

#[test]
fn test_log() {
    assert_eq!(log(0x80, 8), 0x8000);
    assert_eq!(log(0x80, 9), 0x0000);
    assert_eq!(log(0x80, -7i16 as u16), 0x01);
    assert_eq!(log(0x80, -8i16 as u16), 0x00);
    assert_eq!(log(0x8000, -8i16 as u16), 0x80);

    assert_eq!(art(0x80, 8), 0x8000);
    assert_eq!(art(0x80, 9), 0x0000);
    assert_eq!(art(0x80, -7i16 as u16), 0x01);
    assert_eq!(art(0x80, -8i16 as u16), 0x00);
    assert_eq!(art(0x8000, -8i16 as u16), 0xff80);
}

#[test]
fn test_rand() {
    random();
    let mut rand = init();
    assert!(rand.rand(10) - 1 < 10);
    assert_eq!(rand.rand(1), 1);
    assert_eq!(rand.rand(0xff88), 0);
    let a = rand.rand(0x7fff);
    assert_eq!(rand.rand(0x7fff), 2);
    assert_eq!(rand.rand(0xff88), 0);
    let b = rand.rand(0x7fff);
    assert_eq!(a, b);
    assert_eq!(rand.rand(0x88ff), 0);
    let a = rand.rand(0x7fff);
    assert_eq!(rand.rand(0x88ff), 0);
    let b = rand.rand(0x7fff);
    assert_eq!(a, b);
    assert_eq!(rand.rand(0), 0);
}
