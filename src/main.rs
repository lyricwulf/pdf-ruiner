use std::path::Path;

use anyhow::Result;
use clap::Parser;
use serde::Serialize;

use crate::{strategy::RuinStrategy, util::list_files};

mod ruin;
mod strategy;
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

    #[arg(short, long, value_delimiter = ',', default_value = "rect")]
    strategy: Vec<String>,

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
    let input_dir = args.path;

    // Ensure output directory exists
    std::fs::create_dir_all(&args.out)?;

    let filelist = if Path::new(&input_dir).is_dir() {
        // if filepath is a directory, iterate over pdf files
        list_files(&input_dir, "pdf")?
    } else {
        // if filepath is a file, process that file
        vec![input_dir.clone()]
    };

    println!("Found {} PDF files to process.", filelist.len());

    let strategy = args
        .strategy
        .iter()
        .map(|s| match s.as_str() {
            "rect" => strategy::RuinStrategy::Rect,
            "image" => strategy::RuinStrategy::Image,
            "annotation" => strategy::RuinStrategy::Annotation,
            _ => panic!("Unknown strategy: {}", s),
        })
        .collect::<strategy::RuinStrategy>();

    if strategy.contains(RuinStrategy::Image) {
        println!(
            "Warning: Strategy includes images, which significantly increases processing time."
        );
    }

    // Write summary file
    let mut wtr = csv::Writer::from_path("summary.csv")?;
    for (idx, filepath) in filelist.iter().enumerate() {
        eprint!("Processing file {}/{}\r", idx + 1, filelist.len());

        let filepath_path = Path::new(filepath);
        // out_path relative to input filepath (folder)
        let out_path = Path::new(&args.out).join(Path::strip_prefix(filepath_path, &input_dir)?);

        let ruin_result = ruin::ruin_file(filepath, &out_path, &strategy)?;

        wtr.serialize(ruin_result)?;
        wtr.flush()?;
    }

    Ok(())
}
