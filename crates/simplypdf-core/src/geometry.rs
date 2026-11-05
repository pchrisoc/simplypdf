//! Coordinate conversions shared by renderers, overlays, and measurement tools.

/// An (x, y) pair. Each conversion documents the coordinate space it expects.
pub type Point = [f64; 2];

/// Lower-left and upper-right corners in raw PDF user coordinates.
pub type PdfBox = [Point; 2];

/// Invalid or unrepresentable geometry supplied by a document or viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryError {
    InvalidBox,
    InvalidUserUnit,
    InvalidRotation,
    InvalidViewport,
    InvalidPoint,
}

impl std::fmt::Display for GeometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidBox => "page boxes must be finite and enclose a visible area",
            Self::InvalidUserUnit => "PDF UserUnit must be finite and positive",
            Self::InvalidRotation => "PDF rotation must be a multiple of 90 degrees",
            Self::InvalidViewport => "viewport origin and positive scale must be representable",
            Self::InvalidPoint => "point or conversion result is not finite",
        })
    }
}

impl std::error::Error for GeometryError {}

/// Page geometry resolved from PDF metadata before applying any viewer transforms.
#[derive(Debug, Clone, Copy)]
pub struct PageGeometry {
    media_origin: Point,
    user_unit: f64,
    crop_origin: Point,
    crop_size: Point,
    rotation: i32,
}

impl PageGeometry {
    /// Supply inherited/defaulted metadata from the PDF adapter. A missing crop
    /// uses the media box. The visible area is their intersection.
    pub fn new(
        media: PdfBox,
        crop: Option<PdfBox>,
        user_unit: f64,
        clockwise_degrees: i32,
    ) -> Result<Self, GeometryError> {
        let crop = crop.unwrap_or(media);
        for bounds in [media, crop] {
            if !bounds.iter().copied().all(finite)
                || bounds[0][0] >= bounds[1][0]
                || bounds[0][1] >= bounds[1][1]
            {
                return Err(GeometryError::InvalidBox);
            }
        }
        if !user_unit.is_finite() || user_unit <= 0.0 {
            return Err(GeometryError::InvalidUserUnit);
        }
        if clockwise_degrees % 90 != 0 {
            return Err(GeometryError::InvalidRotation);
        }

        let lower = [media[0][0].max(crop[0][0]), media[0][1].max(crop[0][1])];
        let upper = [media[1][0].min(crop[1][0]), media[1][1].min(crop[1][1])];
        let crop_origin = scale(subtract(lower, media[0]), user_unit);
        let crop_end = scale(subtract(upper, media[0]), user_unit);
        let crop_size = subtract(crop_end, crop_origin);
        if !finite(crop_origin)
            || !finite(crop_size)
            || crop_size[0] <= 0.0
            || crop_size[1] <= 0.0
        {
            return Err(GeometryError::InvalidBox);
        }

        Ok(Self {
            media_origin: media[0],
            user_unit,
            crop_origin,
            crop_size,
            rotation: clockwise_degrees.rem_euclid(360),
        })
    }

    /// Convert raw PDF coordinates to unrotated points relative to MediaBox,
    /// with positive Y up. This is the space used for persisted measurements.
    pub fn pdf_to_page(self, point: Point) -> Result<Point, GeometryError> {
        checked_point(scale(subtract(point, self.media_origin), self.user_unit))
    }

    /// Convert canonical page points back to raw PDF user coordinates.
    pub fn page_to_pdf(self, [x, y]: Point) -> Result<Point, GeometryError> {
        let [left, bottom] = self.media_origin;
        checked_point([x / self.user_unit + left, y / self.user_unit + bottom])
    }

    /// Build a transform for one page. The origin is the rotated crop's
    /// top-left in canvas units. Scale is canvas units per canonical point.
    pub fn viewport(self, origin: Point, scale: f64) -> Result<Viewport, GeometryError> {
        let [width, height] = self.crop_size;
        let rotated_size = match self.rotation {
            90 | 270 => [height, width],
            _ => [width, height],
        };
        let size = [rotated_size[0] * scale, rotated_size[1] * scale];
        if !finite(origin) || !finite(size) || size[0] <= 0.0 || size[1] <= 0.0 {
            return Err(GeometryError::InvalidViewport);
        }
        Ok(Viewport { page: self, origin, scale, size })
    }
}

/// Maps canonical page points to canvas coordinates with positive Y down.
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    page: PageGeometry,
    origin: Point,
    scale: f64,
    size: Point,
}

impl Viewport {
    /// Width and height of the visible page in canvas units after rotation.
    pub fn size(self) -> Point {
        self.size
    }

    /// Convert a canonical page point to the canvas. Points outside the crop
    /// can still be transformed. Clipping belongs to the canvas.
    pub fn page_to_canvas(self, point: Point) -> Result<Point, GeometryError> {
        let [x, y] = subtract(point, self.page.crop_origin);
        let [width, height] = self.page.crop_size;
        let [u, v] = match self.page.rotation {
            0 => [x, height - y],
            90 => [y, x],
            180 => [width - x, y],
            270 => [height - y, width - x],
            _ => unreachable!("rotation is validated at construction"),
        };
        let [left, top] = self.origin;
        checked_point([left + u * self.scale, top + v * self.scale])
    }

    /// Invert canvas coordinates for pointer placement and hit testing.
    pub fn canvas_to_page(self, point: Point) -> Result<Point, GeometryError> {
        let [dx, dy] = subtract(point, self.origin);
        let [u, v] = [dx / self.scale, dy / self.scale];
        let [width, height] = self.page.crop_size;
        let [x, y] = match self.page.rotation {
            0 => [u, height - v],
            90 => [v, u],
            180 => [width - u, v],
            270 => [width - v, height - u],
            _ => unreachable!("rotation is validated at construction"),
        };
        let [left, bottom] = self.page.crop_origin;
        checked_point([left + x, bottom + y])
    }
}

fn subtract([x, y]: Point, [left, bottom]: Point) -> Point {
    [x - left, y - bottom]
}

fn scale([x, y]: Point, factor: f64) -> Point {
    [x * factor, y * factor]
}

fn finite([x, y]: Point) -> bool {
    x.is_finite() && y.is_finite()
}

fn checked_point(point: Point) -> Result<Point, GeometryError> {
    if finite(point) {
        Ok(point)
    } else {
        Err(GeometryError::InvalidPoint)
    }
}
