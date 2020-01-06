extern crate rzif;
use self::rzif::*;

struct IO<'a, L: 'a + Iterator, C: 'a + Iterator> {
    output: &'a mut String,
    trans: &'a mut String,
    lines: &'a mut L,
    chars: &'a mut C,
    errors: &'a mut Vec<Error>,
}

impl<'a, L, C> Interface for IO<'a, L, C>
where
    L: Iterator<Item = (String, char)>,
    C: Iterator<Item = char>,
{
    fn write_screen(&mut self, str: &str, _: bool) {
        print!("{}", str);
        self.output.push_str(str);
    }
    fn write_transcript(&mut self, str: &str) {
        self.trans.push_str(str);
    }
    fn write_command(&mut self, _: &str) {}

    fn status(&mut self, _: &str) {}
    fn window_font(&mut self, font: u16) -> bool {
        match font {
            1 | 4 => true,
            _ => false,
        }
    }
    fn window_color(&mut self, _: u16, _: u16) {}
    fn window_style(&mut self, _: u16) {}
    fn window_set(&mut self, _: u16) {}
    fn window_buffer(&mut self, _: u16) {}
    fn window_split(&mut self, _: u16) {}
    fn window_cursor_set(&mut self, _: u16, _: u16) {}
    fn window_cursor_get(&mut self) -> (u16, u16) {
        (0, 0)
    }
    fn window_erase(&mut self, _: u16) {}
    fn window_line(&mut self) {}

    fn read<T: FnMut(&mut Self) -> bool>(
        &mut self,
        _: Vec<char>,
        _: String,
        _: u16,
        _: u16,
        _: T,
    ) -> (String, char) {
        let line = self.lines.next().unwrap();
        print!("{}{}", line.0, line.1);
        line
    }
    fn read_char<T: FnMut(&mut Self) -> bool>(&mut self, _: u16, _: T) -> char {
        self.chars.next().unwrap()
    }
    fn read_file(&mut self) -> String {
        String::new()
    }

    fn bleep(&mut self, _: u16) {}
    fn save(&mut self, _: &[u8]) -> bool {
        false
    }
    fn restore(&mut self) -> Vec<u8> {
        Vec::new()
    }
    fn restore_failed(&mut self, _: SaveError) {}
    fn error(&mut self, error: Error) {
        println!("{:?}", error);
        self.errors.push(error);
    }
}

pub fn main<L, C>(file: Vec<u8>, mut lines: L, mut chars: C) -> (String, String, Vec<Error>)
where
    L: Iterator<Item = (String, char)>,
    C: Iterator<Item = char>,
{
    let mut output = String::new();
    let mut trans = String::new();
    let mut errors = Vec::new();
    let config = Config {
        status: true,
        split: true,
        fixed_default: true,
        color: true,
        bold: true,
        italic: true,
        fixed: true,
        timed: false,
        screen: (77, 14),
        default_color: (2, 9),
        true_color: (0x0000, 0x7fff),
        picture: false,
        error: ErrorLevel::Always,
    };
    rzif::main(
        file,
        config,
        IO {
            output: &mut output,
            trans: &mut trans,
            lines: &mut lines,
            chars: &mut chars,
            errors: &mut errors,
        },
    );
    let lines = lines.collect::<Vec<_>>();
    if lines != Vec::new() {
        panic!("\nUnused input lines: {:?}", lines);
    }
    let chars = chars.collect::<Vec<_>>();
    if chars != Vec::new() {
        panic!("\nUnused input chars: {:?}", chars);
    }
    (output, trans, errors)
}

pub fn check(str: &str, exept: &str, end: bool) {
    let mut str = str.chars();
    let mut exept = exept.chars();
    while let Some(result) = str.next() {
        if let Some(exept) = exept.next() {
            if result == exept {
                print!("{}", result);
            } else {
                println!();
                panic!("\nresult: {:?}\nexepted: {:?}\n", result, exept);
            }
        } else if end {
            println!();
            panic!("\nresult:\n{}\n", str.collect::<String>());
        }
    }
    let exept = exept.collect::<String>();
    if exept != String::new() {
        println!();
        panic!("\nexepted:\n{}\n", exept);
    }
}

pub fn errors(err: Vec<Error>, mut exept: Vec<(Cause, (u16, u16))>) {
    exept.push((Cause::Quit, (0, 0)));
    let mut err = err.into_iter();
    for (cause, data) in exept.into_iter() {
        let err = err.next().unwrap();
        assert_eq!(cause, err.cause);
        assert_eq!(data, err.data);
    }
    assert!(err.next().is_none());
}
