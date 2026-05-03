use crate::{SpringAnim, SpringParams, TweenAnim};

#[inline(always)]
pub(crate) fn spring_step(
    pos: f64,
    target: f64,
    vel: f64,
    p: SpringParams,
    dt: f64,
) -> (f64, f64) {
    let k = p.stiffness as f64;
    let c = p.damping as f64;
    let m = p.mass as f64;

    let displacement = pos - target;
    let accel = (-k * displacement - c * vel) / m;
    let new_vel = vel + accel * dt;
    let new_pos = pos + new_vel * dt;
    (new_pos, new_vel)
}

macro_rules! impl_num {
    ($t:ty) => {
        impl TweenAnim for $t {
            #[inline]
            fn tween(start: &$t, end: &$t, t: f64) -> $t {
                (*start as f64 + (*end as f64 - *start as f64) * t) as $t
            }
        }

        impl SpringAnim for $t {
            type Velocity = f64;

            #[inline]
            fn spring(
                current: &$t,
                target: &$t,
                velocity: &f64,
                params: SpringParams,
                dt: f64,
            ) -> ($t, f64) {
                let (new_pos, new_vel) =
                    spring_step(*current as f64, *target as f64, *velocity, params, dt);
                (new_pos.round() as $t, new_vel)
            }
        }
    };
}

impl_num!(f64);
impl_num!(f32);
impl_num!(usize);
impl_num!(isize);
impl_num!(u64);
impl_num!(i64);
impl_num!(u32);
impl_num!(i32);
impl_num!(u16);
impl_num!(i16);
impl_num!(u8);
impl_num!(i8);
