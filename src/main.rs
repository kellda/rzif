use std::{env, fs, thread, time};
mod utils;

fn main() {
    let mut args = env::args();
    let name = args.next().unwrap_or_else(|| "rzif".to_string());
    let path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("Usage: {} path/to/storyfile [never|once|always|quit]", name);
            return;
        }
    };
    let story = match fs::read(path) {
        Ok(story) => story,
        Err(err) => {
            eprintln!("Error opening the file: {}", err);
            return;
        }
    };
    let level = args.next().unwrap_or_else(|| "always".to_string());

    let config = rzif::Config {
        status: true,
        split: true,
        fixed_default: true,
        color: true,
        bold: true,
        italic: true,
        fixed: true,
        timed: true,
        screen: get_size(),
        default_color: (2, 9),
        true_color: (0x0000, 0x7fff),
        picture: false,
        error: match level.as_str() {
            "never" => rzif::ErrorLevel::Never,
            "once" => rzif::ErrorLevel::Once,
            "always" => rzif::ErrorLevel::Always,
            "quit" => rzif::ErrorLevel::Quit,
            _ => {
                eprintln!("invalid parameter: {}", level);
                thread::sleep(time::Duration::from_secs(2));
                rzif::ErrorLevel::Always
            }
        },
    };

    let io = utils::init(story[0], config.screen.0, config.screen.1);

    rzif::main(story, config, io);
}

fn get_size() -> (u16, u16) {
    let size = Size(0, 0, 0, 0);
    unsafe { ioctl(1, 0x5413, &size) };
    (size.1, size.0)
}

#[repr(C)]
struct Size(u16, u16, u16, u16);

extern "C" {
    fn ioctl(fd: i32, request: i32, ...) -> i32;
}
