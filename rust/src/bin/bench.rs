//! Benchmark and compare solver variants.
//!
//! `bench run` builds the solver (with `--features stats` unless `--no-stats`), runs it
//! several times with `--stats-json`, records peak memory (via BSD/GNU `/usr/bin/time`),
//! checks every produced strategy with `cxx/strategy_check`, then prints the summary of all
//! runs in the output directory. Each label's binary is copied into the output directory,
//! so variants built from different code can be compared side by side:
//!
//! ```bash
//! cargo run --release --bin bench -- run -n 5 -l before -- -q 4 -s 8
//! # ...change the code...
//! cargo run --release --bin bench -- run -n 5 -l after -- -q 4 -s 8
//! ```
//!
//! `bench summary <files>` prints the comparison table for existing statistics files.

use clap::{Args, Parser, Subcommand};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::process::CommandExt,
    time::{Duration, Instant},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const RUST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const SOLVER: &str = "segment-colouring";

#[derive(Parser)]
#[command(about = "Benchmark and compare solver variants")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the solver several times, check each strategy, and print the summary
    Run(RunArgs),
    /// Print the comparison table for statistics files (written by `--stats-json`)
    Summary {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
}

#[derive(Args)]
struct RunArgs {
    /// Number of runs
    #[arg(short = 'n', long, default_value_t = 3)]
    runs: usize,
    /// Label of this variant [default: current git commit, plus "-dirty"]
    #[arg(short, long)]
    label: Option<String>,
    /// Output directory for logs, strategies and statistics; runs of different labels can
    /// share it and are compared side by side
    #[arg(short = 'd', long, default_value = "bench-out")]
    out_dir: PathBuf,
    /// Build without `--features stats` (to measure the counters' overhead)
    #[arg(long)]
    no_stats: bool,
    /// Reuse the binary already copied for this label
    #[arg(long)]
    no_build: bool,
    /// Kill a run (and everything it started) after this many seconds; it is then
    /// reported as `timeout`, with statistics up to the last completed depth
    #[arg(long)]
    timeout: Option<u64>,
    /// Solver arguments, after `--`. Don't pass -o/--output, --stats-json or --label.
    #[arg(last = true)]
    solver_args: Vec<String>,
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Cmd::Run(args) => run(args),
        Cmd::Summary { files } => summary(&files),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------------------
// bench run
// ---------------------------------------------------------------------------------------

fn run(args: RunArgs) -> Result<(), String> {
    let rust_dir = Path::new(RUST_DIR);
    let cxx_dir = rust_dir.join("../cxx");
    let label = args.label.clone().unwrap_or_else(|| default_label(rust_dir));
    fs::create_dir_all(&args.out_dir).map_err(|e| format!("{}: {e}", args.out_dir.display()))?;
    let out_dir = args.out_dir.canonicalize().map_err(|e| e.to_string())?;
    let bin = out_dir.join(format!("bin-{label}"));

    if !args.no_build {
        build_solver(rust_dir, !args.no_stats, &bin)?;
    }
    if !bin.exists() {
        return Err(format!("no binary for label {label}: {}", bin.display()));
    }

    let checker = cxx_dir.join("strategy_check");
    if !checker.exists() {
        run_ok(Command::new("make").arg("-C").arg(&cxx_dir).arg("strategy_check"))?;
    }

    let time = Path::new("/usr/bin/time").exists().then(|| {
        if cfg!(target_os = "macos") {
            "-l"
        } else {
            "-v"
        }
    });

    for i in 1..=args.runs {
        let base = out_dir.join(format!("{label}-{i}"));
        let path = |ext: &str| base.with_extension(ext);
        let (strategy, stats) = (path("strategy"), path("jsonl"));
        let _ = fs::remove_file(&strategy);
        let _ = fs::remove_file(&stats);
        println!(
            "[{label} {i}/{}] {}",
            args.runs,
            args.solver_args.join(" ")
        );

        let mut cmd = match time {
            Some(flag) => {
                let mut c = Command::new("/usr/bin/time");
                c.arg(flag).arg(&bin);
                c
            }
            None => Command::new(&bin),
        };
        cmd.args(&args.solver_args)
            .arg("-o")
            .arg(&strategy)
            .arg("--stats-json")
            .arg(&stats)
            .arg("--label")
            .arg(&label)
            .stdout(create(&path("log"))?)
            .stderr(create(&path("time"))?);
        // Own process group, so a timeout kills the solver too, not just `/usr/bin/time`.
        cmd.process_group(0);
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("cannot run {}: {e}", bin.display()))?;
        let deadline = args.timeout.map(|t| Instant::now() + Duration::from_secs(t));
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break Some(status);
            }
            if deadline.is_some_and(|d| Instant::now() >= d) {
                let group = format!("-{}", child.id());
                let _ = Command::new("kill").args(["-KILL", "--", &group]).status();
                let _ = child.wait();
                break None;
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        let Some(status) = status else {
            eprintln!("  timed out after {}s", args.timeout.unwrap_or_default());
            append_record(&stats, json!({ "type": "timeout", "seconds": args.timeout }))?;
            continue;
        };
        if !status.success() {
            eprintln!(
                "  run failed ({status}), see {} and {}",
                path("log").display(),
                path("time").display()
            );
            continue;
        }

        if let Some(rss) = fs::read_to_string(path("time")).ok().and_then(|t| parse_peak_rss(&t)) {
            append_record(&stats, json!({ "type": "rss", "max_rss_bytes": rss }))?;
        }
        if strategy.exists() {
            let output = Command::new(&checker)
                .arg(&strategy)
                .current_dir(&cxx_dir)
                .output()
                .map_err(|e| format!("cannot run {}: {e}", checker.display()))?;
            let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&output.stderr));
            fs::write(path("check"), &text).map_err(|e| e.to_string())?;
            let ok = output.status.success() && text.contains("Strategy is correct");
            if !ok {
                eprintln!("  STRATEGY CHECK FAILED, see {}", path("check").display());
            }
            append_record(&stats, json!({ "type": "check", "ok": ok }))?;
        }
    }

    let mut files: Vec<PathBuf> = fs::read_dir(&out_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    files.sort();
    summary(&files)
}

