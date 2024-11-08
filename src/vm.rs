use std::mem;

pub type Register = usize;
pub type Address = usize;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Immediate {
    None(),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
}

// (see art_of_vm::assembler::Opcode for an opcode reference)
#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    NOP(),                               // do nothing
    HLT(),                               // end program execution
    INT(Address),                        // call virtual interrupt
    PUSH(Immediate),                     // push immediate to stack
    PUSHR(Register),                     // push reg contents to stack
    POP(Register),                       // pops immediate from stack 
    LDI(Register, Immediate),            // load immediate to register
    CPY(Register, Register),             // cpy contents of reg A into reg B
    JMP(Address),                        // jmp to location
    JE(Address),                         // jmp (if eq) to location
    JNE(Address),                        // jmp (if not eq) to location
    JG(Address),                         // jmp (if greater than) to location
    JL(Address),                         // jmp (if less than) to location
    CMP(Register, Register),             // compare two reg
    DIV(Register, Register),             // div two regs and, pushse result to stack
    ADD(Register, Register),             // add two regs and, pushse result to stack
    SUB(Register, Register),             // sub two regs and, pushse result to stack
    MUL(Register, Register),             // mul two regs and, pushse result to stack
    AND(Register, Register),             // bitwise AND on 2 regs, pushes result to stack
    OR(Register, Register),              // bitwise OR on 2 regs, pushes result to stack
    XOR(Register, Register),             // bitwise XOR on 2 regs, pushes result to stack
    SHR(Register, Immediate),            // shifts reg to the right by immediate, pushes result to stack
    SHL(Register, Immediate),            // shifts reg to the left by immediate, pushes result to stack
    HSTORE(Address, Immediate),          // store immediate to heap at specific address from stack
    HLOAD(Address),                      // load immediate from heap and push to stack
    HSTORER(Address, Register),          // store immediate to heap from register contents
    HLOADR(Register, Address),           // load immediate from heap to register
}

pub struct VirtualMachine {
    instr_ptr: Address,
    instr_mem: Vec<u8>,
    virt_mem: Vec<Immediate>,
    stack: Vec<Immediate>,
    reg: [Immediate; 16],

    flag_eq: bool,
    flag_gt: bool,
    is_exe: bool,
}

impl VirtualMachine {
    pub fn new(instr_mem: Vec<u8>, heap_max: usize) -> Self {
        Self {
            instr_ptr: 0,
            instr_mem,
            virt_mem: vec![Immediate::U8(0); heap_max],
            stack: vec![],
            reg: [Immediate::U8(0); 16],

            flag_eq: false,
            flag_gt: false,
            is_exe: false,
        }
    }

    pub fn exec(&mut self) {
        if self.is_exe {
            return;
        }

        self.is_exe = true;

        while self.instr_ptr < self.instr_mem.len() && self.is_exe {
            let decoded = self.decode();
            self.execute(decoded);
            self.instr_ptr += 1;
        }
    }

