use crate::utils::IO;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

impl IO {
    pub fn file_out(&mut self) -> BufWriter<File> {
        'main: loop {
            print!("\nEnter file name: ");
            let path = PathBuf::from(
                self.read(vec!['\n'], String::new(), ::std::usize::MAX, 0, |_| false)
                    .0,
            );
            if path.exists() {
                print!("This file already exists. ");
                loop {
                    print!("Overwrite ? [y/n] ");
                    let answer = loop {
                        if let Some(answer) = self.getch.getch() {
                            break answer;
                        }
                    };
                    println!("{}", answer);
                    match answer {
                        'y' | 'Y' => break,
                        'n' | 'N' => continue 'main,
                        _ => continue,
                    }
                }
            }
            match File::create(path) {
                Ok(file) => return BufWriter::new(file),
                Err(err) => println!("Error opening the file: {}", err),
            }
        }
    }

    pub fn file_in(&mut self) -> BufReader<File> {
        loop {
            print!("\nEnter file name: ");
            let path = self
                .read(vec!['\n'], String::new(), ::std::usize::MAX, 0, |_| false)
                .0;
            match File::open(path) {
                Ok(file) => return BufReader::new(file),
                Err(err) => println!("Error opening the file: {}", err),
            }
        }
    }
}
