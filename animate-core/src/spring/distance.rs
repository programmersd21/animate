pub trait Distance {
    fn distance(&self, other: &Self) -> f64;
}

macro_rules! impl_distance {
    ($($t:ty),* $(,)?) => {
        $(
            impl Distance for $t {
                #[inline]
                fn distance(&self, other: &$t) -> f64 {
                    (*self as f64 - *other as f64).abs()
                }
            }
        )*
    };
}

impl_distance!(f64, f32, usize, isize, u64, i64, u32, i32, u16, i16, u8, i8);

impl Distance for String {
    #[inline]
    fn distance(&self, other: &String) -> f64 {
        let prefix = self
            .chars()
            .zip(other.chars())
            .take_while(|(a, b)| a == b)
            .count();
        let rem_self = self.chars().count() - prefix;
        let rem_other = other.chars().count() - prefix;
        (rem_self + rem_other) as f64
    }
}

#[cfg(feature = "ratatui")]
impl Distance for ratatui::style::Color {
    #[inline]
    fn distance(&self, other: &ratatui::style::Color) -> f64 {
        use ratatui::style::Color;
        match (self, other) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let dr = *r1 as f64 - *r2 as f64;
                let dg = *g1 as f64 - *g2 as f64;
                let db = *b1 as f64 - *b2 as f64;
                (dr * dr + dg * dg + db * db).sqrt()
            }
            _ => 0.0,
        }
    }
}
