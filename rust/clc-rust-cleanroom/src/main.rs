//! CLC cleanroom vector runner.
//!
//! `cargo run -- --vector <path>`, `--property-cases <path>`, `--crosswalk <path>`,
//! `--dir <path>`, or `--clc-v1`.

#![forbid(unsafe_code)]

use anyhow::{bail, Context};
use clap::Parser;
use clc_rust_cleanroom::{
    report_json, report_junit, report_text, run_vector_dir, run_vector_file, SuiteReport,
};
use std::path::{Path, PathBuf};
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
    /// `--properties` selects the same file.
    #[arg(long, visible_alias = "properties")]
    property_cases: Option<PathBuf>,

    /// One JSON crosswalk file. Same runner as `--vector`.
    #[arg(long)]
    crosswalk: Option<PathBuf>,

    /// Directory of JSON vector files.
    #[arg(long)]
    dir: Option<PathBuf>,

    /// Run the pinned CLC-v1 minor-gate suites and print a summary table.
    #[arg(long)]
    clc_v1: bool,

    /// `vectors/clc-v1` param-bounds and equality suites.
    #[arg(long)]
    param_bounds: bool,

    /// `vectors/clc-v1` bound-meet vectors and the 500 property cases.
    #[arg(long)]
    param_bounds_meet: bool,

    /// `vectors/clc-v1` resolve suite.
    #[arg(long)]
    resolve: bool,

    /// `vectors/clc-v1` constraint-union suites, including collation.
    #[arg(long)]
    constraint_union: bool,

    /// `vectors/clc-v1` scheme-declaration suite.
    #[arg(long)]
    scheme_declaration: bool,

    /// `vectors/clc-v1` quantity suite.
    #[arg(long)]
    quantity: bool,

    /// `vectors/clc-v1` string-domain suite.
    #[arg(long)]
    string_domain: bool,

    /// Optional `vectors/clc-v1` symmetry suite.
    #[arg(long)]
    symmetry: bool,

    /// `text` prints the colored summary and the JSON log. `json` and `junit` print one report.
    #[arg(long, default_value = "text")]
    report: String,
}

