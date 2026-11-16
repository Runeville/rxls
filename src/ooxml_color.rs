//! OOXML color helpers: indexed colors, theme colors, and tinting.
//!
//! SpreadsheetML colors can be specified as RGB, indexed palette entries, or
//! theme colors with an optional tint. The reader resolves these when style and
//! workbook theme information is available; this module exposes the same
//! conversions for callers that have raw indexed/theme+tint values.

use crate::Color;

/// The standard 64-color OOXML indexed palette.
pub const DEFAULT_INDEXED_COLORS: [Color; 64] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0xFF, 0xFF, 0xFF),
    Color::rgb(0xFF, 0x00, 0x00),
    Color::rgb(0x00, 0xFF, 0x00),
    Color::rgb(0x00, 0x00, 0xFF),
    Color::rgb(0xFF, 0xFF, 0x00),
    Color::rgb(0xFF, 0x00, 0xFF),
    Color::rgb(0x00, 0xFF, 0xFF),
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0xFF, 0xFF, 0xFF),
    Color::rgb(0xFF, 0x00, 0x00),
    Color::rgb(0x00, 0xFF, 0x00),
    Color::rgb(0x00, 0x00, 0xFF),
    Color::rgb(0xFF, 0xFF, 0x00),
    Color::rgb(0xFF, 0x00, 0xFF),
    Color::rgb(0x00, 0xFF, 0xFF),
    Color::rgb(0x80, 0x00, 0x00),
    Color::rgb(0x00, 0x80, 0x00),
    Color::rgb(0x00, 0x00, 0x80),
    Color::rgb(0x80, 0x80, 0x00),
    Color::rgb(0x80, 0x00, 0x80),
    Color::rgb(0x00, 0x80, 0x80),
    Color::rgb(0xC0, 0xC0, 0xC0),
    Color::rgb(0x80, 0x80, 0x80),
    Color::rgb(0x99, 0x99, 0xFF),
    Color::rgb(0x99, 0x33, 0x66),
    Color::rgb(0xFF, 0xFF, 0xCC),
    Color::rgb(0xCC, 0xFF, 0xFF),
    Color::rgb(0x66, 0x00, 0x66),
    Color::rgb(0xFF, 0x80, 0x80),
    Color::rgb(0x00, 0x66, 0xCC),
    Color::rgb(0xCC, 0xCC, 0xFF),
    Color::rgb(0x00, 0x00, 0x80),
    Color::rgb(0xFF, 0x00, 0xFF),
    Color::rgb(0xFF, 0xFF, 0x00),
    Color::rgb(0x00, 0xFF, 0xFF),
    Color::rgb(0x80, 0x00, 0x80),
    Color::rgb(0x80, 0x00, 0x00),
    Color::rgb(0x00, 0x80, 0x80),
    Color::rgb(0x00, 0x00, 0xFF),
    Color::rgb(0x00, 0xCC, 0xFF),
    Color::rgb(0xCC, 0xFF, 0xFF),
    Color::rgb(0xCC, 0xFF, 0xCC),
    Color::rgb(0xFF, 0xFF, 0x99),
    Color::rgb(0x99, 0xCC, 0xFF),
    Color::rgb(0xFF, 0x99, 0xCC),
    Color::rgb(0xCC, 0x99, 0xFF),
    Color::rgb(0xFF, 0xCC, 0x99),
    Color::rgb(0x33, 0x66, 0xFF),
    Color::rgb(0x33, 0xCC, 0xCC),
    Color::rgb(0x99, 0xCC, 0x00),
    Color::rgb(0xFF, 0xCC, 0x00),
    Color::rgb(0xFF, 0x99, 0x00),
    Color::rgb(0xFF, 0x66, 0x00),
    Color::rgb(0x66, 0x66, 0x99),
    Color::rgb(0x96, 0x96, 0x96),
    Color::rgb(0x00, 0x33, 0x66),
    Color::rgb(0x33, 0x99, 0x66),
    Color::rgb(0x00, 0x33, 0x00),
    Color::rgb(0x33, 0x33, 0x00),
    Color::rgb(0x99, 0x33, 0x00),
    Color::rgb(0x99, 0x33, 0x66),
    Color::rgb(0x33, 0x33, 0x99),
    Color::rgb(0x33, 0x33, 0x33),
];

