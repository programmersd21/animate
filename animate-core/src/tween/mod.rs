mod alternate;
mod cycle;
mod once;

pub mod easing;

use crate::{Animate, Mode, Once};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Tween<T, E, I, M = Once>
where
    M: Mode,
    T: TweenAnim + PartialEq,
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
{
    pub(crate) state: TweenState<T, E, I>,
    _mode: PhantomData<M>,
}

impl<T, E, I, M> Tween<T, E, I, M>
where
    M: Mode,
    T: TweenAnim + PartialEq + Default,
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
{
    pub fn new(initial: T, duration: f64, easing: E, interp: I) -> Self {
        Self {
            state: TweenState::new(initial, duration, easing, interp),
            _mode: PhantomData,
        }
    }
}


pub trait TweenAnim {
    fn tween(start: &Self, end: &Self, t: f64) -> Self;
}

#[derive(Debug)]
pub(crate) struct TweenState<T, E, I>
where
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
{
    pub current: T,
    pub start: T,
    pub target: T,
    pub started_at: Option<usize>,
    pub pending: bool,
    pub duration: f64,
    pub easing: E,
    pub interp: I,
}

impl<T: Default, E, I> TweenState<T, E, I>
where
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
{
    pub fn new(initial: T, duration: f64, easing: E, interp: I) -> Self {
        Self {
            current: initial,
            start: Default::default(),
            target: Default::default(),
            started_at: None,
            pending: false,
            duration: duration.max(f64::MIN_POSITIVE),
            easing,
            interp,
        }
    }
}

impl<T, E, I, M> std::ops::Deref for Tween<T, E, I, M>
where
    M: Mode,
    T: TweenAnim + PartialEq + Default,
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
    Self: Animate<Value = T>,
{
    type Target = T;
    fn deref(&self) -> &T {
        Animate::get(self)
    }
}

impl<T, E, I, M> std::fmt::Display for Tween<T, E, I, M>
where
    M: Mode,
    T: TweenAnim + PartialEq + Default + std::fmt::Display,
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
    Self: Animate<Value = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Animate::get(self).fmt(f)
    }
}

impl<T, E, I, M> std::ops::AddAssign<T> for Tween<T, E, I, M>
where
    M: Mode,
    T: TweenAnim + PartialEq + Default,
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
    for<'b> &'b T: std::ops::Add<T, Output = T>,
    Self: Animate<Value = T>,
{
    fn add_assign(&mut self, rhs: T) {
        let v = Animate::target(self) + rhs;
        Animate::set(self, v);
    }
}

impl<T, E, I, M> std::ops::SubAssign<T> for Tween<T, E, I, M>
where
    M: Mode,
    T: TweenAnim + PartialEq + Default,
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
    for<'b> &'b T: std::ops::Sub<T, Output = T>,
    Self: Animate<Value = T>,
{
    fn sub_assign(&mut self, rhs: T) {
        let v = Animate::target(self) - rhs;
        Animate::set(self, v);
    }
}

impl<T, E, I, M> std::ops::MulAssign<T> for Tween<T, E, I, M>
where
    M: Mode,
    T: TweenAnim + PartialEq + Default,
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
    for<'b> &'b T: std::ops::Mul<T, Output = T>,
    Self: Animate<Value = T>,
{
    fn mul_assign(&mut self, rhs: T) {
        let v = Animate::target(self) * rhs;
        Animate::set(self, v);
    }
}

impl<T, E, I, M> std::ops::DivAssign<T> for Tween<T, E, I, M>
where
    M: Mode,
    T: TweenAnim + PartialEq + Default,
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
    for<'b> &'b T: std::ops::Div<T, Output = T>,
    Self: Animate<Value = T>,
{
    fn div_assign(&mut self, rhs: T) {
        let v = Animate::target(self) / rhs;
        Animate::set(self, v);
    }
}
