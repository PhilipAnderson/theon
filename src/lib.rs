pub mod array;
pub mod ops;
pub mod partition;
pub mod query;
pub mod space;

// Foreign implementation modules. These are empty unless the corresponding
// geometry features are enabled.
mod cgmath;
mod mint;
mod nalgebra;

use std::cmp::Ordering;

use arrayvec::ArrayVec;
use decorum::R64;
use num::{self, Num, NumCast, One, Zero};

pub mod prelude {
    pub use crate::query::Intersection as _;
    pub use crate::Lattice as _;
}

trait Half {
    fn half(self) -> Self;
}

impl<T> Half for T
where
    T: Num + One,
{
    fn half(self) -> Self {
        self / (Self::one() + Self::one())
    }
}

pub trait Composite {
    type Item;
}

impl<T> Composite for (T, T) {
    type Item = T;
}

impl<T> Composite for (T, T, T) {
    type Item = T;
}

pub trait IntoItems: Composite {
    type Output: IntoIterator<Item = Self::Item>;

    fn into_items(self) -> Self::Output;
}

impl<T> IntoItems for (T, T) {
    type Output = ArrayVec<[T; 2]>;

    fn into_items(self) -> Self::Output {
        ArrayVec::from([self.0, self.1])
    }
}

impl<T> IntoItems for (T, T, T) {
    type Output = ArrayVec<[T; 3]>;

    fn into_items(self) -> Self::Output {
        ArrayVec::from([self.0, self.1, self.2])
    }
}

pub trait FromItems: Composite + Sized {
    fn from_items<I>(items: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self::Item>;
}

impl<T> FromItems for (T, T) {
    fn from_items<I>(items: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self::Item>,
    {
        let mut items = items.into_iter().take(2);
        match (items.next(), items.next()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

impl<T> FromItems for (T, T, T) {
    fn from_items<I>(items: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self::Item>,
    {
        let mut items = items.into_iter().take(3);
        match (items.next(), items.next(), items.next()) {
            (Some(a), Some(b), Some(c)) => Some((a, b, c)),
            _ => None,
        }
    }
}

pub trait Converged: Composite {
    fn converged(value: Self::Item) -> Self;
}

impl<T> Converged for (T, T)
where
    T: Clone,
{
    fn converged(value: Self::Item) -> Self {
        (value.clone(), value)
    }
}

impl<T> Converged for (T, T, T)
where
    T: Clone,
{
    fn converged(value: Self::Item) -> Self {
        (value.clone(), value.clone(), value)
    }
}

pub trait Lattice: PartialOrd + Sized {
    fn meet(&self, other: &Self) -> Self;

    fn join(&self, other: &Self) -> Self;

    fn meet_join(&self, other: &Self) -> (Self, Self) {
        (self.meet(other), self.join(other))
    }

    fn partial_min<'a>(&'a self, other: &'a Self) -> Option<&'a Self> {
        match self.partial_cmp(other) {
            Some(Ordering::Greater) => Some(other),
            Some(_) => Some(self),
            None => None,
        }
    }

    fn partial_max<'a>(&'a self, other: &'a Self) -> Option<&'a Self> {
        match self.partial_cmp(other) {
            Some(Ordering::Less) => Some(other),
            Some(_) => Some(self),
            None => None,
        }
    }

    fn partial_ordered_pair<'a>(&'a self, other: &'a Self) -> Option<(&'a Self, &'a Self)> {
        match self.partial_cmp(other) {
            Some(Ordering::Less) => Some((self, other)),
            Some(_) => Some((other, self)),
            None => None,
        }
    }

    fn partial_clamp<'a>(&'a self, min: &'a Self, max: &'a Self) -> Option<&'a Self> {
        let _ = (min, max);
        unimplemented!() // TODO:
    }
}

impl<T> Lattice for T
where
    T: Copy + PartialOrd + Sized,
{
    fn meet(&self, other: &Self) -> Self {
        if *self <= *other {
            *self
        }
        else {
            *other
        }
    }

    fn join(&self, other: &Self) -> Self {
        if *self >= *other {
            *self
        }
        else {
            *other
        }
    }
}

pub trait IteratorExt: Iterator + Sized {
    fn compose<T>(self) -> Option<T>
    where
        T: Composite<Item = Self::Item> + FromItems,
    {
        T::from_items(self)
    }
}

impl<I> IteratorExt for I where I: Iterator + Sized {}

pub fn lerp<T>(a: T, b: T, f: R64) -> T
where
    T: Num + NumCast,
{
    let f = num::clamp(f, Zero::zero(), One::one());
    let af = <R64 as NumCast>::from(a).unwrap() * (R64::one() - f);
    let bf = <R64 as NumCast>::from(b).unwrap() * f;
    <T as NumCast>::from(af + bf).unwrap()
}

fn partial_min<T>(a: T, b: T) -> T
where
    T: Copy + Lattice,
{
    *a.partial_min(&b).unwrap()
}

fn partial_max<T>(a: T, b: T) -> T
where
    T: Copy + Lattice,
{
    *a.partial_max(&b).unwrap()
}
