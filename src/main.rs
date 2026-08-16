use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};

use ghostmark::{
    encode::{check_capacity, watermark},
    homoglyph::HOMOGLYPHS,
    identify::identify_ranked,
};

/// Homoglyph-based text watermarking
#[derive(Parser)]
#[command(name = "ghostmark", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Watermark a cover text for each recipient in a list
    Encode(EncodeArgs),
    /// Identify which recipient a leaked text most likely came from
    Identify(IdentifyArgs),
}

#[derive(Args)]
struct EncodeArgs {
    /// Path to the cover text file (the message to watermark)
    #[arg(short, long)]
    cover: PathBuf,

    /// Path to a CSV file of recipients
    #[arg(short, long)]
    recipients: PathBuf,

    /// Path to a file containing the secret key
    #[arg(short, long)]
    key_file: PathBuf,

    /// Directory to write one watermarked file per recipient into
    #[arg(short, long, default_value = "./watermarked_out")]
    out_dir: PathBuf,
}

#[derive(Args)]
struct IdentifyArgs {
    /// Path to the original cover text file
    #[arg(short, long)]
    cover: PathBuf,

    /// Path to the leaked text file to identify
    #[arg(short, long)]
    leaked: PathBuf,

    /// Path to a CSV file of candidate recipients
    #[arg(short, long)]
    recipients: PathBuf,

    /// Path to the secret key file (must match what was used to encode).
    #[arg(short, long)]
    key_file: PathBuf,

    /// Show every candidate's score
    #[arg(long, default_value_t = false)]
    all: bool,
}

/// Get the list of recipients (probably emails)
fn read_recipients(path: &Path) -> Result<Vec<String>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("failed to open recipients file: {}", path.display()))?;

    let mut recipients = Vec::new();
    for result in rdr.records() {
        let record = result.context("failed to parse a row in the recipients CSV")?;
        if let Some(first) = record.get(0) {
            let trimmed = first.trim();

            if !trimmed.is_empty() {
                recipients.push(trimmed.to_string());
            }
        }
    }

    if recipients.is_empty() {
        bail!("no recipients found in {}", path.display());
    }
    Ok(recipients)
}

/// Get the secret key from the disk
fn read_key(path: &Path) -> Result<Vec<u8>> {
    let key =
        fs::read(path).with_context(|| format!("failed to read key file: {}", path.display()))?;

    Ok(key)
}

/// Encode the text for each recipient
fn run_encode(args: EncodeArgs) -> Result<()> {
    let cover_text = fs::read_to_string(&args.cover)
        .with_context(|| format!("failed to read cover text: {}", args.cover.display()))?;
    let recipients = read_recipients(&args.recipients)?;
    let secret_key = read_key(&args.key_file)?;

    let num_hg = HOMOGLYPHS.candidate_positions(&cover_text).len();
    println!("\nHomoglyph channel: {num_hg} eligible letter positions\n");
    println!("{}\n", check_capacity(recipients.len(), num_hg));

    fs::create_dir_all(&args.out_dir).with_context(|| {
        format!(
            "failed to create output directory: {}",
            args.out_dir.display()
        )
    })?;

    // Write the result to file and in the console
    for r in &recipients {
        let wm = watermark(&cover_text, r, &secret_key);
        let safe_name = r.replace(['@', '/', '\\', ' '], "_");
        let out_path = args.out_dir.join(format!("{safe_name}.txt"));
        fs::write(&out_path, &wm)
            .with_context(|| format!("failed to write {}", out_path.display()))?;
        println!("  {r:35} -> {}", out_path.display());
    }

    println!("\nDone. {} recipient(s) watermarked.", recipients.len());
    Ok(())
}

/// Identify who leaked the text
fn run_identify(args: IdentifyArgs) -> Result<()> {
    let cover_text = fs::read_to_string(&args.cover)
        .with_context(|| format!("failed to read cover text: {}", args.cover.display()))?;
    let leaked_text = fs::read_to_string(&args.leaked)
        .with_context(|| format!("failed to read leaked text: {}", args.leaked.display()))?;
    let recipients = read_recipients(&args.recipients)?;
    let secret_key = read_key(&args.key_file)?;

    let ranked = identify_ranked(&cover_text, &leaked_text, &recipients, &secret_key);
    if ranked.is_empty() {
        println!("No homoglyph-eligible positions found, nothing to compare.");
        return Ok(());
    }

    let shown = if args.all {
        ranked.len()
    } else {
        ranked.len().min(5)
    };

    println!("{:<35} {:>12} {:>10}", "Candidate", "Matches", "Rate");
    println!("{}", "-".repeat(60));
    for c in ranked.iter().take(shown) {
        println!(
            "{:<35} {:>8}/{:<3} {:>9.1}%",
            c.recipient_id,
            c.matches,
            c.total_bits,
            c.match_rate() * 100.0
        );
    }
    if !args.all && ranked.len() > shown {
        println!(
            "... and {} more (use --all to show everyone)",
            ranked.len() - shown
        );
    }

    let best = &ranked[0];
    println!(
        "\nMost likely source: {} ({} matches out of {})",
        best.recipient_id, best.matches, best.total_bits
    );
    if ranked.len() > 1 {
        let runner_up = &ranked[1];
        if best.matches == runner_up.matches {
            println!(
                "Warning: tied with {}, result is ambiguous, consider a longer cover text next time.",
                runner_up.recipient_id
            );
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Encode(e) => {
            run_encode(e)?;
        }
        Command::Identify(i) => {
            run_identify(i)?;
        }
    }

    Ok(())
}
