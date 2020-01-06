use crate::{err::*, *};

pub fn init<I: Interface>(data: Vec<u8>, config: Config, interface: &mut I) -> Result<(), Error> {
    // or Result<!, Error>
    let mut mem = fatal(mem::new(data))?;
    let header = fatal(header::init(&mut mem, &config))?;
    let mut rand = alu::init();
    let text = fatal(text::init(&mem, &header))?;
    let mut state = state::init(&mem);
    let mut out = out::init();
    let mut screen = screen::init(&mem, &config);
    let mut input = input::init(&mem);
    let obj = obj::init(&mem);
    let dict = dict::init(&mem);
    let restart = state.save(&mem).unwrap();
    let mut undo = [state.save(&mem).unwrap()];
    let mut err_said = [false; CAUSE_COUNT];
    run(
        &mut mem,
        &mut rand,
        &text,
        &mut state,
        &mut out,
        &mut screen,
        &mut input,
        &header,
        &obj,
        &dict,
        interface,
        &restart,
        &mut undo,
        &config,
        &mut err_said,
    )
}

pub fn run<I: Interface>(
    mem: &mut mem::Mem,
    rand: &mut alu::Random,
    text: &text::Text,
    state: &mut state::State,
    out: &mut out::Output,
    screen: &mut screen::Screen,
    input: &mut input::Input,
    header: &header::Header,
    obj: &obj::Object,
    dict: &dict::Dict,
    interface: &mut I,
    restart: &state::Saved,
    undo: &mut [state::Saved],
    config: &Config,
    err_said: &mut [bool],
) -> Result<(), Error> {
    // or Result<!, Error>
    loop {
        let addr = state.pc;
        let instr = trace(fatal(instr::decode(mem, state, addr)), Trace::Decode(addr))?;
        let result = opcode::exec(
            &instr, mem, rand, text, state, out, screen, input, header, obj, dict, interface,
            restart, undo, config, err_said,
        );

        if let Err(err) = trace(result, Trace::Exec(addr)) {
            if err.fatal {
                return Err(err);
            } else {
                match config.error {
                    ErrorLevel::Never => {}
                    ErrorLevel::Once => {
                        let id = err.cause as usize;
                        if !err_said[id] {
                            interface.error(err);
                            err_said[id] = true;
                        }
                    }
                    ErrorLevel::Always => interface.error(err),
                    ErrorLevel::Quit => return Err(err),
                }
            }
        }
    }
}
