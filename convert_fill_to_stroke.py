import logging
import re

import pikepdf
from pikepdf import Pdf


def hex_to_color_command(hex):
    hex = hex.lstrip("#")
    # convert hex to r g b floats
    (r, g, b) = tuple(int(hex[i : i + 2], 16) / 255.0 for i in (0, 2, 4))
    return f"{r} {g} {b} RG "


def convert_fill_to_stroke(input_path, output_path, color=""):
    # color, red is "1 0 0 RG " and note the space at the end !
    # Open the PDF
    pdf = Pdf.open(input_path)

    color_str = hex_to_color_command(color) if color else ""

    total_replacement_count = 0

    # Process each page
    for page_num, page in enumerate(pdf.pages, 1):
        logging.debug(f"Processing page {page_num}...")

        # Get the content stream(s)
        if "/Contents" in page:
            try:
                page_contents = page.obj.Contents

                # Check if it's an array by trying to get its length
                try:
                    # If this works, it's an array
                    num_streams = len(page_contents)
                    # It's an array - concatenate all streams
                    all_content = []
                    for i in range(num_streams):
                        stream = page_contents[i]
                        all_content.append(stream.read_bytes().decode("latin-1"))
                    content_str = "\n".join(all_content)
                except TypeError:
                    # Not an array - it's a single stream
                    content_str = page_contents.read_bytes().decode("latin-1")

                # Replace fill operators with stroke operators and set stroke color to red
                # RGB red is: 1 0 0 RG (for stroke color)
                # We'll replace the fill operator with the specified stroke color + stroke operator
                replacements = [
                    (r"(?<=\s)f(?=\s)", f"{color_str}S"),  # fill -> specified stroke
                    (
                        r"(?<=\s)F(?=\s)",
                        f"{color_str}S",
                    ),  # fill (obsolete) -> specified stroke
                    (
                        r"(?<=\s)f\*(?=\s)",
                        f"{color_str}S",
                    ),  # fill (even-odd) -> specified stroke
                    (
                        r"(?<=\s)B(?=\s)",
                        f"{color_str}S",
                    ),  # fill and stroke -> specified stroke only
                    (
                        r"(?<=\s)B\*(?=\s)",
                        f"{color_str}S",
                    ),  # fill and stroke (even-odd) -> specified stroke
                    (
                        r"(?<=\s)b(?=\s)",
                        f"{color_str}s",
                    ),  # close, fill and stroke -> specified close and stroke
                    (
                        r"(?<=\s)b\*(?=\s)",
                        f"{color_str}s",
                    ),  # close, fill and stroke (even-odd) -> specified close and stroke
                    # Also handle cases at start/end of stream
                    (r"^f(?=\s)", f"{color_str}S", re.MULTILINE),
                    (r"^F(?=\s)", f"{color_str}S", re.MULTILINE),
                    (r"^f\*(?=\s)", f"{color_str}S", re.MULTILINE),
                    (r"^B(?=\s)", f"{color_str}S", re.MULTILINE),
                    (r"^B\*(?=\s)", f"{color_str}S", re.MULTILINE),
                    (r"^b(?=\s)", f"{color_str}s", re.MULTILINE),
                    (r"^b\*(?=\s)", f"{color_str}s", re.MULTILINE),
                ]

                original_content = content_str
                replacement_count = 0

                for item in replacements:
                    if len(item) == 3:
                        pattern, replacement, flags = item
                        new_content = re.sub(
                            pattern, replacement, content_str, flags=flags
                        )
                        count = len(re.findall(pattern, content_str, flags=flags))
                        replacement_count += count
                        content_str = new_content
                    else:
                        pattern, replacement = item
                        new_content = re.sub(pattern, replacement, content_str)
                        count = len(re.findall(pattern, content_str))
                        replacement_count += count
                        content_str = new_content

                # Update the content stream
                page.Contents = pdf.make_stream(content_str.encode("latin-1"))

                total_replacement_count += replacement_count
            except Exception as e:
                logging.warning(f"  Warning: Could not process page {page_num}: {e}")

    pdf.save(output_path)
    logging.debug(f"{total_replacement_count} fill rects in {output_path}")

    return total_replacement_count


# Example usage
if __name__ == "__main__":
    input_file = "input.pdf"
    output_file = "output_stroked.pdf"

    try:
        convert_fill_to_stroke(input_file, output_file)
    except Exception as e:
        logging.error(f"Error: {e}")
