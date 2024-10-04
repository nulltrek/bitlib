use std::convert::From;

pub trait Pow {
    fn pow(self, exponent: i32) -> Self;
}

impl Pow for i32 {
    fn pow(self, exponent: i32) -> Self {
        self.pow(exponent as u32)
    }
}

impl Pow for u64 {
    fn pow(self, exponent: i32) -> Self {
        self.pow(exponent as u32)
    }
}

impl Pow for i64 {
    fn pow(self, exponent: i32) -> Self {
        self.pow(exponent as u32)
    }
}

impl Pow for i128 {
    fn pow(self, exponent: i32) -> Self {
        self.pow(exponent as u32)
    }
}

impl Pow for u128 {
    fn pow(self, exponent: i32) -> Self {
        self.pow(exponent as u32)
    }
}
