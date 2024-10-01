use core::ops;
use std::fmt;

use crate::traits;

impl traits::Pow for i64 {
    fn pow(self, exponent: i32) -> i64 {
        self.pow(exponent as u32)
    }
}

/// Defines a curve in the form:
///    y^2 = x^3 + ax + b
///
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Curve<T> {
    a: T,
    b: T,
}

impl<T> fmt::Display for Curve<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.a, self.b)
    }
}

#[derive(PartialEq, Debug)]
pub enum Coords<T> {
    Inf,
    Def { x: T, y: T },
}

impl<T> fmt::Display for Coords<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Coords::Inf => write!(f, "inf"),
            Coords::Def { x, y } => write!(f, "{}, {}", x, y),
        }
    }
}

/// The point only supports curves in the form:
///    y^2 = x^3 + Ax + B
///
#[derive(PartialEq, Debug)]
pub struct Point<T>
where
    T: ops::Add<T, Output = T>
        + ops::Add<i64, Output = T>
        + ops::Sub<T, Output = T>
        + ops::Sub<i64>
        + ops::Mul<T, Output = T>
        + ops::Mul<i64, Output = T>
        + ops::Div<T, Output = T>
        + traits::Pow
        + PartialEq
        + Default
        + std::fmt::Display
        + Copy,
{
    curve: Curve<T>,
    coords: Coords<T>,
}

impl<T> Point<T>
where
    T: ops::Add<T, Output = T>
        + ops::Add<i64, Output = T>
        + ops::Sub<T, Output = T>
        + ops::Sub<i64>
        + ops::Mul<T, Output = T>
        + ops::Mul<i64, Output = T>
        + ops::Div<T, Output = T>
        + traits::Pow
        + PartialEq
        + Default
        + std::fmt::Display
        + Copy,
{
    pub fn new(curve: Curve<T>, coords: Coords<T>) -> Point<T> {
        match coords {
            Coords::Inf => (),
            Coords::Def { x, y } => {
                if y * y != x.pow(3) + x * curve.a + curve.b {
                    panic!(
                        "The point ({}, {}) is not on the curve ({}, {})",
                        x, y, curve.a, curve.b
                    )
                }
            }
        }
        Point { curve, coords }
    }
}

impl<T> fmt::Display for Point<T>
where
    T: ops::Add<T, Output = T>
        + ops::Add<i64, Output = T>
        + ops::Sub<T, Output = T>
        + ops::Sub<i64>
        + ops::Mul<T, Output = T>
        + ops::Mul<i64, Output = T>
        + ops::Div<T, Output = T>
        + traits::Pow
        + PartialEq
        + Default
        + std::fmt::Display
        + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point({})_{}", self.curve, self.coords)
    }
}

impl<T> ops::Add<Point<T>> for Point<T>
where
    T: ops::Add<T, Output = T>
        + ops::Add<i64, Output = T>
        + ops::Sub<T, Output = T>
        + ops::Sub<i64>
        + ops::Mul<T, Output = T>
        + ops::Mul<i64, Output = T>
        + ops::Div<T, Output = T>
        + traits::Pow
        + PartialEq
        + Default
        + std::fmt::Display
        + Copy,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.curve != other.curve {
            panic!(
                "Curve parameters are different between {} and {}",
                self, other
            )
        }

        let (self_x, self_y) = match self.coords {
            Coords::Inf => return other,
            Coords::Def { x, y } => (x, y),
        };
        let (other_x, other_y) = match other.coords {
            Coords::Inf => return self,
            Coords::Def { x, y } => (x, y),
        };

        if self_x == other_x && self_y == T::default() && other_y == T::default() {
            return Self::new(self.curve, Coords::Inf);
        }
        // Points are equal
        if self_x == other_x && self_y == other_y {
            let slope = (self_x * self_x * 3 + self.curve.a) / (self_y * 2);
            let x = slope * slope - self_x * 2;
            let y = slope * (self_x - x) - self_y;
            return Self::new(self.curve, Coords::Def { x, y });
        }
        // Points are mirrored on X axis, forming a vertical line
        if self_x == other_x && self_y != other_y {
            return Self::new(self.curve, Coords::Inf);
        }

        // Points are different
        let slope = (other_y - self_y) / (other_x - self_x);
        let x = slope * slope - self_x - other_x;
        let y = slope * (self_x - x) - self_y;
        return Self::new(self.curve, Coords::Def { x, y });
    }
}