    fn decode_immed(&mut self) -> Immediate {
        self.instr_ptr += 2;

        match self.instr_mem[self.instr_ptr-1] {
            0 => Immediate::U8(self.instr_mem[self.instr_ptr] as u8),
            1 => Immediate::I8(self.instr_mem[self.instr_ptr] as i8),
            2 => {
                let size = mem::size_of::<u16>();
                let val = u16::from_le_bytes(
                    self.instr_mem[self.instr_ptr..][..size].try_into().unwrap()
                );

                self.instr_ptr += size-1;
                Immediate::U16(val)
            },
            3 => {
                let size = mem::size_of::<i16>();
                let val = i16::from_le_bytes(
                    self.instr_mem[self.instr_ptr..][..size].try_into().unwrap()
                );

                self.instr_ptr += size-1;
                Immediate::I16(val)
            },
            4 => {
                let size = mem::size_of::<u32>();
                let val = u32::from_le_bytes(
                    self.instr_mem[self.instr_ptr..][..size].try_into().unwrap()
                );

                self.instr_ptr += size-1;
                Immediate::U32(val)
            },
            5 => {
                let size = mem::size_of::<i32>();
                let val = i32::from_le_bytes(
                    self.instr_mem[self.instr_ptr..][..size].try_into().unwrap()
                );

                self.instr_ptr += size-1;
                Immediate::I32(val)
            },
            6 => {
                let size = mem::size_of::<u64>();
                let val = u64::from_le_bytes(
                    self.instr_mem[self.instr_ptr..][..size].try_into().unwrap()
                );

                self.instr_ptr += size-1;
                Immediate::U64(val)
            },
            7 => {
                let size = mem::size_of::<i64>();
                let val = i64::from_le_bytes(
                    self.instr_mem[self.instr_ptr..][..size].try_into().unwrap()
                );

                self.instr_ptr += size-1;
                Immediate::I64(val)
            },
            8 => {
                let size = mem::size_of::<f32>();
                let val = f32::from_le_bytes(
                    self.instr_mem[self.instr_ptr..][..size].try_into().unwrap()
                );

                self.instr_ptr += size-1;
                Immediate::F32(val)
            },
            9 => {
                let size = mem::size_of::<f64>();
                let val = f64::from_le_bytes(
                    self.instr_mem[self.instr_ptr..][..size].try_into().unwrap()
                );

                self.instr_ptr += size-1;
                Immediate::F64(val)
            },
            _ => Immediate::None(),
        }
    }

