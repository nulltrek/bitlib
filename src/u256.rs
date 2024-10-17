use num_bigint::BigUint;
use num_traits::cast::FromPrimitive;
use primitive_types;
use std::fmt;
use std::ops::{Add, Mul, Rem, Shr, Sub};

#[derive(PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct U256(primitive_types::U256);

impl U256 {
    pub fn from_hex(hex: &str) -> U256 {
        U256(primitive_types::U256::from_str_radix(hex, 16).unwrap())
    }
}

impl Default for U256 {
    fn default() -> Self {
        U256(primitive_types::U256::zero())
    }
}

impl fmt::Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl Add for U256 {
    type Output = U256;

    fn add(self, other: U256) -> Self::Output {
        U256(self.0 + other.0)
    }
}

impl Sub for U256 {
    type Output = U256;

    fn sub(self, other: U256) -> Self::Output {
        U256(self.0 - other.0)
    }
}

impl Mul for U256 {
    type Output = U256;

    fn mul(self, other: U256) -> Self::Output {
        U256(self.0 * other.0)
    }
}

impl Rem for U256 {
    type Output = U256;

    fn rem(self, other: U256) -> Self::Output {
        U256(self.0 % other.0)
    }
}

impl Shr<u32> for U256 {
    type Output = U256;

    fn shr(self, other: u32) -> Self::Output {
        U256(self.0 >> other)
    }
}

impl Shr<U256> for U256 {
    type Output = U256;

    fn shr(self, other: U256) -> Self::Output {
        U256(self.0 >> other.0)
    }
}

impl FromPrimitive for U256 {
    fn from_i64(n: i64) -> Option<Self> {
        Some(U256(primitive_types::U256::from(n as u64)))
    }
    fn from_u64(n: u64) -> Option<Self> {
        Some(U256(primitive_types::U256::from(n)))
    }
}

impl U256 {
    pub fn to_big_uint(&self) -> BigUint {
        BigUint::from_bytes_be(&self.0.to_big_endian())
    }
    pub fn from_big_uint(value: &BigUint) -> U256 {
        U256(primitive_types::U256::from_big_endian(
            value.to_bytes_be().as_slice(),
        ))
    }
}

mod tests {
    use super::*;

    #[test]
    fn u256_constructor() {
        let a = U256::from_hex("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
        let b = U256::from_hex("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
        assert_eq!(a, b);
    }

    #[test]
    fn u256_biguint_conversion() {
        let value =
            U256::from_hex("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");
        let big_num = value.to_big_uint();
        assert_eq!(U256::from_big_uint(&big_num), value);
    }
}
