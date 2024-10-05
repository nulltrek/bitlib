use core::ops::{self, Add, Mul, Rem, Sub};
use num_traits::cast::FromPrimitive;
use std::cmp::{PartialEq, PartialOrd};
use std::convert::From;
use std::fmt::{self, Display};

use crate::traits::{ModPow, Pow};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct FieldElement<const PRIME: u32, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    num: T,
}

impl<const PRIME: u32, T> Default for FieldElement<PRIME, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    fn default() -> Self {
        Self { num: T::default() }
    }
}

impl<const PRIME: u32, T> FieldElement<PRIME, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    pub fn new(num: T) -> FieldElement<PRIME, T> {
        if num >= T::from_u32(PRIME).unwrap() {
            panic!("The value cannot be greater than the PRIME")
        }
        return FieldElement { num };
    }
}

impl<const PRIME: u32, T> fmt::Display for FieldElement<PRIME, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FieldElement_{}_{}", PRIME, self.num)
    }
}

impl<const PRIME: u32, T> From<u32> for FieldElement<PRIME, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    fn from(value: u32) -> Self {
        Self::new(T::from_u32(value).unwrap())
    }
}

impl<const PRIME: u32, T> ops::Add<FieldElement<PRIME, T>> for FieldElement<PRIME, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        let prime = T::from_u32(PRIME).unwrap();
        Self {
            num: (self.num + other.num) % prime,
        }
    }
}

impl<const PRIME: u32, T> ops::Sub<FieldElement<PRIME, T>> for FieldElement<PRIME, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        let prime = T::from_u32(PRIME).unwrap();
        if self.num >= other.num {
            Self {
                num: (self.num - other.num) % prime,
            }
        } else {
            Self {
                num: (prime - ((other.num - self.num) % prime)),
            }
        }
    }
}

impl<const PRIME: u32, T> ops::Mul<FieldElement<PRIME, T>> for FieldElement<PRIME, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        let prime = T::from_u32(PRIME).unwrap();
        Self {
            num: (self.num * other.num) % prime,
        }
    }
}

impl<const PRIME: u32, T> Pow for FieldElement<PRIME, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    fn pow(self, exponent: i32) -> FieldElement<PRIME, T> {
        let exponent = exponent.rem_euclid(PRIME as i32 - 1);
        Self {
            num: self
                .num
                .mod_pow(T::from_i32(exponent).unwrap(), T::from_u32(PRIME).unwrap()),
        }
    }
}

impl<const PRIME: u32, T> ops::Div<FieldElement<PRIME, T>> for FieldElement<PRIME, T>
where
    T: PartialOrd
        + Rem<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + ModPow
        + FromPrimitive
        + Default
        + Clone
        + Copy
        + Display,
{
    type Output = Self;

    fn div(self, other: Self) -> Self {
        self * other.pow(PRIME as i32 - 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn field_element_new_greater_than_prime_error() {
        FieldElement::<7, i32>::new(15);
    }

    #[test]
    fn field_element_assert_eq() {
        assert_eq!(
            FieldElement::<7, i32>::new(5),
            FieldElement::<7, i32>::new(5)
        );
        assert_eq!(FieldElement::<7, i32>::new(5), 5.into());
    }

    #[test]
    fn field_element_add() {
        assert_eq!(
            FieldElement::<19, i32>::new(7) + FieldElement::<19, i32>::new(8),
            15.into()
        );
        assert_eq!(FieldElement::<19, i32>::new(11) + 17.into(), 9.into());
        assert_eq!(FieldElement::<19, i32>::new(9) + 10.into(), 0.into());
    }

    #[test]
    fn field_element_sub() {
        assert_eq!(
            FieldElement::<19, i32>::new(11) - FieldElement::<19, i32>::new(9),
            2.into()
        );
        assert_eq!(FieldElement::<19, i32>::new(0) - 9.into(), 10.into());
        assert_eq!(FieldElement::<19, i32>::new(6) - 13.into(), 12.into());
    }

    #[test]
    fn field_element_mul() {
        assert_eq!(
            FieldElement::<19, i32>::new(5) * FieldElement::<19, i32>::new(3),
            15.into()
        );
        assert_eq!(FieldElement::<19, i32>::new(8) * 17.into(), 3.into());
    }

    #[test]
    fn field_element_pow() {
        assert_eq!(FieldElement::<19, i32>::new(7).pow(3), 1.into());
        assert_eq!(FieldElement::<19, i64>::new(9).pow(12), 7.into());
        assert_eq!(FieldElement::<19, i32>::new(1).pow(18), 1.into());
        assert_eq!(FieldElement::<19, i32>::new(5).pow(18), 1.into());
        assert_eq!(FieldElement::<19, i32>::new(9).pow(18), 1.into());

        assert_eq!(FieldElement::<19, i64>::new(7).pow(-1), 11.into());
        assert_eq!(FieldElement::<19, u64>::new(7).pow(-1), 11.into());
    }

    #[test]
    fn field_element_div() {
        assert_eq!(
            FieldElement::<19, i64>::new(2) / FieldElement::<19, i64>::new(7),
            3.into()
        );
        assert_eq!(FieldElement::<19, i64>::new(7) / 5.into(), 9.into());
    }
}
