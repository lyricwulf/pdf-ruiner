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
uv sync
uv run python ruin.py your_file.pdf
```

## Development

Build executable with pyinstaller

```bash
# replace with your site packages path
uv tool run pyinstaller --onefile --nowindow --clean --paths ./.venv/lib/python3.13/site-packages/ --name PdfRuin ruin.py
```