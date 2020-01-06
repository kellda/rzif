use crate::utils::IO;
use std::time::{Duration, Instant};

impl IO {
    pub fn read<T: FnMut(&mut Self) -> bool>(
        &mut self,
        terminate: Vec<char>,
        left: String,
        max: usize,
        time: u16,
        mut callback: T,
    ) -> (String, char) {
        let mut buf = left.chars().collect::<Vec<_>>();
        let mut i = buf.len();
        let mut h = self.hist.len();
        let mut ins = true;
        let timed = time != 0;
        let time = Duration::from_millis(u64::from(time) * 100);
        let mut last = Instant::now();
        loop {
            if timed && last.elapsed() >= time {
                self.printed = false;
                if callback(self) {
                    return (end(self, &buf), '\0');
                }
                last = Instant::now();
                if self.printed {
                    print!("{}", buf.iter().collect::<String>());
                    let offset = buf.len() - i;
                    if offset != 0 {
                        print!("\x1b[{}D", offset);
                    }
                }
            }
            if let Some(char) = self.getch.getch() {
                if char == '\n' || terminate.contains(&char) {
                    return (end(self, &buf), char);
                }
                match char {
                    '\x01' => {
                        if i > 0 {
                            print!("\x1b[{}D", i);
                            i = 0;
                        }
                    }
                    '\x02' => ins = !ins,
                    '\x03' => {
                        if i < buf.len() {
                            buf.remove(i);
                            print!("\x1b[K");
                            draw(&buf[i..], buf.len() - i)
                        }
                    }
                    '\x04' => {
                        if i < buf.len() {
                            print!("\x1b[{}C", buf.len() - i);
                            i = buf.len();
                        }
                    }
                    '\x05' | '\x06' => {} // page up / down
                    '\x08' => {
                        if i != 0 {
                            i -= 1;
                            buf.remove(i);
                            print!("\x1b[D\x1b[K");
                            draw(&buf[i..], buf.len() - i);
                        }
                    }
                    '\u{81}' => {
                        if h != 0 {
                            h -= 1;
                            buf = init(&self.hist[h], i);
                            i = buf.len();
                        }
                    }
                    '\u{82}' => {
                        if h < self.hist.len() {
                            h += 1;
                            if h == self.hist.len() {
                                h -= 1;
                                buf = init("", i);
                                i = 0;
                            } else {
                                buf = init(&self.hist[h], i);
                                i = buf.len();
                            }
                        }
                    }
                    '\u{83}' => {
                        if i > 0 {
                            i -= 1;
                            print!("\x1b[D");
                        }
                    }
                    '\u{84}' => {
                        if i < buf.len() {
                            i += 1;
                            print!("\x1b[C");
                        }
                    }
                    '\0'..='\x1f' | '\u{80}'..='\u{9f}' => {}
                    _ => {
                        if buf.len() < max {
                            if ins {
                                buf.insert(i, char);
                                draw(&buf[i..], buf.len() - i - 1);
                            } else {
                                if i == buf.len() {
                                    buf.push(char);
                                } else {
                                    buf[i] = char;
                                }
                                print!("{}", char);
                            }
                            i += 1;
                        }
                    }
                }
            }
        }
    }
}

fn draw(buf: &[char], back: usize) {
    print!("{}", buf.iter().collect::<String>());
    if back != 0 {
        print!("\x1b[{}D", back);
    }
}

fn init(str: &str, i: usize) -> Vec<char> {
    if i != 0 {
        print!("\x1b[{}D", i);
    }
    print!("\x1b[K{}", str);
    str.chars().collect()
}

fn end(io: &mut IO, buf: &[char]) -> String {
    println!();
    let str = buf.iter().collect::<String>();
    io.hist.push(str.clone());
    str
}
