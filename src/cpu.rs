#[derive(Debug)]
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,

    memory: [u8; 0xffff],
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing,
}

#[derive(Debug)]
pub struct OpCode {
    opcode: u8,
    mnemonic: Mnemonic,
    bytes: u8,
    cycles: u8,
    addr_mode: AddressingMode,
}

impl OpCode {
    const fn new(
        opcode: u8,
        mnemonic: Mnemonic,
        bytes: u8,
        cycles: u8,
        addr_mode: AddressingMode,
    ) -> Self {
        Self {
            opcode,
            mnemonic,
            bytes,
            cycles,
            addr_mode,
        }
    }
}

#[derive(Debug)]
pub enum Mnemonic {
    ADC,
    BRK,
    INX,
    LDA,
    STA,
    TAX,
}

pub const CPU_OPCODES: &[OpCode] = &[
    OpCode::new(0x69, Mnemonic::ADC, 2, 2, AddressingMode::Immediate),
    OpCode::new(0x65, Mnemonic::ADC, 2, 3, AddressingMode::ZeroPage),
    OpCode::new(0x75, Mnemonic::ADC, 2, 4, AddressingMode::ZeroPageX),
    OpCode::new(0x6d, Mnemonic::ADC, 3, 4, AddressingMode::Absolute),
    OpCode::new(0x7d, Mnemonic::ADC, 3, 4, AddressingMode::AbsoluteX),
    OpCode::new(0x79, Mnemonic::ADC, 3, 4, AddressingMode::AbsoluteY),
    OpCode::new(0x61, Mnemonic::ADC, 2, 6, AddressingMode::IndirectX),
    OpCode::new(0x71, Mnemonic::ADC, 2, 5, AddressingMode::IndirectY),
    OpCode::new(0x00, Mnemonic::BRK, 1, 7, AddressingMode::NoneAddressing),
    OpCode::new(0xe8, Mnemonic::INX, 1, 2, AddressingMode::NoneAddressing),
    OpCode::new(0xa9, Mnemonic::LDA, 2, 2, AddressingMode::Immediate),
    OpCode::new(0xa5, Mnemonic::LDA, 2, 3, AddressingMode::ZeroPage),
    OpCode::new(0xb5, Mnemonic::LDA, 2, 4, AddressingMode::ZeroPageX),
    OpCode::new(0xad, Mnemonic::LDA, 3, 4, AddressingMode::Absolute),
    OpCode::new(0xbd, Mnemonic::LDA, 3, 4, AddressingMode::AbsoluteX),
    OpCode::new(0xb9, Mnemonic::LDA, 3, 4, AddressingMode::AbsoluteY),
    OpCode::new(0xa1, Mnemonic::LDA, 2, 6, AddressingMode::IndirectX),
    OpCode::new(0xb1, Mnemonic::LDA, 2, 5, AddressingMode::IndirectY),
    OpCode::new(0x85, Mnemonic::STA, 2, 3, AddressingMode::ZeroPage),
    OpCode::new(0x95, Mnemonic::STA, 2, 4, AddressingMode::ZeroPageX),
    OpCode::new(0x8d, Mnemonic::STA, 3, 4, AddressingMode::Absolute),
    OpCode::new(0x9d, Mnemonic::STA, 3, 5, AddressingMode::AbsoluteX),
    OpCode::new(0x99, Mnemonic::STA, 3, 5, AddressingMode::AbsoluteY),
    OpCode::new(0x81, Mnemonic::STA, 2, 6, AddressingMode::IndirectX),
    OpCode::new(0x91, Mnemonic::STA, 2, 6, AddressingMode::IndirectY),
    OpCode::new(0xaa, Mnemonic::TAX, 1, 2, AddressingMode::NoneAddressing),
];

pub const CARRY: u8 = 0b0000_0001;
pub const ZERO: u8 = 0b0000_0010;
pub const OVERFLOW: u8 = 0b0100_0000;
pub const SIGN: u8 = 0b1000_0000;

impl CPU {
    pub fn new() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,

