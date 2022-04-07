#![feature(const_trait_impl)]

pub type MemoryAddress = u64;
pub type RegisterAddress = u8;

pub enum Location {
    Memory(MemoryAddress),
    Register(RegisterAddress),
}

pub enum Opcode {
    Stop,

    Load,
    Move,
    Copy,
    Jump,

    Calc,
    Cmp,
    Bin,

    Illegal,
}

impl const From<u8> for Opcode {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Opcode::Stop,

            1 => Opcode::Load,
            2 => Opcode::Move,
            3 => Opcode::Copy,
            4 => Opcode::Jump,

            5 => Opcode::Calc,
            6 => Opcode::Cmp,
            7 => Opcode::Bin,

            _ => Opcode::Illegal,
        }
    }
}

pub enum CalcMethod {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
}

pub enum CmpMethod {
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Neq,
}

pub enum BinMethod {
    And {
        op1: RegisterAddress,
        op2: RegisterAddress,
        out: RegisterAddress,
    },
    Or {
        op1: RegisterAddress,
        op2: RegisterAddress,
        out: RegisterAddress,
    },
}

pub enum Instruction {
    Stop,

    Load {
        value: f64,
        register: RegisterAddress,
    },
    Move {
        src: Location,
        dst: Location,
    },
    Copy {
        src: Location,
        dst: Location,
    },
    Jump {
        offset: usize,
    },

    Calc {
        method: CalcMethod,
        op1: RegisterAddress,
        op2: RegisterAddress,
        dst: RegisterAddress,
    },
    Cmp {
        method: CmpMethod,
        op1: RegisterAddress,
        op2: RegisterAddress,
        dst: RegisterAddress,
    },
    Bin {
        method: BinMethod,
    },
}

struct Memory {
    data: Vec<u8>,
}

impl Memory {
    const DATA_SIZE: usize = 2048;

    fn set(&mut self, address: MemoryAddress, value: u8) {
        self.data[address as usize] = value;
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            data: vec![0; Self::DATA_SIZE],
        }
    }
}

pub struct Vm {
    registers: [u64; Self::REGISTER_COUNT],

    callstack: Vec<MemoryAddress>,
    byte_code: Vec<u8>,

    memory: Memory,

    pointer: usize,
}

impl Vm {
    const REGISTER_COUNT: usize = 32;

    pub fn new(byte_code: &[u8]) -> Self {
        Self {
            registers: [0; Self::REGISTER_COUNT],

            callstack: vec![],
            byte_code: byte_code.to_vec(),

            memory: Memory::default(),

            pointer: 0,
        }
    }

    fn take_u16(&mut self) -> Option<u16> {
        if self.pointer + 2 > self.byte_code.len() {
            return None
        }

        let bytes = unsafe {
            [
                *self.byte_code.get_unchecked(self.pointer + 1),
                *self.byte_code.get_unchecked(self.pointer + 2),
            ]
        };

        self.pointer += 2;

        Some(u16::from_le_bytes(bytes))
    }

    fn take_u32(&mut self) -> Option<u32> {
        if self.pointer + 8 > self.byte_code.len() {
            return None
        }

        let bytes = unsafe {
            [
                *self.byte_code.get_unchecked(self.pointer + 1),
                *self.byte_code.get_unchecked(self.pointer + 2),
                *self.byte_code.get_unchecked(self.pointer + 3),
                *self.byte_code.get_unchecked(self.pointer + 4),
            ]
        };

        self.pointer += 8;

        Some(u32::from_le_bytes(bytes))
    }

    fn take_u64(&mut self) -> Option<u64> {
        if self.pointer + 8 > self.byte_code.len() {
            return None
        }

        let bytes = unsafe {
            [
                *self.byte_code.get_unchecked(self.pointer + 1),
                *self.byte_code.get_unchecked(self.pointer + 2),
                *self.byte_code.get_unchecked(self.pointer + 3),
                *self.byte_code.get_unchecked(self.pointer + 4),
                *self.byte_code.get_unchecked(self.pointer + 5),
                *self.byte_code.get_unchecked(self.pointer + 6),
                *self.byte_code.get_unchecked(self.pointer + 7),
                *self.byte_code.get_unchecked(self.pointer + 8),
            ]
        };

        self.pointer += 8;

        Some(u64::from_le_bytes(bytes))
    }

    fn execute(&mut self) -> Option<bool> {
        let byte = self.byte_code[self.pointer];
        let opcode = Opcode::from(byte);

        match opcode {
            Opcode::Stop => Some(true),

            Opcode::Load => {
                let register = self.take_u16()? as usize;
                let value = self.take_u64()?;

                if register <= Self::REGISTER_COUNT {
                    return Some(false);
                }

                self.registers[register] = value;

                Some(true)
            }
            Opcode::Move => todo!(),
            Opcode::Copy => todo!(),
            Opcode::Jump => {
                let value = self.take_u32()? as usize;

                if value > self.byte_code.len() {
                    return None
                }

                self.pointer = value;

                Some(true)
            },

            Opcode::Calc => todo!(),
            Opcode::Cmp => todo!(),
            Opcode::Bin => todo!(),

            Opcode::Illegal => todo!(),
        }
    }
}
