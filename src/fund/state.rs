use crate::{err::*, machine, mem::Mem, rout, *};

pub struct State {
    stack: Vec<Vec<u16>>,
    pub pc: usize,
    call: Vec<Frame>,
    vars: u16,
    interrupt: Option<usize>,
}

pub struct Saved {
    pub mem: Vec<u8>,
    pub stack: Vec<Vec<u16>>,
    pub pc: usize,
    pub call: Vec<Frame>,
}

#[derive(Clone)]
pub struct Frame {
    pub ret: usize,
    pub local: Vec<u16>,
    pub args: u16,
    pub store: bool,
}

impl State {
    pub fn save(&self, mem: &Mem) -> Result<Saved, Error> {
        if self.interrupt.is_some() {
            return error(Cause::SaveInterrupt, (0, 0));
        }
        Ok(Saved {
            mem: mem.save(),
            stack: self.stack.clone(),
            pc: self.pc,
            call: self.call.clone(),
        })
    }

    pub fn restore(&mut self, mem: &mut Mem, saved: &Saved) {
        mem.restore(&saved.mem);
        self.stack = saved.stack.clone();
        self.pc = saved.pc;
        self.call = saved.call.clone();
    }

    pub fn get_var(&mut self, mem: &Mem, var: u16) -> Result<u16, Error> {
        match var {
            0x00 => self
                .stack
                .last_mut()
                .unwrap()
                .pop()
                .ok_or_else(|| err(Cause::StackUnderflow, (0, 0))),
            0x01..=0x0f => Ok(*self
                .call
                .last()
                .ok_or_else(|| err(Cause::NoLocalInMain, (0, 0)))?
                .local
                .get(var as usize - 1)
                .ok_or_else(|| err(Cause::NoLocal, (var - 1, 0)))?),
            0x10..=0xff => mem.loadw(self.vars + 2 * (var - 0x10)),
            _ => error(Cause::NoVar, (var, 0)),
        }
    }

    pub fn set_var(&mut self, mem: &mut Mem, var: u16, data: u16) -> Result<(), Error> {
        match var {
            0x00 => self.stack.last_mut().unwrap().push(data),
            0x01..=0x0f => {
                *self
                    .call
                    .last_mut()
                    .ok_or_else(|| err(Cause::NoLocalInMain, (0, 0)))?
                    .local
                    .get_mut(var as usize - 1)
                    .ok_or_else(|| err(Cause::NoLocal, (var - 1, 0)))? = data
            }
            0x10..=0xff => mem.storew(self.vars + 2 * (var - 0x10), data)?,
            _ => return error(Cause::NoVar, (var, 0)),
        }
        Ok(())
    }

    pub fn get_inplace(&self) -> Result<u16, Error> {
        Ok(*self
            .stack
            .last()
            .unwrap()
            .last()
            .ok_or_else(|| err(Cause::StackUnderflow, (0, 0)))?)
    }

    pub fn set_inplace(&mut self, data: u16) -> Result<(), Error> {
        *self
            .stack
            .last_mut()
            .unwrap()
            .last_mut()
            .ok_or_else(|| err(Cause::StackUnderflow, (0, 0)))? = data;
        Ok(())
    }

    pub fn call(
        &mut self,
        mem: &mut Mem,
        addr: usize,
        args: Vec<u16>,
        store: bool,
    ) -> Result<(), Error> {
        if addr == 0 {
            if store {
                let var = mem[self.pc - 1];
                self.set_var(mem, var.into(), 0)?;
            }
            return Ok(());
        }
        let (mut local, pc) = trace(fatal(rout::info(mem, addr)), Trace::Rout(addr))?;
        let len = local.len().min(args.len());
        local[..len].clone_from_slice(&args[..len]);
        self.stack.push(Vec::new());
        self.call.push(Frame {
            ret: self.pc,
            local,
            args: args.len() as u16,
            store,
        });
        self.pc = pc;
        Ok(())
    }

    pub fn ret(&mut self, mem: &mut Mem, value: u16) -> Result<(), Error> {
        let frame = fatal(
            self.call
                .pop()
                .ok_or_else(|| err(Cause::MainReturned, (0, 0))),
        )?;
        self.stack.pop();
        self.pc = frame.ret;
        if self.interrupt == Some(self.call.len()) {
            return fatal(error(Cause::MainReturned, (value, 0)));
        }
        if frame.store {
            let &var = fatal(
                mem.get(self.pc - 1)
                    .ok_or_else(|| err(Cause::PcOut, (0, 0))),
            )?;
            self.set_var(mem, var.into(), value)?;
        }
        Ok(())
    }

    pub fn catch(&self) -> u16 {
        self.call.len() as u16
    }

    pub fn throw(&mut self, mem: &mut Mem, index: u16, value: u16) -> Result<(), Error> {
        let index = index as usize;
        self.call.truncate(index);
        self.stack.truncate(index);
        self.ret(mem, value)
    }

