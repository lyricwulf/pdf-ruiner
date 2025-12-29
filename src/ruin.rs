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
    let pdfium_binding =
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib"))
            .or_else(|_| Pdfium::bind_to_system_library())?;
    let pdfium = Pdfium::new(pdfium_binding);

    let begin_modify_time = std::time::Instant::now();

    let document_bytes = std::fs::read(filepath)?;

    // Open the document twice. Could there be a way to avoid re-parsing?
    let original_document = pdfium.load_pdf_from_byte_vec(document_bytes.clone(), None)?;
    let ruined_document = pdfium.load_pdf_from_byte_vec(document_bytes, None)?;

    let mut pages_changed = vec![];

    // Process each page
    for (page_index, mut page) in ruined_document.pages().iter().enumerate() {
        let mut modified = false;

        // Defer page regeneration until after all modifications
        page.set_content_regeneration_strategy(PdfPageContentRegenerationStrategy::Manual);

        if strategy.contains(RuinStrategy::Annotation) {
            let handle_generic_annotation = |annotation: &PdfPageAnnotation| {
                println!(
                    "{:?} {:.1?} => pg.{}, \"{}\"",
                    annotation.annotation_type(),
                    bounds_dimensions(&annotation.bounds().unwrap()),
                    page_index + 1,
                    filepath,
                );
            };

            for mut annotation in page.annotations().iter() {
                if annotation.is_hidden() || !annotation.is_printed() {
                    continue;
                }
                match &annotation {
                    PdfPageAnnotation::Circle(circle) => handle_generic_annotation(&annotation),
                    PdfPageAnnotation::FreeText(freetext) => {
                        println!(
                            "Annotation::FreeText {}",
                            freetext.contents().unwrap_or_default()
                        );
                    }
                    PdfPageAnnotation::Highlight(highlight) => {
                        handle_generic_annotation(&annotation)
                    }
                    PdfPageAnnotation::Ink(ink) => handle_generic_annotation(&annotation),
                    PdfPageAnnotation::Link(link) => {
                        // let annotation_label = if let Some(action) = link.link()?.action() {
                        //     match action {
                        //         PdfAction::LocalDestination(dest) => "<LocalDestination>",
                        //         PdfAction::RemoteDestination(dest) => "<RemoteDestination>",
                        //         PdfAction::EmbeddedDestination(dest) => "<EmbeddedDestination>",
                        //         PdfAction::Launch(launch) => "<Launch>",
                        //         // PdfAction::Uri(uri) => match uri.uri() {
                        //         //     Ok(uri_str) => {
                        //         //         println!("{}", uri_str);
                        //         //         "<URI>"
                        //         //     }
                        //         //     Err(_) => "<No URI>",
                        //         // },
                        //         PdfAction::Unsupported(unsupported) => "<Unsupported>",
                        //         _ => "",
                        //     }
                        // } else {
                        //     "<No Action>"
                        // };
                        // if !annotation_label.is_empty() {
                        //     println!("Annotation::Link {}", annotation_label);
                        //     continue;
                        // }
                    }
                    PdfPageAnnotation::Popup(popup) => {
                        annotation.set_is_hidden(true)?;
                        annotation.set_is_printed(false)?;
                        annotation.set_bounds(PdfRect::zero())?;
                        modified = true;
                    }
                    PdfPageAnnotation::Square(square) => handle_generic_annotation(&annotation),
                    PdfPageAnnotation::Squiggly(squiggly) => handle_generic_annotation(&annotation),
                    PdfPageAnnotation::Stamp(stamp) => {
                        annotation.set_is_hidden(true)?;
                        annotation.set_is_printed(false)?;
                        annotation.set_bounds(PdfRect::zero())?;
                        modified = true;
                    }
                    PdfPageAnnotation::Strikeout(strikeout) => {
                        handle_generic_annotation(&annotation)
                    }
                    PdfPageAnnotation::Text(text) => {
                        println!("Annotation::Text {}", text.contents().unwrap_or_default());
                    }
                    PdfPageAnnotation::Underline(underline) => {
                        handle_generic_annotation(&annotation)
                    }
                    PdfPageAnnotation::Widget(widget) => {}
                    PdfPageAnnotation::XfaWidget(xfawidget) => {
                        handle_generic_annotation(&annotation)
                    }
                    PdfPageAnnotation::Redacted(redacted) => handle_generic_annotation(&annotation),
                    PdfPageAnnotation::Unsupported(f) => {
                        f.get_type();
                    }
                    _ => handle_generic_annotation(&annotation),
                }
            }
        }

        // Iterate through all objects on the page
        for mut object in page.objects().iter() {
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
                    let width = img_obj.width()?;
                    let height = img_obj.height()?;
                    let most_frequent_ratio = {
                        let processed_image = img_obj.get_processed_bitmap(&ruined_document)?;
                        let processed_image = processed_image.as_image().to_luma8();
                        // Check if the image is mostly uniform
                        let histogram = imageproc::stats::histogram(&processed_image).channels;
                        let most_frequent_pixel_count = histogram[0].iter().max().unwrap_or(&0);
                        let total_pixels = (width * height) as u32;

                        *most_frequent_pixel_count as f32 / total_pixels as f32
                    };
                    if most_frequent_ratio >= 0.99 {
                        // Replace with a blank image
                        let blank_image = PdfBitmap::empty(
                            width as _,
                            height as _,
                            PdfBitmapFormat::BGRA,
                            pdfium.bindings(),
                        )?;
                        img_obj.set_bitmap(&blank_image)?;
                        modified = true;
                    }
                }
                _ => {}
            }
        }

        // Regenerate the page content stream if we made changes
        if modified {
            page.regenerate_content()?;
            pages_changed.push(page_index as PdfPageIndex);
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

fn bounds_dimensions(bounds: &PdfRect) -> (f32, f32) {
    let width = bounds.right() - bounds.left();
    let height = bounds.top() - bounds.bottom();
    (width.value, height.value)
}
