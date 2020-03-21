// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::UnknownUnit;
use approxeq::ApproxEq;
use approxord::{min, max};
use length::Length;
use scale::Scale;
use size::{Size2D, Size3D};
#[cfg(feature = "mint")]
use mint;
use num::*;
use num_traits::NumCast;
use vector::{Vector2D, Vector3D, vec2, vec3};
use core::fmt;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use core::marker::PhantomData;
use core::cmp::{Eq, PartialEq};
use core::hash::{Hash};
#[cfg(feature = "serde")]
use serde;

/// A 2d Point tagged with a unit.
#[repr(C)]
pub struct Point2D<T, U> {
    pub x: T,
    pub y: T,
    #[doc(hidden)]
    pub _unit: PhantomData<U>,
}

impl<T: Copy, U> Copy for Point2D<T, U> {}

impl<T: Clone, U> Clone for Point2D<T, U> {
    fn clone(&self) -> Self {
        Point2D {
            x: self.x.clone(),
            y: self.y.clone(),
            _unit: PhantomData,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, T, U> serde::Deserialize<'de> for Point2D<T, U>
    where T: serde::Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let (x, y) = serde::Deserialize::deserialize(deserializer)?;
        Ok(Point2D { x, y, _unit: PhantomData })
    }
}

#[cfg(feature = "serde")]
impl<T, U> serde::Serialize for Point2D<T, U>
    where T: serde::Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        (&self.x, &self.y).serialize(serializer)
    }
}

impl<T, U> Eq for Point2D<T, U> where T: Eq {}

impl<T, U> PartialEq for Point2D<T, U>
    where T: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T, U> Hash for Point2D<T, U>
    where T: Hash
{
    fn hash<H: ::core::hash::Hasher>(&self, h: &mut H) {
        self.x.hash(h);
        self.y.hash(h);
    }
}

mint_vec!(Point2D[x, y] = Point2);

impl<T: fmt::Debug, U> fmt::Debug for Point2D<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?},{:?})", self.x, self.y)
    }
}

impl<T: fmt::Display, U> fmt::Display for Point2D<T, U> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "({},{})", self.x, self.y)
    }
}

impl<T: Default, U> Default for Point2D<T, U> {
    fn default() -> Self {
        Point2D::new(Default::default(), Default::default())
    }
}


impl<T, U> Point2D<T, U> {
    /// Constructor, setting all components to zero.
    #[inline]
    pub fn origin() -> Self
    where
        T: Zero,
    {
        point2(Zero::zero(), Zero::zero())
    }

    /// The same as [`origin()`](#method.origin).
    #[inline]
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Self::origin()
    }

    /// Constructor taking scalar values directly.
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Point2D {
            x,
            y,
            _unit: PhantomData,
        }
    }

    /// Constructor taking properly Lengths instead of scalar values.
    #[inline]
    pub fn from_lengths(x: Length<T, U>, y: Length<T, U>) -> Self {
        point2(x.0, y.0)
    }

    /// Tag a unitless value with units.
    #[inline]
    pub fn from_untyped(p: Point2D<T, UnknownUnit>) -> Self {
        point2(p.x, p.y)
    }
}

impl<T: Copy, U> Point2D<T, U> {
    /// Create a 3d point from this one, using the specified z value.
    #[inline]
    pub fn extend(&self, z: T) -> Point3D<T, U> {
        point3(self.x, self.y, z)
    }

    /// Cast this point into a vector.
    ///
    /// Equivalent to subtracting the origin from this point.
    #[inline]
    pub fn to_vector(&self) -> Vector2D<T, U> {
        Vector2D {
            x: self.x,
            y: self.y,
            _unit: PhantomData,
        }
    }

    /// Swap x and y.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::{Point2D, point2};
    /// enum Mm {}
    ///
    /// let point: Point2D<_, Mm> = point2(1, -8);
    ///
    /// assert_eq!(point.yx(), point2(-8, 1));
    /// ```
    #[inline]
    pub fn yx(&self) -> Self {
        point2(self.y, self.x)
    }

    /// Drop the units, preserving only the numeric value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::{Point2D, point2};
    /// enum Mm {}
    ///
    /// let point: Point2D<_, Mm> = point2(1, -8);
    ///
    /// assert_eq!(point.x, point.to_untyped().x);
    /// assert_eq!(point.y, point.to_untyped().y);
    /// ```
    #[inline]
    pub fn to_untyped(&self) -> Point2D<T, UnknownUnit> {
        point2(self.x, self.y)
    }

    /// Cast the unit, preserving the numeric value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::{Point2D, point2};
    /// enum Mm {}
    /// enum Cm {}
    ///
    /// let point: Point2D<_, Mm> = point2(1, -8);
    ///
    /// assert_eq!(point.x, point.cast_unit::<Cm>().x);
    /// assert_eq!(point.y, point.cast_unit::<Cm>().y);
    /// ```
    #[inline]
    pub fn cast_unit<V>(&self) -> Point2D<T, V> {
        point2(self.x, self.y)
    }

