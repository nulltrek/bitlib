use crate::u256::U256;
use core::ops::{Add, Div, Mul, Rem, Shr, Sub};
use num_traits::cast::{FromPrimitive, ToPrimitive};
use num_traits::pow::Pow;
use std::fmt::{self, Display};

pub trait Field<T>: Display {
    fn add(&self, a: &T, b: &T) -> T;
    fn sub(&self, a: &T, b: &T) -> T;
    fn mul(&self, a: &T, b: &T) -> T;
    fn div(&self, a: &T, b: &T) -> T;
    fn pow(&self, a: &T, exponent: &T) -> T;
    fn eq(&self, a: &T, b: &T) -> bool;
    fn neq(&self, a: &T, b: &T) -> bool {
        !self.eq(a, b)
    }
    fn from_u8(value: u8) -> T;
}

pub trait FieldElement:
    Default
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Rem<Output = Self>
    + Shr<Output = Self>
    + FromPrimitive
    + Copy
    + Clone
    + Display
{
}

impl FieldElement for u32 {}
impl FieldElement for i32 {}
impl FieldElement for u64 {}
impl FieldElement for i64 {}
impl FieldElement for u128 {}
impl FieldElement for i128 {}
impl FieldElement for U256 {}

pub struct InfiniteField;

impl<T> Field<T> for InfiniteField
where
    T: FieldElement + Div<Output = T> + Pow<u32, Output = T> + ToPrimitive,
{
    fn add(&self, a: &T, b: &T) -> T {
        a.add(*b)
    }
    fn sub(&self, a: &T, b: &T) -> T {
        a.sub(*b)
    }
    fn mul(&self, a: &T, b: &T) -> T {
        a.mul(*b)
    }
    fn div(&self, a: &T, b: &T) -> T {
        a.div(*b)
    }
    fn pow(&self, a: &T, exponent: &T) -> T {
        a.pow(exponent.to_u32().unwrap())
    }
    fn eq(&self, a: &T, b: &T) -> bool {
        a == b
    }
    fn from_u8(value: u8) -> T {
        T::from_u8(value).unwrap()
    }
}

impl fmt::Display for InfiniteField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "inf")
    }
}

pub trait FiniteFieldElement: FieldElement + Rem<Output = Self> {}

impl FiniteFieldElement for u32 {}
impl FiniteFieldElement for i32 {}
impl FiniteFieldElement for u64 {}
impl FiniteFieldElement for i64 {}
impl FiniteFieldElement for u128 {}
impl FiniteFieldElement for i128 {}
impl FiniteFieldElement for U256 {}

pub struct FiniteField<T>
where
    T: FiniteFieldElement,
{
    pub prime: T,
}

impl<T> FiniteField<T>
where
    T: FiniteFieldElement,
{
    pub fn new(prime: T) -> Self {
        Self { prime }
    }
}

impl<T> Field<T> for FiniteField<T>
where
    T: FiniteFieldElement,
{
    fn add(&self, a: &T, b: &T) -> T {
        a.add(*b) % self.prime
    }
    fn sub(&self, a: &T, b: &T) -> T {
        if a >= b {
            a.sub(*b) % self.prime
        } else {
            self.prime - (b.sub(*a) % self.prime)
        }
    }
    fn mul(&self, a: &T, b: &T) -> T {
        a.mul(*b) % self.prime
    }
    fn div(&self, a: &T, b: &T) -> T {
        self.mul(a, &self.pow(b, &(self.prime - Self::from_u8(2))))
    }

    /// Modular exponentiation
    /// See: https://en.wikipedia.org/wiki/Modular_exponentiation#Right-to-left_binary_method
    fn pow(&self, a: &T, exponent: &T) -> T {
        let zero = T::default();
        let one = Self::from_u8(1);
        let two = Self::from_u8(2);

        if self.prime == one {
            return zero;
        }

        let mut exp = *exponent;
        let mut base = *a % self.prime;
        let mut result = one;
        while exp > zero {
            if exp % two == one {
                result = (result * base) % self.prime;
            }
            exp = exp >> one;
            base = (base * base) % self.prime;
        }
        return result;
    }
    fn eq(&self, a: &T, b: &T) -> bool {
        (*a % self.prime) == (*b % self.prime)
    }
    fn from_u8(value: u8) -> T {
        T::from_u8(value).unwrap()
    }
}

