use crate::{err::*, header::Header, mem::Mem};

pub struct Text {
    table: Option<usize>,
    unicode: Option<usize>,
    abbr: u16,
    v: u8,
}

static A0: [u8; 26] = *b"abcdefghijklmnopqrstuvwxyz";
static A1: [u8; 26] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
static A2: [u8; 26] = *b"\0\r0123456789.,!?_#'\"/\\-:()";
static A2V1: [u8; 26] = *b"\00123456789.,!?_#'\"/\\<-:()";
static UNICODE: [char; 69] = [
    'ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß', '»', '«', 'ë', 'ï', 'ÿ', 'Ë', 'Ï', 'á', 'é', 'í', 'ó', 'ú',
    'ý', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Ý', 'à', 'è', 'ì', 'ò', 'ù', 'À', 'È', 'Ì', 'Ò', 'Ù', 'â', 'ê',
    'î', 'ô', 'û', 'Â', 'Ê', 'Î', 'Ô', 'Û', 'å', 'Å', 'ø', 'Ø', 'ã', 'ñ', 'õ', 'Ã', 'Ñ', 'Õ', 'æ',
    'Æ', 'ç', 'Ç', 'þ', 'ð', 'Þ', 'Ð', '£', 'œ', 'Œ', '¡', '¿',
];

pub use self::decode::*;
mod decode;
pub use self::encode::*;
mod encode;

pub fn init(mem: &Mem, header: &Header) -> Result<Text, Error> {
    let v = mem[0];
    let addr = mem.byte(mem.loadw(0x34).unwrap());
    let table = if v >= 5 && addr != 0 {
        if addr + 78 > mem.len() {
            return error(Cause::AlphabetOut, (addr as u16, 0));
        }
        Some(addr)
    } else {
        None
    };
    let addr = header.get_extension(mem, 3)?;
    let unicode = if v >= 5 && addr != 0 {
        Some(addr as usize)
    } else {
        None
    };
    Ok(Text {
        table,
        unicode,
        abbr: mem.loadw(0x18).unwrap(),
        v,
    })
}

#[cfg(test)]
use crate::{header, mem};

#[test]
fn test_init() {
    let mut data = mem::default();
    data[0x00] = 5;
    data[0x18] = 1;
    let mut mem = mem::new(data.clone()).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(text.table, None);
    assert_eq!(text.abbr, 0x100);
    assert_eq!(text.v, 5);
    data[0x35] = 0x40;
    data.extend(vec![0; 78]);
    let mut mem = mem::new(data.clone()).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(text.table, Some(0x40));
    assert_eq!(text.abbr, 0x100);
    assert_eq!(text.v, 5);
    data[0x00] = 4;
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = init(&mem, &header).unwrap();
    assert_eq!(text.table, None);
    assert_eq!(text.abbr, 0x100);
    assert_eq!(text.v, 4);
}