    /// Cast into an array with x and y.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::{Point2D, point2};
    /// enum Mm {}
    ///
    /// let point: Point2D<_, Mm> = point2(1, -8);
    ///
    /// assert_eq!(point.to_array(), [1, -8]);
    /// ```
    #[inline]
    pub fn to_array(&self) -> [T; 2] {
        [self.x, self.y]
    }

    /// Cast into a tuple with x and y.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::{Point2D, point2};
    /// enum Mm {}
    ///
    /// let point: Point2D<_, Mm> = point2(1, -8);
    ///
    /// assert_eq!(point.to_tuple(), (1, -8));
    /// ```
    #[inline]
    pub fn to_tuple(&self) -> (T, T) {
        (self.x, self.y)
    }

    /// Convert into a 3d point with z-coordinate equals to zero.
    #[inline]
    pub fn to_3d(&self) -> Point3D<T, U>
    where
        T: Zero,
    {
        point3(self.x, self.y, Zero::zero())
    }

    /// Linearly interpolate between this point and another point.
    ///
    /// When `t` is `One::one()`, returned value equals to `other`,
    /// otherwise equals to `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use euclid::point2;
    /// use euclid::default::Point2D;
    ///
    /// let first: Point2D<_> = point2(0.0, 10.0);
    /// let last:  Point2D<_> = point2(8.0, -4.0);
    ///
    /// assert_eq!(first.lerp(last, -1.0), point2(-8.0,  24.0));
    /// assert_eq!(first.lerp(last,  0.0), point2( 0.0,  10.0));
    /// assert_eq!(first.lerp(last,  0.5), point2( 4.0,   3.0));
    /// assert_eq!(first.lerp(last,  1.0), point2( 8.0,  -4.0));
    /// assert_eq!(first.lerp(last,  2.0), point2(16.0, -18.0));
    /// ```
    #[inline]
    pub fn lerp(&self, other: Self, t: T) -> Self
    where
        T: One + Sub<Output = T> + Mul<Output = T> + Add<Output = T>,
    {
        let one_t = T::one() - t;
        point2(
            one_t * self.x + t * other.x,
            one_t * self.y + t * other.y,
        )
    }
}

impl<T: PartialOrd, U> Point2D<T, U> {
    #[inline]
    pub fn min(self, other: Self) -> Self {
        point2(min(self.x, other.x), min(self.y, other.y))
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        point2(max(self.x, other.x), max(self.y, other.y))
    }

    /// Returns the point each component of which clamped by corresponding
    /// components of `start` and `end`.
    ///
    /// Shortcut for `self.max(start).min(end)`.
    #[inline]
    pub fn clamp(&self, start: Self, end: Self) -> Self
    where
        T: Copy,
    {
        self.max(start).min(end)
    }
}

impl<T: Copy + Add<T, Output = T>, U> Point2D<T, U> {
    #[inline]
    pub fn add_size(&self, other: &Size2D<T, U>) -> Self {
        point2(self.x + other.width, self.y + other.height)
    }
}

impl<T: Add<T, Output = T>, U> Add<Size2D<T, U>> for Point2D<T, U> {
    type Output = Self;
    #[inline]
    fn add(self, other: Size2D<T, U>) -> Self {
        point2(self.x + other.width, self.y + other.height)
    }
}

impl<T: Copy + Add<T, Output = T>, U> AddAssign<Vector2D<T, U>> for Point2D<T, U> {
    #[inline]
    fn add_assign(&mut self, other: Vector2D<T, U>) {
        *self = *self + other
    }
}

impl<T: Copy + Sub<T, Output = T>, U> SubAssign<Vector2D<T, U>> for Point2D<T, U> {
    #[inline]
    fn sub_assign(&mut self, other: Vector2D<T, U>) {
        *self = *self - other
    }
}

impl<T: Add<T, Output = T>, U> Add<Vector2D<T, U>> for Point2D<T, U> {
    type Output = Self;
    #[inline]
    fn add(self, other: Vector2D<T, U>) -> Self {
        point2(self.x + other.x, self.y + other.y)
    }
}

impl<T: Sub<T, Output = T>, U> Sub for Point2D<T, U> {
    type Output = Vector2D<T, U>;
    #[inline]
    fn sub(self, other: Self) -> Vector2D<T, U> {
        vec2(self.x - other.x, self.y - other.y)
    }
}

impl<T: Sub<T, Output = T>, U> Sub<Vector2D<T, U>> for Point2D<T, U> {
    type Output = Self;
    #[inline]
    fn sub(self, other: Vector2D<T, U>) -> Self {
        point2(self.x - other.x, self.y - other.y)
    }
}