impl<T> fmt::Display for FiniteField<T>
where
    T: FiniteFieldElement,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prime: {}", self.prime)
    }
}

pub struct FiniteFieldU256 {
    pub prime: U256,
}

impl FiniteFieldU256 {
    pub fn new(prime: U256) -> Self {
        Self { prime }
    }
}

impl Field<U256> for FiniteFieldU256 {
    fn add(&self, a: &U256, b: &U256) -> U256 {
        let a = a.to_big_uint();
        let b = b.to_big_uint();
        let prime = self.prime.to_big_uint();
        U256::from_big_uint(&(a.add(b) % prime))
    }

    fn sub(&self, a: &U256, b: &U256) -> U256 {
        if *a >= *b {
            (*a).sub(*b) % self.prime
        } else {
            self.prime - (b.sub(*a) % self.prime)
        }
    }

    fn mul(&self, a: &U256, b: &U256) -> U256 {
        let a = a.to_big_uint();
        let b = b.to_big_uint();
        let prime = self.prime.to_big_uint();
        let result = (a * b) % prime;
        U256::from_big_uint(&result)
    }

    fn div(&self, a: &U256, b: &U256) -> U256 {
        self.mul(a, &self.pow(b, &(self.prime - Self::from_u8(2))))
    }

    /// Modular exponentiation
    /// See: https://en.wikipedia.org/wiki/Modular_exponentiation#Right-to-left_binary_method
    fn pow(&self, a: &U256, exponent: &U256) -> U256 {
        let zero = U256::default();
        let one = Self::from_u8(1);
        let two = Self::from_u8(2);

        if self.prime == one {
            return zero;
        }

        let prime = self.prime.to_big_uint();
        let mut exp = *exponent;
        let mut base = (*a % self.prime).to_big_uint();
        let mut result = one.to_big_uint();
        while exp > zero {
            if exp % two == one {
                result = (result * &base) % &prime;
            }
            exp = exp >> one;
            base = (&base * &base) % &prime;
        }
        return U256::from_big_uint(&result);
    }

    fn eq(&self, a: &U256, b: &U256) -> bool {
        (*a % self.prime) == (*b % self.prime)
    }
    fn from_u8(value: u8) -> U256 {
        U256::from_u8(value).unwrap()
    }
}

impl fmt::Display for FiniteFieldU256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prime: {}", self.prime)
    }
}

