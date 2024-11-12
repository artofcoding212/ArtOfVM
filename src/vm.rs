use std::{char, fs::{read_to_string, self}, mem};

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
    HSTORE(Address),                     // store immediate from stack to heap at address
    HSTORER(Register),                   // store immediate from stack to heap at address from register
    HLOAD(Address),                      // load immediate from heap and push to stack
    HLOADR(Register),                    // load immediate from heap at address from register and push to stack
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
            virt_mem: vec![Immediate::None(); heap_max],
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
            }),
            24 => Instruction::HSTORER({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }),
            25 => Instruction::HLOAD({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Address
            }),
            26 => Instruction::HLOADR({
                self.instr_ptr += 1;
                self.instr_mem[self.instr_ptr] as Register
            }),
            _ => Instruction::NOP(),
        }
    }

    fn execute(&mut self, instr: Instruction) {
        match instr {
            Instruction::NOP() => {},
            Instruction::HLT() => self.is_exe = false,
            Instruction::INT(i) => match i {
                /*
                    WRITE interrupt
                    params:
                        start ptr to buf (u8/u16/u32/u64)
                    desc:
                        writes buf to stdout

                example printing 'A':

                push 65 ('A' in unicode)
                str 0
                push 0
                int 0
                */
                0 => {
                    let mut buf = String::new();
                    let mut addr: usize = match self.stack.pop() {
                        Some(i) => match i {
                            Immediate::U8(i) => i as usize,
                            Immediate::U16(i) => i as usize,
                            Immediate::U32(i) => i as usize,
                            Immediate::U64(i) => i as usize,
                            _ => panic!("valid addresses to WRITE interrupt are u8, u16, u32, & u64"),
                        },
                        _ => panic!("expected ptr to heap provided to WRITE interrupt"),
                    };

                    while self.virt_mem[addr] != Immediate::U32(0) {
                        buf.push(match char::from_u32(
                            match self.virt_mem[addr] {
                                Immediate::U32(i) => i,
                                _ => panic!("expected U32 as unicode char provided within message string for WRITE interrupt"),
                            }
                        ) {
                            Some(c) => c,
                            _ => panic!("invalid char provided within message string for WRITE interrupt"),
                        });
                        addr += 1;
                    }

                    print!("{buf}");
                },
                /* 
                    HEAP_ALLOC interrupt
                    params:
                        requested alloc size (in immediates, as u8/u16/u32/u64)
                    desc:
                        zeroes out first available heap region & returns ptr to it,
                        if no available heap regions were found, expands the heap with 0s and returns ptr to it
                
                example allocating string 'A':

                push 0 ('\0' or null terminator in unicode)
                push 65 ('A' in unicode)

                push 2
                int 1

                pop R1
                ldi R3 1
                add R1 R3
                pop R2

                strR R1
                strR R2
                */
                1 => {
                    let to_alloc: usize = match self.stack.pop() {
                        Some(i) => match i {
                            Immediate::U8(i) => i as usize,
                            Immediate::U16(i) => i as usize,
                            Immediate::U32(i) => i as usize,
                            Immediate::U64(i) => i as usize,
                            _ => panic!("valid alloc size types to HEAP_ALLOC interrupt are u8, u16, u32, & u64"),
                        },
                        _ => panic!("expected alloc size provided to HEAP_ALLOC interrupt"),
                    };

                    let mut curr_free = 0;
                    let mut ptr: Option<usize> = None;

                    for (i, cell) in self.virt_mem.iter().enumerate() {
                        if let Immediate::None() = cell {
                            curr_free += 1;
                            if curr_free >= to_alloc {
                                ptr = Some(((i as isize - to_alloc as isize) + 1) as usize);
                                break;
                            }
                        } else {
                            curr_free = 0;
                        }
                    }

                    if let None = ptr {
                        let addr = self.virt_mem.len().clone()-1;

                        for _ in 0..to_alloc {
                            self.virt_mem.push(Immediate::U8(0));
                        }

                        self.stack.push(Immediate::U64(addr as u64));
                        return;
                    }

                    let addr = ptr.unwrap();

                    for i in addr..addr+to_alloc {
                        self.virt_mem[i] = Immediate::U8(0);
                    }

                    self.stack.push(Immediate::U64(addr.clone() as u64));
                },
                /* 
                    READ_FILE interrupt
                    params:
                        start ptr to file path (u8/u16/u32/u64)
                    desc:
                        pushes ptr to buffer in heap then a 1 if successful (1 would be at the top of the stack)
                        pushes two 0s if unsucessful (err happened)
                
                example reading 'A.txt':

                push 0 ('\0' or null terminator in unicode)
                push 116 ('t' in unicode)
                push 120 ('x' in unicode)
                push 116 ('t' in unicode)
                push 46 ('.' in unicode)
                push 65 ('A' in unicode)

                str 0
                str 1
                str 2
                str 3
                str 4
                str 5

                push 0
                int 2
                */
                2 => {
                    let mut path = String::new();
                    let mut addr: usize = match self.stack.pop() {
                        Some(i) => match i {
                            Immediate::U8(i) => i as usize,
                            Immediate::U16(i) => i as usize,
                            Immediate::U32(i) => i as usize,
                            Immediate::U64(i) => i as usize,
                            _ => panic!("valid addresses to READ_FILE interrupt are u8, u16, u32, & u64"),
                        },
                        _ => panic!("expected ptr to heap provided to READ_FILE interrupt"),
                    };

                    while self.virt_mem[addr] != Immediate::U32(0) {
                        path.push(match char::from_u32(
                            match self.virt_mem[addr] {
                                Immediate::U32(i) => i,
                                _ => panic!("expected U32 as unicode char provided within file path string for READ_FILE interrupt"),
                            }
                        ) {
                            Some(c) => c,
                            _ => panic!("invalid char provided within file path string for READ_FILE interrupt"),
                        });
                        addr += 1;
                    }

                    match read_to_string(path) {
                        Ok(s) => {
                            let len = s.chars().count();
                            self.stack.push(Immediate::U64(len as u64));

                            self.execute(Instruction::INT(1));
                            let buf_start = match self.stack.pop().unwrap() { Immediate::U64(addr) => addr as usize, _ => unreachable!() };

                            for (i, ch) in s.chars().into_iter().enumerate() {
                                self.virt_mem[buf_start+i] = Immediate::U32(ch as u32);
                            }

                            self.virt_mem[buf_start+len] = Immediate::U32(0);
                            self.stack.extend_from_slice(&[Immediate::U64(buf_start as u64), Immediate::U8(1)]);
                        },
                        _ => self.stack.extend_from_slice(&[Immediate::U64(0), Immediate::U8(0)]),
                    }
                },
                /* 
                    WRITE_FILE interrupt
                    params:
                        start ptr to buf (u8/u16/u32/u64) (first arg)
                        start ptr to file path (u8/u16/u32/u64)
                    desc:
                        attempts to write to file or create file if nonexistant with buf, pushes 0 if err, 1 if success
                
                example writing 'A' to 'A.txt':

                push 0 ('\0' or null terminator in unicode)
                push 116 ('t' in unicode)
                push 120 ('x' in unicode)
                push 116 ('t' in unicode)
                push 46 ('.' in unicode)
                push 65 ('A' in unicode)
                str 0
                str 1
                str 2
                str 3
                str 4
                str 5

                push 0 ('\0' or null terminator in unicode)
                push 65 ('A' in unicode)
                str 6
                str 7

                push 0
                push 6
                int 3
                */
                3 => {
                    let mut buf = String::new();
                    let mut buf_addr: usize = match self.stack.pop() {
                        Some(i) => match i {
                            Immediate::U8(i) => i as usize,
                            Immediate::U16(i) => i as usize,
                            Immediate::U32(i) => i as usize,
                            Immediate::U64(i) => i as usize,
                            _ => panic!("valid addresses to WRITE_FILE interrupt are u8, u16, u32, & u64"),
                        },
                        _ => panic!("expected ptr to heap provided to WRITE_FILE interrupt"),
                    };

                    while self.virt_mem[buf_addr] != Immediate::U32(0) {
                        buf.push(match char::from_u32(
                            match self.virt_mem[buf_addr] {
                                Immediate::U32(i) => i,
                                _ => panic!("expected U32 as unicode char provided within buffer string for WRITE_FILE interrupt"),
                            }
                        ) {
                            Some(c) => c,
                            _ => panic!("invalid char provided within buffer string for WRITE_FILE interrupt"),
                        });
                        buf_addr += 1;
                    }

                    let mut path = String::new();
                    let mut path_addr: usize = match self.stack.pop() {
                        Some(i) => match i {
                            Immediate::U8(i) => i as usize,
                            Immediate::U16(i) => i as usize,
                            Immediate::U32(i) => i as usize,
                            Immediate::U64(i) => i as usize,
                            _ => panic!("valid addresses to WRITE_FILE interrupt are u8, u16, u32, & u64"),
                        },
                        _ => panic!("expected ptr to heap provided to WRITE_FILE interrupt"),
                    };

                    while self.virt_mem[path_addr] != Immediate::U32(0) {
                        path.push(match char::from_u32(
                            match self.virt_mem[path_addr] {
                                Immediate::U32(i) => i,
                                _ => panic!("expected U32 as unicode char provided within path string for WRITE_FILE interrupt"),
                            }
                        ) {
                            Some(c) => c,
                            _ => panic!("invalid char provided within path string for WRITE_FILE interrupt"),
                        });
                        path_addr += 1;
                    }

                    match fs::write(path, buf) {
                        Ok(_) => self.stack.push(Immediate::U8(1)),
                        _ => self.stack.push(Immediate::U8(0)),
                    }
                },
                /* 
                    PANIC interrupt
                    params:
                        start ptr to panic message (u64)
                    desc:
                        prints out panic message to stderr then exits with error code 1
                
                    example panicking with 'A':

                    push 0 ('\0' or null terminator in unicode)
                    push 65 ('A' in unicode)
                    
                    str 0
                    str 1

                    push 0
                    int 4
                */
                4 => {
                    let mut buf = String::new();
                    let mut addr: usize = match self.stack.pop() {
                        Some(i) => match i {
                            Immediate::U8(i) => i as usize,
                            Immediate::U16(i) => i as usize,
                            Immediate::U32(i) => i as usize,
                            Immediate::U64(i) => i as usize,
                            _ => panic!("valid addresses to PANIC interrupt are u8, u16, u32, & u64"),
                        },
                        _ => panic!("expected ptr to heap provided to PANIC interrupt"),
                    };

                    while self.virt_mem[addr] != Immediate::U32(0) {
                        buf.push(match char::from_u32(
                            match self.virt_mem[addr] {
                                Immediate::U32(i) => i,
                                _ => panic!("expected U32 as unicode char provided within message string for PANIC interrupt"),
                            }
                        ) {
                            Some(c) => c,
                            _ => panic!("invalid char provided within message string for PANIC interrupt"),
                        });
                        addr += 1;
                    }

                    eprintln!("panicked with err message:\n{buf}");
                    std::process::exit(1);
                },
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
            Instruction::HSTORE(addr) => self.virt_mem[addr] = match self.stack.pop() {
                Some(i) => i,
                _ => panic!("expected value on stack for HSTORE instruction"),
            },
            Instruction::HSTORER(reg) => self.virt_mem[match self.reg[reg] {
                Immediate::U8(i) => i as usize,
                Immediate::U16(i) => i as usize,
                Immediate::U32(i) => i as usize,
                Immediate::U64(i) => i as usize,
                _ => panic!("valid addresses to HSTORER are u8, u16, u32, & u64"),
            }] = match self.stack.pop() {
                Some(i) => i,
                _ => panic!("expected value on stack for HSTORER instruction"),
            },
            Instruction::HLOAD(addr) => self.stack.push(self.virt_mem[addr]),
            Instruction::HLOADR(reg) => self.stack.push(self.virt_mem[match self.reg[reg] {
                Immediate::U8(i) => i as usize,
                Immediate::U16(i) => i as usize,
                Immediate::U32(i) => i as usize,
                Immediate::U64(i) => i as usize,
                _ => panic!("valid addresses to HLOADR are u8, u16, u32, & u64"),
            }]),
        }
    }
}