pub struct Sample {
    value: [u8; 4],
}

impl Sample {
    pub fn r(&self) -> u8 {
        self.value[0]
    }

    pub fn g(&self) -> u8 {
        self.value[1]
    }

    pub fn b(&self) -> u8 {
        self.value[2]
    }

    pub fn a(&self) -> u8 {
        self.value[3]
    }
}

impl From<[u8; 4]> for Sample {
    fn from(value: [u8; 4]) -> Self {
        Self { value }
    }
}