    pub fn arg_count(&self) -> Result<u16, Error> {
        Ok(self
            .call
            .last()
            .ok_or_else(|| err(Cause::NoLocalInMain, (0, 0)))?
            .args)
    }

    pub fn interrupt<I: Interface>(
        &mut self,
        addr: usize,
        mem: &mut Mem,
        rand: &mut alu::Random,
        text: &text::Text,
        out: &mut out::Output,
        screen: &mut screen::Screen,
        input: &mut input::Input,
        header: &header::Header,
        obj: &obj::Object,
        dict: &dict::Dict,
        interface: &mut I,
        restart: &Saved,
        undo: &mut [Saved],
        config: &Config,
        err_said: &mut [bool],
    ) -> Option<u16> {
        if self.interrupt.is_some() {
            return None;
        }
        let interrupt = self.call.len();
        self.interrupt = Some(interrupt);
        if let Err(err) = self.call(mem, addr, Vec::new(), false) {
            match config.error {
                ErrorLevel::Never => {}
                ErrorLevel::Once => {
                    let id = err.cause as usize;
                    if !err_said[id] {
                        interface.error(err);
                        err_said[id] = true;
                    }
                }
                ErrorLevel::Always | ErrorLevel::Quit => interface.error(err),
            }
        }

        let result = machine::run(
            mem, rand, text, self, out, screen, input, header, obj, dict, interface, restart, undo,
            config, err_said,
        )
        .unwrap_err();
        self.interrupt = None;
        if result.cause == Cause::MainReturned {
            Some(result.data.0)
        } else {
            interface.error(result);
            self.call.truncate(interrupt);
            self.stack.truncate(interrupt);
            None
        }
    }
}

pub fn init(mem: &Mem) -> State {
    State {
        stack: vec![Vec::new()],
        pc: mem.byte(mem.loadw(0x06).unwrap()),
        call: Vec::new(),
        vars: mem.loadw(0x0c).unwrap(),
        interrupt: None,
    }
}

#[cfg(test)]
use crate::mem;

#[test]
fn test_init() {
    let mut data = mem::default();
    data[0x07] = 0x40;
    data[0x0d] = 0x10;
    data.push(0);
    let mem = mem::new(data).unwrap();
    let state = init(&mem);
    assert_eq!(state.stack, vec![Vec::new()]);
    assert_eq!(state.pc, 0x40);
    assert_eq!(state.call.len(), 0);
    assert_eq!(state.vars, 0x10);
}

#[test]
fn test_save_restore() {
    let mut data = mem::default();
    data[0x0f] = 0x44;
    data.extend(vec![0x10, 0x20, 0x30, 0x40, 0x04, 0x03, 0x02, 0x01]);
    let mut mem = mem::new(data.clone()).unwrap();
    let mut state = State {
        stack: vec![vec![1, 2], vec![3, 4]],
        pc: 10,
        call: vec![Frame {
            ret: 20,
            local: vec![5, 6],
            args: 2,
            store: true,
        }],
        vars: 0,
        interrupt: None,
    };

    let mut save = state.save(&mem).unwrap();
    mem.storew(0x40, 0).unwrap();
    mem.storew(0x42, 0).unwrap();
    state.stack = vec![];
    state.pc = 0;
    state.call = vec![];

    assert_eq!(save.mem[..], data[..0x44]);
    assert_eq!(save.stack, vec![vec![1, 2], vec![3, 4]]);
    assert_eq!(save.pc, 10);
    assert_eq!(save.call.len(), 1);
    assert_eq!(save.call[0].ret, 20);
    assert_eq!(save.call[0].local, vec![5, 6]);
    assert_eq!(save.call[0].args, 2);
    assert_eq!(save.call[0].store, true);

    state.restore(&mut mem, &save);
    save.mem = vec![];
    save.stack = vec![];
    save.pc = 0;
    save.call = vec![];

    assert_eq!(mem.loadw(0x40).unwrap(), 0x1020);
    assert_eq!(mem.loadw(0x42).unwrap(), 0x3040);
    assert_eq!(state.stack, vec![vec![1, 2], vec![3, 4]]);
    assert_eq!(state.pc, 10);
    assert_eq!(state.call.len(), 1);
    assert_eq!(state.call[0].ret, 20);
    assert_eq!(state.call[0].local, vec![5, 6]);
    assert_eq!(state.call[0].args, 2);
    assert_eq!(state.call[0].store, true);
}

