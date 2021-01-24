use rzif::{Error, *};
use std::{
    fs::File,
    io::*,
    time::{Duration, Instant},
};

mod buffer;
mod color;
mod err;
mod file;
mod getch;
mod read;

pub struct IO {
    v: u8,
    size: (u16, u16),
    split: u16,
    current: u16,
    buffer: (String, u16, u16),
    buffering: bool,
    getch: getch::Getch,
    hist: Vec<String>,
    printed: bool,
    transcript: Option<BufWriter<File>>,
    cmd_out: Option<BufWriter<File>>,
    cmd_in: Option<BufReader<File>>,
}

impl Interface for IO {
    fn write_screen(&mut self, str: &str, _: bool) {
        self.buffer(str, true);
        self.printed = true;
    }

    fn write_transcript(&mut self, str: &str) {
        if self.transcript.is_none() {
            self.transcript = Some(self.file_out());
        }
        self.transcript
            .as_mut()
            .unwrap()
            .write_all(str.as_bytes())
            .unwrap();
    }

    fn write_command(&mut self, str: &str) {
        if self.cmd_out.is_none() {
            self.cmd_out = Some(self.file_out());
        }
        self.cmd_out
            .as_mut()
            .unwrap()
            .write_all(str.as_bytes())
            .unwrap();
    }

    fn status(&mut self, str: &str) {
        print!("\x1b7\x1b[H\x1b[7m{}\x1b8", str);
    }

    fn window_font(&mut self, font: u16) -> bool {
        match font {
            1 | 4 => true,
            _ => false,
        }
    }

    fn window_color(&mut self, foreground: u16, background: u16) {
        self.color(foreground as i16, true);
        self.color(background as i16, false);
    }

    fn window_style(&mut self, style: u16) {
        if style == 0 {
            self.buffer("\x1b[m", false);
            return;
        }
        if style & 0x01 != 0 {
            self.buffer("\x1b[7m", false);
        }
        if style & 0x02 != 0 {
            self.buffer("\x1b[1m", false);
        }
        if style & 0x04 != 0 {
            self.buffer("\x1b[4m", false);
        }
        if style >= 16 {
            panic!();
        }
    }

    fn window_set(&mut self, window: u16) {
        if window == self.current {
            return;
        }
        self.current = window;
        match window {
            0 => print!("\x1b[{};{}r\x1b8", self.split + 1, self.size.1),
            1 => print!("\x1b7\x1b[H"),
            _ => panic!(),
        }
    }

    fn window_buffer(&mut self, mode: u16) {
        match mode {
            1 => self.buffering = true,
            0 => {
                self.flush();
                self.buffering = false;
            }
            _ => panic!(),
        }
    }

    fn window_split(&mut self, lines: u16) {
        if self.current != 0 {
            return;
        }
        self.split = lines;
        if self.v == 3 {
            print!("\x1b7\x1b[{};{}r\x1b[1J\x1b8", lines + 1, self.size.1);
        } else {
            print!("\x1b7\x1b[{};{}r\x1b8", lines + 1, self.size.1);
        }
    }

    fn window_cursor_set(&mut self, x: u16, y: u16) {
        print!("\x1b[{};{}H", x, y);
    }

    fn window_cursor_get(&mut self) -> (u16, u16) {
        use std::io::*;
        self.getch.flush();
        print!("\x1b[6n");
        stdout().flush().unwrap();
        let mut buf = String::new();
        while stdin().read_to_string(&mut buf).unwrap() == 0 {}
        let mut chars = buf.chars();
        assert_eq!(chars.next(), Some('\x1b'));
        assert_eq!(chars.next(), Some('['));
        let mut buf = String::new();
        let mut char = chars.next().unwrap();
        let mut pos = (0, 0);
        while char != ';' {
            buf.push(char);
            char = chars.next().unwrap();
        }
        pos.0 = buf.parse().unwrap();
        buf.clear();
        char = chars.next().unwrap();
        while char != 'R' {
            buf.push(char);
            char = chars.next().unwrap();
        }
        pos.1 = buf.parse().unwrap();
        pos
    }

    fn window_erase(&mut self, window: u16) {
        self.flush();
        match window {
            0 | 1 => {
                let pos = match window {
                    0 => {
                        if self.v < 5 {
                            self.size.1
                        } else {
                            self.split + 1
                        }
                    }
                    1 => 0,
                    _ => panic!(),
                };
                print!("\x1b[{}H\x1b[{}J\x1b[{}H", self.split + 1, window, pos);
            }
            0xffff => {
                print!(
                    "\x1b[0r\x1b[2J\x1b[{}H",
                    if self.v < 5 { self.size.1 } else { 1 }
                );
                self.split = 0;
                self.current = 0;
            }
            _ => panic!(),
        }
    }

    fn window_line(&mut self) {
        self.buffer("\x1b[K", false);
    }

    fn read<T: FnMut(&mut Self) -> bool>(
        &mut self,
        terminate: Vec<char>,
        left: String,
        max: u16,
        time: u16,
        callback: T,
    ) -> (String, char) {
        self.flush();
        self.read(terminate, left, max as usize, time, callback)
    }

    fn read_char<T: FnMut(&mut Self) -> bool>(&mut self, time: u16, mut callback: T) -> char {
        self.flush();
        let timed = time != 0;
        let time = Duration::from_millis(u64::from(time) * 100);
        let mut last = Instant::now();
        loop {
            if timed && last.elapsed() >= time {
                if callback(self) {
                    return '\0';
                }
                last = Instant::now();
            }
            if let Some(char) = self.getch.getch() {
                return char;
            }
        }
    }

    fn read_file(&mut self) -> String {
        if self.cmd_in.is_none() {
            self.cmd_in = Some(self.file_in());
        }
        let mut line = String::new();
        self.cmd_in.as_mut().unwrap().read_line(&mut line).unwrap();
        line
    }

    fn bleep(&mut self, _: u16) {
        print!("\x07");
    }

    fn save(&mut self, data: &[u8]) -> bool {
        let mut file = self.file_out();
        file.write_all(data).is_ok()
    }

    fn restore(&mut self) -> Vec<u8> {
        let mut file = self.file_in();
        let mut result = Vec::new();
        file.read_to_end(&mut result).unwrap();
        result
    }

    fn restore_failed(&mut self, cause: SaveError) {
        if let SaveError::GamesDiffer = cause {
            println!("This file wasn't saved from this game");
        } else {
            println!("Corrupted save file: {:?}", cause);
        }
    }

    fn error(&mut self, error: Error) {
        self.flush();
        if error.cause == Cause::Quit {
            return;
        }
        err::display(error);
    }
}

pub fn init(v: u8, w: u16, h: u16) -> IO {
    print!("\x1b[2J");
    if v <= 3 {
        print!("\x1b[2;{}r", h);
    }
    if v <= 4 {
        print!("\x1b[{}H", h);
    } else {
        print!("\x1b[H");
    }
    IO {
        v,
        size: (w, h),
        split: 0,
        current: 0,
        buffer: (String::new(), 0, 0),
        buffering: true,
        getch: getch::init(),
        hist: Vec::new(),
        printed: false,
        transcript: None,
        cmd_out: None,
        cmd_in: None,
    }
}

impl Drop for IO {
    fn drop(&mut self) {
        print!("\x1b[m\x1b[0r\x1b[{}H", self.size.1);
    }
}
