use pdfium_render::prelude::*;

// Helper function to determine if a path object is a rectangle
pub fn is_rectangle(path: &PdfPagePathObject) -> bool {
    // A rectangle typically has 4 or 5 segments (5 if closed with a line back to start)
    // Check if it has the right number of segments for a rectangle
    let segment_count = path.segments().len();
    if !(segment_count == 4 || segment_count == 5) {
        return false;
    }

    // Check the height and width from the bounding box
    let min_size = PdfPoints::new(4.0); // Minimum size to consider
    let bbox = path.bounds().unwrap();
    let width = bbox.width();
    let height = bbox.height();
    if width < min_size || height < min_size {
        return false;
    }

    // Additional checks could be done to verify it's actually rectangular
    // (e.g., checking angles, parallel sides, etc.)

    true
}
