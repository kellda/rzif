use rzif::{Cause::*, Error, Trace::*};

pub fn display(error: Error) {
    let Error {
        cause,
        data,
        mut trace,
        fatal,
    } = error;

    if fatal {
        eprint!("Fatal error: ");
    } else {
        eprint!("Error: ");
    };

    match cause {
        Quit => eprintln!("the game has quit"),
        BadVer => eprintln!("the version {} is not supported", data.0),
        TooShort => eprintln!("the file is too short to hold the header"),
        StaticOut => out("static memory", data.0),
        WriteOut => eprintln!("tried to write above static memory mark at ${:04x}", data.0),
        ReadOut => eprintln!("tried to read out of bounds at ${:04x}", data.0),
        DivByZero => eprintln!("tried to divide by zero"),
        MissingOperand => eprintln!("exepted {} operands, found only {}", data.0, data.1),
        BadOpcode => eprintln!(
            "opcode {}:{:x} don't exist",
            match data.0 {
                0 => "0OP",
                1 => "1OP",
                2 => "2OP",
                3 => "VAR",
                4 => "EXT",
                _ => unreachable!(),
            },
            data.1
        ),
        StackUnderflow => eprintln!("the stack underflowed"),
        NoVar => invalid("variable", data.0),
        NoLocal => println!("local variable {} don't exists", data.0),
        NoLocalInMain => eprintln!("can't access local variable from the main routine"),
        MainReturned => eprintln!("the main routine returned"),
        PcOut => eprintln!("the program counter is out of bounds"),
        TooManyLocals => eprintln!("a routine can't have more that 15 local variables"),
        TextBufferOut => out("text buffer", data.0),
        TerminatingOut => out("terminating characters table", data.0),
        StrOut => eprintln!("tried to print a string that is out of bounds"),
        AlphabetOut => out("alphabet table", data.0),
        UnicodeOut => out("unicode translation table", data.0),
        BadUnicodeChar => eprintln!("unicode char U+{:04x} is not valid", data.0),
        BadZSCIIChar => invalid("ZSCII char", data.0),
        NestedAbbr => eprintln!("tried to use an abbreviation in another one"),
        AbbrIncompleteZSCII => eprintln!("the abbreviation ended with an incomplete ZSCII char"),
        BadObj => invalid("obect", data.0),
        NotChildOfParent => eprintln!("object {} is not a child of his parent {}", data.0, data.1),
        BadAttr => invalid("attribute", data.0),
        BadProp => invalid("property", data.0),
        GetLongProp | PutLongProp => eprintln!(
            "the propery {} of the object {} is longer that two bytes",
            data.1, data.0
        ),
        NoProp => eprintln!(
            "the property {} of the object {} don't exists",
            data.1, data.0
        ),
        BadInputStream => invalid("intput stream", data.0),
        BadOutputStream => invalid("output stream", data.0),
        NoOutputS3 => eprintln!("can't disable output stream 3 while it isn't enabled"),
        OutputS3Overflow => eprintln!("can't enable output stream 3 more than 16 times"),
        BadColor => invalid("color", data.0),
        SaveInterrupt => eprintln!("can't save during an interupt routine"),
    }

    if trace.is_empty() {
        eprintln!("  while initializing the interpreter");
    } else {
        while let Some(frame) = trace.pop() {
            let (text, addr) = match frame {
                Decode(addr) => ("decoding instruction", addr),
                Exec(addr) => ("executing instruction", addr),
                String(addr) => ("decoding string", addr),
                Abbr(addr) => ("decoding abbreviation", addr),
                Rout(addr) => ("calling routine", addr),
            };
            eprintln!("  while {} at ${:x}", text, addr);
        }
    }
}

fn out(text: &str, addr: u16) {
    eprintln!("the {} at ${:04x} is out of bounds", text, addr);
}

fn invalid(text: &str, num: u16) {
    eprintln!("{} {} is not valid", text, num);
}
