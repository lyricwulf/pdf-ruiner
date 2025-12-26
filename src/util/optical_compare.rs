use anyhow::Result;
use image::{GrayImage, ImageBuffer};
use mupdf::{Colorspace, Matrix, document::Document};
use ndarray::{Array3, s};
use pdfium_render::prelude::*;

fn render_pdfium_page_to_image(pdf: &PdfDocument, page_index: PdfPageIndex) -> Result<GrayImage> {
    let page = pdf.pages().get(page_index)?;
    let render_config = PdfRenderConfig::new();
    let img = page.render_with_config(&render_config)?.as_image();

    Ok(img.to_luma8())
}

fn render_mupdf_page_to_image(doc: &Document, page_index: u16) -> Result<GrayImage> {
    let page = doc.load_page(page_index as i32)?;

    let pix = page.to_pixmap(&Matrix::IDENTITY, &Colorspace::device_rgb(), false, false)?;

    let width = pix.width() as u32;
    let height = pix.height() as u32;
    let raw = pix.samples();

    let img = ImageBuffer::from_raw(width, height, raw.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Failed to create image from rendered page"))?;

    Ok(img)
}

pub fn optical_compare(
    pdf1: &PdfDocument,
    pdf2: &PdfDocument,
    pages: Option<&[PdfPageIndex]>,
    min_average_value: f32,
) -> Result<(f32, String)> {
    // let mupdf_doc1 = Document::from_bytes(&pdf1.save_to_bytes()?, "")?;
    // let mupdf_doc2 = Document::from_bytes(&pdf2.save_to_bytes()?, "")?;

    let mut differences = Vec::new();

    // Compare each page
    let page_count: PdfPageIndex = pdf1.pages().len();

    for page_num in page_iter(pages, page_count as u16) {
        let img1 = render_pdfium_page_to_image(pdf1, page_num)?;
        let img2 = render_pdfium_page_to_image(pdf2, page_num)?;

        // let img1 = render_mupdf_page_to_image(&mupdf_doc1, page_num)?;
        // let img2 = render_mupdf_page_to_image(&mupdf_doc2, page_num)?;Z

        let (width, height) = img1.dimensions();

        let arr1 = image_to_array(&img1);
        let arr2 = image_to_array(&img2);

        // Subtract images to find differences
        let diff = subtract_images(&arr2, &arr1);

        // Only include pixels brighter than a threshold to reduce noise
        let diff_thresholded = threshold_image(&diff, 64);
        if !has_nonzero(&diff_thresholded) {
            continue;
        }

        // Multiply by blurred
        let blurred =
            imageproc::filter::box_filter(&array_to_image(&diff_thresholded, width, height), 2, 2);
        let weighted = multiply_image(&diff_thresholded, &image_to_array(&blurred));
        let weighted_thresholded = threshold_image(&weighted, 32);

        // Calculate average difference value
        let diff_average_value = calculate_average_value(&weighted_thresholded);

        // Save the diff image for inspection
        // let output_path = format!("pdf_shake_page{}.png", page_num + 1);
        // array_to_image(&weighted_thresholded, width, height).save(&output_path)?;
        // let output_path = format!("pdf_contrast_page{}.png", page_num + 1);
        // blurred.save(&output_path)?;
        // let output_path = format!("pdf_diff_raw_page{}.png", page_num + 1);
        // array_to_image(&diff, width, height).save(&output_path)?;

        if diff_average_value < min_average_value {
            continue;
        }

        differences.push((page_num + 1, diff_average_value));
    }

    if differences.is_empty() {
        return Ok((0.0, String::new()));
    }

    let max_difference = differences
        .iter()
        .map(|(_, diff)| *diff)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    let diff_pages: String = differences
        .iter()
        .map(|(page, _)| page.to_string())
        .collect::<Vec<_>>()
        .join(",");

    Ok((max_difference, diff_pages))
}

// Subtract img1 from img2 using ndarray for speed
fn subtract_images(arr2: &Array3<u8>, arr1: &Array3<u8>) -> Array3<u8> {
    // Perform subtraction with saturation
    let diff = arr2.mapv(|v| v as i16) - arr1.mapv(|v| v as i16);

    diff.mapv(|v| v.max(0) as u8)
}

// Apply shake map: multiply offset(1,1) to remove lines
fn apply_shake_map(img: &GrayImage) -> GrayImage {
    let (width, height) = img.dimensions();
    let arr = image_to_array(img);

    // Create offset arrays
    let mut offset_x = Array3::<u8>::zeros((height as usize, width as usize, 3));

    // Shift by 1 pixel in X and Y
    offset_x
        .slice_mut(s![0..height as usize - 1, 0..width as usize - 1, ..])
        .assign(&arr.slice(s![1.., 1.., ..]));
    offset_x
        .slice_mut(s![height as usize - 1, width as usize - 1, ..])
        .assign(&arr.slice(s![height as usize - 1, width as usize - 1, ..]));

    // Multiply the two offset arrays
    let result = (offset_x.mapv(|v| v as u16) * arr.mapv(|v| v as u16)).mapv(|v| (v / 255) as u8); // Normalize back to u8
    let result = result.mapv(|v| v as u8);

    array_to_image(&result, width, height)
}

fn multiply_image(arr1: &Array3<u8>, arr2: &Array3<u8>) -> Array3<u8> {
    (arr1.mapv(|v| v as u16) * arr2.mapv(|v| v as u16)).mapv(|v| (v / 255) as u8)
}

fn threshold_image(arr: &Array3<u8>, threshold: u8) -> Array3<u8> {
    arr.mapv(|v| if v >= threshold { v } else { 0 })
}

// Check if image has any non-zero pixels
fn has_nonzero(arr: &Array3<u8>) -> bool {
    arr.iter().any(|&v| v != 0)
}

// Calculate average pixel value (normalized to 0.0-1.0)
fn calculate_average_value(arr: &Array3<u8>) -> f32 {
    let sum: u64 = arr.iter().map(|&v| v as u64).sum();
    let total_elements = arr.len() as f32;
    sum as f32 / total_elements / 255.0
}

// Helper: Convert GrayImage to ndarray
fn image_to_array(img: &GrayImage) -> Array3<u8> {
    let (width, height) = img.dimensions();
    let raw = img.as_raw();

    let channels = 1; // Grayscale has 1 channel

    Array3::from_shape_vec((height as usize, width as usize, channels), raw.clone()).unwrap()
}

// Helper: Convert ndarray to GrayImage
fn array_to_image(arr: &Array3<u8>, width: u32, height: u32) -> GrayImage {
    let flat: Vec<u8> = arr.iter().copied().collect();
    ImageBuffer::from_raw(width, height, flat).unwrap()
}

fn page_iter(pages: Option<&[u16]>, page_count: u16) -> Box<dyn Iterator<Item = u16> + '_> {
    match pages {
        Some(pages) => Box::new(pages.iter().copied()),
        None => Box::new(0..page_count),
    }
}
