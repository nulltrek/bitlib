use crate::fields::{
    add as fadd, div as fdiv, eq as feq, mul as fmul, pow as fpow, sub as fsub, Field, FieldElement,
};
use std::fmt;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Point<T> {
    Inf,
    Coords { x: T, y: T },
}

impl<T> Point<T> {
    pub fn coords(x: T, y: T) -> Self {
        Point::Coords { x, y }
    }
}

impl<T> fmt::Display for Point<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Point::Inf => write!(f, "inf"),
            Point::Coords { x, y } => write!(f, "x: {}, y: {}", x, y),
        }
    }
}

/// Defines a curve in the form:
///    y^2 = x^3 + ax + b
///
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Curve<T, FIELD>
where
    FIELD: Field<T>,
    T: FieldElement,
{
    field: FIELD,
    a: T,
    b: T,
}

impl<T, FIELD> Curve<T, FIELD>
where
    FIELD: Field<T>,
    T: FieldElement,
{
    pub fn new(field: FIELD, a: T, b: T) -> Self {
        return Curve { field, a, b };
    }

    pub fn has_point(&self, point: &Point<T>) -> bool {
        let f = &self.field;
        match point {
            Point::Inf => true,
            Point::Coords { x, y } => feq!(
                f,
                fmul!(f, y, y),
                fadd!(
                    f,
                    fpow!(f, x, FIELD::from_u8(3)),
                    fadd!(f, fmul!(f, x, self.a), self.b)
                )
            ),
        }
    }

    pub fn add(&self, p1: &Point<T>, p2: &Point<T>) -> Point<T> {
        let (p1_x, p1_y) = match p1 {
            Point::Inf => return *p2,
            Point::Coords { x, y } => (x, y),
        };
        let (p2_x, p2_y) = match p2 {
            Point::Inf => return *p1,
            Point::Coords { x, y } => (x, y),
        };

        if p1_x == p2_x && p1_y == p2_y && *p1_y == T::default() {
            return Point::Inf;
        }

        let f = &self.field;

        // Points are equal
        if p1_x == p2_x && p1_y == p2_y {
            let slope = fdiv!(
                f,
                fadd!(
                    f,
                    fmul!(f, FIELD::from_u8(3), fpow!(f, p1_x, FIELD::from_u8(2))),
                    self.a
                ),
                fmul!(f, FIELD::from_u8(2), p1_y)
            );

            let x = fsub!(f, fmul!(f, slope, slope), fmul!(f, FIELD::from_u8(2), p1_x));
            let y = fsub!(f, fmul!(f, slope, fsub!(f, p1_x, x)), p1_y);
            return Point::coords(x, y);
        }

        // Points are mirrored on X axis, forming a vertical line
        if p1_x == p2_x && p1_y != p2_y {
            return Point::Inf;
        }

        // Points are different
        let slope = fdiv!(f, fsub!(f, p2_y, p1_y), fsub!(f, p2_x, p1_x));
        let x = fsub!(f, fsub!(f, fmul!(f, slope, slope), p1_x), p2_x);
        let y = fsub!(f, fmul!(f, slope, fsub!(f, p1_x, x)), p1_y);
        return Point::coords(x, y);
    }

    pub fn mul(&self, scalar: &T, p: &Point<T>) -> Point<T> {
        let zero = FIELD::from_u8(0);
        let one = FIELD::from_u8(1);
        let two = FIELD::from_u8(2);

        let mut coeff = *scalar;
        let mut result = Point::Inf;
        let mut current = *p;
        while coeff > zero {
            if coeff % two == one {
                result = self.add(&result, &current);
            }
            current = self.add(&current, &current);
            coeff = coeff >> one;
        }
        return result;
    }
}

impl<T, FIELD> fmt::Display for Curve<T, FIELD>
where
    FIELD: Field<T>,
    T: FieldElement,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a: {}, b: {} on field: {}", self.a, self.b, self.field)
    }
}

