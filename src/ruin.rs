use crate::strategy::RuinStrategy;
use anyhow::Result;
use pdfium_render::prelude::*;

use crate::{
    RuinedInfo,
    util::{is_rectangle, optical_compare},
};

pub fn ruin_file(
    filepath: &str,
    out_path: &std::path::Path,
    strategy: &RuinStrategy,
) -> Result<RuinedInfo> {
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
    eprint!(
        "Found file {:<.2} MB: {}\r",
        original_bytes.len() as f64 / 1024.0 / 1024.0,
        filepath,
    );
    let ruined_document = pdfium.load_pdf_from_byte_vec(original_bytes, None)?;

    let mut pages_changed = vec![];

    // Process each page
    for (page_index, mut page) in ruined_document.pages().iter().enumerate() {
        let mut modified = false;

        // Defer page regeneration until after all modifications
        page.set_content_regeneration_strategy(PdfPageContentRegenerationStrategy::Manual);

        // Get all objects on the page
        let objects = page.objects();

        // Iterate through all objects
        for mut object in objects.iter() {
            match object {
                PdfPageObject::Path(ref mut path_obj) => {
                    if !(strategy.contains(RuinStrategy::Rect)) {
                        continue;
                    }

                    // Check if this path object is a rectangle
                    if is_rectangle(path_obj) {
                        // Remove fill and ensure stroke is enabled
                        path_obj.set_fill_and_stroke_mode(PdfPathFillMode::None, true)?;

                        // Optionally set stroke color and width if needed
                        // path_obj.set_fill_color(PdfColor::new(255, 255, 0, 64))?;
                        // path_obj.set_stroke_color(PdfColor::new(255, 0, 0, 255))?;
                        // path_obj.set_stroke_width(PdfPoints::new(1.0))?;
                        modified = true;
                    }
                }
                PdfPageObject::Image(ref mut img_obj) => {
                    if !(strategy.contains(RuinStrategy::Image)) {
                        continue;
                    }
                    // let image = img_obj.get_raw_bitmap()?;
                    // let processed_image = img_obj.get_processed_bitmap(&ruined_document)?;
                    let _filter_count = img_obj.filters().len();
                    let _filter_names = img_obj
                        .filters()
                        .iter()
                        .map(|f| f.name().to_owned())
                        .collect::<Vec<_>>()
                        .join(", ");
                }
                _ => {}
            }
        }

        // Regenerate the page content stream if we made changes
        if modified {
            pages_changed.push(page_index as PdfPageIndex);
            page.regenerate_content()?;
        }
    }

    if pages_changed.is_empty() {
        return Ok(RuinedInfo {
            file_name: filepath.to_string(),
            max_difference: 0.0,
            diff_pages: String::new(),
            modify_time: begin_modify_time.elapsed().as_secs_f32(),
            analyze_time: 0.0,
        });
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
