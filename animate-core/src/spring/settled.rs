pub trait Settled {
    fn magnitude(&self) -> f64;

    #[inline]
    fn is_within_epsilon(&self, epsilon: f32) -> bool {
        self.magnitude() < epsilon as f64
    }
}

#[cfg(feature = "ratatui")]
impl Settled for [f64; 3] {
    #[inline]
    fn magnitude(&self) -> f64 {
        self.iter().map(|v| v * v).sum::<f64>().sqrt()
    }
}

macro_rules! impl_settled {
    (s: $($t:ty),* $(,)?) => {
        $(
            impl Settled for $t {
                #[inline]
                fn magnitude(&self) -> f64 {
                    self.abs() as f64
                }
            }
        )*
    };

    (u: $($t:ty),* $(,)?) => {
        $(
            impl Settled for $t {
                #[inline]
                fn magnitude(&self) -> f64 {
                    *self as f64
                }
            }
        )*
    };
}

impl_settled!(s: f64, f32, isize, i64, i32, i16, i8);
impl_settled!(u: usize, u64, u32, u16, u8);