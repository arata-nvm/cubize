#[derive(Debug, Default)]
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u8,
}

pub const ZERO: u8 = 0b0000_0010;
pub const SIGN: u8 = 0b1000_0000;

impl CPU {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let opcode = program[self.inc_pc()];

            match opcode {
                0x00 => return,
                0xa9 => {
                    let param = program[self.inc_pc()];
                    self.lda(param);
                }
                0xaa => self.tax(),
                0xe8 => self.inx(),
                _ => unimplemented!(),
            }
        }
    }

    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
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

    fn inc_pc(&mut self) -> usize {
        let cur_pc = self.program_counter;
        self.program_counter += 1;
        cur_pc as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.get_flag(ZERO));
        assert!(!cpu.get_flag(SIGN));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.get_flag(ZERO));
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x10;
        cpu.interpret(vec![0xaa, 0x00]);
        assert_eq!(cpu.register_x, 0x10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 0x1);
    }
}
