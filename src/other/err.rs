#[derive(Clone, Debug)]
/// Information about the error that occured
pub struct Error {
    /// the cause of the error
    pub cause: Cause,
    /// some information about the error
    pub data: (u16, u16),
    /// backtrace
    pub trace: Vec<Trace>,
    /// is this error fatal ?
    pub fatal: bool,
}

pub fn err(cause: Cause, data: (u16, u16)) -> Error {
    Error {
        cause,
        data,
        trace: Vec::new(),
        fatal: false,
    }
}

pub fn error<T>(cause: Cause, data: (u16, u16)) -> Result<T, Error> {
    // or Result<!, Error>
    Err(Error {
        cause,
        data,
        trace: Vec::new(),
        fatal: false,
    })
}

pub fn fatal<T>(mut result: Result<T, Error>) -> Result<T, Error> {
    if let Err(ref mut err) = result {
        err.fatal = true;
    }
    result
}

pub fn trace<T>(mut result: Result<T, Error>, trace: Trace) -> Result<T, Error> {
    if let Err(ref mut err) = result {
        err.trace.push(trace);
    }
    result
}

#[derive(Clone, Copy, Debug)]
/// A frame of the backtrace
pub enum Trace {
    /// Decoding opcode
    Decode(usize),
    /// Executing opcode
    Exec(usize),
    /// Decoding string
    String(usize),
    /// Decoding abbreviation
    Abbr(usize),
    /// Calling routine
    Rout(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// The cause of the error that occured
pub enum Cause {
    /// The game has quit
    Quit,
    /// Storyfile was not in a supported version\
    /// data: the version of this storyfile
    BadVer,
    /// Storyfile too short (it must be at least $40 bytes long to hold the header)\
    /// data: length of the storyfile
    TooShort,
    /// End of static memory out of bounds\
    /// data: end of the static memory
    StaticOut,
    /// Tried to write outside of dynamic memory\
    /// data: address writen
    WriteOut,
    /// Tried to read out of bounds\
    /// data: address readed
    ReadOut,
    /// Tried to divide by zero
    DivByZero,
    /// Not enough operands given
    /// data: minimum nuber of operands, current number of operands
    MissingOperand,
    /// This opcode is invalid\
    /// data: operand count, opcode number
    BadOpcode,
    /// The stack underflowed
    StackUnderflow,
    /// Invalid variable id\
    /// data: variable id
    NoVar,
    /// Non-existent local variable\
    /// data: variable id
    NoLocal,
    /// Access to a local variable from main routine
    NoLocalInMain,
    /// Main routine returned
    MainReturned,
    /// Program counter out of bounds
    PcOut,
    /// Routine called that have more than 15 local variables\
    /// data: number of local variables
    TooManyLocals,
    /// Text buffer out of bounds\
    /// data: address of the text buffer
    TextBufferOut,
    /// Terminating charachters table out of bounds\
    /// data: address of the table
    TerminatingOut,
    /// Part of string out of bounds\
    /// The address of the string will be in the backtrace
    StrOut,
    /// Alphabet table out of bounds\
    /// data: address of the table
    AlphabetOut,
    /// Unicode translation table out of bounds\
    /// data: address of the table
    UnicodeOut,
    /// Invalid unicode char was encontered\
    /// data: unicode char
    BadUnicodeChar,
    /// Invalid ZSCII char was encontered\
    /// data: ZSCII char
    BadZSCIIChar,
    /// Abbreviation containing another one\
    /// The addresses of the abbreviations will be in the backtrace
    NestedAbbr,
    /// Abbreviation ending with an incomplete char\
    /// The address of the abbreviation will be in the backtrace
    AbbrIncompleteZSCII,
    /// Invalid object\
    /// data: object number
    BadObj,
    /// Object not child of his parent\
    /// data: number of the child and his parent
    NotChildOfParent,
    /// Invalid attribute\
    /// data: attribute number
    BadAttr,
    /// Invalid property\
    /// data: property number
    BadProp,
    /// Got property larger than two bytes\
    /// data: object, property
    GetLongProp,
    /// Set property larger than two bytes\
    /// data: object, property
    PutLongProp,
    /// Use of a non-existent property\
    /// data: object, property
    NoProp,
    /// Invalid output stream\
    /// data: stream number
    BadInputStream,
    /// Invalid input stream\
    /// data: stream number
    BadOutputStream,
    /// Output stream 3 disabled while not enabled
    NoOutputS3,
    /// output stream 3 enabled for the 17th
    OutputS3Overflow,
    /// Invalid color\
    /// data: color
    BadColor,
    /// Save during interupt\
    SaveInterrupt,
}

pub const CAUSE_COUNT: usize = Cause::SaveInterrupt as usize + 1;

#[derive(Clone, Copy, Debug)]
/// Why the restore failed
pub enum SaveError {
    /// The save file is misformed
    BadSave,
    /// The compressed data ends with an incomplete run
    CMemIncomplete,
    /// The compressed data is longer that the static memory
    CMemLonger(usize, usize),
    /// The uncompressed data isn't the same size as the dynamic memory
    UMemBadSize(usize, usize),
    /// The `CMem` or `UMem` chunk is missing
    MissingMem,
    /// There are two `CMem` or `UMem` chunks
    TwoMem,
    /// The `Stks` chunk is missing
    MissingStks,
    /// There are two `Stks` chunks
    TwoStks,
    /// There are two `IFhd` chunks
    TwoIFhd,
    /// This wasn't saved by this game
    GamesDiffer,
}
