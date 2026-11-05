//! Quartz adapter. The viewer only receives RGBA pixels and shared page geometry.

use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use objc2_core_foundation::{CFRetained, CFURL, CGAffineTransform, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorSpace, CGContext, CGImageAlphaInfo, CGPDFBox, CGPDFDictionary,
    CGPDFDocument, CGPDFPage,
};
use simplypdf_core::geometry::{PageGeometry, PdfBox};

pub struct Document(CFRetained<CGPDFDocument>);

pub struct RasterPage {
    pub geometry: PageGeometry,
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
    pub uv_max: [f32; 2],
}

impl Document {
    pub fn open(path: &Path) -> Result<Self, String> {
        let path = std::path::absolute(path).map_err(|error| error.to_string())?;
        let bytes = path.as_os_str().as_bytes();
        let length = isize::try_from(bytes.len()).map_err(|_| "Path is too long")?;
        // CFURL reads exactly the byte count from the live path buffer.
        let url = unsafe { CFURL::from_file_system_representation(None, bytes.as_ptr(), length, false) }
            .ok_or("Could not create a file URL")?;
        let document = CGPDFDocument::with_url(Some(&url)).ok_or("Could not open the PDF")?;
        if !CGPDFDocument::is_unlocked(Some(&document)) {
            return Err("This PDF requires a password".into());
        }
        Ok(Self(document))
    }

    pub fn page_count(&self) -> usize {
        CGPDFDocument::number_of_pages(Some(&self.0))
    }

    pub fn render(&self, index: usize) -> Result<RasterPage, String> {
        let page = CGPDFDocument::page(Some(&self.0), index + 1).ok_or("Page unavailable")?;
        let media = CGPDFPage::box_rect(Some(&page), CGPDFBox::MediaBox);
        let crop = CGPDFPage::box_rect(Some(&page), CGPDFBox::CropBox);
        let rotation = CGPDFPage::rotation_angle(Some(&page));
        let mut user_unit = 1.0;
        let dictionary = CGPDFPage::dictionary(Some(&page));
        // The dictionary is borrowed from the retained page and the key is NUL-terminated.
        if !dictionary.is_null() {
            unsafe { CGPDFDictionary::number(dictionary, std::ptr::NonNull::from(c"UserUnit").cast(), &mut user_unit) };
        }
        let geometry = PageGeometry::new(box_corners(media), Some(box_corners(crop)), user_unit, rotation)
            .map_err(|error| error.to_string())?;
        let [width, height] = geometry.viewport([0.0, 0.0], 1.0)
            .map_err(|error| error.to_string())?.size();
        // Bound a single RGBA page to 16 MiB during this synchronous spike.
        let scale = (2048.0 / width.max(height)).min(2.0);
        let draw_size = [width * scale, height * scale];
        let size = draw_size.map(|side| side.ceil() as usize);
        let [width, height] = size;
        let mut rgba = vec![0; width * height * 4];
        let color = CGColorSpace::new_device_rgb().ok_or("Could not create RGB color space")?;
        // The buffer is sized for the supplied stride and outlives the context.
        let context = unsafe {
            CGBitmapContextCreate(
                rgba.as_mut_ptr().cast(), width, height, 8, width * 4, Some(&color),
                CGImageAlphaInfo::PremultipliedLast.0,
            )
        }.ok_or("Could not allocate a PDF raster")?;
        let bounds = CGRect::new(CGPoint::ZERO, CGSize::new(width as f64, height as f64));
        CGContext::set_rgb_fill_color(Some(&context), 1.0, 1.0, 1.0, 1.0);
        CGContext::fill_rect(Some(&context), bounds);
        // Build Quartz's raw-PDF-to-bitmap affine map from the same viewport
        // used for clicks. Quartz bitmap drawing has positive Y up.
        let view = geometry.viewport([0.0, 0.0], scale).map_err(|error| error.to_string())?;
        let map = |raw| {
            let [x, y] = view.page_to_canvas(geometry.pdf_to_page(raw)?)?;
            Ok::<_, simplypdf_core::geometry::GeometryError>([x, height as f64 - y])
        };
        let transform = (|| {
            let [tx, ty] = map([0.0, 0.0])?;
            let [x1, y1] = map([1.0, 0.0])?;
            let [x2, y2] = map([0.0, 1.0])?;
            Ok::<_, simplypdf_core::geometry::GeometryError>(CGAffineTransform {
                a: x1 - tx, b: y1 - ty, c: x2 - tx, d: y2 - ty, tx, ty,
            })
        })().map_err(|error| error.to_string())?;
        CGContext::concat_ctm(Some(&context), transform);
        CGContext::draw_pdf_page(Some(&context), Some(&page));
        drop(context);
        let uv_max = [(draw_size[0] / width as f64) as f32, (draw_size[1] / height as f64) as f32];
        Ok(RasterPage { geometry, size, rgba, uv_max })
    }
}

fn box_corners(rect: CGRect) -> PdfBox {
    let CGPoint { x, y } = rect.origin;
    [[x, y], [x + rect.size.width, y + rect.size.height]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_ink_matches_the_pointer_transform_at_every_rotation() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/page-geometry.pdf");
        let document = Document::open(&fixture).unwrap();
        assert_eq!(document.page_count(), 4);

        // Midpoint of the horizontal edge before rotation, from the fixture manifest.
        let expected = [[140, 480], [120, 140], [660, 120], [480, 660]];
        for (index, [x, y]) in expected.into_iter().enumerate() {
            let page = document.render(index).unwrap();
            let view = page.geometry.viewport([0.0, 0.0], 2.0).unwrap();
            assert_eq!(view.size(), page.size.map(|side| side as f64));
            let pixel = (y * page.size[0] + x) * 4;
            assert!(page.rgba[pixel] < 64, "expected ink on page {}", index + 1);
            let point = view.canvas_to_page([x as f64, y as f64]).unwrap();
            assert_eq!(page.geometry.page_to_pdf(point).unwrap(), [65.0, 80.0]);
        }
    }
}
