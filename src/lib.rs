//! A z-machine interpreter
//!
//! The entry point is the `main` function.
//! Take a look at the binary part of this crate to see an example of how to use.
//! This interpreter follow [The Z-Machine Standards Document](http://inform-fiction.org/zmachine/standards/z1point1/index.html) version 1.1.

#![allow(clippy::too_many_arguments)]

use self::fund::*;
mod fund {
    pub mod alu;
    pub mod instr;
    pub mod mem;
    pub mod rout;
    pub mod state;
    pub mod text;
}

use self::io::*;
mod io {
    pub mod input;
    pub mod out;
    pub mod screen;
    pub mod sound;
}

use self::tables::*;
mod tables {
    pub mod dict;
    pub mod header;
    pub mod obj;
    pub mod opcode;
}

use self::other::*;
mod other {
    pub mod err;
    pub mod interface;
    pub mod machine;
    pub mod quetzal;
}

pub mod doc;
pub use self::interface::*;

/// Starts the z-machine interpreter
///
/// The first argument is the contents of the storyfile to play.
/// The second is the configuration of your inferface.
/// The third is the callback functions this crate use to interact with your interface.
/// Before calling this function, the screen must be prepared as described [here](crate::doc#starting-a-game).\
/// This function returns at the end of the game
pub fn main<I: Interface>(file: Vec<u8>, config: Config, mut interface: I) {
    let error = machine::init(file, config, &mut interface).unwrap_err();
    interface.error(error);
}
