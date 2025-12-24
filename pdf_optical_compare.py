import numpy as np
import pymupdf as fitz
from PIL import ImageChops


def optical_compare(
    input_path1,
    input_path2,
    # diff_threshold=64,
    min_average_value=0.05 / 100,
):
    # Open the two PDFs
    pdf1 = fitz.open(input_path1)
    pdf2 = fitz.open(input_path2)

    differences = []

    # Compare each page
    for page_num in range(len(pdf1)):
        page1 = pdf1.load_page(page_num)
        page2 = pdf2.load_page(page_num)

        # Render pages to images
        pix1 = page1.get_pixmap().pil_image()
        pix2 = page2.get_pixmap().pil_image()

        # Subtract images to find differences
        diff = ImageChops.subtract(pix2, pix1)

        # Apply threshold to reduce noise
        # FIXME: Subtract reduces all values, interacts poorly with multiply
        # value_threshold = Image.new(
        #     pix1.mode, pix1.size, (diff_threshold, diff_threshold, diff_threshold)
        # )
        # diff = ImageChops.subtract(diff, value_threshold, scale=)

        # Shake to remove vertical and horizontal lines
        shake_map = ImageChops.multiply(
            ImageChops.offset(diff, 1, 0),
            ImageChops.offset(diff, 0, 1),
        )

        # Check diff is not empty
        bbox = shake_map.getbbox()
        if not bbox:
            continue  # No differences found

        # Calculate total average value
        shake_map_bytes = np.array(shake_map)
        diff_average_value = np.sum(shake_map_bytes) / shake_map_bytes.size / 255
        if diff_average_value < min_average_value:
            continue

        # Save the diff image for inspection
        diff.save(f"{input_path2}_diff{page_num + 1}.png")
        print(
            f"Difference found on page {page_num + 1}: {(diff_average_value * 100):.2f}%."
        )

        differences.append((page_num + 1, diff_average_value))

    pdf1.close()
    pdf2.close()

    if not differences:
        return (0, [])

    max_difference = max([d[1] for d in differences])

    print(f"Total differences found: {len(differences)}")
    print(f"Maximum difference found: {(max_difference * 100):.2f}%")
    return (max_difference, [d[0] for d in differences])