/// Current commit (read-only git), with `-dirty` if the working tree differs from it.
fn default_label(rust_dir: &Path) -> String {
    let git = |args: &[&str]| Command::new("git").arg("-C").arg(rust_dir).args(args).output();
    let Ok(rev) = git(&["rev-parse", "--short", "HEAD"]) else {
        return "run".to_string();
    };
    if !rev.status.success() {
        return "run".to_string();
    }
    let mut label = String::from_utf8_lossy(&rev.stdout).trim().to_string();
    if git(&["diff", "--quiet", "HEAD"]).is_ok_and(|o| !o.status.success()) {
        label.push_str("-dirty");
    }
    label
}

/// Build the solver binary and copy it to `dest`, so later rebuilds don't change it.
fn build_solver(rust_dir: &Path, stats: bool, dest: &Path) -> Result<(), String> {
    let mut cmd = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    cmd.current_dir(rust_dir)
        .args(["build", "--release", "-q", "--bin", SOLVER]);
    if stats {
        cmd.args(["--features", "stats"]);
    }
    run_ok(&mut cmd)?;
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| rust_dir.join("target"));
    let built = target.join("release").join(SOLVER);
    fs::copy(&built, dest).map_err(|e| format!("copy {}: {e}", built.display()))?;
    Ok(())
}

fn run_ok(cmd: &mut Command) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{cmd:?}: {e}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("{cmd:?} failed: {status}"))
}

