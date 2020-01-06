use crate::utils::IO;

impl IO {
    pub fn buffer(&mut self, str: &str, count: bool) {
        if self.buffering && self.current == 0 {
            if count {
                for char in str.chars() {
                    self.buffer.1 += 1;
                    self.buffer.2 += 1;
                    if char == ' ' || char == '\n' {
                        if self.buffer.1 > self.size.0 {
                            println!();
                            self.buffer.1 = self.buffer.2;
                        }
                        print!("{}{}", self.buffer.0, char);
                        self.buffer.0.clear();
                        self.buffer.2 = 0;
                        if char == '\n' {
                            self.buffer.1 = 0;
                        }
                    } else {
                        self.buffer.0.push(char);
                    }
                }
            } else {
                self.buffer.0 += str;
            }
        } else {
            print!("{}", str);
        }
    }

    pub fn flush(&mut self) {
        print!("{}", self.buffer.0);
        self.buffer.0.clear();
        self.buffer.1 = 0;
        self.buffer.2 = 0;
    }
}
