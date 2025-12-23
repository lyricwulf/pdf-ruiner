import argparse
import csv
import logging
import os

from convert_fill_to_stroke import convert_fill_to_stroke

logging.basicConfig(level=logging.INFO)


def main():
    parser = argparse.ArgumentParser(
        prog="pdf-ruiner",
        description="ruins your pdf files in a terrible way",
    )

    parser.add_argument("input_path")
    parser.add_argument("-o", "--output", default="ruined")
    parser.add_argument("-c", "--color")

    args = parser.parse_args()

    # read files in directory
    dir_or_file = args.input_path
    out_dir = args.output

    # ensure output directory
    os.makedirs(out_dir, exist_ok=True)

    out_csv_path = "summary.csv"
    summary_data = [["filename", "fill_rects_converted"]]

    file_list = []
    if os.path.isdir(dir_or_file):
        dirname = dir_or_file
        file_list = os.listdir(dirname)
    else:
        dirname = os.path.dirname(dir_or_file)
        file_list = [os.path.basename(dir_or_file)]

    for i, filename in enumerate(file_list):
        if filename.endswith(".pdf"):
            input_path = os.path.join(dirname, filename)
            print(f"Processing {i + 1}/{len(file_list)}: {input_path}...")
            # return console cursor to start of line
            print("\033[F\033[K", end="")
            output_path = os.path.join(out_dir, filename)

            num_rects = convert_fill_to_stroke(
                input_path, output_path, color=args.color
            )
            summary_data.append([filename, num_rects])

    # write summary
    with open(out_csv_path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerows(summary_data)

    print(f"Summary written to {out_csv_path}")
    print("Done.")


if __name__ == "__main__":
    main()