fn create(path: &Path) -> Result<File, String> {
    File::create(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn append_record(path: &Path, record: Value) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    writeln!(file, "{record}").map_err(|e| e.to_string())
}

/// Peak RSS in bytes from BSD `time -l` (bytes) or GNU `time -v` (kbytes) output.
fn parse_peak_rss(time_output: &str) -> Option<u64> {
    for line in time_output.lines() {
        if line.contains("maximum resident set size") {
            return line.split_whitespace().next()?.parse().ok();
        }
        if line.contains("Maximum resident set size") {
            let kbytes: u64 = line.split_whitespace().last()?.parse().ok()?;
            return Some(kbytes * 1024);
        }
    }
    None
}

// ---------------------------------------------------------------------------------------
// bench summary
// ---------------------------------------------------------------------------------------

const CALLERS: [&str; 4] = ["root", "move", "front", "back"];
const OUTCOMES: [&str; 5] = ["win", "lose_deep", "lose_shallow", "in_progress", "unknown"];

/// One solver run, read from its statistics file.
#[derive(Default)]
struct RunData {
    label: String,
    params: String,
    stats_enabled: bool,
    depths: Vec<Value>,
    end: Option<Value>,
    rss: Option<f64>,
    check: Option<bool>,
    timed_out: bool,
}

impl RunData {
    fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut run = RunData {
            label: "?".into(),
            ..Default::default()
        };
        for (n, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
            let rec: Value = serde_json::from_str(line)
                .map_err(|e| format!("{}:{}: {e}", path.display(), n + 1))?;
            match rec["type"].as_str() {
                Some("run") => {
                    run.label = rec["label"].as_str().unwrap_or("?").to_string();
                    run.stats_enabled = rec["stats_enabled"].as_bool().unwrap_or(false);
                    run.params = format!(
                        "MQ={} MS={} threads={}{}{}",
                        rec["mq"],
                        rec["ms"],
                        rec["threads"],
                        if rec["store_at_limit"] == true { " store-at-limit" } else { "" },
                        if run.stats_enabled { "" } else { " (no stats)" }
                    );
                }
                Some("depth") => {
                    // Fallback when `/usr/bin/time` isn't available: the solver's own peak.
                    if let Some(peak) = rec["memory"]["peak_rss_bytes"].as_f64() {
                        run.rss = Some(run.rss.map_or(peak, |r: f64| r.max(peak)));
                    }
                    run.depths.push(rec)
                }
                Some("end") => run.end = Some(rec),
                Some("rss") => run.rss = rec["max_rss_bytes"].as_f64(),
                Some("check") => run.check = rec["ok"].as_bool(),
                Some("timeout") => run.timed_out = true,
                _ => {}
            }
        }
        Ok(run)
    }

    /// A counter summed over all depth iterations; `None` without `--features stats`.
    fn counter(&self, name: &str) -> Option<f64> {
        let totals: Vec<f64> = self
            .depths
            .iter()
            .filter_map(|d| d.get("counters")?.get(name)?.get("total")?.as_f64())
            .collect();
        (!totals.is_empty()).then(|| totals.iter().sum())
    }

    fn lookups(&self, caller: &str) -> Option<f64> {
        OUTCOMES
            .iter()
            .map(|o| self.counter(&format!("lookup_{caller}_{o}")))
            .sum()
    }

    /// One outcome summed over all callers.
    fn outcome(&self, outcome: &str) -> Option<f64> {
        CALLERS
            .iter()
            .map(|c| self.counter(&format!("lookup_{c}_{outcome}")))
            .sum()
    }

    fn metrics(&self) -> Vec<(String, Metric)> {
        let end = self.end.as_ref();
        let text = |s: &str| Metric::Text(s.to_string());
        let mut m = vec![
            (
                "result".to_string(),
                text(match end.and_then(|e| e["result"].as_str()) {
                    Some(result) => result,
                    None if self.timed_out => "timeout",
                    None => "unfinished",
                }),
            ),
            (
                "total time [s]".into(),
                Metric::Num(end.and_then(|e| e["total_ms"].as_f64()).map(|ms| ms / 1000.0)),
            ),
            (
                "final depth".into(),
                Metric::Num(self.depths.last().and_then(|d| d["depth"].as_f64())),
            ),
            ("iterations".into(), Metric::Num(Some(self.depths.len() as f64))),
            (
                "stored states".into(),
                // A timed-out run has no `end`; use the last completed depth instead.
                Metric::Num(end.and_then(|e| e["visited_states"].as_f64()).or_else(|| {
                    self.depths.last().and_then(|d| d["map"]["states"].as_f64())
                })),
            ),
            (
                "peak RSS [MB]".into(),
                Metric::Num(self.rss.map(|b| b / (1024.0 * 1024.0))),
            ),
            (
                "strategy check".into(),
                text(match self.check {
                    Some(true) => "ok",
                    Some(false) => "FAILED",
                    None => "-",
                }),
            ),
        ];
        if self.stats_enabled {
            let all_lookups: Option<f64> = CALLERS.iter().map(|c| self.lookups(c)).sum();
            m.extend([
                ("lookups".into(), Metric::Num(all_lookups)),
                ("expansions".into(), Metric::Num(self.counter("expansion"))),
                ("colour branches".into(), Metric::Num(self.counter("colour_branch"))),
                (
                    "re-searched (shallow)".into(),
                    Metric::Num(self.outcome("lose_shallow")),
                ),
                ("in-progress hits".into(), Metric::Num(self.outcome("in_progress"))),
                ("map writes".into(), Metric::Num(self.counter("status_write"))),
            ]);
            for caller in ["move", "front", "back"] {
                let unknown = self.counter(&format!("lookup_{caller}_unknown"));
                let rate = match (unknown, self.lookups(caller)) {
                    (Some(u), Some(l)) if l > 0.0 => Some(100.0 * u / l),
                    _ => None,
                };
                m.push((format!("unknown % ({caller})"), Metric::Percent(rate)));
            }
        }
        m
    }
}

enum Metric {
    Num(Option<f64>),
    Percent(Option<f64>),
    Text(String),
}

