use crate::vm::Immediate;

/* TODO: labels

HashMap<String, usize> under Assembler for labels & their bit position
Vec<(usize, String)> under Assembler for bits to be replaced with labels' starting bit

*/

use std::collections::HashMap;

pub struct Assembler {
    machine_c: Vec<u8>,
    lbls: HashMap<String, usize>,
    lbl_replaces: Vec<(usize, String)>,
    src: Vec<char>,
    bit: usize,
    ch: char,
    i: usize,
}

pub enum Opcode {
    NOP = 0,       // _
    HLT = 1,       // hlt
    INT = 2,       // int
    PUSH = 3,      // $ [immed]
    PUSHR = 4,     // $$ [reg]
    POP = 5,       // % [reg]
    LDI = 6,       // @ [reg] [immed]
    CPY = 7,       // : [reg] [reg]
    JMP = 8,       // // [lbl] (//R [reg] for raw opcode translation)
    JE = 9,        // /= [lbl] (/=R [reg] for raw opcode translation)
    JNE = 10,      // /! [lbl] (/!R [reg] for raw opcode translation)
    JG = 11,       // /> [lbl] (/>R [reg] for raw opcode translation)
    JL = 12,       // /< [lbl] (/=R [reg] for raw opcode translation)
    CMP = 13,      // = [reg] [reg]
    ADD = 14,      // + [reg] [reg]
    SUB = 15,      // - [reg] [reg]
    MUL = 16,      // * [reg] [reg]
    DIV = 17,      // / [reg] [reg]
    AND = 18,      // & [reg] [reg]
    OR = 19,       // | [reg] [reg]
    XOR = 20,      // ^ [reg] [reg]
    SHR = 21,      // > [reg] [immed]
    SHL = 22,      // < [reg] [immed]
    HSTORE = 23,   // str [addr]
    HSTORER = 24,  // strR [reg]
    HLOAD = 25,    // ld [addr]
    HLOADR = 26,   // ldR [reg]
} 

impl Assembler {
    pub fn new(src: String) -> Self {
        Self {
            machine_c: vec![],
            lbls: HashMap::new(),
            lbl_replaces: vec![],
            src: src.chars().collect(),
            ch: src.chars().nth(0).unwrap_or('\0'),
            bit: 0,
            i: 0,
        }
    }

    pub fn assemble(&mut self) -> Vec<u8> {
        while self.ch != '\0' {
            self.assemble_instr();
        }

        for (i, name) in self.lbl_replaces.iter() {
            let bit = match self.lbls.get(name) {
                Some(bit) => *bit,
                _ => panic!("unknown label {name:?}"),
            };

            self.machine_c.insert((*i)-1, bit as u8);
        }

        self.machine_c.push(1);
        return self.machine_c.clone();
    }

    fn assemble_instr(&mut self) {
        if self.ch == '.' {
            self.adv();

            let name = self.rd_til_ws();
            self.lbls.insert(name, self.bit.clone());

            return;
        }

        let opcode = self.rd_til_ws();
        self.bit += 1;
        
        match opcode.as_str() {
            "_" => self.machine_c.push(Opcode::NOP as u8),
            "hlt" => self.machine_c.push(Opcode::HLT as u8),
            "int" => {
                self.machine_c.push(Opcode::INT as u8);

                let int = match self.addr() {
                    Some(n) => n as u8,
                    _ => panic!("expected interrupt number after INT instr"),
                };

                self.machine_c.push(int);
            },
            "$" => {
                self.machine_c.push(Opcode::PUSH as u8);
                
                let immed = match self.immed() {
                    Some(i) => i,
                    _ => panic!("expected immediate after PUSH instr"),
                };

                self.psh_encoded_immed(immed);
            },
            "$$" => {
                self.machine_c.push(Opcode::PUSHR as u8);

                let reg = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected register after PUSHR instr"),
                } as u8;

                self.machine_c.push(reg);
            },
            "%" => {
                self.machine_c.push(Opcode::POP as u8);

                let reg = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected register after POP instr"),
                } as u8;

                self.machine_c.push(reg);
            },
            "@" => {
                self.machine_c.push(Opcode::LDI as u8);

                let reg = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected register after LDI instr"),
                } as u8;

