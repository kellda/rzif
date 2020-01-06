pub use crate::err::{Cause, Error, SaveError, Trace};

/// The configuration of your interface
///
/// This tell to the interpreter what your interface can do. It use it to set/clear header flags. See the [interface](trait.interface.html) trait and [the format of the header](http://inform-fiction.org/zmachine/standards/z1point1/sect11.html) for more information.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Can your interface draw a status line ?
    pub status: bool,
    /// Can your interface split the screen ?
    pub split: bool,
    /// Is a fixed-pitch font the default ?
    pub fixed_default: bool,
    /// Can your interface make text colored ?
    pub color: bool,
    /// Can your interface make text bold ?
    pub bold: bool,
    /// Can your interface make text italic ?
    pub italic: bool,
    /// Can your interface make text fixed-pitch ?
    pub fixed: bool,
    /// Do your interface support timed input ?
    pub timed: bool,
    /// What is the size of the screen (width, height) ?
    pub screen: (u16, u16),
    /// What are the default [colors](doc/index.html#color-codes) (foreground, background) ?
    pub default_color: (u8, u8),
    /// What are the defaults [true colors](doc/color/index.html#true-colors) (foreground, background) ?
    pub true_color: (u16, u16),
    /// Do your interface support [character graphics font](http://inform-fiction.org/zmachine/standards/z1point1/sect16.html) ?
    pub picture: bool,
    /// When to report errors (see `[ErrorLevel](enum.ErrorLevel.html)`)
    pub error: ErrorLevel,
}

/// When to report errors
#[derive(Clone, Copy, Debug)]
pub enum ErrorLevel {
    /// Never report any error
    Never,
    /// Report the error only the first time
    Once,
    /// Always report any error
    Always,
    /// Quit on any error
    Quit,
}

#[cfg(test)]
pub static DEFAULT: Config = Config {
    status: true,
    split: true,
    fixed_default: true,
    color: true,
    bold: true,
    italic: true,
    fixed: true,
    timed: true,
    screen: (255, 255),
    default_color: (0, 0),
    true_color: (0, 0),
    picture: false,
    error: ErrorLevel::Always,
};

/// Callbacks to your interface
///
/// These functions are called when the interpreter needs to do outputs or get inputs.
#[cfg(not(test))]
pub trait Interface {
    /// Print a `text` to the screen, possibly in `fixed`-pitch font.
    fn write_screen(&mut self, text: &str, fixed: bool);
    /// Write a `text` to the [transcript](doc/index.html#transcript).
    fn write_transcript(&mut self, text: &str);
    /// Write a `text` to the [commands file](doc/index.html#command-file).
    fn write_command(&mut self, text: &str);

    /// Write the `text` in the status line (the top line of the screen).
    fn status(&mut self, text: &str);
    /// Set the [`font`](doc/index.html#fonts). Returns true if the font is available, false otherwise.
    fn window_font(&mut self, font: u16) -> bool;
    /// Set the current `foreground` and `background` [true colors](doc/index.html#true-colors).
    fn window_color(&mut self, foreground: u16, background: u16);
    /// Set the [`style`](doc/index.html#styles) to apply to the text.
    fn window_style(&mut self, style: u16);
    /// Select the current [`window`](doc/index.html#windows).
    fn window_set(&mut self, window: u16);
    /// Set the [`buffer`](doc/index.html#buffering) mode.
    fn window_buffer(&mut self, buffer: u16);
    /// [Split](doc/index.html#spliting) the screen.
    fn window_split(&mut self, lines: u16);
    /// Move the cursor to (`x`; `y`).
    fn window_cursor_set(&mut self, x: u16, y: u16);
    /// Get the position of the [cursor](doc/index.html#cursor).
    fn window_cursor_get(&mut self) -> (u16, u16);
    /// [Erase](doc/index.html#erasing-a-window) a `window`.
    fn window_erase(&mut self, window: u16);
    /// [Erase](doc/index.html#erasing-a-line) a line.
    fn window_line(&mut self);

    /// Read a line treminated by a `terminating` character, with the text `preload` already given and of maximum length `maxlen`.
    fn read<F: FnMut(&mut Self) -> bool>(
        &mut self,
        terminating: Vec<char>,
        preload: String,
        maxlen: u16,
        time: u16,
        routine: F,
    ) -> (String, char);
    /// Read a char.
    fn read_char<T: FnMut(&mut Self) -> bool>(&mut self, time: u16, callback: T) -> char;
    /// Read a line from a [command file](doc/index.html#command-file).
    fn read_file(&mut self) -> String;

    /// Emit the given [`bleep`](doc/index.html#bleeps).
    fn bleep(&mut self, bleep: u16);
    /// [Save](doc/index.html#save) some `data` into a file. Returns true in case of success, false otherwise.
    fn save(&mut self, data: &[u8]) -> bool;
    /// [Restore](doc/index.html#save) some data from a file. In case of failure an empty `Vec` can be returned.
    fn restore(&mut self) -> Vec<u8>;
    /// Called if the resore fail
    fn restore_failed(&mut self, cause: SaveError);
    /// Called on error
    fn error(&mut self, error: Error);
}

#[cfg(test)]
pub trait Interface {
    fn write_screen(&mut self, _: &str, _: bool) {}
    fn write_transcript(&mut self, _: &str) {}
    fn write_command(&mut self, _: &str) {}

    fn status(&mut self, _: &str) {}
    fn window_font(&mut self, _: u16) -> bool {
        false
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
        (String::new(), '\n')
    }
    fn read_char<T: FnMut(&mut Self) -> bool>(&mut self, _: u16, _: T) -> char {
        ' '
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
        panic!("Error: {:?}", error);
    }
}
