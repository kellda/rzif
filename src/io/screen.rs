use crate::{
    err::*,
    interface::{Config, Interface},
    mem::Mem,
    obj::Object,
    state::State,
    text::Text,
};

pub struct Screen {
    v: u8,
    font: u16,
    width: usize,
}

impl Screen {
    pub fn status<I: Interface>(
        &self,
        mem: &Mem,
        text: &Text,
        state: &mut State,
        obj: &Object,
        interface: &mut I,
    ) -> Result<(), Error> {
        let mut name = obj.name(mem, text, state.get_var(mem, 0x10)?)?;
        if name.chars().count() > self.width {
            let mut last_word = 0;
            let mut last_char = 0;
            for (n, (i, char)) in name.char_indices().enumerate() {
                last_char = i;
                if n + 1 >= self.width {
                    break;
                }
                if char == ' ' {
                    last_word = i;
                }
            }
            if last_word == 0 {
                name.truncate(last_char);
            } else {
                name.truncate(last_word + 1);
            }
            name.push('…');
        }
        if name.len() + 9 > self.width {
            interface.status(&format!("{:1$}", name, self.width));
            return Ok(());
        }
        let score = if self.v == 3 && mem[1] & 0x02 != 0 {
            let hours = state.get_var(mem, 0x11)?;
            if hours < 12 {
                format!("{:02}:{:02} AM", hours, state.get_var(mem, 0x12)?)
            } else {
                format!("{:02}:{:02} PM", hours - 12, state.get_var(mem, 0x12)?)
            }
        } else {
            format!(
                "{}/{}",
                state.get_var(mem, 0x11)? as i16,
                state.get_var(mem, 0x12)? as i16
            )
        };
        let status = format!("{:2$} {:8}", name, score, self.width - 9);
        interface.status(&status);
        Ok(())
    }

    pub fn font<I: Interface>(&mut self, interface: &mut I, font: u16) -> u16 {
        if font == 0 {
            return self.font;
        }
        if interface.window_font(font) {
            let old = self.font;
            self.font = font;
            return old;
        }
        0
    }

    pub fn color<I: Interface>(
        &self,
        interface: &mut I,
        foreground: u16,
        background: u16,
    ) -> Result<(), Error> {
        let foreground = get_true(foreground)?;
        let background = get_true(background)?;
        interface.window_color(foreground, background);
        Ok(())
    }

    pub fn true_color<I: Interface>(&self, interface: &mut I, foreground: u16, background: u16) {
        interface.window_color(foreground, background);
    }

    pub fn style<I: Interface>(&self, interface: &mut I, style: u16) {
        interface.window_style(style);
    }

    pub fn window<I: Interface>(&self, interface: &mut I, window: u16) {
        interface.window_set(window);
    }

    pub fn buffer<I: Interface>(&self, interface: &mut I, mode: u16) {
        interface.window_buffer(mode);
    }

    pub fn split_window<I: Interface>(&self, interface: &mut I, height: u16) {
        interface.window_split(height);
    }

    pub fn set_cursor<I: Interface>(&self, interface: &mut I, line: u16, column: u16) {
        interface.window_cursor_set(line, column);
    }

    pub fn get_cursor<I: Interface>(&self, interface: &mut I) -> (u16, u16) {
        interface.window_cursor_get()
    }

    pub fn erase_window<I: Interface>(&self, interface: &mut I, window: u16) {
        interface.window_erase(window);
    }

    pub fn erase_line<I: Interface>(&self, interface: &mut I) {
        interface.window_line();
    }
}

fn get_true(color: u16) -> Result<u16, Error> {
    Ok(match color {
        0 => 0xfffe,
        1 => 0xffff,
        2 => 0x0000,
        3 => 0x001D,
        4 => 0x0340,
        5 => 0x03BD,
        6 => 0x59A0,
        7 => 0x7C1F,
        8 => 0x77A0,
        9 => 0x7FFF,
        _ => return error(Cause::BadColor, (color, 0)),
    })
}

pub fn init(mem: &Mem, config: &Config) -> Screen {
    Screen {
        v: mem[0],
        font: 1,
        width: config.screen.0 as usize,
    }
}

#[cfg(test)]
struct Output(String);

#[cfg(test)]
impl Interface for Output {
    fn status(&mut self, str: &str) {
        self.0 = str.to_string();
    }
    fn window_font(&mut self, font: u16) -> bool {
        font == 1 || font == 4
    }
}

#[cfg(test)]
use crate::{header, interface, mem, obj, state, text};

#[test]
fn test() {
    let mut data = mem::default();
    data[0x00] = 0x03;
    data[0x0b] = 0x46;
    data[0x0d] = 0x40;
    data[0x0f] = 0x46;
    data.extend(vec![0; 0x4c]);
    data.extend(vec![
        0x8d, 0x05, 0x11, 0xaa, 0x46, 0x34, 0x00, 0x9c, 0x52, 0xf1, 0xa4, 0xa5,
    ]);
    let mut mem = mem::new(data).unwrap();
    let header = header::init_test(&mut mem);
    let text = text::init(&mem, &header).unwrap();
    let mut state = state::init(&mem);
    let mut screen = init(&mem, &interface::DEFAULT);
    let obj = obj::init(&mem);
    let mut output = Output(String::new());

    assert_eq!(screen.font(&mut output, 0), 1);
    assert_eq!(screen.font(&mut output, 4), 1);
    assert_eq!(screen.font(&mut output, 2), 0);
    assert_eq!(screen.font(&mut output, 0), 4);
    assert_eq!(screen.font(&mut output, 3), 0);
    assert_eq!(screen.font(&mut output, 1), 4);

    state.set_var(&mut mem, 0x10, 1).unwrap();
    screen.width = 4;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hel…");

    screen.width = 6;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hello…");

    screen.width = 7;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hello …");

    screen.width = 10;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hello …   ");

    screen.width = 11;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hello World");

    screen.width = 19;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hello World        ");

    screen.width = 20;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hello World 0/0     ");

    state.set_var(&mut mem, 0x11, -99i16 as u16).unwrap();
    state.set_var(&mut mem, 0x12, 9999).unwrap();
    screen.width = 21;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hello World  -99/9999");

    let flags = mem.loadb(0x01).unwrap();
    mem.storeb(0x01, flags | 0x02).unwrap();
    state.set_var(&mut mem, 0x11, 1).unwrap();
    state.set_var(&mut mem, 0x12, 5).unwrap();
    screen.width = 22;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hello World   01:05 AM");

    state.set_var(&mut mem, 0x11, 22).unwrap();
    state.set_var(&mut mem, 0x12, 30).unwrap();
    screen.width = 23;
    screen
        .status(&mem, &text, &mut state, &obj, &mut output)
        .unwrap();
    assert_eq!(&output.0, "Hello World    10:30 PM");
}