                self.machine_c.push(reg);

                let immed = match self.immed() {
                    Some(i) => i,
                    _ => panic!("expected immed after register after LDI instr"),
                };

                self.psh_encoded_immed(immed);
            },
            ":" => {
                self.machine_c.push(Opcode::CPY as u8);

                let reg_a = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after CPY instr"),
                } as u8;

                let reg_b = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after CPY instr"),
                } as u8;

                self.machine_c.push(reg_a);
                self.machine_c.push(reg_b);
            },
            "//" => {
                self.machine_c.push(Opcode::JMP as u8);
                self.lbl();
            },
            "/=" => {
                self.machine_c.push(Opcode::JE as u8);
                self.lbl();
            },
            "/!" => {
                self.machine_c.push(Opcode::JNE as u8);
                self.lbl();
            },
            "/>" => {
                self.machine_c.push(Opcode::JG as u8);
                self.lbl();
            },
            "/<" => {
                self.machine_c.push(Opcode::JL as u8);
                self.lbl();
            },
            "//R" => {
                self.machine_c.push(Opcode::JMP as u8);

                let addr = match self.addr() {
                    Some(a) => a,
                    _ => panic!("expected addr after JMP (raw) instr"),
                } as u8;

                self.machine_c.push(addr);
            },
            "/=R" => {
                self.machine_c.push(Opcode::JE as u8);

                let addr = match self.addr() {
                    Some(a) => a,
                    _ => panic!("expected addr after JE (raw) instr"),
                } as u8;

                self.machine_c.push(addr);
            },
            "/!R" => {
                self.machine_c.push(Opcode::JNE as u8);

                let addr = match self.addr() {
                    Some(a) => a,
                    _ => panic!("expected addr after JNE (raw) instr"),
                } as u8;

                self.machine_c.push(addr);
            },
            "/>R" => {
                self.machine_c.push(Opcode::JG as u8);

                let addr = match self.addr() {
                    Some(a) => a,
                    _ => panic!("expected addr after JG (raw) instr"),
                } as u8;

                self.machine_c.push(addr);
            },
            "/<R" => {
                self.machine_c.push(Opcode::JL as u8);

                let addr = match self.addr() {
                    Some(a) => a,
                    _ => panic!("expected addr after JL (raw) instr"),
                } as u8;

                self.machine_c.push(addr);
            },
            "=" => {
                self.machine_c.push(Opcode::CMP as u8);

                let reg_a = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after CMP instr"),
                } as u8;

                let reg_b = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after CMP instr"),
                } as u8;

                self.machine_c.push(reg_a);
                self.machine_c.push(reg_b);
            },
            "+" => {
                self.machine_c.push(Opcode::ADD as u8);

                let reg_a = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after ADD instr"),
                } as u8;

                let reg_b = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after ADD instr"),
                } as u8;

                self.machine_c.push(reg_a);
                self.machine_c.push(reg_b);
            },
            "-" => {
                self.machine_c.push(Opcode::SUB as u8);

                let reg_a = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after SUB instr"),
                } as u8;
                
                let reg_b = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after SUB instr"),
                } as u8;

                self.machine_c.push(reg_a);
                self.machine_c.push(reg_b);
            },
            "*" => {
                self.machine_c.push(Opcode::MUL as u8);

                let reg_a = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after MUL instr"),
                } as u8;

                let reg_b = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after MUL instr"),
                } as u8;

                self.machine_c.push(reg_a);
                self.machine_c.push(reg_b);
            },
            "/" => {
                self.machine_c.push(Opcode::DIV as u8);

                let reg_a = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after DIV instr"),
                } as u8;

                let reg_b = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after DIV instr"),
                } as u8;

                self.machine_c.push(reg_a);
                self.machine_c.push(reg_b);
            },
            "&" => {
                self.machine_c.push(Opcode::AND as u8);

                let reg_a = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after AND instr"),
                } as u8;

                let reg_b = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after AND instr"),
                } as u8;

                self.machine_c.push(reg_a);
                self.machine_c.push(reg_b);
            },
            "|" => {
                self.machine_c.push(Opcode::OR as u8);

                let reg_a = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after OR instr"),
                } as u8;

                let reg_b = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after OR instr"),
                } as u8;

                self.machine_c.push(reg_a);
                self.machine_c.push(reg_b);
            },
            "^" => {
                self.machine_c.push(Opcode::XOR as u8);

                let reg_a = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after XOR instr"),
                } as u8;

                let reg_b = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected two registers after XOR instr"),
                } as u8;

                self.machine_c.push(reg_a);
                self.machine_c.push(reg_b);
            },
            ">" => {
                self.machine_c.push(Opcode::SHR as u8);

                let reg = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected a register after SHR instr"),
                } as u8;

                self.machine_c.push(reg);
                
                let immed = match self.immed() {
                    Some(i) => i,
                    _ => panic!("expected immed after reg after SHR instr"),
                };

                self.psh_encoded_immed(immed);
            },
            "<" => {
                self.machine_c.push(Opcode::SHL as u8);

                let reg = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected a register after SHL instr"),
                } as u8;

                self.machine_c.push(reg);
                
                let immed = match self.immed() {
                    Some(i) => i,
                    _ => panic!("expected immed after reg after SHL instr"),
                };

                self.psh_encoded_immed(immed);
            },
            "str" => {
                self.machine_c.push(Opcode::HSTORE as u8);

                let addr = match self.addr() {
                    Some(r) => r,
                    _ => panic!("expected a addr after HSTORE instr"),
                } as u8;

                self.machine_c.push(addr);
            },
            "strR" => {
                self.machine_c.push(Opcode::HSTORER as u8);

                let reg = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected a reg after HSTORES instr"),
                } as u8;

                self.machine_c.push(reg);
            },
            "ld" => {
                self.machine_c.push(Opcode::HLOAD as u8);

                let addr = match self.addr() {
                    Some(r) => r,
                    _ => panic!("expected a addr after HLOAD instr"),
                } as u8;

                self.machine_c.push(addr);
            },
            "ldR" => {
                self.machine_c.push(Opcode::HLOADR as u8);

                let reg = match self.reg() {
                    Some(r) => r,
                    _ => panic!("expected a reg after HLOADR instr"),
                } as u8;

                self.machine_c.push(reg);
            },
            _ => panic!("invalid opcode {opcode:?}"),
        };
    }

    fn rd_til_ws(&mut self) -> String {
        let mut buf = String::new();

        while match self.ch { '\0'|'\n'|'\r'|'\t'|' ' => false, _ => true } {
            buf.push(self.ch);
            self.adv();
        }

        while match self.ch { '\n'|'\r'|' '|'\t' => true, _ => false } {
            self.adv();
        }

        return buf;
    }

    fn lbl(&mut self) {
        let name = self.rd_til_ws();

        self.bit += 1; // to account for the addr
        self.lbl_replaces.push((self.bit.clone(), name));
    }

    fn psh_encoded_immed(&mut self, immed: Immediate) {
        let mut encoded: Vec<u8> = vec![];
    
        match immed {
            Immediate::None() => {
                self.bit += 1;
                encoded.push(255)
            },
            Immediate::U8(i) => {
                self.bit += 2;
                encoded.push(0);
                encoded.push(i);
            },
            Immediate::I8(i) => {
                self.bit += 2;
                encoded.push(1);
                encoded.push(i as u8);
            },
            Immediate::U16(i) => {
                self.bit += 3;
                encoded.push(2);
                encoded.extend_from_slice(&i.to_le_bytes());
            },
            Immediate::I16(i) => {
                self.bit += 3;
                encoded.push(3);
                encoded.extend_from_slice(&i.to_le_bytes());
            },
            Immediate::U32(i) => {
                self.bit += 5;
                encoded.push(4);
                encoded.extend_from_slice(&i.to_le_bytes());
            },
            Immediate::I32(i) => {
                self.bit += 5;
                encoded.push(5);
                encoded.extend_from_slice(&i.to_le_bytes());
            },
            Immediate::U64(i) => {
                self.bit += 9;
                encoded.push(6);
                encoded.extend_from_slice(&i.to_le_bytes());
            },
            Immediate::I64(i) => {
                self.bit += 9;
                encoded.push(7);
                encoded.extend_from_slice(&i.to_le_bytes());
            },
            Immediate::F32(i) => {
                self.bit += 5;
                encoded.push(8);
                encoded.extend_from_slice(&i.to_le_bytes());
            },
            Immediate::F64(i) => {
                self.bit += 9;
                encoded.push(9);
                encoded.extend_from_slice(&i.to_le_bytes());
            },
        }

        self.machine_c.extend(encoded);
    }

    fn reg(&mut self) -> Option<usize> {
        let mut reg_v: Vec<char> = self.rd_til_ws().chars().collect();
        self.bit += 1;

        if reg_v.len() < 2 {
            return None;
        }

        if reg_v[0] != 'R' {
            return None;
        }

        reg_v.remove(0);

        let mut reg_n_s = String::new();

        for c in reg_v.into_iter() {
            reg_n_s.push(c);
        }

        match reg_n_s.parse::<usize>() {
            Ok(n) => Some(n),
            Err(_) => None,
        }
    }

    fn addr(&mut self) -> Option<usize> {
        let addr_v: Vec<char> = self.rd_til_ws().chars().collect();
        let mut addr_s = String::new();
        self.bit += 1;

        for c in addr_v.into_iter() {
            addr_s.push(c);
        }

        match addr_s.parse::<usize>() {
            Ok(n) => Some(n),
            Err(_) => None,
        }
    }

    fn immed(&mut self) -> Option<Immediate> {
        let mut int: Vec<char> = self.rd_til_ws().chars().collect();

        if int.len() < 4 {
            return None;
        }

        let is_neg = {
            if int[0] == '-' {
                int.remove(0);
                true
            } else {
                false
            }
        };

        let t: char = int[0].clone();
        int.remove(0);

        match t {
            'i' => {},
            'u' => {},
            'f' => {},
            _ => return None,
        };

        if t == 'u' && is_neg {
            return None;
        }

        let mut int_type = t.to_string();

        while int[0] != '$' && int.len() > 0 {
            int_type.push(int[0].clone());
            int.remove(0);
        }

        if int[0] != '$' {
            return None;
        }

        int.remove(0);

        let mut int_str = String::new();

        while int.len() > 0 {
            int_str.push(int[0].clone());
            int.remove(0);
        }

        return Some(match int_type.as_str() {
            "u8" => Immediate::U8(int_str.parse::<u8>().unwrap()),
            "u16" => Immediate::U16(int_str.parse::<u16>().unwrap()),
            "u32" => Immediate::U32(int_str.parse::<u32>().unwrap()),
            "u64" => Immediate::U64(int_str.parse::<u64>().unwrap()),
            "i8" => Immediate::I8({
                let i = int_str.parse::<i8>().unwrap();
                if is_neg { -i } else { i }
            }),
            "i16" => Immediate::I16({
                let i = int_str.parse::<i16>().unwrap();
                if is_neg { -i } else { i }
            }),
            "i32" => Immediate::I32({
                let i = int_str.parse::<i32>().unwrap();
                if is_neg { -i } else { i }
            }),
            "i64" => Immediate::I64({
                let i = int_str.parse::<i64>().unwrap();
                if is_neg { -i } else { i }
            }),
            "f32" => Immediate::F32({
                let f = int_str.parse::<f32>().unwrap();
                if is_neg { -f } else { f }
            }),
            "f64" => Immediate::F64({
                let f = int_str.parse::<f64>().unwrap();
                if is_neg { -f } else { f }
            }),
            _ => return None,
        });
    }

    fn adv(&mut self) {
        if self.ch == '\0' {
            return;
        }

        self.i += 1;
        self.ch = self.src[self.i];
    }
}