            memory: [0; 0xffff],
        }
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xfffc, 0x8000);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xfffc);
    }

    pub fn run(&mut self) {
        use self::Mnemonic::*;

        loop {
            let opcode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            let mut done = false;
            for op in CPU_OPCODES {
                if op.opcode == opcode {
                    match op.mnemonic {
                        ADC => self.adc(op.addr_mode),
                        BRK => return,
                        INX => self.inx(),
                        LDA => self.lda(op.addr_mode),
                        STA => self.sta(op.addr_mode),
                        TAX => self.tax(),
                    }

                    self.program_counter += op.bytes as u16 - 1;
                    done = true;
                    break;
                }
            }

            if !done {
                unimplemented!("opcode: {:x}", opcode);
            }
        }
    }

    fn adc(&mut self, mode: AddressingMode) {
        let a = self.register_a;
        let m = self.mem_read(self.get_operand_address(mode));
        let c = self.get_flag(CARRY) as u8;

        let (a_m, overflow1) = (a as i8).overflowing_add(m as i8);
        let (result, overflow2) = a_m.overflowing_add(c as i8);
        let result_carry = (a as u16).wrapping_add(m as u16).wrapping_add(c as u16) >> 8;
        let result_sign = result >> 7;

        self.register_a = result as u8;
        self.set_flag(CARRY, result_carry != 0);
        self.set_flag(ZERO, result == 0);
        self.set_flag(OVERFLOW, overflow1 | overflow2);
        self.set_flag(SIGN, result_sign != 0);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_flags(self.register_x);
    }

    fn lda(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_flags(self.register_a);
    }

    fn sta(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_flags(self.register_x);
    }

    fn update_flags(&mut self, value: u8) {
        self.set_flag(ZERO, value == 0);
        self.set_flag(SIGN, value & 0b1000_0000 != 0);
    }

    fn set_flag(&mut self, flag: u8, status: bool) {
        if status {
            self.status |= flag;
        } else {
            self.status &= !flag;
        }
    }

    fn get_flag(&mut self, flag: u8) -> bool {
        self.status & flag != 0
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let lo = (data & 0xff) as u8;
        let hi = (data >> 8) as u8;
        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi);
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn get_operand_address(&self, mode: AddressingMode) -> u16 {
        use self::AddressingMode::*;

        match mode {
            Immediate => self.program_counter,
            ZeroPage => self.mem_read(self.program_counter) as u16,
            Absolute => self.mem_read_u16(self.program_counter),

            ZeroPageX => {
                let base = self.mem_read(self.program_counter);
                base.wrapping_add(self.register_x) as u16
            }
            ZeroPageY => {
                let base = self.mem_read(self.program_counter);
                base.wrapping_add(self.register_y) as u16
            }

            AbsoluteX => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_x as u16)
            }
            AbsoluteY => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_y as u16)
            }

            IndirectX => {
                let base = self.mem_read(self.program_counter) as u16;
                let addr = base.wrapping_add(self.register_x as u16);
                self.mem_read_u16(addr)
            }
            IndirectY => {
                let addr = self.mem_read(self.program_counter) as u16;
                self.mem_read_u16(addr) + self.register_y as u16
            }

            NoneAddressing => panic!("{:?} is not supported", mode),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.get_flag(ZERO));
        assert!(!cpu.get_flag(SIGN));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.get_flag(ZERO));
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x10, 0xaa, 0x00]);
        assert_eq!(cpu.register_x, 0x10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 0x1);
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_sta_move_a_to_memory() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x10, 0x85, 0xff, 0x00]);
        assert_eq!(cpu.mem_read(0x00ff), 0x10);
    }

    #[test]
    fn test_adc() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x10, 0x69, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x15);
        assert!(!cpu.get_flag(CARRY));
        assert!(!cpu.get_flag(ZERO));
        assert!(!cpu.get_flag(OVERFLOW));
        assert!(!cpu.get_flag(SIGN));
    }

    #[test]
    fn test_adc_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x7f, 0x69, 0x01, 0x00]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(!cpu.get_flag(CARRY));
        assert!(!cpu.get_flag(ZERO));
        assert!(cpu.get_flag(OVERFLOW));
        assert!(cpu.get_flag(SIGN));
    }

    #[test]
    fn test_adc_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x69, 0x01, 0x00]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.get_flag(CARRY));
        assert!(cpu.get_flag(ZERO));
        assert!(!cpu.get_flag(OVERFLOW));
        assert!(!cpu.get_flag(SIGN));
    }
}
