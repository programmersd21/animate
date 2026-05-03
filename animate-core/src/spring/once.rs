use crate::spring::{Distance, Settled};
use crate::{Animate, IS_ANIMATING, Once, SpringAnim, Spring, SpringParams, delta};

impl<T, I> Animate for Spring<T, I, Once>
where
    T: SpringAnim + PartialEq + Distance,
    T::Velocity: Settled,
    I: Fn(&T, &T, &T::Velocity, SpringParams, f64) -> (T, T::Velocity),
{
    type Value = T;

    fn update(&mut self) {
        if !self.state.active {
            return;
        }

        let dt = delta();
        if dt == 0.0 {
            return;
        }

        let (new_pos, new_vel) = (self.state.interp)(
            &self.state.current,
            &self.state.target,
            &self.state.velocity,
            self.state.params,
            dt,
        );

        let delta = new_pos.distance(&self.state.target);

        if super::has_settled(delta, &new_vel, self.state.params.epsilon) {
            self.state.current = std::mem::take(&mut self.state.target);
            self.state.velocity = T::Velocity::default();
            self.state.active = false;
        } else {
            self.state.current = new_pos;
            self.state.velocity = new_vel;
            IS_ANIMATING.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn get(&self) -> &T {
        &self.state.current
    }

    fn set(&mut self, target: T) {
        self.state.target = target;
        self.state.active = true;
    }

    fn target(&self) -> &T {
        if self.state.active {
            &self.state.target
        } else {
            &self.state.current
        }
    }
}
