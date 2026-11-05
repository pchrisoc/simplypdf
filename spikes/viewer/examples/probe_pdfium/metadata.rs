//! Resolves PDF page-tree metadata for the PDFium comparison probe.

use std::collections::HashSet;
use std::error::Error;

use lopdf::{Dictionary, Document, Object, ObjectId};
use simplypdf_core::geometry::{PageGeometry, PdfBox};

/// Extract canonical geometry in page-tree order from the same bytes sent to PDFium.
pub fn read_pages(bytes: &[u8]) -> Result<Vec<PageGeometry>, Box<dyn Error>> {
    let document = Document::load_mem(bytes)?;
    document
        .get_pages()
        .into_iter()
        .map(|(number, id)| {
            page_geometry(&document, id).map_err(|error| format!("Page {number}: {error}").into())
        })
        .collect()
}

fn page_geometry(document: &Document, id: ObjectId) -> Result<PageGeometry, Box<dyn Error>> {
    let media = inherited(document, id, b"MediaBox")?.ok_or("missing MediaBox")?;
    let crop = inherited(document, id, b"CropBox")?
        .map(|value| page_box(document, value))
        .transpose()?;
    let rotation = inherited(document, id, b"Rotate")?
        .map(Object::as_i64)
        .transpose()?
        .unwrap_or(0);
    // UserUnit belongs to the page dictionary. A parent's value has no effect.
    let unit = entry(document, document.get_dictionary(id)?, b"UserUnit")?
        .map(number)
        .transpose()?
        .unwrap_or(1.0);

    Ok(PageGeometry::new(
        page_box(document, media)?,
        crop,
        unit,
        rotation.rem_euclid(360) as i32,
    )?)
}

fn inherited<'a>(
    document: &'a Document,
    mut id: ObjectId,
    key: &[u8],
) -> Result<Option<&'a Object>, Box<dyn Error>> {
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(id) {
            return Err("cycle in PDF page parents".into());
        }
        let dictionary = document.get_dictionary(id)?;
        if let Some(value) = entry(document, dictionary, key)? {
            return Ok(Some(value));
        }
        // Preserve the parent object ID for cycle detection.
        match dictionary.get(b"Parent") {
            Ok(Object::Null) | Err(lopdf::Error::DictKey(_)) => return Ok(None),
            Ok(parent) => id = parent.as_reference()?,
            Err(error) => return Err(error.into()),
        }
    }
}

fn entry<'a>(
    document: &'a Document,
    dictionary: &'a Dictionary,
    key: &[u8],
) -> lopdf::Result<Option<&'a Object>> {
    match dictionary.get_deref(key, document) {
        Ok(Object::Null) | Err(lopdf::Error::DictKey(_)) => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(error) => Err(error),
    }
}

fn page_box(document: &Document, object: &Object) -> Result<PdfBox, Box<dyn Error>> {
    let [left, bottom, right, top] = object.as_array()?.as_slice() else {
        return Err("a page box must contain four coordinates".into());
    };
    let coordinate = |value| number(document.dereference(value)?.1);
    Ok([
        [coordinate(left)?, coordinate(bottom)?],
        [coordinate(right)?, coordinate(top)?],
    ])
}

fn number(object: &Object) -> lopdf::Result<f64> {
    match object {
        Object::Integer(value) => Ok(*value as f64),
        _ => object.as_float().map(f64::from),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::dictionary;

    #[test]
    fn inherits_nearest_box_and_rotation_with_indirect_values() {
        let mut doc = Document::new();
        let coordinate = doc.add_object(-10);
        let media = doc.add_object(vec![coordinate.into(), 20.into(), 310.into(), 260.into()]);
        let root = doc.add_object(dictionary! {
            "MediaBox" => media,
            "Rotate" => 90,
            "UserUnit" => 99,
        });
        let parent = doc.add_object(dictionary! {
            "Parent" => root,
            "CropBox" => vec![30.into(), 50.into(), 230.into(), 200.into()],
            "Rotate" => -90,
        });
        let id = doc.add_object(dictionary! { "Parent" => parent });
        let geometry = page_geometry(&doc, id).unwrap();
        let view = geometry.viewport([0.0, 0.0], 1.0).unwrap();
        assert_eq!(view.size(), [150.0, 200.0]);
        assert_eq!(geometry.pdf_to_page([50.0, 80.0]).unwrap(), [60.0, 60.0]);
        assert_eq!(view.page_to_canvas([60.0, 60.0]).unwrap(), [120.0, 180.0]);
    }

    #[test]
    fn local_values_override_parents_and_null_crop_uses_media() {
        let mut doc = Document::new();
        let parent = doc.add_object(dictionary! { "Rotate" => 90 });
        let id = doc.add_object(dictionary! {
            "Parent" => parent,
            "MediaBox" => vec![0.into(), 0.into(), 100.into(), 40.into()],
            "CropBox" => Object::Null,
            "Rotate" => 0,
            "UserUnit" => 1.5_f32,
        });
        let geometry = page_geometry(&doc, id).unwrap();
        assert_eq!(geometry.viewport([0.0, 0.0], 1.0).unwrap().size(), [150.0, 60.0]);
    }

    #[test]
    fn rejects_cycles_and_malformed_local_metadata() {
        let mut doc = Document::new();
        let id = doc.new_object_id();
        doc.objects.insert(id, dictionary! { "Parent" => id }.into());
        assert_eq!(page_geometry(&doc, id).unwrap_err().to_string(), "cycle in PDF page parents");

        let parent = doc.add_object(dictionary! {
            "MediaBox" => vec![0.into(), 0.into(), 100.into(), 40.into()],
        });
        let id = doc.add_object(dictionary! {
            "Parent" => parent,
            "MediaBox" => vec![0.into(), 100.into()],
        });
        assert_eq!(
            page_geometry(&doc, id).unwrap_err().to_string(),
            "a page box must contain four coordinates"
        );
    }
}