struct ClcSuite {
    label: &'static str,
    file: &'static str,
    required: bool,
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
    let classic = args.vector.is_some()
        || args.dir.is_some()
        || args.property_cases.is_some()
        || args.crosswalk.is_some();
    let suites = selected_clc_suites(&args);
    if !classic && suites.is_empty() {
        bail!(
            "pass --vector <path>, --property-cases <path>, --properties <path>, --crosswalk <path>, --dir <path>, or --clc-v1"
        );
    }
    let mut report = SuiteReport::default();
    if let Some(path) = &args.vector {
        report.merge(run_vector_file(path).with_context(|| format!("read {}", path.display()))?);
    }
    if let Some(path) = &args.property_cases {
        report.merge(run_vector_file(path).with_context(|| format!("read {}", path.display()))?);
    }
    if let Some(path) = &args.crosswalk {
        report.merge(run_vector_file(path).with_context(|| format!("read {}", path.display()))?);
    }
    if let Some(path) = &args.dir {
        report.merge(run_vector_dir(path).with_context(|| format!("read {}", path.display()))?);
    }
    if classic {
        match args.report.as_str() {
            "json" => println!("{}", report_json(&report)),
            "junit" => println!("{}", report_junit(&report)),
            _ => println!("{}", report_text(&report)),
        }
    }
    let mut clc_failed = 0usize;
    if !suites.is_empty() {
        clc_failed = run_clc_suites(&args, &suites)?;
    }
    if report.failed() > 0 || clc_failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn selected_clc_suites(args: &Args) -> Vec<ClcSuite> {
    let mut out = Vec::new();
    let all = args.clc_v1;
    if all || args.param_bounds {
        push_unique(
            &mut out,
            ClcSuite {
                label: "param-bounds",
                file: "param-bounds-vectors.json",
                required: true,
            },
        );
        push_unique(
            &mut out,
            ClcSuite {
                label: "param-bounds-equality",
                file: "param-bounds-equality-vectors.json",
                required: true,
            },
        );
    }
    if all || args.param_bounds_meet {
        push_unique(
            &mut out,
            ClcSuite {
                label: "param-bounds-meet",
                file: "param-bounds-meet-vectors.json",
                required: true,
            },
        );
        push_unique(
            &mut out,
            ClcSuite {
                label: "param-bounds-meet-property",
                file: "param-bounds-meet-property-cases.json",
                required: true,
            },
        );
    }
    if all || args.resolve {
        push_unique(
            &mut out,
            ClcSuite {
                label: "resolve",
                file: "resolve-vectors.json",
                required: true,
            },
        );
    }
    if all || args.constraint_union {
        push_unique(
            &mut out,
            ClcSuite {
                label: "constraint-union",
                file: "constraint-union-vectors.json",
                required: true,
            },
        );
        push_unique(
            &mut out,
            ClcSuite {
                label: "constraint-union-collation",
                file: "constraint-union-collation-vectors.json",
                required: true,
            },
        );
    }
    if all || args.scheme_declaration {
        push_unique(
            &mut out,
            ClcSuite {
                label: "scheme-declaration",
                file: "scheme-declaration-vectors.json",
                required: true,
            },
        );
    }
    if all || args.quantity {
        push_unique(
            &mut out,
            ClcSuite {
                label: "quantity",
                file: "quantity-vectors.json",
                required: true,
            },
        );
    }
    if all || args.string_domain {
        push_unique(
            &mut out,
            ClcSuite {
                label: "string-domain",
                file: "string-domain-vectors.json",
                required: true,
            },
        );
    }
    if all || args.symmetry {
        push_unique(
            &mut out,
            ClcSuite {
                label: "symmetry (optional)",
                file: "symmetry-vectors.json",
                required: !args.clc_v1 && args.symmetry,
            },
        );
    }
    out
}

fn push_unique(out: &mut Vec<ClcSuite>, suite: ClcSuite) {
    if out.iter().any(|item| item.file == suite.file) {
        return;
    }
    out.push(suite);
}

fn run_clc_suites(args: &Args, suites: &[ClcSuite]) -> anyhow::Result<usize> {
    let mut rows = Vec::new();
    let mut failed_required = 0usize;
    let mut failed_optional = 0usize;
    for suite in suites {
        let path = clc_path(suite.file);
        let report = run_vector_file(&path).with_context(|| format!("read {}", path.display()))?;
        let failed = report.failed();
        if suite.required {
            failed_required += failed;
        } else {
            failed_optional += failed;
        }
        if failed > 0 {
            for item in report.outcomes.iter().filter(|item| !item.passed) {
                eprintln!(
                    "FAIL  {id}  {kind}  {detail}",
                    id = item.id,
                    kind = item.kind,
                    detail = item.detail
                );
            }
        }
        rows.push((suite, report));
    }
    match args.report.as_str() {
        "json" => println!("{}", clc_json(&rows)),
        "junit" => {
            let mut merged = SuiteReport::default();
            for (_, report) in &rows {
                merged.merge(report.clone());
            }
            println!("{}", report_junit(&merged));
        }
        _ => print_clc_table(&rows),
    }
    Ok(failed_required + failed_optional)
}

fn clc_path(file: &str) -> PathBuf {
    let relative = Path::new("vectors").join("clc-v1").join(file);
    if relative.exists() {
        return relative;
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vectors")
        .join("clc-v1")
        .join(file)
}

fn print_clc_table(rows: &[( &ClcSuite, SuiteReport)]) {
    println!("CLC-v1 minor-gate  draft-wei-capability-language-core  CLC-1.19");
    println!(
        "{:<32} {:>7} {:>7} {:>7}",
        "suite", "cases", "passed", "failed"
    );
    let mut required_cases = 0usize;
    let mut required_passed = 0usize;
    let mut optional_cases = 0usize;
    let mut optional_passed = 0usize;
    for (suite, report) in rows {
        let cases = report.outcomes.len();
        let passed = report.passed();
        println!(
            "{:<32} {:>7} {:>7} {:>7}",
            suite.label,
            cases,
            passed,
            report.failed()
        );
        if suite.required {
            required_cases += cases;
            required_passed += passed;
        } else {
            optional_cases += cases;
            optional_passed += passed;
        }
    }
    println!(
        "{:<32} {:>7} {:>7} {:>7}",
        "REQUIRED",
        required_cases,
        required_passed,
        required_cases - required_passed
    );
    if optional_cases > 0 {
        println!(
            "{:<32} {:>7} {:>7} {:>7}",
            "OPTIONAL",
            optional_cases,
            optional_passed,
            optional_cases - optional_passed
        );
    }
}

fn clc_json(rows: &[(&ClcSuite, SuiteReport)]) -> String {
    let suites: Vec<serde_json::Value> = rows
        .iter()
        .map(|(suite, report)| {
            serde_json::json!({
                "suite": suite.label,
                "file": suite.file,
                "required": suite.required,
                "cases": report.outcomes.len(),
                "passed": report.passed(),
                "failed": report.failed(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "spec": "draft-wei-capability-language-core",
        "revision": "CLC-1.19",
        "profile": "minor-gate",
        "suites": suites,
    }))
    .unwrap_or_else(|_| "{}".to_string())
}
