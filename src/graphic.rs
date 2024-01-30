use macroquad::prelude::Color;

/// Scales (brightens/darkens) a color by a given factor.
/// I know the behavior is just wrong, but it's enough for now.
pub fn scale_color(color: Color, factor: f32) -> Color {
    Color::new(
        color.r * factor,
        color.g * factor,
        color.b * factor,
        color.a,
    )
}
