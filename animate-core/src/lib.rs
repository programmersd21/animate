pub mod spring;
pub mod tween;
pub mod types;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub use spring::*;
pub use tween::easing;
pub use tween::*;

pub static FRAME_TIME: AtomicUsize = AtomicUsize::new(0);
pub static LAST_DELTA: AtomicUsize = AtomicUsize::new(0);
pub static IS_ANIMATING: AtomicBool = AtomicBool::new(false);

pub trait Animate {
    type Value;
    fn update(&mut self);
    fn get(&self) -> &Self::Value;
    fn set(&mut self, target: Self::Value);
    fn target(&self) -> &Self::Value;
}

pub trait Mode {}

#[derive(Debug, Clone, Copy, Default)]
pub struct Once;
impl Mode for Once {}

#[derive(Debug, Clone, Copy, Default)]
pub struct Cycle;
impl Mode for Cycle {}

#[derive(Debug, Clone, Copy, Default)]
pub struct Alternate;
impl Mode for Alternate {}

#[inline(always)]
pub fn tick(delta: usize) {
    FRAME_TIME.fetch_add(delta, Ordering::Relaxed);
    LAST_DELTA.store(delta, Ordering::Relaxed);
    IS_ANIMATING.store(false, Ordering::Relaxed);
}

pub fn frame() -> usize {
    FRAME_TIME.load(Ordering::Relaxed)
}

#[inline]
pub fn delta() -> f64 {
    let delta_ms = crate::LAST_DELTA.load(Ordering::Relaxed);
    (delta_ms as f64 / 1000.0).max(f64::MIN_POSITIVE)
}

pub fn is_animating() -> bool {
    IS_ANIMATING.load(Ordering::Relaxed)
}