impl<T: Copy + Mul<T, Output = T>, U> Mul<T> for Point2D<T, U> {
    type Output = Self;
    #[inline]
    fn mul(self, scale: T) -> Self {
        point2(self.x * scale, self.y * scale)
    }
}

impl<T: Copy + Mul<T, Output = T>, U> MulAssign<T> for Point2D<T, U> {
    #[inline]
    fn mul_assign(&mut self, scale: T) {
        *self = *self * scale
    }
}

impl<T: Copy + Div<T, Output = T>, U> Div<T> for Point2D<T, U> {
    type Output = Self;
    #[inline]
    fn div(self, scale: T) -> Self {
        point2(self.x / scale, self.y / scale)
    }
}

impl<T: Copy + Div<T, Output = T>, U> DivAssign<T> for Point2D<T, U> {
    #[inline]
    fn div_assign(&mut self, scale: T) {
        *self = *self / scale
    }
}

impl<T: Copy + Mul<T, Output = T>, U1, U2> Mul<Scale<T, U1, U2>> for Point2D<T, U1> {
    type Output = Point2D<T, U2>;
    #[inline]
    fn mul(self, scale: Scale<T, U1, U2>) -> Point2D<T, U2> {
        point2(self.x * scale.get(), self.y * scale.get())
    }
}

impl<T: Copy + Div<T, Output = T>, U1, U2> Div<Scale<T, U1, U2>> for Point2D<T, U2> {
    type Output = Point2D<T, U1>;
    #[inline]
    fn div(self, scale: Scale<T, U1, U2>) -> Point2D<T, U1> {
        point2(self.x / scale.get(), self.y / scale.get())
    }
}

impl<T: Round, U> Point2D<T, U> {
    /// Rounds each component to the nearest integer value.
    ///
    /// This behavior is preserved for negative values (unlike the basic cast).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::point2;
    /// enum Mm {}
    ///
    /// assert_eq!(point2::<_, Mm>(-0.1, -0.8).round(), point2::<_, Mm>(0.0, -1.0))
    /// ```
    #[inline]
    #[must_use]
    pub fn round(&self) -> Self {
        point2(self.x.round(), self.y.round())
    }
}

impl<T: Ceil, U> Point2D<T, U> {
    /// Rounds each component to the smallest integer equal or greater than the original value.
    ///
    /// This behavior is preserved for negative values (unlike the basic cast).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::point2;
    /// enum Mm {}
    ///
    /// assert_eq!(point2::<_, Mm>(-0.1, -0.8).ceil(), point2::<_, Mm>(0.0, 0.0))
    /// ```
    #[inline]
    #[must_use]
    pub fn ceil(&self) -> Self {
        point2(self.x.ceil(), self.y.ceil())
    }
}

impl<T: Floor, U> Point2D<T, U> {
    /// Rounds each component to the biggest integer equal or lower than the original value.
    ///
    /// This behavior is preserved for negative values (unlike the basic cast).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::point2;
    /// enum Mm {}
    ///
    /// assert_eq!(point2::<_, Mm>(-0.1, -0.8).floor(), point2::<_, Mm>(-1.0, -1.0))
    /// ```
    #[inline]
    #[must_use]
    pub fn floor(&self) -> Self {
        point2(self.x.floor(), self.y.floor())
    }
}

impl<T: NumCast + Copy, U> Point2D<T, U> {
    /// Cast from one numeric representation to another, preserving the units.
    ///
    /// When casting from floating point to integer coordinates, the decimals are truncated
    /// as one would expect from a simple cast, but this behavior does not always make sense
    /// geometrically. Consider using `round()`, `ceil()` or `floor()` before casting.
    #[inline]
    pub fn cast<NewT: NumCast>(&self) -> Point2D<NewT, U> {
        self.try_cast().unwrap()
    }

    /// Fallible cast from one numeric representation to another, preserving the units.
    ///
    /// When casting from floating point to integer coordinates, the decimals are truncated
    /// as one would expect from a simple cast, but this behavior does not always make sense
    /// geometrically. Consider using `round()`, `ceil()` or `floor()` before casting.
    pub fn try_cast<NewT: NumCast>(&self) -> Option<Point2D<NewT, U>> {
        match (NumCast::from(self.x), NumCast::from(self.y)) {
            (Some(x), Some(y)) => Some(point2(x, y)),
            _ => None,
        }
    }

    // Convenience functions for common casts

    /// Cast into an `f32` point.
    #[inline]
    pub fn to_f32(&self) -> Point2D<f32, U> {
        self.cast()
    }

    /// Cast into an `f64` point.
    #[inline]
    pub fn to_f64(&self) -> Point2D<f64, U> {
        self.cast()
    }