    fn decode(&mut self) -> Instruction {
        match self.instr_mem[self.instr_ptr] {
            0 => {
                self.instr_ptr += 1;
                Instruction::NOP()
            },
            1 => {
                self.instr_ptr += 1;
                Instruction::HLT()
            },
            2 => Instruction::INT({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Address
            }),
            3 => Instruction::PUSH(self.decode_immed()),
            4 => Instruction::PUSHR({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }),
            5 => Instruction::POP({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }),
            6 => Instruction::LDI({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }, self.decode_immed()),
            7 => Instruction::CPY({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Register),
            8 => Instruction::JMP({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }),
            9 => Instruction::JE({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }),
            10 => Instruction::JNE({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }),
            11 => Instruction::JG({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }),
            12 => Instruction::JL({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }),
            13 => Instruction::CMP({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Register),
            14 => Instruction::ADD({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Register),
            15 => Instruction::SUB({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Register),
            16 => Instruction::MUL({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Register),
            17 => Instruction::DIV({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Register),
            18 => Instruction::AND({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Register),
            19 => Instruction::OR({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Register),
            20 => Instruction::XOR({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Register),
            21 => Instruction::SHR({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }, self.decode_immed()),
            22 => Instruction::SHL({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }, self.decode_immed()),
            23 => Instruction::HSTORE({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Address
            }, self.decode_immed()),
            24 => Instruction::HLOAD({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Address
            }),
            25 => Instruction::HSTORER({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Address
            }, self.instr_mem[self.instr_ptr] as Register),
            26 => Instruction::HLOADR({
                self.instr_ptr += 2;
                self.instr_mem[self.instr_ptr-1] as Register
            }, self.instr_mem[self.instr_ptr] as Address),
            _ => Instruction::NOP(),
        }
    }

    fn execute(&mut self, instr: Instruction) {
        match instr {
            Instruction::NOP() => {},
            Instruction::HLT() => self.is_exe = false,
            Instruction::INT(i) => match i {
                /* WRITE interrupt, takes unicode char
                example:
                push 65 ('A' in unicode)
                int 0
                */
                0 => print!("{}", 
                    match char::from_u32(
                        match self.stack.pop() {
                            Some(v) => match v {
                                Immediate::U32(i) => {
                                    i
                                },
                                _ => panic!("unciode char argument to WRITE interrupt must be u32"),      
                            },
                            _ => panic!("expected unicode char on stack when calling WRITE interrupt"),
                        }
                    ) {
                        Some(c) => c,
                        _ => panic!("invalid unicode char provided to WRITE interrupt"),
                    }
                ),
                _ => panic!("unknown interrupt '{i}'"),
            },
            Instruction::PUSH(immed) => {
                self.stack.push(immed)
            },
            Instruction::PUSHR(reg) => self.stack.push(self.reg[reg]),
            Instruction::POP(reg) => match self.stack.pop() {
                Some(immed) => self.reg[reg] = immed,
                _ => panic!("attempted to pop off value from stack when no values are on the stack"),
            },
            Instruction::LDI(reg, immed) => self.reg[reg] = immed,
            Instruction::CPY(reg_a, reg_b) => self.reg[reg_b] = self.reg[reg_a].clone(),
            Instruction::JMP(addr) => self.instr_ptr = addr-1,
            Instruction::JE(addr) => match self.flag_eq {
                true => self.execute(Instruction::JMP(addr)),
                _ => {},
            },
            Instruction::JNE(addr) => match self.flag_eq {
                false => self.execute(Instruction::JMP(addr)),
                _ => {},
            },
            Instruction::JG(addr) => match self.flag_gt {
                true => self.execute(Instruction::JMP(addr)),
                _ => {},
            },
            Instruction::JL(addr) => match self.flag_gt {
                false => self.execute(Instruction::JMP(addr)),
                _ => {},
            },
            Instruction::CMP(reg_a, reg_b) => {
                let r1 = self.reg[reg_a];
                let r2 = self.reg[reg_b];
                self.flag_eq = r1 == r2;
                self.flag_gt = r1 > r2;
            },
            Instruction::ADD(reg_a, reg_b) => match (self.reg[reg_a], self.reg[reg_b]) {
                (Immediate::I8(a), Immediate::I8(b)) => self.stack.push(Immediate::I8(a+b)),
                (Immediate::I16(a), Immediate::I16(b)) => self.stack.push(Immediate::I16(a+b)),
                (Immediate::I32(a), Immediate::I32(b)) => self.stack.push(Immediate::I32(a+b)),
                (Immediate::I64(a), Immediate::I64(b)) => self.stack.push(Immediate::I64(a+b)),
                (Immediate::U8(a), Immediate::U8(b)) => self.stack.push(Immediate::U8(a+b)),
                (Immediate::U16(a), Immediate::U16(b)) => self.stack.push(Immediate::U16(a+b)),
                (Immediate::U32(a), Immediate::U32(b)) => self.stack.push(Immediate::U32(a+b)),
                (Immediate::U64(a), Immediate::U64(b)) => self.stack.push(Immediate::U64(a+b)),
                (Immediate::F32(a), Immediate::F32(b)) => self.stack.push(Immediate::F32(a+b)),
                (Immediate::F64(a), Immediate::F64(b)) => self.stack.push(Immediate::F64(a+b)),
                _ => panic!("can only add two registers if they store the same type of value"),
            },
            Instruction::SUB(reg_a, reg_b) => match (self.reg[reg_a], self.reg[reg_b]) {
                (Immediate::I8(a), Immediate::I8(b)) => self.stack.push(Immediate::I8(a-b)),
                (Immediate::I16(a), Immediate::I16(b)) => self.stack.push(Immediate::I16(a-b)),
                (Immediate::I32(a), Immediate::I32(b)) => self.stack.push(Immediate::I32(a-b)),
                (Immediate::I64(a), Immediate::I64(b)) => self.stack.push(Immediate::I64(a-b)),
                (Immediate::U8(a), Immediate::U8(b)) => self.stack.push(Immediate::U8(a-b)),
                (Immediate::U16(a), Immediate::U16(b)) => self.stack.push(Immediate::U16(a-b)),
                (Immediate::U32(a), Immediate::U32(b)) => self.stack.push(Immediate::U32(a-b)),
                (Immediate::U64(a), Immediate::U64(b)) => self.stack.push(Immediate::U64(a-b)),
                (Immediate::F32(a), Immediate::F32(b)) => self.stack.push(Immediate::F32(a-b)),
                (Immediate::F64(a), Immediate::F64(b)) => self.stack.push(Immediate::F64(a-b)),
                _ => panic!("can only sub two registers if they store the same type of value"),
            },
            Instruction::MUL(reg_a, reg_b) => match (self.reg[reg_a], self.reg[reg_b]) {
                (Immediate::I8(a), Immediate::I8(b)) => self.stack.push(Immediate::I8(a*b)),
                (Immediate::I16(a), Immediate::I16(b)) => self.stack.push(Immediate::I16(a*b)),
                (Immediate::I32(a), Immediate::I32(b)) => self.stack.push(Immediate::I32(a*b)),
                (Immediate::I64(a), Immediate::I64(b)) => self.stack.push(Immediate::I64(a*b)),
                (Immediate::U8(a), Immediate::U8(b)) => self.stack.push(Immediate::U8(a-b)),
                (Immediate::U16(a), Immediate::U16(b)) => self.stack.push(Immediate::U16(a*b)),
                (Immediate::U32(a), Immediate::U32(b)) => self.stack.push(Immediate::U32(a*b)),
                (Immediate::U64(a), Immediate::U64(b)) => self.stack.push(Immediate::U64(a*b)),
                (Immediate::F32(a), Immediate::F32(b)) => self.stack.push(Immediate::F32(a*b)),
                (Immediate::F64(a), Immediate::F64(b)) => self.stack.push(Immediate::F64(a*b)),
                _ => panic!("can only mul two registers if they store the same type of value"),
            },
            Instruction::DIV(reg_a, reg_b) => match (self.reg[reg_a], self.reg[reg_b]) {
                (Immediate::I8(a), Immediate::I8(b)) => self.stack.push(Immediate::I8(a/b)),
                (Immediate::I16(a), Immediate::I16(b)) => self.stack.push(Immediate::I16(a/b)),
                (Immediate::I32(a), Immediate::I32(b)) => self.stack.push(Immediate::I32(a/b)),
                (Immediate::I64(a), Immediate::I64(b)) => self.stack.push(Immediate::I64(a/b)),
                (Immediate::U8(a), Immediate::U8(b)) => self.stack.push(Immediate::U8(a/b)),
                (Immediate::U16(a), Immediate::U16(b)) => self.stack.push(Immediate::U16(a/b)),
                (Immediate::U32(a), Immediate::U32(b)) => self.stack.push(Immediate::U32(a/b)),
                (Immediate::U64(a), Immediate::U64(b)) => self.stack.push(Immediate::U64(a/b)),
                (Immediate::F32(a), Immediate::F32(b)) => self.stack.push(Immediate::F32(a/b)),
                (Immediate::F64(a), Immediate::F64(b)) => self.stack.push(Immediate::F64(a/b)),
                _ => panic!("can only div two registers if they store the same type of value"),
            },
            Instruction::AND(reg_a, reg_b) => match (self.reg[reg_a], self.reg[reg_b]) {
                (Immediate::I8(a), Immediate::I8(b)) => self.stack.push(Immediate::I8(a&b)),
                (Immediate::I16(a), Immediate::I16(b)) => self.stack.push(Immediate::I16(a&b)),
                (Immediate::I32(a), Immediate::I32(b)) => self.stack.push(Immediate::I32(a&b)),
                (Immediate::I64(a), Immediate::I64(b)) => self.stack.push(Immediate::I64(a&b)),
                (Immediate::U8(a), Immediate::U8(b)) => self.stack.push(Immediate::U8(a&b)),
                (Immediate::U16(a), Immediate::U16(b)) => self.stack.push(Immediate::U16(a&b)),
                (Immediate::U32(a), Immediate::U32(b)) => self.stack.push(Immediate::U32(a&b)),
                (Immediate::U64(a), Immediate::U64(b)) => self.stack.push(Immediate::U64(a&b)),
                _ => panic!("can only bitwise and two registers if they store the same type of value"),
            },
            Instruction::OR(reg_a, reg_b) => match (self.reg[reg_a], self.reg[reg_b]) {
                (Immediate::I8(a), Immediate::I8(b)) => self.stack.push(Immediate::I8(a|b)),
                (Immediate::I16(a), Immediate::I16(b)) => self.stack.push(Immediate::I16(a|b)),
                (Immediate::I32(a), Immediate::I32(b)) => self.stack.push(Immediate::I32(a|b)),
                (Immediate::I64(a), Immediate::I64(b)) => self.stack.push(Immediate::I64(a|b)),
                (Immediate::U8(a), Immediate::U8(b)) => self.stack.push(Immediate::U8(a|b)),
                (Immediate::U16(a), Immediate::U16(b)) => self.stack.push(Immediate::U16(a|b)),
                (Immediate::U32(a), Immediate::U32(b)) => self.stack.push(Immediate::U32(a|b)),
                (Immediate::U64(a), Immediate::U64(b)) => self.stack.push(Immediate::U64(a|b)),
    
                _ => panic!("can only bitwise or two registers if they store the same type of value"),
            },
            Instruction::XOR(reg_a, reg_b) => match (self.reg[reg_a], self.reg[reg_b]) {
                (Immediate::I8(a), Immediate::I8(b)) => self.stack.push(Immediate::I8(a^b)),
                (Immediate::I16(a), Immediate::I16(b)) => self.stack.push(Immediate::I16(a^b)),
                (Immediate::I32(a), Immediate::I32(b)) => self.stack.push(Immediate::I32(a^b)),
                (Immediate::I64(a), Immediate::I64(b)) => self.stack.push(Immediate::I64(a^b)),
                (Immediate::U8(a), Immediate::U8(b)) => self.stack.push(Immediate::U8(a^b)),
                (Immediate::U16(a), Immediate::U16(b)) => self.stack.push(Immediate::U16(a^b)),
                (Immediate::U32(a), Immediate::U32(b)) => self.stack.push(Immediate::U32(a^b)),
                (Immediate::U64(a), Immediate::U64(b)) => self.stack.push(Immediate::U64(a^b)),
                _ => panic!("can only bitwise and two registers if they store the same type of value"),
            },
            Instruction::SHR(reg, immed) => match (self.reg[reg], immed) {
                (Immediate::I8(a), Immediate::I8(b)) => self.stack.push(Immediate::I8(a>>b)),
                (Immediate::I16(a), Immediate::I16(b)) => self.stack.push(Immediate::I16(a>>b)),
                (Immediate::I32(a), Immediate::I32(b)) => self.stack.push(Immediate::I32(a>>b)),
                (Immediate::I64(a), Immediate::I64(b)) => self.stack.push(Immediate::I64(a>>b)),
                (Immediate::U8(a), Immediate::U8(b)) => self.stack.push(Immediate::U8(a>>b)),
                (Immediate::U16(a), Immediate::U16(b)) => self.stack.push(Immediate::U16(a>>b)),
                (Immediate::U32(a), Immediate::U32(b)) => self.stack.push(Immediate::U32(a>>b)),
                (Immediate::U64(a), Immediate::U64(b)) => self.stack.push(Immediate::U64(a>>b)),
                _ => panic!("can only right shift if reg and immed are of the same type of value"),
            },
            Instruction::SHL(reg, immed) => match (self.reg[reg], immed) {
                (Immediate::I8(a), Immediate::I8(b)) => self.stack.push(Immediate::I8(a<<b)),
                (Immediate::I16(a), Immediate::I16(b)) => self.stack.push(Immediate::I16(a<<b)),
                (Immediate::I32(a), Immediate::I32(b)) => self.stack.push(Immediate::I32(a<<b)),
                (Immediate::I64(a), Immediate::I64(b)) => self.stack.push(Immediate::I64(a<<b)),
                (Immediate::U8(a), Immediate::U8(b)) => self.stack.push(Immediate::U8(a<<b)),
                (Immediate::U16(a), Immediate::U16(b)) => self.stack.push(Immediate::U16(a<<b)),
                (Immediate::U32(a), Immediate::U32(b)) => self.stack.push(Immediate::U32(a<<b)),
                (Immediate::U64(a), Immediate::U64(b)) => self.stack.push(Immediate::U64(a<<b)),
                _ => panic!("can only left shift if reg and immed are of the same type of value"),
            },
            Instruction::HSTORE(addr, immed) => self.virt_mem[addr] = immed,
            Instruction::HLOAD(addr) => self.stack.push(self.virt_mem[addr]),
            Instruction::HSTORER(addr, reg) => self.virt_mem[addr] = self.reg[reg],
            Instruction::HLOADR(addr, reg) => self.reg[reg] = self.virt_mem[addr],
        }
    }
}