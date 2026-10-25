//! CLC cleanroom vector runner.
//!
//! `cargo run -- --vector <path>` or `cargo run -- --dir <path>`.

#![forbid(unsafe_code)]

use anyhow::{bail, Context};
use clap::Parser;
use clc_rust_cleanroom::{report_json, report_junit, report_text, run_vector_dir, run_vector_file, SuiteReport};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    author = "Justin D. Kintzele (J Diesel NY / ECR-WG)",
    version,
    about = "CLC independent cleanroom vector runner (CLC-1.19)"
)]
struct Args {
    /// One JSON vector file.
    #[arg(long)]
    vector: Option<PathBuf>,

    /// One JSON property-cases file (e.g. property-cases.json).
    #[arg(long)]
    property_cases: Option<PathBuf>,

    /// Directory of JSON vector files.
    #[arg(long)]
    dir: Option<PathBuf>,

    /// `text` prints the colored summary and the JSON log. `json` and `junit` print one report.
    #[arg(long, default_value = "text")]
    report: String,
}

fn main() -> ExitCode {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}

fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.vector.is_none() && args.dir.is_none() && args.property_cases.is_none() {
        bail!("pass --vector <path>, --property-cases <path>, or --dir <path>");
    }
    let mut report = SuiteReport::default();
    if let Some(path) = &args.vector {
        report.merge(run_vector_file(path).with_context(|| format!("read {}", path.display()))?);
    }
    if let Some(path) = &args.property_cases {
        report.merge(run_vector_file(path).with_context(|| format!("read {}", path.display()))?);
    }
    if let Some(path) = &args.dir {
        report.merge(run_vector_dir(path).with_context(|| format!("read {}", path.display()))?);
    }
    match args.report.as_str() {
        "json" => println!("{}", report_json(&report)),
        "junit" => println!("{}", report_junit(&report)),
        _ => println!("{}", report_text(&report)),
    }
    if report.failed() > 0 {
        std::process::exit(1);
    }
    Ok(())
}
