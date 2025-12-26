use pdfium_render::prelude::*;

// Helper function to determine if a path object is a rectangle
pub fn is_rectangle(object: &PdfPageObject) -> bool {
    if let Some(path) = object.as_path_object() {
        // A rectangle typically has 4 or 5 segments (5 if closed with a line back to start)
        let segment_count = path.segments().len();

        // Check if it has the right number of segments for a rectangle
        if segment_count == 4 || segment_count == 5 {
            // Additional checks could be done here to verify it's actually rectangular
            // (e.g., checking angles, parallel sides, etc.)
            return true;
        }
    }
    false
}
