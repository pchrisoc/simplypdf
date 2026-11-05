//! Writes one deterministic, synthetic PDF with four page rotations.

use std::fmt::Write;
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let path = std::env::args_os().nth(1).map(PathBuf::from).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "pass the output PDF path")
    })?;
    std::fs::write(path, geometry_pdf())
}

fn geometry_pdf() -> String {
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Count 4 /Kids [4 0 R 5 0 R 6 0 R 7 0 R] >>".to_owned(),
    ];
    let drawing = "2 w\n50 80 m\n80 80 l\n80 120 l\nh\nS\n";
    objects.push(format!(
        "<< /Length {} >>\nstream\n{drawing}endstream",
        drawing.len()
    ));
    for degrees in [0, 90, 180, 270] {
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [-10 20 310 260] \
             /CropBox [30 50 230 200] /UserUnit 2 /Rotate {degrees} \
             /Resources << >> /Contents 3 0 R >>"
        ));
    }

    // PDF cross-reference entries are byte offsets from the start of the file.
    let mut pdf = String::from("%PDF-1.6\n");
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        writeln!(pdf, "{} 0 obj\n{object}\nendobj", index + 1).unwrap();
    }
    let xref = pdf.len();
    let count = offsets.len() + 1;
    writeln!(pdf, "xref\n0 {count}\n0000000000 65535 f ").unwrap();
    for offset in offsets {
        writeln!(pdf, "{offset:010} 00000 n ").unwrap();
    }
    writeln!(
        pdf,
        "trailer\n<< /Size {count} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF"
    )
    .unwrap();
    pdf
}