fn summary(files: &[PathBuf]) -> Result<(), String> {
    let mut by_label: BTreeMap<String, Vec<RunData>> = BTreeMap::new();
    for file in files {
        let run = RunData::load(file)?;
        by_label.entry(run.label.clone()).or_default().push(run);
    }
    if by_label.is_empty() {
        return Err("no statistics files".into());
    }
    let labels: Vec<&String> = by_label.keys().collect();

    // Rows in first-seen order; one cell per label.
    let mut rows: Vec<(String, BTreeMap<&String, String>)> = Vec::new();
    for (label, runs) in &by_label {
        set_cell(&mut rows, "runs", label, runs.len().to_string());
        let mut params: Vec<&str> = runs.iter().map(|r| r.params.as_str()).collect();
        params.sort();
        params.dedup();
        set_cell(&mut rows, "params", label, params.join(", "));

        let per_run: Vec<Vec<(String, Metric)>> = runs.iter().map(RunData::metrics).collect();
        for (i, (key, first)) in per_run[0].iter().enumerate() {
            let cell = match first {
                Metric::Text(_) => {
                    let mut counts: Vec<(String, usize)> = Vec::new();
                    for run in &per_run {
                        if let Some((_, Metric::Text(t))) = run.get(i) {
                            match counts.iter_mut().find(|(v, _)| v == t) {
                                Some((_, n)) => *n += 1,
                                None => counts.push((t.clone(), 1)),
                            }
                        }
                    }
                    counts
                        .iter()
                        .map(|(v, n)| if *n > 1 { format!("{v} ×{n}") } else { v.clone() })
                        .collect::<Vec<_>>()
                        .join(", ")
                }
                Metric::Num(_) | Metric::Percent(_) => {
                    let values: Vec<f64> = per_run
                        .iter()
                        .filter_map(|run| match run.get(i) {
                            Some((_, Metric::Num(v) | Metric::Percent(v))) => *v,
                            _ => None,
                        })
                        .collect();
                    let unit = if matches!(first, Metric::Percent(_)) { "%" } else { "" };
                    spread(&values, unit)
                }
            };
            set_cell(&mut rows, key, label, cell);
        }
    }

    let width0 = rows.iter().map(|(k, _)| k.chars().count()).max().unwrap_or(0);
    let widths: Vec<usize> = labels
        .iter()
        .map(|l| {
            rows.iter()
                .map(|(_, cells)| cells.get(l).map_or(0, |c| c.chars().count()))
                .chain([l.chars().count()])
                .max()
                .unwrap_or(0)
        })
        .collect();
    let pad = |s: &str, w: usize| format!("{s}{}", " ".repeat(w.saturating_sub(s.chars().count())));
    println!();
    let mut header = vec![" ".repeat(width0)];
    header.extend(labels.iter().zip(&widths).map(|(l, w)| pad(l, *w)));
    println!("{}", header.join("  ").trim_end());
    for (key, cells) in &rows {
        let mut line = vec![pad(key, width0)];
        line.extend(
            labels
                .iter()
                .zip(&widths)
                .map(|(l, w)| pad(cells.get(l).map_or("", String::as_str), *w)),
        );
        println!("{}", line.join("  ").trim_end());
    }
    Ok(())
}

fn set_cell<'a>(
    rows: &mut Vec<(String, BTreeMap<&'a String, String>)>,
    key: &str,
    label: &'a String,
    cell: String,
) {
    match rows.iter_mut().find(|(k, _)| k == key) {
        Some((_, cells)) => {
            cells.insert(label, cell);
        }
        None => rows.push((key.to_string(), BTreeMap::from([(label, cell)]))),
    }
}

/// `mean (min–max)` of the values, or `-` if there are none.
fn spread(values: &[f64], unit: &str) -> String {
    if values.is_empty() {
        return "-".into();
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let (min, max) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &v| (a.min(v), b.max(v)));
    if min == max {
        format!("{}{unit}", fmt_num(mean))
    } else {
        format!("{}{unit} ({}–{})", fmt_num(mean), fmt_num(min), fmt_num(max))
    }
}

/// Small non-integers with two decimals, everything else rounded with thousands separators.
fn fmt_num(x: f64) -> String {
    if x.abs() < 100.0 && x.fract() != 0.0 {
        return format!("{x:.2}");
    }
    let digits = format!("{}", x.round().abs() as u64);
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if x < 0.0 {
        out.insert(0, '-');
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn formats_numbers() {
        assert_eq!(fmt_num(0.583), "0.58");
        assert_eq!(fmt_num(14.0), "14");
        assert_eq!(fmt_num(11_026_267.67), "11,026,268");
        assert_eq!(fmt_num(426_134.0), "426,134");
        assert_eq!(fmt_num(-1234.0), "-1,234");
        assert_eq!(spread(&[1.0, 2.0, 3.0], "%"), "2% (1–3)");
        assert_eq!(spread(&[5.0, 5.0], ""), "5");
        assert_eq!(spread(&[], ""), "-");
    }

    #[test]
    fn parses_peak_rss() {
        let bsd = "        0.60 real         8.07 user\n            58277888  maximum resident set size\n";
        assert_eq!(parse_peak_rss(bsd), Some(58_277_888));
        let gnu = "\tUser time (seconds): 8.07\n\tMaximum resident set size (kbytes): 56912\n";
        assert_eq!(parse_peak_rss(gnu), Some(56_912 * 1024));
        assert_eq!(parse_peak_rss("nothing here"), None);
    }
}
