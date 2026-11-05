//! Standalone PDFium compatibility probe for the repository's synthetic fixture.

use std::error::Error;
use std::ffi::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::ptr;

use libloading::Library;
use simplypdf_core::geometry::PageGeometry;

type Handle = *mut c_void;
type OpenDocument = unsafe extern "system" fn(*const c_void, usize, *const c_char) -> Handle;
type CreateBitmap = unsafe extern "system" fn(c_int, c_int, c_int, *mut c_void, c_int) -> Handle;
type RenderPage = unsafe extern "system" fn(Handle, Handle, c_int, c_int, c_int, c_int, c_int, c_int);
type PageToDevice = unsafe extern "system" fn(
    Handle, c_int, c_int, c_int, c_int, c_int, f64, f64, *mut c_int, *mut c_int,
) -> c_int;

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("pass the absolute path to the PDFium dynamic library")?;
    if !path.is_absolute() {
        return Err("the library path must be absolute".into());
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/page-geometry.pdf");
    let bytes = std::fs::read(fixture)?;

    // Only load an explicitly supplied, trusted PDFium library. The signatures
    // below match the public chromium/8138 headers. No handles escape the probe.
    let library = unsafe { Library::new(path) }?;
    let probe = Pdfium::load(&library)?;
    probe.check_fixture(&bytes)
}

struct Pdfium<'a> {
    init: libloading::Symbol<'a, unsafe extern "system" fn()>,
    shutdown: libloading::Symbol<'a, unsafe extern "system" fn()>,
    open: libloading::Symbol<'a, OpenDocument>,
    close: libloading::Symbol<'a, unsafe extern "system" fn(Handle)>,
    count: libloading::Symbol<'a, unsafe extern "system" fn(Handle) -> c_int>,
    load_page: libloading::Symbol<'a, unsafe extern "system" fn(Handle, c_int) -> Handle>,
    close_page: libloading::Symbol<'a, unsafe extern "system" fn(Handle)>,
    width: libloading::Symbol<'a, unsafe extern "system" fn(Handle) -> f32>,
    height: libloading::Symbol<'a, unsafe extern "system" fn(Handle) -> f32>,
    bitmap: libloading::Symbol<'a, CreateBitmap>,
    destroy_bitmap: libloading::Symbol<'a, unsafe extern "system" fn(Handle)>,
    render: libloading::Symbol<'a, RenderPage>,
    page_to_device: libloading::Symbol<'a, PageToDevice>,
}

impl<'a> Pdfium<'a> {
    fn load(library: &'a Library) -> Result<Self, libloading::Error> {
        // Each symbol is typed against the release's fpdfview.h.
        unsafe {
            Ok(Self {
                init: library.get(b"FPDF_InitLibrary")?,
                shutdown: library.get(b"FPDF_DestroyLibrary")?,
                open: library.get(b"FPDF_LoadMemDocument64")?,
                close: library.get(b"FPDF_CloseDocument")?,
                count: library.get(b"FPDF_GetPageCount")?,
                load_page: library.get(b"FPDF_LoadPage")?,
                close_page: library.get(b"FPDF_ClosePage")?,
                width: library.get(b"FPDF_GetPageWidthF")?,
                height: library.get(b"FPDF_GetPageHeightF")?,
                bitmap: library.get(b"FPDFBitmap_CreateEx")?,
                destroy_bitmap: library.get(b"FPDFBitmap_Destroy")?,
                render: library.get(b"FPDF_RenderPageBitmap")?,
                page_to_device: library.get(b"FPDF_PageToDevice")?,
            })
        }
    }

    fn check_fixture(&self, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        // The fixture buffer remains live until the document is closed.
        unsafe {
            (self.init)();
            let doc = (self.open)(bytes.as_ptr().cast(), bytes.len(), ptr::null());
            let result = if doc.is_null() {
                Err("PDFium could not open the fixture".into())
            } else {
                let result = self.check_pages(doc);
                (self.close)(doc);
                result
            };
            (self.shutdown)();
            result
        }
    }

    fn check_pages(&self, doc: Handle) -> Result<(), Box<dyn Error>> {
        // doc belongs to the active library. Each page is closed before doc.
        unsafe {
            if (self.count)(doc) != 4 {
                return Err("the fixture should have four pages".into());
            }
            for (index, rotation) in [0, 90, 180, 270].into_iter().enumerate() {
                let page = (self.load_page)(doc, index as c_int);
                if page.is_null() {
                    return Err(format!("could not load fixture page {}", index + 1).into());
                }
                let result = self.check_page(page, rotation);
                (self.close_page)(page);
                result?;
            }
        }
        Ok(())
    }

    fn check_page(&self, page: Handle, rotation: i32) -> Result<(), Box<dyn Error>> {
        let geometry = PageGeometry::new(
            [[-10.0, 20.0], [310.0, 260.0]],
            Some([[30.0, 50.0], [230.0, 200.0]]),
            2.0,
            rotation,
        )?;
        let viewport = geometry.viewport([0.0, 0.0], 2.0)?;
        let [width, height] = viewport.size().map(|side| side as c_int);
        let reported_without_user_unit = [width as f32 / 4.0, height as f32 / 4.0];
        let raw = [65.0, 80.0];
        let expected = viewport.page_to_canvas(geometry.pdf_to_page(raw)?)?;
        let mut x = 0;
        let mut y = 0;
        // The page is live. Format 3 is BGRx and uses four bytes per pixel.
        // The bitmap borrows the initialized white buffer until destruction.
        let (reported, mapped, ink) = unsafe {
            let reported = [(self.width)(page), (self.height)(page)];
            let mut pixels = vec![255_u8; (width * height * 4) as usize];
            let bitmap = (self.bitmap)(width, height, 3, pixels.as_mut_ptr().cast(), width * 4);
            if bitmap.is_null() {
                return Err("PDFium could not create the fixture bitmap".into());
            }
            (self.render)(bitmap, page, 0, 0, width, height, 0, 0);
            (self.destroy_bitmap)(bitmap);
            let mapped = (self.page_to_device)(page, 0, 0, width, height, 0, raw[0], raw[1], &mut x, &mut y);
            let offset = ((expected[1] as c_int * width + expected[0] as c_int) * 4) as usize;
            (reported, mapped, pixels[offset..offset + 3].iter().all(|&channel| channel < 64))
        };
        let actual = [f64::from(x), f64::from(y)];
        if mapped == 0 || actual != expected || !ink
            || geometry.page_to_pdf(viewport.canvas_to_page(actual)?)? != raw
        {
            return Err(format!("{rotation}°: expected ink at {expected:?}, mapped to {actual:?}").into());
        }
        if reported != reported_without_user_unit {
            return Err(format!(
                "{rotation}°: PDFium now reports {reported:?}; expected unscaled size {reported_without_user_unit:?}"
            ).into());
        }
        println!("{rotation:>3}°: reported {reported:?}, raster {width}×{height}, ink and pointer {actual:?}");
        Ok(())
    }
}
