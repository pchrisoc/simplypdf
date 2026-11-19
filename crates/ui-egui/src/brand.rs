//! SimplyPDF owner-provided visual identity, selected from the resolved UI theme.
use egui::{ColorImage, Context, Id, Image, Response, Sense, TextureHandle, TextureOptions, Ui, Vec2};

const LIGHT: &[u8] = include_bytes!("../../../assets/app-icon/simplypdf-light-128.png");
const DARK: &[u8] = include_bytes!("../../../assets/app-icon/simplypdf-dark-128.png");

pub(crate) fn show(ui: &mut Ui, dark: bool, size: f32) -> Response {
    let size = Vec2::splat(size);
    match texture(ui.ctx(), dark) {
        Some(texture) => ui.add(Image::from_texture(&texture).fit_to_exact_size(size)),
        None => ui.allocate_response(size, Sense::hover()),
    }
}

fn texture(ctx: &Context, dark: bool) -> Option<TextureHandle> {
    let (name, bytes) = if dark { ("simplypdf-brand-dark", DARK) } else { ("simplypdf-brand-light", LIGHT) };
    let id = Id::unique(name);
    if let Some(cached) = ctx.data(|data| data.get_temp::<Option<TextureHandle>>(id)) {
        return cached;
    }

    let texture = match image::load_from_memory_with_format(bytes, image::ImageFormat::Png) {
        Ok(image) => {
            let rgba = image.into_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            Some(ctx.load_texture(name, pixels, TextureOptions::LINEAR))
        }
        Err(error) => {
            log::warn!("Could not decode embedded logo {name}: {error}");
            None
        }
    };
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_logo_textures_load_and_are_reused() {
        let ctx = Context::default();
        let light = texture(&ctx, false).expect("light logo texture");
        let dark = texture(&ctx, true).expect("dark logo texture");
        assert_eq!(light.size(), [128, 128]);
        assert_eq!(dark.size(), [128, 128]);
        assert_ne!(light.id(), dark.id());
        assert_eq!(texture(&ctx, false).expect("cached light texture").id(), light.id());
        assert_eq!(texture(&ctx, true).expect("cached dark texture").id(), dark.id());
    }

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
