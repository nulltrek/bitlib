use core::ops;
use std::fmt;

/// The point only supports curves in the form:
///    y^2 = x^3 + Ax + B
///
#[derive(PartialEq, Debug)]
pub enum Point<const A: i64, const B: i64> {
    Inf,
    Def { x: i64, y: i64 },
}

impl<const A: i64, const B: i64> Point<A, B> {
    pub fn new(x: i64, y: i64) -> Point<A, B> {
        if y * y != x.pow(3) + A * x + B {
            panic!(
                "The point ({}, {}) is not on the curve ({}, {})",
                x, y, A, B
            )
        }
        return Point::Def { x, y };
    }
}

impl<const A: i64, const B: i64> fmt::Display for Point<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Point::Inf => write!(f, "Point(inf)_{}_{}", A, B),
            Point::Def { x, y } => write!(f, "Point({}, {})_{}_{}", x, y, A, B),
        }
    }
}

impl<const A: i64, const B: i64> ops::Add<Point<A, B>> for Point<A, B> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Point::Inf => other,
            Point::Def {
                x: self_x,
                y: self_y,
            } => match other {
                Point::Inf => self,
                Point::Def {
                    x: other_x,
                    y: other_y,
                } => {
                    // Points are equal, tangent to the curve at y = 0
                    if self_x == other_x && self_y == 0 && other_y == 0 {
                        return Point::Inf;
                    }
                    // Points are equal
                    if self_x == other_x && self_y == other_y {
                        let slope = (3 * self_x * self_x + A) / 2 * self_y;
                        let x = slope * slope - 2 * self_x;
                        let y = slope * (self_x - x) - self_y;
                        return Point::new(x, y);
                    }
                    // Points are mirrored on X axis, forming a vertical line
                    if self_x == other_x && self_y != other_y {
                        return Point::Inf;
                    }

                    // Points are different
                    let slope = (other_y - self_y) / (other_x - self_x);
                    let x = slope * slope - self_x - other_x;
                    let y = slope * (self_x - x) - self_y;
                    return Self::new(x, y);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn point_not_on_curve() {
        Point::<5, 7>::new(5, 7);
    }

    #[test]
    fn point_constructor() {
        Point::<5, 7>::new(-1, -1);
        Point::<5, 7>::new(18, 77);
    }

    #[test]
    fn point_equality() {
        assert_eq!(Point::<0, 7>::Inf, Point::<0, 7>::Inf);
        assert_ne!(Point::<5, 7>::Inf, Point::<5, 7>::new(-1, -1));
        assert_eq!(Point::<5, 7>::new(-1, -1), Point::<5, 7>::new(-1, -1));
        assert_ne!(Point::<5, 7>::new(-1, -1), Point::<5, 7>::new(18, 77));
    }

    #[test]
    fn point_sum() {
        assert_eq!(
            Point::<5, 7>::Inf + Point::<5, 7>::new(-1, -1),
            Point::<5, 7>::new(-1, -1)
        );
        assert_eq!(
            Point::<5, 7>::new(-1, -1) + Point::<5, 7>::new(2, 5),
            Point::<5, 7>::new(3, -7)
        );

        // Points are equal, tangent to the curve at y = 0
        assert_eq!(
            Point::<1, 2>::new(-1, 0) + Point::<1, 2>::new(-1, 0),
            Point::<1, 2>::Inf
        );
        // Points are equal
        assert_eq!(
            Point::<5, 7>::new(-1, -1) + Point::<5, 7>::new(-1, -1),
            Point::<5, 7>::new(18, 77)
        );

        // Points are mirrored on X axis, forming a vertical line
        assert_eq!(
            Point::<1, 4>::new(0, 2) + Point::<1, 4>::new(0, -2),
            Point::<1, 4>::Inf
        );

        // Points are different
        assert_eq!(
            Point::<5, 7>::new(2, 5) + Point::<5, 7>::new(-1, -1),
            Point::<5, 7>::new(3, -7)
        );
    }
}
