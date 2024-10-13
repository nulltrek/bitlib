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
    + Pow<u32, Output = Self>
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

pub struct InfiniteField;

impl<T> Field<T> for InfiniteField
where
    T: FieldElement + Div<Output = T> + ToPrimitive,
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

pub struct FiniteField<T>
where
    T: FiniteFieldElement,
{
    prime: T,
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
        a.mul(self.pow(b, &(self.prime - Self::from_u8(2))))
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

// macro_rules! field {
//     ($field:ident, $op:ident, $a:expr, $b:expr) => {
//         $field.$op(&$a, &$b)
//     };
// }
macro_rules! eq {
    ($field:ident, $a:expr, $b:expr) => {
        $field.eq(&$a, &$b)
    };
}
macro_rules! add {
    ($field:ident, $a:expr, $b:expr) => {
        $field.add(&$a, &$b)
    };
}
macro_rules! sub {
    ($field:ident, $a:expr, $b:expr) => {
        $field.sub(&$a, &$b)
    };
}
macro_rules! mul {
    ($field:ident, $a:expr, $b:expr) => {
        $field.mul(&$a, &$b)
    };
}
macro_rules! div {
    ($field:ident, $a:expr, $b:expr) => {
        $field.div(&$a, &$b)
    };
}
macro_rules! pow {
    ($field:ident, $a:expr, $b:expr) => {
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
}
