use core::ops;
use std::cmp::PartialEq;
use std::fmt;

#[derive(PartialEq, Debug)]
pub struct FieldElement<const PRIME: u32> {
    num: i64,
}

impl<const PRIME: u32> FieldElement<PRIME> {
    pub fn new(num: i64) -> FieldElement<PRIME> {
        if num >= PRIME as i64 {
            panic!("The value cannot be greater than the PRIME")
        }
        if num < 0 {
            panic!("The value cannot be less than zero")
        }
        return FieldElement { num };
    }
}

impl<const PRIME: u32> fmt::Display for FieldElement<PRIME> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FieldElement_{}_{}", PRIME, self.num)
    }
}

impl<const PRIME: u32> PartialEq<i64> for FieldElement<PRIME> {
    fn eq(&self, other: &i64) -> bool {
        self.num == *other
    }
}

impl<const PRIME: u32> ops::Add<FieldElement<PRIME>> for FieldElement<PRIME> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            num: (self.num + other.num) % PRIME as i64,
        }
    }
}

impl<const PRIME: u32> ops::Add<i64> for FieldElement<PRIME> {
    type Output = Self;

    fn add(self, other: i64) -> Self {
        Self {
            num: (self.num + other) % PRIME as i64,
        }
    }
}

impl<const PRIME: u32> ops::Sub<FieldElement<PRIME>> for FieldElement<PRIME> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            num: (self.num - other.num).rem_euclid(PRIME as i64),
        }
    }
}

impl<const PRIME: u32> ops::Sub<i64> for FieldElement<PRIME> {
    type Output = Self;

    fn sub(self, other: i64) -> Self {
        Self {
            num: (self.num - other).rem_euclid(PRIME as i64),
        }
    }
}

impl<const PRIME: u32> ops::Mul<FieldElement<PRIME>> for FieldElement<PRIME> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            num: (self.num * other.num) % PRIME as i64,
        }
    }
}

impl<const PRIME: u32> ops::Mul<u64> for FieldElement<PRIME> {
    type Output = Self;

    fn mul(self, other: u64) -> Self {
        Self {
            num: (self.num * other as i64) % PRIME as i64,
        }
    }
}

impl<const PRIME: u32> FieldElement<PRIME> {
    pub fn pow(self, exponent: i32) -> FieldElement<PRIME> {
        let n = exponent.rem_euclid(PRIME as i32 - 1);
        return FieldElement {
            num: self.num.pow(n as u32) % PRIME as i64,
        };
    }
}

impl<const PRIME: u32> ops::Div<FieldElement<PRIME>> for FieldElement<PRIME> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        self * other.pow(PRIME as i32 - 2)
    }
}

impl<const PRIME: u32> ops::Div<u64> for FieldElement<PRIME> {
    type Output = Self;

    fn div(self, other: u64) -> Self {
        self * Self::new(other as i64).pow(PRIME as i32 - 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn field_element_new_less_than_zero_error() {
        FieldElement::<7>::new(-10);
    }

    #[test]
    #[should_panic]
    fn field_element_new_greater_than_prime_error() {
        FieldElement::<7>::new(15);
    }

    #[test]
    fn field_element_assert_eq() {
        assert_eq!(FieldElement::<7>::new(5), FieldElement::<7>::new(5));
        assert_eq!(FieldElement::<7>::new(5), 5);
    }

    #[test]
    fn field_element_add() {
        assert_eq!(FieldElement::<19>::new(7) + FieldElement::<19>::new(8), 15);
        assert_eq!(FieldElement::<19>::new(11) + 17, 9);
        assert_eq!(FieldElement::<19>::new(9) + 10, 0);
    }

    #[test]
    fn field_element_sub() {
        assert_eq!(FieldElement::<19>::new(11) - FieldElement::<19>::new(9), 2);
        assert_eq!(FieldElement::<19>::new(0) - 9, 10);
        assert_eq!(FieldElement::<19>::new(6) - 13, 12);
    }

    #[test]
    fn field_element_mul() {
        assert_eq!(FieldElement::<19>::new(5) * FieldElement::<19>::new(3), 15);
        assert_eq!(FieldElement::<19>::new(8) * 17, 3);
    }

    #[test]
    fn field_element_pow() {
        assert_eq!(FieldElement::<19>::new(7).pow(3), 1);
        assert_eq!(FieldElement::<19>::new(9).pow(12), 7);
        assert_eq!(FieldElement::<19>::new(1).pow(18), 1);
        assert_eq!(FieldElement::<19>::new(5).pow(18), 1);
        assert_eq!(FieldElement::<19>::new(9).pow(18), 1);

        assert_eq!(FieldElement::<19>::new(7).pow(-1), 11);
    }

    #[test]
    fn field_element_div() {
        assert_eq!(FieldElement::<19>::new(2) / FieldElement::<19>::new(7), 3);
        assert_eq!(FieldElement::<19>::new(7) / 5, 9);
    }
}
