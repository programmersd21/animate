mod alternate;
mod cycle;
mod once;
mod distance;
mod settled;

pub use distance::Distance;
pub use settled::Settled;

use crate::{Animate, Mode, Once};
use std::marker::PhantomData;

pub struct Spring<T, I, M = Once>
where
    M: Mode,
    T: SpringAnim,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
{
    pub(crate) state: SpringState<T, I>,
    _mode: PhantomData<M>,
    pub(crate) advancing: bool,
}

impl<T, I, M> Spring<T, I, M>
where
    M: Mode,
    T: SpringAnim,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
{
    pub fn new(initial: T, params: SpringParams, interp: I) -> Self {
        Self {
            state: SpringState::new(initial, params, interp),
            _mode: PhantomData,
            advancing: true,
        }
    }

    pub fn velocity(&self) -> f64
    where
        T::Velocity: Settled,
    {
        self.state.velocity.magnitude()
    }
}


pub trait SpringAnim: Sized + Default {
    fn spring(
        current: &Self,
        target: &Self,
        velocity: &Self::Velocity,
        params: SpringParams,
        delta: f64,
    ) -> (Self, Self::Velocity);

    type Velocity: Default + std::fmt::Debug;
}

#[derive(Clone, Copy, Debug)]
pub struct SpringParams {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
    pub epsilon: f32,
}

impl Default for SpringParams {
    fn default() -> Self {
        Self {
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
            epsilon: 0.001,
        }
    }
}

impl SpringParams {
    pub fn new(stiffness: f32, damping: f32, mass: f32) -> Self {
        Self {
            stiffness,
            damping,
            mass,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub(crate) struct SpringState<T: SpringAnim, I>
where
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
{
    pub current: T,
    pub origin: T,
    pub target: T,
    pub velocity: T::Velocity,
    pub active: bool,
    pub pending: bool,
    pub params: SpringParams,
    pub interp: I,
}

impl<T, I> SpringState<T, I>
where
    T: SpringAnim,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
{
    pub fn new(initial: T, params: SpringParams, interp: I) -> Self {
        Self {
            current: initial,
            origin: T::default(),
            target: T::default(),
            velocity: T::Velocity::default(),
            active: false,
            pending: false,
            params,
            interp,
        }
    }
}

impl<T, I, M> std::ops::Deref for Spring<T, I, M>
where
    M: Mode,
    T: SpringAnim,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
    Self: Animate<Value = T>,
{
    type Target = T;
    fn deref(&self) -> &T {
        Animate::get(self)
    }
}

impl<T, I, M> std::fmt::Display for Spring<T, I, M>
where
    M: Mode,
    T: SpringAnim + std::fmt::Display,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
    Self: Animate<Value = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Animate::get(self).fmt(f)
    }
}

impl<T, I, M> std::ops::AddAssign<T> for Spring<T, I, M>
where
    M: Mode,
    T: SpringAnim,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
    for<'b> &'b T: std::ops::Add<T, Output = T>,
    Self: Animate<Value = T>,
{
    fn add_assign(&mut self, rhs: T) {
        let v = Animate::target(self) + rhs;
        Animate::set(self, v);
    }
}

impl<T, I, M> std::ops::SubAssign<T> for Spring<T, I, M>
where
    M: Mode,
    T: SpringAnim,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
    for<'b> &'b T: std::ops::Sub<T, Output = T>,
    Self: Animate<Value = T>,
{
    fn sub_assign(&mut self, rhs: T) {
        let v = Animate::target(self) - rhs;
        Animate::set(self, v);
    }
}

impl<T, I, M> std::ops::MulAssign<T> for Spring<T, I, M>
where
    M: Mode,
    T: SpringAnim,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
    for<'b> &'b T: std::ops::Mul<T, Output = T>,
    Self: Animate<Value = T>,
{
    fn mul_assign(&mut self, rhs: T) {
        let v = Animate::target(self) * rhs;
        Animate::set(self, v);
    }
}

impl<T, I, M> std::ops::DivAssign<T> for Spring<T, I, M>
where
    M: Mode,
    T: SpringAnim,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
    for<'b> &'b T: std::ops::Div<T, Output = T>,
    Self: Animate<Value = T>,
{
    fn div_assign(&mut self, rhs: T) {
        let v = Animate::target(self) / rhs;
        Animate::set(self, v);
    }
}

#[inline]
fn has_settled<V: Settled>(delta: f64, velocity: &V, epsilon: f32) -> bool {
    delta.abs() < epsilon as f64 && velocity.is_within_epsilon(epsilon)
}
