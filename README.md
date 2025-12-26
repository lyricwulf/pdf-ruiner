# pdf-ruiner

ruins your pdf files in a terrible way.

all solid rectangles are converted to hollow rectangles, completely ruining the
appearance of the document.

might be useful if you hate pdfs.

## Usage

With executable

```bash
PdfRuin your_file.pdf
PdfRuin input_directory
PdfRuin input_directory -o out
```

From source

```bash
cargo
cargo run --release -- your_file.pdf
```

## Development

Build executable 

```bash
cargo build --release
```