#[test]
fn test_vars() {
    let mut data = mem::default();
    data[0x0d] = 0x40;
    data[0x0f] = 0x42;
    data.extend(vec![0x01, 0x02, 0x03]);
    let mut mem = mem::new(data).unwrap();
    let mut state = init(&mem);

    state.set_var(&mut mem, 0x00, 0x1020).unwrap();
    state.set_var(&mut mem, 0x00, 0x3040).unwrap();
    assert_eq!(state.get_inplace().unwrap(), 0x3040);
    assert_eq!(state.stack, vec![vec![0x1020, 0x3040]]);
    state.set_inplace(0x5060).unwrap();
    assert_eq!(state.get_var(&mem, 0x00).unwrap(), 0x5060);
    assert_eq!(state.get_var(&mem, 0x00).unwrap(), 0x1020);

    state.call(&mut mem, 0x40, vec![], false).unwrap();
    assert_eq!(state.get_var(&mem, 0x01).unwrap(), 0x0203);
    state.set_var(&mut mem, 0x01, 0x1234).unwrap();
    assert_eq!(state.call[0].local[0], 0x1234);
    assert_eq!(state.get_var(&mem, 0x10).unwrap(), 0x0102);
    state.set_var(&mut mem, 0x10, 0x0304).unwrap();
    assert_eq!(mem.loadw(0x40).unwrap(), 0x0304);

    mem.storeb(0x40, 0x01).unwrap();
    state.pc = 0x41;
    state.call(&mut mem, 0x40, vec![], true).unwrap();
    state.ret(&mut mem, 0xa5).unwrap();
    assert_eq!(state.get_var(&mem, 0x01).unwrap(), 0xa5);
    state.pc = 0x41;
    state.call(&mut mem, 0x40, vec![], false).unwrap();
    state.ret(&mut mem, 0x5a).unwrap();
    assert_eq!(state.get_var(&mem, 0x01).unwrap(), 0xa5);
}

#[test]
fn test_call_ret() {
    let mut data = mem::default();
    data.extend(vec![0; 2]);
    data[0x00] = 5;
    data[0x41] = 2;
    let mut mem = mem::new(data).unwrap();
    let mut state = init(&mem);
    state.set_var(&mut mem, 0, 0x1234).unwrap();

    state.call(&mut mem, 0x40, vec![1, 2, 3], false).unwrap();
    assert_eq!(state.pc, 0x41);
    assert_eq!(state.stack, vec![vec![0x1234], vec![]]);
    assert_eq!(state.call.len(), 1);
    assert_eq!(state.call[0].ret, 0);
    assert_eq!(state.call[0].local, vec![]);
    assert_eq!(state.call[0].args, 3);
    assert_eq!(state.call[0].store, false);

    state.call(&mut mem, 0x41, vec![1, 2, 3], false).unwrap();
    assert_eq!(state.pc, 0x42);
    assert_eq!(state.stack, vec![vec![0x1234], vec![], vec![]]);
    assert_eq!(state.call.len(), 2);
    assert_eq!(state.call[1].ret, 0x41);
    assert_eq!(state.call[1].local, vec![1, 2]);
    assert_eq!(state.call[1].args, 3);
    assert_eq!(state.call[1].store, false);

    state.ret(&mut mem, 0).unwrap();
    assert_eq!(state.pc, 0x41);
    assert_eq!(state.stack, vec![vec![0x1234], vec![]]);
    assert_eq!(state.call.len(), 1);
    assert_eq!(state.call[0].ret, 0);
    assert_eq!(state.call[0].local, vec![]);
    assert_eq!(state.call[0].args, 3);
    assert_eq!(state.call[0].store, false);

    state.set_var(&mut mem, 0, 0x5678).unwrap();
    state.set_var(&mut mem, 0, 0x9abc).unwrap();
    assert_eq!(state.get_var(&mem, 0).unwrap(), 0x9abc);
    state.ret(&mut mem, 0).unwrap();
    assert_eq!(state.get_var(&mem, 0).unwrap(), 0x1234);
    assert_eq!(state.pc, 0);
    assert_eq!(state.stack, vec![vec![]]);
    assert_eq!(state.call.len(), 0);

    state.call(&mut mem, 0x41, vec![4], false).unwrap();
    assert_eq!(state.call[0].local, vec![4, 0]);
    assert_eq!(state.call[0].args, 1);
}

#[test]
fn test_catch_throw() {
    let mut data = mem::default();
    data[0x00] = 5;
    data.push(1);
    let mut mem = mem::new(data).unwrap();
    let mut state = init(&mem);
    assert_eq!(state.catch(), 0);
    state.call(&mut mem, 0x40, vec![], false).unwrap();
    state.pc = 0x41;
    state.call(&mut mem, 0x40, vec![], true).unwrap();
    assert_eq!(state.catch(), 2);
    state.call(&mut mem, 0x40, vec![], false).unwrap();
    state.throw(&mut mem, 2, 0xff).unwrap();
    assert_eq!(state.call.len(), 1);
    assert_eq!(state.get_var(&mem, 0x01).unwrap(), 0xff);
}