// macro_rules! field {
//     ($field:ident, $op:ident, $a:expr, $b:expr) => {
//         $field.$op(&$a, &$b)
//     };
// }
macro_rules! eq {
    ($field:expr, $a:expr, $b:expr) => {
        $field.eq(&$a, &$b)
    };
}
macro_rules! add {
    ($field:expr, $a:expr, $b:expr) => {
        $field.add(&$a, &$b)
    };
}
macro_rules! sub {
    ($field:expr, $a:expr, $b:expr) => {
        $field.sub(&$a, &$b)
    };
}
macro_rules! mul {
    ($field:expr, $a:expr, $b:expr) => {
        $field.mul(&$a, &$b)
    };
}
macro_rules! div {
    ($field:expr, $a:expr, $b:expr) => {
        $field.div(&$a, &$b)
    };
}
macro_rules! pow {
    ($field:expr, $a:expr, $b:expr) => {
        $field.pow(&$a, &$b)
    };
}
pub(crate) use add;
pub(crate) use div;
pub(crate) use eq;
pub(crate) use mul;
pub(crate) use pow;
pub(crate) use sub;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infinite_field() {
        assert!(eq!(InfiniteField, 2_u32, 2));
        assert_eq!(add!(InfiniteField, 1_u32, 1_u32), 2_u32);
        assert_eq!(sub!(InfiniteField, 1_u32, 1_u32), 0_u32);
        assert_eq!(mul!(InfiniteField, 2_u32, 4_u32), 8_u32);
        assert_eq!(pow!(InfiniteField, 2_u32, 4), 16_u32);

        assert_eq!(add!(InfiniteField, 1_i32, 7_i32), 8_i32);
        assert_eq!(sub!(InfiniteField, 1_i32, 4_i32), -3_i32);
        assert_eq!(mul!(InfiniteField, 2_i32, -4_i32), -8_i32);
        assert_eq!(pow!(InfiniteField, 2_i32, 4), 16_i32);
    }

    #[test]
    fn finite_field() {
        let f = FiniteField::<u32>::new(7);

        assert!(eq!(f, 2_u32, 2));
        assert!(eq!(f, 9_u32, 2));
        assert_eq!(add!(f, 1, 1), 2_u32);
        assert_eq!(sub!(f, 1, 1), 0_u32);
        assert_eq!(mul!(f, 2, 4), 1_u32);
        assert_eq!(pow!(f, 2, 4), 2_u32);

        let f = FiniteField::<u32>::new(31);
        assert_eq!(div!(f, 3, 24), 4_u32);
        assert_eq!(div!(f, 1, 4913), 29_u32);
        assert_eq!(mul!(f, 11, div!(f, 1, 256)), 13_u32);
    }

    #[test]
    fn finite_field_mod_pow() {
        let f = FiniteField::<u32>::new(497);
        assert_eq!(pow!(f, 4_u32, 13), 445);

        let f = FiniteField::<i64>::new(19);
        assert_eq!(pow!(f, 7_i64, 3), 1);
        assert_eq!(pow!(f, 9_i64, 12), 7);

        let f = FiniteField::<i32>::new(19);
        assert_eq!(pow!(f, 7_i32, 3), 1);
        assert_eq!(pow!(f, 9_i32, 12), 7);
    }

    #[test]
    fn finite_field_u256_div() {
        let f = FiniteFieldU256::new(U256::from_hex("1f"));

        assert_eq!(
            mul!(
                f,
                U256::from_hex("b"),
                div!(f, U256::from_hex("1"), U256::from_hex("100"))
            ),
            U256::from_hex("d")
        );

        let f = FiniteFieldU256::new(U256::from_hex(
            "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
        ));
        assert_eq!(
            div!(
                f,
                U256::from_hex("ec208baa0fc1c19f708a9ca96fdeff3ac3f230bb4a7ba4aede4942ad003c0f60",),
                U256::from_hex("68342ceff8935ededd102dd876ffd6ba72d6a427a3edb13d26eb0781cb423c4")
            ),
            U256::from_dec(
                "94501631981587311006704454863488507556501540117424777655513324157345498405185"
            )
        );

        assert_eq!(
            pow!(
                f,
                U256::from_hex("68342ceff8935ededd102dd876ffd6ba72d6a427a3edb13d26eb0781cb423c4"),
                U256::from_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141")
                // U256::from_hex("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f")
                    - U256::from_dec("2")
            ),
            U256::from_dec(
                "100323378640741192763451357826607979131075829390957642811683296770329292192481"
            )
        )
    }

    #[test]
    fn finite_field_secp256k1_point() {
        let prime =
            U256::from_hex("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
        let f = FiniteFieldU256::new(prime);

        let x = U256::from_hex("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");
        let y = U256::from_hex("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8");
        assert_eq!(
            pow!(f, y, U256::from_u32(2).unwrap()),
            add!(
                f,
                pow!(f, x, U256::from_u32(3).unwrap()),
                U256::from_u32(7).unwrap()
            )
        );
    }
}