    /// Cast into an `usize` point, truncating decimals if any.
    ///
    /// When casting from floating point points, it is worth considering whether
    /// to `round()`, `ceil()` or `floor()` before the cast in order to obtain
    /// the desired conversion behavior.
    #[inline]
    pub fn to_usize(&self) -> Point2D<usize, U> {
        self.cast()
    }

    /// Cast into an `u32` point, truncating decimals if any.
    ///
    /// When casting from floating point points, it is worth considering whether
    /// to `round()`, `ceil()` or `floor()` before the cast in order to obtain
    /// the desired conversion behavior.
    #[inline]
    pub fn to_u32(&self) -> Point2D<u32, U> {
        self.cast()
    }

    /// Cast into an i32 point, truncating decimals if any.
    ///
    /// When casting from floating point points, it is worth considering whether
    /// to `round()`, `ceil()` or `floor()` before the cast in order to obtain
    /// the desired conversion behavior.
    #[inline]
    pub fn to_i32(&self) -> Point2D<i32, U> {
        self.cast()
    }

    /// Cast into an i64 point, truncating decimals if any.
    ///
    /// When casting from floating point points, it is worth considering whether
    /// to `round()`, `ceil()` or `floor()` before the cast in order to obtain
    /// the desired conversion behavior.
    #[inline]
    pub fn to_i64(&self) -> Point2D<i64, U> {
        self.cast()
    }
}

impl<T: ApproxEq<T>, U> ApproxEq<Point2D<T, U>> for Point2D<T, U> {
    #[inline]
    fn approx_epsilon() -> Self {
        point2(T::approx_epsilon(), T::approx_epsilon())
    }

    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        self.x.approx_eq(&other.x) && self.y.approx_eq(&other.y)
    }

    #[inline]
    fn approx_eq_eps(&self, other: &Self, eps: &Self) -> bool {
        self.x.approx_eq_eps(&other.x, &eps.x) && self.y.approx_eq_eps(&other.y, &eps.y)
    }
}

impl<T, U> Into<[T; 2]> for Point2D<T, U> {
    fn into(self) -> [T; 2] {
        [self.x, self.y]
    }
}

impl<T, U> From<[T; 2]> for Point2D<T, U> {
    fn from([x, y]: [T; 2]) -> Self {
        point2(x, y)
    }
}

impl<T, U> Into<(T, T)> for Point2D<T, U> {
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T, U> From<(T, T)> for Point2D<T, U> {
    fn from(tuple: (T, T)) -> Self {
        point2(tuple.0, tuple.1)
    }
}



/// A 3d Point tagged with a unit.
#[repr(C)]
pub struct Point3D<T, U> {
    pub x: T,
    pub y: T,
    pub z: T,
    #[doc(hidden)]
    pub _unit: PhantomData<U>,
}

mint_vec!(Point3D[x, y, z] = Point3);

impl<T: Copy, U> Copy for Point3D<T, U> {}

impl<T: Clone, U> Clone for Point3D<T, U> {
    fn clone(&self) -> Self {
        Point3D {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone(),
            _unit: PhantomData,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, T, U> serde::Deserialize<'de> for Point3D<T, U>
    where T: serde::Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let (x, y, z) = serde::Deserialize::deserialize(deserializer)?;
        Ok(Point3D { x, y, z, _unit: PhantomData })
    }
}

#[cfg(feature = "serde")]
impl<T, U> serde::Serialize for Point3D<T, U>
    where T: serde::Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        (&self.x, &self.y, &self.z).serialize(serializer)
    }
}

impl<T, U> Eq for Point3D<T, U> where T: Eq {}

impl<T, U> PartialEq for Point3D<T, U>
    where T: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl<T, U> Hash for Point3D<T, U>
    where T: Hash
{
    fn hash<H: ::core::hash::Hasher>(&self, h: &mut H) {
        self.x.hash(h);
        self.y.hash(h);
        self.z.hash(h);
    }
}

impl<T: fmt::Debug, U> fmt::Debug for Point3D<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?},{:?},{:?})", self.x, self.y, self.z)
    }
}

impl<T: fmt::Display, U> fmt::Display for Point3D<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{})", self.x, self.y, self.z)
    }
}

impl<T: Default, U> Default for Point3D<T, U> {
    fn default() -> Self {
        Point3D::new(Default::default(), Default::default(), Default::default())
    }
}


impl<T, U> Point3D<T, U> {
    /// Constructor, setting all components to zero.
    #[inline]
    pub fn origin() -> Self
    where
        T: Zero,
    {
        point3(Zero::zero(), Zero::zero(), Zero::zero())
    }

    /// The same as [`origin()`](#method.origin).
    #[inline]
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Self::origin()
    }

    /// Constructor taking scalar values directly.
    #[inline]
    pub const fn new(x: T, y: T, z: T) -> Self {
        Point3D {
            x,
            y,
            z,
            _unit: PhantomData,
        }
    }

    /// Constructor taking properly Lengths instead of scalar values.
    #[inline]
    pub fn from_lengths(x: Length<T, U>, y: Length<T, U>, z: Length<T, U>) -> Self {
        point3(x.0, y.0, z.0)
    }

    /// Tag a unitless value with units.
    #[inline]
    pub fn from_untyped(p: Point3D<T, UnknownUnit>) -> Self {
        point3(p.x, p.y, p.z)
    }
}

