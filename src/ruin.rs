use anyhow::Result;
use pdfium_render::prelude::*;

use crate::{
    RuinedInfo,
    util::{is_rectangle, optical_compare},
};

pub fn ruin_file(filepath: &str, out_path: &std::path::Path) -> Result<RuinedInfo> {
    // Initialize Pdfium
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );

    let begin_modify_time = std::time::Instant::now();

    // Open the PDF document
    let original_document = pdfium.load_pdf_from_file(filepath, None)?;
    // is there a better way to clone this?
    let original_bytes = original_document.save_to_bytes()?;
    println!(
        "Found file {:<.2} MB: {}",
        original_bytes.len() as f64 / 1024.0 / 1024.0,
        filepath,
    );
    let ruined_document = pdfium.load_pdf_from_byte_vec(original_bytes, None)?;

    let mut pages_changed = vec![];

    // Process each page
    for (page_index, mut page) in ruined_document.pages().iter().enumerate() {
        // Get all objects on the page
        let objects = page.objects();

        let mut modified = false;

        // Iterate through all objects
        for mut object in objects.iter() {
            // Check if the object is a path (rectangles are path objects)
            if object.object_type() == PdfPageObjectType::Path {
                // Check if this path object is a rectangle
                if is_rectangle(&object) {
                    // Remove fill and ensure stroke is enabled
                    if let Some(path) = object.as_path_object_mut() {
                        path.set_fill_and_stroke_mode(PdfPathFillMode::None, true)?;
                        if !modified {
                            modified = true;
                            pages_changed.push(page_index as PdfPageIndex);
                        }

                        // Optionally set stroke color and width if needed
                        // path.set_stroke_color(PdfColor::new(255, 0, 0, 255))?;
                        // path.set_stroke_width(PdfPoints::new(1.0))?;
                    }
                }
            }
        }

        // Regenerate the page content stream if we made changes
        if modified {
            page.regenerate_content()?;
        }
    }

    let begin_compare_time = std::time::Instant::now();

    let (max_difference, diff_pages) = optical_compare(
        &original_document,
        &ruined_document,
        Some(&pages_changed),
        0.01 / 100.0,
    )?;

    // Save the modified PDF
    ruined_document.save_to_file(out_path)?;

    let modify_time = begin_compare_time
        .duration_since(begin_modify_time)
        .as_secs_f32();
    let analyze_time = begin_compare_time.elapsed().as_secs_f32();

    Ok(RuinedInfo {
        file_name: filepath.to_string(),
        max_difference,
        diff_pages,
        modify_time,
        analyze_time,
    })
}
