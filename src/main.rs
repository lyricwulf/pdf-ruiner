use anyhow::Result;
use clap::Parser;
use serde::Serialize;

use crate::util::list_files;

mod ruin;
mod util;

// Args
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to process
    path: String,

    /// Output directory
    #[arg(short, long, default_value = "ruined")]
    out: String,

    #[arg(short, long)]
    color: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct RuinedInfo {
    file_name: String,
    max_difference: f32,
    diff_pages: String,
    modify_time: f32,
    analyze_time: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let filepath = args.path;

    // Ensure output directory exists
    std::fs::create_dir_all(&args.out)?;

    let filelist = if std::path::Path::new(&filepath).is_dir() {
        // if filepath is a directory, iterate over pdf files
        list_files(&filepath, "pdf")?
    } else {
        // if filepath is a file, process that file
        vec![filepath.clone()]
    };

    let mut wtr = csv::Writer::from_path("summary.csv")?;
    for filepath in &filelist {
        let out_path = std::path::Path::new(&args.out)
            .join(std::path::Path::new(filepath).file_name().unwrap());
        let ruin_result = ruin::ruin_file(filepath, &out_path)?;

        wtr.serialize(ruin_result)?;
        wtr.flush()?;
    }

    Ok(())
}