#[cfg(test)]
mod tests {
    use crate::fields::FieldElement;

    use super::*;
    use std::panic;

    #[test]
    #[should_panic]
    fn point_not_on_curve() {
        Point::<i64>::new(Curve { a: 5, b: 7 }, Coords::Def { x: 5, y: 7 });
    }

    #[test]
    fn point_constructor() {
        let curve = Curve { a: 5, b: 7 };
        Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 });
        Point::<i64>::new(curve, Coords::Def { x: 18, y: 77 });
    }

    #[test]
    fn point_equality() {
        let curve = Curve { a: 0, b: 7 };
        assert_eq!(
            Point::<i64>::new(curve, Coords::Inf),
            Point::new(curve, Coords::Inf)
        );

        let curve = Curve { a: 5, b: 7 };
        assert_ne!(
            Point::<i64>::new(curve, Coords::Inf),
            Point::new(curve, Coords::Def { x: -1, y: -1 })
        );
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 }),
            Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 })
        );
        assert_ne!(
            Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 }),
            Point::<i64>::new(curve, Coords::Def { x: 18, y: 77 })
        );
    }

    #[test]
    fn point_sum() {
        let curve = Curve { a: 5, b: 7 };
        assert_eq!(
            Point::<i64>::new(curve, Coords::Inf)
                + Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 }),
            Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 })
        );
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 })
                + Point::<i64>::new(curve, Coords::Def { x: 2, y: 5 }),
            Point::<i64>::new(curve, Coords::Def { x: 3, y: -7 })
        );

        let curve = Curve { a: 1, b: 2 };
        // Points are equal, tangent to the curve at y = 0
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: -1, y: 0 })
                + Point::<i64>::new(curve, Coords::Def { x: -1, y: 0 }),
            Point::<i64>::new(curve, Coords::Inf)
        );

        let curve = Curve { a: 5, b: 7 };
        // Points are equal
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 })
                + Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 }),
            Point::<i64>::new(curve, Coords::Def { x: 18, y: 77 })
        );

        let curve = Curve { a: 1, b: 4 };
        // Points are mirrored on X axis, forming a vertical line
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: 0, y: 2 })
                + Point::<i64>::new(curve, Coords::Def { x: 0, y: -2 }),
            Point::<i64>::new(curve, Coords::Inf)
        );

        let curve = Curve { a: 5, b: 7 };
        // Points are different
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: 2, y: 5 })
                + Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 }),
            Point::<i64>::new(curve, Coords::Def { x: 3, y: -7 })
        );
    }

    #[test]
    fn point_valid_field_elements() {
        let curve = Curve {
            a: FieldElement::<223>::new(0),
            b: FieldElement::<223>::new(7),
        };

        for pair in [(192, 105), (17, 56), (1, 193)].iter() {
            let x = FieldElement::<223>::new(pair.0);
            let y = FieldElement::<223>::new(pair.1);
            Point::<FieldElement<223>>::new(curve, Coords::Def { x, y });
        }
    }

    #[test]
    fn point_invalid_field_elements() {
        let curve = Curve {
            a: FieldElement::<223>::new(0),
            b: FieldElement::<223>::new(7),
        };

        for pair in [(200, 119), (42, 99)].iter() {
            let result = panic::catch_unwind(|| {
                let x = FieldElement::<223>::new(pair.0);
                let y = FieldElement::<223>::new(pair.1);
                Point::<FieldElement<223>>::new(curve, Coords::Def { x, y });
            });
            assert!(result.is_err());
        }
    }
}
