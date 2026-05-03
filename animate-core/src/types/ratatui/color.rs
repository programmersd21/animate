#![cfg(feature = "ratatui")]

use crate::types::num::spring_step;
use crate::{SpringAnim, SpringParams, TweenAnim};
use ratatui::style::Color;

impl TweenAnim for Color {
    fn tween(start: &Color, end: &Color, t: f64) -> Color {
        match (rgb(*start), rgb(*end)) {
            (Some((sr, sg, sb)), Some((er, eg, eb))) => {
                let r = u8::tween(&sr, &er, t);
                let g = u8::tween(&sg, &eg, t);
                let b = u8::tween(&sb, &eb, t);
                Color::Rgb(r, g, b)
            }
            _ => *end,
        }
    }
}

impl SpringAnim for Color {
    type Velocity = [f64; 3];

    fn spring(
        current: &Color,
        target: &Color,
        velocity: &[f64; 3],
        params: SpringParams,
        dt: f64,
    ) -> (Color, [f64; 3]) {
        let (cr, cg, cb) = rgb(*current).unwrap_or((0, 0, 0));
        let (tr, tg, tb) = rgb(*target).unwrap_or((0, 0, 0));
        let [vr, vg, vb] = *velocity;

        let (nr, nvr) = spring_step(cr as f64, tr as f64, vr, params, dt);
        let (ng, nvg) = spring_step(cg as f64, tg as f64, vg, params, dt);
        let (nb, nvb) = spring_step(cb as f64, tb as f64, vb, params, dt);

        let clamp_u8 = |v: f64| v.clamp(0.0, 255.0).round() as u8;
        (
            Color::Rgb(clamp_u8(nr), clamp_u8(ng), clamp_u8(nb)),
            [nvr, nvg, nvb],
        )
    }
}

fn rgb(c: Color) -> Option<(u8, u8, u8)> {
    match c {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}
