//! SimplyPDF owner-provided visual identity, selected from the resolved UI theme.
use egui::{Image, Vec2};

const LIGHT: &[u8] = include_bytes!("../../../assets/app-icon/simplypdf-light-128.png");
const DARK: &[u8] = include_bytes!("../../../assets/app-icon/simplypdf-dark-128.png");

pub(crate) fn image(dark: bool, size: f32) -> Image<'static> {
    let (uri, bytes) = if dark { ("bytes://simplypdf-brand-dark.png", DARK) } else { ("bytes://simplypdf-brand-light.png", LIGHT) };
    Image::from_bytes(uri, bytes).fit_to_exact_size(Vec2::splat(size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_variants_are_distinct_decodable_rgba_images() {
        let light = image::load_from_memory(LIGHT).expect("light logo must decode").to_rgba8();
        let dark = image::load_from_memory(DARK).expect("dark logo must decode").to_rgba8();
        assert_eq!(light.dimensions(), (128, 128));
        assert_eq!(dark.dimensions(), (128, 128));
        assert_eq!(light.get_pixel(64, 64)[3], 0);
        assert_eq!(dark.get_pixel(64, 64)[3], 255);
        assert_eq!(&dark.get_pixel(64, 64).0[..3], &[38, 110, 129]);
    }
}
