use core::ops::{Mul, Rem, Shr};
use num_traits::{One, Zero};

pub trait Pow {
    fn pow(self, exponent: i32) -> Self;
}

/// Modular exponentiation
/// See: https://en.wikipedia.org/wiki/Modular_exponentiation#Right-to-left_binary_method
pub trait ModPow:
    Copy
    + PartialEq
    + PartialOrd
    + Zero
    + One
    + Mul
    + Shr<Output = Self>
    + Rem<Output = Self>
    + Rem<Output = Self>
{
    fn mod_pow(self, exponent: Self, modulus: Self) -> Self {
        let zero = Zero::zero();
        let one = One::one();
        let two = one + one;

        if modulus == one {
            return zero;
        }

        let mut exp = exponent;
        let mut base = self % modulus;
        let mut result = one;
        while exp > zero {
            if exp % two == one {
                result = (result * base) % modulus;
            }
            exp = exp >> one;
            base = (base * base) % modulus;
        }
        return result;
    }
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

impl ModPow for i32 {}
impl ModPow for u64 {}
impl ModPow for i64 {}
impl ModPow for i128 {}
impl ModPow for u128 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_pow() {
        assert_eq!(4_i64.mod_pow(13_i64, 497_i64), 445);
        assert_eq!(7_i64.mod_pow(3_i64, 19_i64), 1);
        assert_eq!(9_i64.mod_pow(12_i64, 19_i64), 7);
        assert_eq!(7_i32.mod_pow(3_i32, 19_i32), 1);
        assert_eq!(9_i32.mod_pow(12_i32, 19_i32), 7);
    }
}
