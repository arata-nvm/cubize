#[derive(Debug, Default)]
pub struct CPU {
    pub register_a: u8,
    pub status: u8,
    pub program_counter: u8,
}

impl CPU {
    pub fn new() -> Self {
        Self::default()
    }
}