macro_rules! has_point {
    ($curve:expr, $a:expr) => {
        $curve.has_point(&$a)
    };
}
macro_rules! add {
    ($curve:expr, $a:expr, $b:expr) => {
        $curve.add(&$a, &$b)
    };
}
macro_rules! mul {
    ($curve:expr, $a:expr, $b:expr) => {
        $curve.mul(&$a, &$b)
    };
}
pub(crate) use add;
pub(crate) use has_point;
pub(crate) use mul;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::{FiniteField, InfiniteField};

    #[test]
    fn inf_field_point_not_on_curve() {
        let c = Curve::<u64, InfiniteField>::new(InfiniteField, 5, 7);
        assert!(!has_point!(c, Point::coords(5, 7)))
    }

    #[test]
    fn inf_field_point_on_curve() {
        let c = Curve::<i64, InfiniteField>::new(InfiniteField, 5, 7);
        assert!(has_point!(c, Point::Inf));
        assert!(has_point!(c, Point::coords(-1, -1)));

        let c = Curve::<u64, InfiniteField>::new(InfiniteField, 5, 7);
        assert!(has_point!(c, Point::Inf));
        assert!(has_point!(c, Point::coords(18, 77)));
    }

    #[test]
    fn inf_field_point_sum() {
        let c = Curve::<i64, InfiniteField>::new(InfiniteField, 5, 7);
        assert_eq!(add!(c, Point::Inf, Point::Inf), Point::Inf);
        assert_eq!(
            add!(c, Point::Inf, Point::coords(-1, -1)),
            Point::coords(-1, -1)
        );
        assert_eq!(
            add!(c, Point::coords(-1, -1), Point::coords(2, 5)),
            Point::coords(3, -7)
        );

        let c = Curve::<i64, InfiniteField>::new(InfiniteField, 1, 2);
        // Points are equal, tangent to the curve at y = 0
        assert_eq!(
            add!(c, Point::coords(-1, 0), Point::coords(-1, 0)),
            Point::Inf
        );

        let c = Curve::<i64, InfiniteField>::new(InfiniteField, 5, 7);
        // Points are equal
        assert_eq!(
            add!(c, Point::coords(-1, -1), Point::coords(-1, -1)),
            Point::coords(18, 77)
        );

        let c = Curve::<i64, InfiniteField>::new(InfiniteField, 1, 4);
        // Points are mirrored on X axis, forming a vertical line
        assert_eq!(
            add!(c, Point::coords(0, 2), Point::coords(0, -2)),
            Point::Inf
        );

        let c = Curve::<i64, InfiniteField>::new(InfiniteField, 5, 7);
        // Points are different
        assert_eq!(
            add!(c, Point::coords(2, 5), Point::coords(-1, -1)),
            Point::coords(3, -7)
        );
    }

    #[test]
    fn finite_field_point_on_curve() {
        let f = FiniteField::new(223);
        let c = Curve::<u64, FiniteField<u64>>::new(f, 0, 7);

        for pair in [(192, 105), (17, 56), (1, 193)].iter() {
            assert!(has_point!(c, Point::coords(pair.0, pair.1)));
        }
    }

    #[test]
    fn finite_field_point_not_on_curve() {
        let f = FiniteField::new(223);
        let c = Curve::<u64, FiniteField<u64>>::new(f, 0, 7);

        for pair in [(200, 119), (42, 99)].iter() {
            assert!(!has_point!(c, Point::coords(pair.0, pair.1)));
        }
    }

    #[test]
    fn finite_field_point_sum() {
        let f = FiniteField::new(223);
        let c = Curve::<u128, FiniteField<u128>>::new(f, 0, 7);

        for tuple in [
            ((170, 142), (60, 139), (220, 181)),
            ((47, 71), (17, 56), (215, 68)),
            ((143, 98), (76, 66), (47, 71)),
        ] {
            assert_eq!(
                add!(
                    c,
                    Point::coords(tuple.0 .0, tuple.0 .1),
                    Point::coords(tuple.1 .0, tuple.1 .1)
                ),
                Point::coords(tuple.2 .0, tuple.2 .1)
            );
        }
    }

    #[test]
    fn scalar_mul_field_elements() {
        let f = FiniteField::new(223);
        let c = Curve::<u128, FiniteField<u128>>::new(f, 0, 7);

        for tuple in [
            (2, (192, 105), (49, 71)),
            (2, (143, 98), (64, 168)),
            (2, (47, 71), (36, 111)),
            (4, (47, 71), (194, 51)),
            (8, (47, 71), (116, 55)),
        ] {
            assert_eq!(
                mul!(c, tuple.0, Point::coords(tuple.1 .0, tuple.1 .1)),
                Point::coords(tuple.2 .0, tuple.2 .1)
            );
        }
    }
}