/// Default Office theme palette in SpreadsheetML/DrawingML canonical slot order:
/// `lt1`, `dk1`, `lt2`, `dk2`, `accent1`..`accent6`, `hlink`, `folHlink`.
pub const DEFAULT_OFFICE_THEME_COLORS: [Color; 12] = [
    Color::rgb(0xFF, 0xFF, 0xFF),
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0xE7, 0xE6, 0xE6),
    Color::rgb(0x44, 0x54, 0x6A),
    Color::rgb(0x44, 0x72, 0xC4),
    Color::rgb(0xED, 0x7D, 0x31),
    Color::rgb(0xA5, 0xA5, 0xA5),
    Color::rgb(0xFF, 0xC0, 0x00),
    Color::rgb(0x5B, 0x9B, 0xD5),
    Color::rgb(0x70, 0xAD, 0x47),
    Color::rgb(0x05, 0x63, 0xC1),
    Color::rgb(0x95, 0x4F, 0x72),
];

/// Apply an OOXML tint to `color`.
///
/// Negative tint darkens the color toward black; positive tint lightens it
/// toward white. Values outside the usual `[-1.0, 1.0]` range are applied using
/// the same formula and then clamped to valid channel bounds.
pub fn apply_tint(color: Color, tint: f64) -> Color {
    fn channel(value: u8, tint: f64) -> u8 {
        let value = f64::from(value);
        let tinted = if tint < 0.0 {
            value * (1.0 + tint)
        } else {
            value * (1.0 - tint) + 255.0 * tint
        };
        tinted.round().clamp(0.0, 255.0) as u8
    }

    let [red, green, blue] = color.as_rgb();
    Color::rgb(
        channel(red, tint),
        channel(green, tint),
        channel(blue, tint),
    )
}

/// Resolve an indexed OOXML color to RGB.
///
/// `custom_indexed_colors` is the workbook's custom indexed palette (from
/// `<colors><indexedColors>` in `styles.xml`), if any. Entries missing from the
/// custom palette fall back to [`DEFAULT_INDEXED_COLORS`]. `tint` is applied
/// when finite.
pub fn indexed_color_rgb(
    index: u32,
    tint: Option<f64>,
    custom_indexed_colors: &[Color],
) -> Option<Color> {
    let idx = usize::try_from(index).ok()?;
    let color = custom_indexed_colors
        .get(idx)
        .copied()
        .or_else(|| DEFAULT_INDEXED_COLORS.get(idx).copied())?;
    Some(apply_optional_tint(color, tint))
}

/// Resolve a theme color slot to RGB.
///
/// `theme_colors` must be in canonical slot order: `lt1`, `dk1`, `lt2`, `dk2`,
/// `accent1`..`accent6`, `hlink`, `folHlink`. Use [`DEFAULT_OFFICE_THEME_COLORS`]
/// for the standard Office theme when the workbook theme is unavailable. `tint`
/// is applied when finite.
pub fn theme_color_rgb(
    theme_index: u32,
    tint: Option<f64>,
    theme_colors: &[Color; 12],
) -> Option<Color> {
    let idx = usize::try_from(theme_index).ok()?;
    let color = *theme_colors.get(idx)?;
    Some(apply_optional_tint(color, tint))
}

/// Resolve a theme color slot from an optional palette to RGB.
///
/// This is useful when a workbook theme may omit or fail to resolve individual
/// slots.
pub fn theme_color_rgb_opt(
    theme_index: u32,
    tint: Option<f64>,
    theme_colors: &[Option<Color>; 12],
) -> Option<Color> {
    let idx = usize::try_from(theme_index).ok()?;
    let color = theme_colors.get(idx).copied().flatten()?;
    Some(apply_optional_tint(color, tint))
}

/// Resolve a theme color slot to RGB using the default Office theme palette.
pub fn default_theme_color_rgb(theme_index: u32, tint: Option<f64>) -> Option<Color> {
    theme_color_rgb(theme_index, tint, &DEFAULT_OFFICE_THEME_COLORS)
}

pub(crate) fn apply_optional_tint(color: Color, tint: Option<f64>) -> Color {
    match tint {
        Some(tint) if tint.is_finite() => apply_tint(color, tint),
        _ => color,
    }
}