impl<T: Copy, U> Point3D<T, U> {
    /// Cast this point into a vector.
    ///
    /// Equivalent to subtracting the origin to this point.
    #[inline]
    pub fn to_vector(&self) -> Vector3D<T, U> {
        Vector3D {
            x: self.x,
            y: self.y,
            z: self.z,
            _unit: PhantomData,
        }
    }

    /// Returns a 2d point using this point's x and y coordinates
    #[inline]
    pub fn xy(&self) -> Point2D<T, U> {
        point2(self.x, self.y)
    }

    /// Returns a 2d point using this point's x and z coordinates
    #[inline]
    pub fn xz(&self) -> Point2D<T, U> {
        point2(self.x, self.z)
    }

    /// Returns a 2d point using this point's x and z coordinates
    #[inline]
    pub fn yz(&self) -> Point2D<T, U> {
        point2(self.y, self.z)
    }

    /// Cast into an array with x, y and z.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::{Point3D, point3};
    /// enum Mm {}
    ///
    /// let point: Point3D<_, Mm> = point3(1, -8, 0);
    ///
    /// assert_eq!(point.to_array(), [1, -8, 0]);
    /// ```
    #[inline]
    pub fn to_array(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    #[inline]
    pub fn to_array_4d(&self) -> [T; 4]
    where
        T: One,
    {
        [self.x, self.y, self.z, One::one()]
    }

    /// Cast into a tuple with x, y and z.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::{Point3D, point3};
    /// enum Mm {}
    ///
    /// let point: Point3D<_, Mm> = point3(1, -8, 0);
    ///
    /// assert_eq!(point.to_tuple(), (1, -8, 0));
    /// ```
    #[inline]
    pub fn to_tuple(&self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }

    #[inline]
    pub fn to_tuple_4d(&self) -> (T, T, T, T)
    where
        T: One,
    {
        (self.x, self.y, self.z, One::one())
    }

    /// Drop the units, preserving only the numeric value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::{Point3D, point3};
    /// enum Mm {}
    ///
    /// let point: Point3D<_, Mm> = point3(1, -8, 0);
    ///
    /// assert_eq!(point.x, point.to_untyped().x);
    /// assert_eq!(point.y, point.to_untyped().y);
    /// assert_eq!(point.z, point.to_untyped().z);
    /// ```
    #[inline]
    pub fn to_untyped(&self) -> Point3D<T, UnknownUnit> {
        point3(self.x, self.y, self.z)
    }

    /// Cast the unit, preserving the numeric value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::{Point3D, point3};
    /// enum Mm {}
    /// enum Cm {}
    ///
    /// let point: Point3D<_, Mm> = point3(1, -8, 0);
    ///
    /// assert_eq!(point.x, point.cast_unit::<Cm>().x);
    /// assert_eq!(point.y, point.cast_unit::<Cm>().y);
    /// assert_eq!(point.z, point.cast_unit::<Cm>().z);
    /// ```
    #[inline]
    pub fn cast_unit<V>(&self) -> Point3D<T, V> {
        point3(self.x, self.y, self.z)
    }

    /// Convert into a 2d point.
    #[inline]
    pub fn to_2d(&self) -> Point2D<T, U> {
        self.xy()
    }

    /// Linearly interpolate between this point and another point.
    ///
    /// When `t` is `One::one()`, returned value equals to `other`,
    /// otherwise equals to `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use euclid::point3;
    /// use euclid::default::Point3D;
    ///
    /// let first: Point3D<_> = point3(0.0, 10.0, -1.0);
    /// let last:  Point3D<_> = point3(8.0, -4.0,  0.0);
    ///
    /// assert_eq!(first.lerp(last, -1.0), point3(-8.0,  24.0, -2.0));
    /// assert_eq!(first.lerp(last,  0.0), point3( 0.0,  10.0, -1.0));
    /// assert_eq!(first.lerp(last,  0.5), point3( 4.0,   3.0, -0.5));
    /// assert_eq!(first.lerp(last,  1.0), point3( 8.0,  -4.0,  0.0));
    /// assert_eq!(first.lerp(last,  2.0), point3(16.0, -18.0,  1.0));
    /// ```
    #[inline]
    pub fn lerp(&self, other: Self, t: T) -> Self
    where
        T: One + Sub<Output = T> + Mul<Output = T> + Add<Output = T>,
    {
        let one_t = T::one() - t;
        point3(
            one_t * self.x + t * other.x,
            one_t * self.y + t * other.y,
            one_t * self.z + t * other.z,
        )
    }
}

impl<T: PartialOrd, U> Point3D<T, U> {
    #[inline]
    pub fn min(self, other: Self) -> Self {
        point3(
            min(self.x, other.x),
            min(self.y, other.y),
            min(self.z, other.z),
        )
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        point3(
            max(self.x, other.x),
            max(self.y, other.y),
            max(self.z, other.z),
        )
    }

