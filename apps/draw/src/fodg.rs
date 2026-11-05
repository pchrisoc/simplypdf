//! Flat OpenDocument drawing output for the shapes supported by this app.

use std::io::{self, Write};

use simplypdf_draw::document::{Document, ShapeKind};

pub fn write(document: &Document, mut output: impl Write) -> io::Result<()> {
    writeln!(output, r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
 xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
 office:version="1.3" office:mimetype="application/vnd.oasis.opendocument.graphics">
 <office:automatic-styles>
  <style:style style:name="Shape" style:family="graphic">
   <style:graphic-properties draw:fill="solid" draw:fill-color="#bfdcf7"
    draw:stroke="solid" svg:stroke-color="#1e4f7d" svg:stroke-width="0.4mm"/>
  </style:style>"##)?;
    for (index, page) in document.pages().iter().enumerate() {
        let [width, height] = page.size.map(mm);
        let [left, top, right, bottom] = page.margins.map(mm);
        writeln!(output, r#"  <style:page-layout style:name="Layout{index}">
   <style:page-layout-properties fo:page-width="{width}" fo:page-height="{height}"
    fo:margin-left="{left}" fo:margin-top="{top}" fo:margin-right="{right}" fo:margin-bottom="{bottom}"/>
  </style:page-layout>"#)?;
    }

    writeln!(output, " </office:automatic-styles>\n <office:master-styles>\n  <draw:layer-set>")?;
    for layer in document.layers() {
        writeln!(output, r#"   <draw:layer draw:name="{}"/>"#, attribute(&layer.name))?;
    }
    writeln!(output, "  </draw:layer-set>")?;
    for index in 0..document.pages().len() {
        writeln!(output, r#"  <style:master-page style:name="Master{index}" style:page-layout-name="Layout{index}"/>"#)?;
    }
    writeln!(output, " </office:master-styles>\n <office:body>\n  <office:drawing>")?;
    for (index, page) in document.pages().iter().enumerate() {
        writeln!(output, r#"   <draw:page draw:name="Page {}" draw:master-page-name="Master{index}">"#, index + 1)?;
        for shape in &page.shapes {
            let element = match shape.kind {
                ShapeKind::Rectangle => "rect",
                ShapeKind::Ellipse => "ellipse",
            };
            let layer = document.layers().iter().find(|layer| layer.id == shape.layer)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Shape layer is missing"))?;
            let [x, y] = shape.min.map(mm);
            let width = mm(shape.max[0] - shape.min[0]);
            let height = mm(shape.max[1] - shape.min[1]);
            writeln!(
                output,
                r#"    <draw:{element} draw:layer="{}" draw:style-name="Shape" svg:x="{x}" svg:y="{y}" svg:width="{width}" svg:height="{height}"/>"#,
                attribute(&layer.name),
            )?;
        }
        writeln!(output, "   </draw:page>")?;
    }
    writeln!(output, "  </office:drawing>\n </office:body>\n</office:document>")
}

fn mm(value: u32) -> String {
    format!("{}.{:02}mm", value / 100, value % 100)
}

fn attribute(value: &str) -> String {
    value.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use simplypdf_draw::document::{A4_PORTRAIT, Shape};

    #[test]
    fn exports_page_geometry_layers_and_both_shapes() {
        let mut drawing = Document::new(A4_PORTRAIT);
        let layer = drawing.add_layer("QA & \"Review\"".into()).unwrap();
        drawing.add_shape(0, Shape::new(ShapeKind::Rectangle, layer, [123, 200], [456, 800]).unwrap());
        drawing.add_page_after(0);
        drawing.add_shape(1, Shape::new(ShapeKind::Ellipse, 0, [100, 200], [800, 900]).unwrap());
        let mut bytes = Vec::new();
        write(&drawing, &mut bytes).unwrap();
        let xml = String::from_utf8(bytes).unwrap();
        assert!(xml.contains(r#"fo:page-width="210.00mm" fo:page-height="297.00mm""#));
        assert!(xml.contains(r#"<draw:rect draw:layer="QA &amp; &quot;Review&quot;""#));
        assert!(xml.contains(r#"svg:x="1.23mm" svg:y="2.00mm" svg:width="3.33mm" svg:height="6.00mm""#));
        assert!(xml.contains(r#"<draw:ellipse draw:layer="layout""#));
        assert_eq!(xml.matches("<draw:page ").count(), 2);
    }
}
