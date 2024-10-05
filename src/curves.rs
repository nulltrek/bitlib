use core::ops;
use std::fmt;

use crate::traits;

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
        + ops::Sub<T, Output = T>
        + ops::Mul<T, Output = T>
        + ops::Div<T, Output = T>
        + traits::Pow
        + std::convert::From<u32>
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
        + ops::Sub<T, Output = T>
        + ops::Mul<T, Output = T>
        + ops::Div<T, Output = T>
        + std::convert::From<u32>
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
        + ops::Sub<T, Output = T>
        + ops::Mul<T, Output = T>
        + ops::Div<T, Output = T>
        + traits::Pow
        + std::convert::From<u32>
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
        + ops::Sub<T, Output = T>
        + ops::Mul<T, Output = T>
        + ops::Div<T, Output = T>
        + std::convert::From<u32>
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
            let slope = (T::from(3) * self_x * self_x + self.curve.a) / (T::from(2) * self_y);
            let x = (slope * slope) - (T::from(2) * self_x);
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
        Point::<u64>::new(Curve::<u64> { a: 5, b: 7 }, Coords::Def { x: 5, y: 7 });
    }

    #[test]
    fn point_constructor() {
        Point::<i64>::new(Curve::<i64> { a: 5, b: 7 }, Coords::Def { x: -1, y: -1 });
        Point::<u64>::new(Curve::<u64> { a: 5, b: 7 }, Coords::Def { x: 18, y: 77 });
    }

    #[test]
    fn point_equality() {
        let curve = Curve::<u64> { a: 0, b: 7 };
        assert_eq!(
            Point::<u64>::new(curve, Coords::Inf),
            Point::<u64>::new(curve, Coords::Inf)
        );

        let curve = Curve::<i64> { a: 5, b: 7 };
        assert_ne!(
            Point::<i64>::new(curve, Coords::Inf),
            Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 })
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
        let curve = Curve::<i64> { a: 5, b: 7 };
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

        let curve = Curve::<i64> { a: 1, b: 2 };
        // Points are equal, tangent to the curve at y = 0
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: -1, y: 0 })
                + Point::<i64>::new(curve, Coords::Def { x: -1, y: 0 }),
            Point::<i64>::new(curve, Coords::Inf)
        );

        let curve = Curve::<i64> { a: 5, b: 7 };
        // Points are equal
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 })
                + Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 }),
            Point::<i64>::new(curve, Coords::Def { x: 18, y: 77 })
        );

        let curve = Curve::<i64> { a: 1, b: 4 };
        // Points are mirrored on X axis, forming a vertical line
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: 0, y: 2 })
                + Point::<i64>::new(curve, Coords::Def { x: 0, y: -2 }),
            Point::<i64>::new(curve, Coords::Inf)
        );

        let curve = Curve::<i64> { a: 5, b: 7 };
        // Points are different
        assert_eq!(
            Point::<i64>::new(curve, Coords::Def { x: 2, y: 5 })
                + Point::<i64>::new(curve, Coords::Def { x: -1, y: -1 }),
            Point::<i64>::new(curve, Coords::Def { x: 3, y: -7 })
        );
    }

    #[test]
    fn point_valid_field_elements() {
        let curve = Curve::<FieldElement<223, u64>> {
            a: FieldElement::<223, u64>::new(0),
            b: FieldElement::<223, u64>::new(7),
        };

        for pair in [(192, 105), (17, 56), (1, 193)].iter() {
            let x = FieldElement::<223, u64>::new(pair.0);
            let y = FieldElement::<223, u64>::new(pair.1);
            Point::<FieldElement<223, u64>>::new(curve, Coords::Def { x, y });
        }
    }

    #[test]
    fn point_invalid_field_elements() {
        let curve = Curve::<FieldElement<223, u64>> {
            a: FieldElement::<223, u64>::new(0),
            b: FieldElement::<223, u64>::new(7),
        };

        for pair in [(200, 119), (42, 99)].iter() {
            let result = panic::catch_unwind(|| {
                let x = FieldElement::<223, u64>::new(pair.0);
                let y = FieldElement::<223, u64>::new(pair.1);
                Point::<FieldElement<223, u64>>::new(curve, Coords::Def { x, y });
            });
            assert!(result.is_err());
        }
    }

    #[test]
    fn point_sum_field_elements() {
        type F223 = FieldElement<223, u128>;

        let curve = Curve {
            a: F223::new(0),
            b: F223::new(7),
        };
        for tuple in [
            ((170, 142), (60, 139), (220, 181)),
            ((47, 71), (17, 56), (215, 68)),
            ((143, 98), (76, 66), (47, 71)),
        ] {
            assert_eq!(
                Point::<F223>::new(
                    curve,
                    Coords::Def {
                        x: F223::new(tuple.0 .0),
                        y: F223::new(tuple.0 .1),
                    }
                ) + Point::<F223>::new(
                    curve,
                    Coords::Def {
                        x: F223::new(tuple.1 .0),
                        y: F223::new(tuple.1 .1),
                    }
                ),
                Point::<F223>::new(
                    curve,
                    Coords::Def {
                        x: F223::new(tuple.2 .0),
                        y: F223::new(tuple.2 .1),
                    }
                )
            );
        }
    }
}