    /// Returns the point each component of which clamped by corresponding
    /// components of `start` and `end`.
    ///
    /// Shortcut for `self.max(start).min(end)`.
    #[inline]
    pub fn clamp(&self, start: Self, end: Self) -> Self
    where
        T: Copy,
    {
        self.max(start).min(end)
    }
}

impl<T: Copy + Add<T, Output = T>, U> Point3D<T, U> {
    #[inline]
    pub fn add_size(&self, other: &Size3D<T, U>) -> Self {
        point3(self.x + other.width, self.y + other.height, self.z + other.depth)
    }
}

impl<T: Copy + Add<T, Output = T>, U> AddAssign<Vector3D<T, U>> for Point3D<T, U> {
    #[inline]
    fn add_assign(&mut self, other: Vector3D<T, U>) {
        *self = *self + other
    }
}

impl<T: Copy + Sub<T, Output = T>, U> SubAssign<Vector3D<T, U>> for Point3D<T, U> {
    #[inline]
    fn sub_assign(&mut self, other: Vector3D<T, U>) {
        *self = *self - other
    }
}

impl<T: Add<T, Output = T>, U> Add<Vector3D<T, U>> for Point3D<T, U> {
    type Output = Self;
    #[inline]
    fn add(self, other: Vector3D<T, U>) -> Self {
        point3(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<T: Sub<T, Output = T>, U> Sub for Point3D<T, U> {
    type Output = Vector3D<T, U>;
    #[inline]
    fn sub(self, other: Self) -> Vector3D<T, U> {
        vec3(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<T: Sub<T, Output = T>, U> Sub<Vector3D<T, U>> for Point3D<T, U> {
    type Output = Self;
    #[inline]
    fn sub(self, other: Vector3D<T, U>) -> Self {
        point3(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<T: Copy + Mul<T, Output = T>, U> Mul<T> for Point3D<T, U> {
    type Output = Self;
    #[inline]
    fn mul(self, scale: T) -> Self {
        point3(self.x * scale, self.y * scale, self.z * scale)
    }
}

impl<T: Copy + Mul<T, Output = T>, U1, U2> Mul<Scale<T, U1, U2>> for Point3D<T, U1> {
    type Output = Point3D<T, U2>;
    #[inline]
    fn mul(self, scale: Scale<T, U1, U2>) -> Point3D<T, U2> {
        point3(self.x * scale.get(), self.y * scale.get(), self.z * scale.get())
    }
}

impl<T: Copy + Div<T, Output = T>, U> Div<T> for Point3D<T, U> {
    type Output = Self;
    #[inline]
    fn div(self, scale: T) -> Self {
        point3(self.x / scale, self.y / scale, self.z / scale)
    }
}

impl<T: Copy + Div<T, Output = T>, U1, U2> Div<Scale<T, U1, U2>> for Point3D<T, U2> {
    type Output = Point3D<T, U1>;
    #[inline]
    fn div(self, scale: Scale<T, U1, U2>) -> Point3D<T, U1> {
        point3(self.x / scale.get(), self.y / scale.get(), self.z / scale.get())
    }
}

impl<T: Round, U> Point3D<T, U> {
    /// Rounds each component to the nearest integer value.
    ///
    /// This behavior is preserved for negative values (unlike the basic cast).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::point3;
    /// enum Mm {}
    ///
    /// assert_eq!(point3::<_, Mm>(-0.1, -0.8, 0.4).round(), point3::<_, Mm>(0.0, -1.0, 0.0))
    /// ```
    #[inline]
    #[must_use]
    pub fn round(&self) -> Self {
        point3(self.x.round(), self.y.round(), self.z.round())
    }
}

impl<T: Ceil, U> Point3D<T, U> {
    /// Rounds each component to the smallest integer equal or greater than the original value.
    ///
    /// This behavior is preserved for negative values (unlike the basic cast).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::point3;
    /// enum Mm {}
    ///
    /// assert_eq!(point3::<_, Mm>(-0.1, -0.8, 0.4).ceil(), point3::<_, Mm>(0.0, 0.0, 1.0))
    /// ```
    #[inline]
    #[must_use]
    pub fn ceil(&self) -> Self {
        point3(self.x.ceil(), self.y.ceil(), self.z.ceil())
    }
}

impl<T: Floor, U> Point3D<T, U> {
    /// Rounds each component to the biggest integer equal or lower than the original value.
    ///
    /// This behavior is preserved for negative values (unlike the basic cast).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use euclid::point3;
    /// enum Mm {}
    ///
    /// assert_eq!(point3::<_, Mm>(-0.1, -0.8, 0.4).floor(), point3::<_, Mm>(-1.0, -1.0, 0.0))
    /// ```
    #[inline]
    #[must_use]
    pub fn floor(&self) -> Self {
        point3(self.x.floor(), self.y.floor(), self.z.floor())
    }
}

impl<T: NumCast + Copy, U> Point3D<T, U> {
    /// Cast from one numeric representation to another, preserving the units.
    ///
    /// When casting from floating point to integer coordinates, the decimals are truncated
    /// as one would expect from a simple cast, but this behavior does not always make sense
    /// geometrically. Consider using `round()`, `ceil()` or `floor()` before casting.
    #[inline]
    pub fn cast<NewT: NumCast>(&self) -> Point3D<NewT, U> {
        self.try_cast().unwrap()
    }

    /// Fallible cast from one numeric representation to another, preserving the units.
    ///
    /// When casting from floating point to integer coordinates, the decimals are truncated
    /// as one would expect from a simple cast, but this behavior does not always make sense
    /// geometrically. Consider using `round()`, `ceil()` or `floor()` before casting.
    pub fn try_cast<NewT: NumCast>(&self) -> Option<Point3D<NewT, U>> {
        match (
            NumCast::from(self.x),
            NumCast::from(self.y),
            NumCast::from(self.z),
        ) {
            (Some(x), Some(y), Some(z)) => Some(point3(x, y, z)),
            _ => None,
        }
    }

    // Convenience functions for common casts

    /// Cast into an `f32` point.
    #[inline]
    pub fn to_f32(&self) -> Point3D<f32, U> {
        self.cast()
    }

    /// Cast into an `f64` point.
    #[inline]
    pub fn to_f64(&self) -> Point3D<f64, U> {
        self.cast()
    }

    /// Cast into an `usize` point, truncating decimals if any.
    ///
    /// When casting from floating point points, it is worth considering whether
    /// to `round()`, `ceil()` or `floor()` before the cast in order to obtain
    /// the desired conversion behavior.
    #[inline]
    pub fn to_usize(&self) -> Point3D<usize, U> {
        self.cast()
    }

    /// Cast into an `u32` point, truncating decimals if any.
    ///
    /// When casting from floating point points, it is worth considering whether
    /// to `round()`, `ceil()` or `floor()` before the cast in order to obtain
    /// the desired conversion behavior.
    #[inline]
    pub fn to_u32(&self) -> Point3D<u32, U> {
        self.cast()
    }

    /// Cast into an `i32` point, truncating decimals if any.
    ///
    /// When casting from floating point points, it is worth considering whether
    /// to `round()`, `ceil()` or `floor()` before the cast in order to obtain
    /// the desired conversion behavior.
    #[inline]
    pub fn to_i32(&self) -> Point3D<i32, U> {
        self.cast()
    }

    /// Cast into an `i64` point, truncating decimals if any.
    ///
    /// When casting from floating point points, it is worth considering whether
    /// to `round()`, `ceil()` or `floor()` before the cast in order to obtain
    /// the desired conversion behavior.
    #[inline]
    pub fn to_i64(&self) -> Point3D<i64, U> {
        self.cast()
    }
}

impl<T: ApproxEq<T>, U> ApproxEq<Point3D<T, U>> for Point3D<T, U> {
    #[inline]
    fn approx_epsilon() -> Self {
        point3(
            T::approx_epsilon(),
            T::approx_epsilon(),
            T::approx_epsilon(),
        )
    }

    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        self.x.approx_eq(&other.x) && self.y.approx_eq(&other.y) && self.z.approx_eq(&other.z)
    }

    #[inline]
    fn approx_eq_eps(&self, other: &Self, eps: &Self) -> bool {
        self.x.approx_eq_eps(&other.x, &eps.x) && self.y.approx_eq_eps(&other.y, &eps.y)
            && self.z.approx_eq_eps(&other.z, &eps.z)
    }
}

impl<T, U> Into<[T; 3]> for Point3D<T, U> {
    fn into(self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
}

impl<T, U> From<[T; 3]> for Point3D<T, U> {
    fn from([x, y, z]: [T; 3]) -> Self {
        point3(x, y, z)
    }
}

impl<T, U> Into<(T, T, T)> for Point3D<T, U> {
    fn into(self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }
}

impl<T, U> From<(T, T, T)> for Point3D<T, U> {
    fn from(tuple: (T, T, T)) -> Self {
        point3(tuple.0, tuple.1, tuple.2)
    }
}

/// Shorthand for `Point2D::new(x, y)`.
#[inline]
pub const fn point2<T, U>(x: T, y: T) -> Point2D<T, U> {
    Point2D {
        x,
        y,
        _unit: PhantomData,
    }
}

/// Shorthand for `Point3D::new(x, y)`.
#[inline]
pub const fn point3<T, U>(x: T, y: T, z: T) -> Point3D<T, U> {
    Point3D {
        x,
        y,
        z,
        _unit: PhantomData,
    }
}


#[cfg(test)]
mod point2d {
    use default::Point2D;
    use {point2, vec2};
    use scale::Scale;

    #[cfg(feature = "mint")]
    use mint;

    #[test]
    pub fn test_scalar_mul() {
        let p1: Point2D<f32> = Point2D::new(3.0, 5.0);

        let result = p1 * 5.0;

        assert_eq!(result, Point2D::new(15.0, 25.0));
    }

    #[test]
    pub fn test_min() {
        let p1 = Point2D::new(1.0, 3.0);
        let p2 = Point2D::new(2.0, 2.0);

        let result = p1.min(p2);

        assert_eq!(result, Point2D::new(1.0, 2.0));
    }

    #[test]
    pub fn test_max() {
        let p1 = Point2D::new(1.0, 3.0);
        let p2 = Point2D::new(2.0, 2.0);

        let result = p1.max(p2);

        assert_eq!(result, Point2D::new(2.0, 3.0));
    }

    #[cfg(feature = "mint")]
    #[test]
    pub fn test_mint() {
        let p1 = Point2D::new(1.0, 3.0);
        let pm: mint::Point2<_> = p1.into();
        let p2 = Point2D::from(pm);

        assert_eq!(p1, p2);
    }

    pub enum Mm {}
    pub enum Cm {}

    pub type Point2DMm<T> = super::Point2D<T, Mm>;
    pub type Point2DCm<T> = super::Point2D<T, Cm>;

    #[test]
    pub fn test_add() {
        let p1 = Point2DMm::new(1.0, 2.0);
        let p2 = vec2(3.0, 4.0);

        let result = p1 + p2;

        assert_eq!(result, Point2DMm::new(4.0, 6.0));
    }

    #[test]
    pub fn test_add_assign() {
        let mut p1 = Point2DMm::new(1.0, 2.0);
        p1 += vec2(3.0, 4.0);

        assert_eq!(p1, Point2DMm::new(4.0, 6.0));
    }

    #[test]
    pub fn test_typed_scalar_mul() {
        let p1 = Point2DMm::new(1.0, 2.0);
        let cm_per_mm: Scale<f32, Mm, Cm> = Scale::new(0.1);

        let result = p1 * cm_per_mm;

        assert_eq!(result, Point2DCm::new(0.1, 0.2));
    }

    #[test]
    pub fn test_conv_vector() {
        for i in 0..100 {
            // We don't care about these values as long as they are not the same.
            let x = i as f32 * 0.012345;
            let y = i as f32 * 0.987654;
            let p: Point2D<f32> = point2(x, y);
            assert_eq!(p.to_vector().to_point(), p);
        }
    }

    #[test]
    pub fn test_swizzling() {
        let p: Point2D<i32> = point2(1, 2);
        assert_eq!(p.yx(), point2(2, 1));
    }
}

#[cfg(test)]
mod point3d {
    use default;
    use default::Point3D;
    use {point2, point3};
    #[cfg(feature = "mint")]
    use mint;

    #[test]
    pub fn test_min() {
        let p1 = Point3D::new(1.0, 3.0, 5.0);
        let p2 = Point3D::new(2.0, 2.0, -1.0);

        let result = p1.min(p2);

        assert_eq!(result, Point3D::new(1.0, 2.0, -1.0));
    }

    #[test]
    pub fn test_max() {
        let p1 = Point3D::new(1.0, 3.0, 5.0);
        let p2 = Point3D::new(2.0, 2.0, -1.0);

        let result = p1.max(p2);

        assert_eq!(result, Point3D::new(2.0, 3.0, 5.0));
    }

    #[test]
    pub fn test_conv_vector() {
        use point3;
        for i in 0..100 {
            // We don't care about these values as long as they are not the same.
            let x = i as f32 * 0.012345;
            let y = i as f32 * 0.987654;
            let z = x * y;
            let p: Point3D<f32> = point3(x, y, z);
            assert_eq!(p.to_vector().to_point(), p);
        }
    }

    #[test]
    pub fn test_swizzling() {
        let p: default::Point3D<i32> = point3(1, 2, 3);
        assert_eq!(p.xy(), point2(1, 2));
        assert_eq!(p.xz(), point2(1, 3));
        assert_eq!(p.yz(), point2(2, 3));
    }

    #[cfg(feature = "mint")]
    #[test]
    pub fn test_mint() {
        let p1 = Point3D::new(1.0, 3.0, 5.0);
        let pm: mint::Point3<_> = p1.into();
        let p2 = Point3D::from(pm);

        assert_eq!(p1, p2);
    }
}
