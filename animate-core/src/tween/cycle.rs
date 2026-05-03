use crate::{Animate, Cycle, TweenAnim, Tween, FRAME_TIME, IS_ANIMATING};
use std::sync::atomic::Ordering;

impl<T, E, I> Animate for Tween<T, E, I, Cycle>
where
    T: TweenAnim + PartialEq + Default,
    E: Fn(f64) -> f64,
    I: Fn(&T, &T, f64) -> T,
{
    type Value = T;

    fn update(&mut self) {
        if self.state.pending {
            self.state.start = std::mem::take(&mut self.state.current);
            self.state.pending = false;
        }

        if let Some(start_t) = self.state.started_at {
            if self.state.duration > 0.0 {
                let now = FRAME_TIME.load(Ordering::Relaxed);
                let elapsed = now.saturating_sub(start_t) as f64;
                let t_raw = (elapsed % self.state.duration) / self.state.duration;

                self.state.current =
                    (self.state.interp)(&self.state.start, &self.state.target, (self.state.easing)(t_raw));
            }
            IS_ANIMATING.store(true, Ordering::Relaxed);
        }
    }

    fn get(&self) -> &T {
        &self.state.current
    }

    fn set(&mut self, target: T) {
        let now = FRAME_TIME.load(Ordering::Relaxed);

        self.state.target = target;
        self.state.started_at = Some(now);
        self.state.pending = true;
    }

    fn target(&self) -> &T {
        &self.state.target
    }
}
