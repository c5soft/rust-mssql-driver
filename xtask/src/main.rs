//! Build automation tasks for the rust-mssql-driver workspace.
//!
//! Run with `cargo xtask <command>`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use xshell::{Shell, cmd};

#[derive(Parser)]
#[command(name = "xtask", about = "Build automation for rust-mssql-driver")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run all checks (format, lint, test)
    Ci,
    /// Run cargo fmt --check
    Fmt,
    /// Run clippy with all features
    Clippy,
    /// Run all tests
    Test,
    /// Run cargo-deny checks
    Deny,
    /// Generate documentation
    Doc,
    /// Run benchmarks
    Bench,
    /// Clean build artifacts
    Clean,
    /// Update workspace-hack crate (requires cargo-hakari)
    Hakari,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    // Change to workspace root
    let workspace_root = workspace_root()?;
    sh.change_dir(&workspace_root);

    match cli.command {
        Command::Ci => {
            println!("Running CI checks...");
            fmt(&sh)?;
            clippy(&sh)?;
            test(&sh)?;
            deny(&sh)?;
            println!("All CI checks passed!");
        }
        Command::Fmt => fmt(&sh)?,
        Command::Clippy => clippy(&sh)?,
        Command::Test => test(&sh)?,
        Command::Deny => deny(&sh)?,
        Command::Doc => doc(&sh)?,
        Command::Bench => bench(&sh)?,
        Command::Clean => clean(&sh)?,
        Command::Hakari => hakari(&sh)?,
    }

    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let output = std::process::Command::new("cargo")
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .context("failed to run cargo locate-project")?;

    let path = String::from_utf8(output.stdout)
        .context("invalid UTF-8 in cargo output")?
        .trim()
        .to_string();

    Ok(PathBuf::from(path)
        .parent()
        .context("failed to get workspace root")?
        .to_path_buf())
}

fn fmt(sh: &Shell) -> Result<()> {
    println!("Checking formatting...");
    cmd!(sh, "cargo fmt --all -- --check").run()?;
    println!("Formatting check passed.");
    Ok(())
}

fn clippy(sh: &Shell) -> Result<()> {
    println!("Running clippy...");
    cmd!(
        sh,
        "cargo clippy --all-features --all-targets -- -D warnings"
    )
    .run()?;
    println!("Clippy check passed.");
    Ok(())
}

fn test(sh: &Shell) -> Result<()> {
    println!("Running tests...");
    cmd!(sh, "cargo test --all-features").run()?;
    println!("All tests passed.");
    Ok(())
}

fn deny(sh: &Shell) -> Result<()> {
    println!("Running cargo-deny...");
    cmd!(sh, "cargo deny check").run()?;
    println!("Cargo-deny check passed.");
    Ok(())
}

fn doc(sh: &Shell) -> Result<()> {
    println!("Generating documentation...");
    cmd!(sh, "cargo doc --all-features --no-deps").run()?;
    println!("Documentation generated.");
    Ok(())
}

fn bench(sh: &Shell) -> Result<()> {
    println!("Running benchmarks...");
    cmd!(sh, "cargo bench").run()?;
    Ok(())
}

fn clean(sh: &Shell) -> Result<()> {
    println!("Cleaning build artifacts...");
    cmd!(sh, "cargo clean").run()?;
    println!("Clean complete.");
    Ok(())
}

fn hakari(sh: &Shell) -> Result<()> {
    println!("Updating workspace-hack...");
    cmd!(sh, "cargo hakari generate").run()?;
    cmd!(sh, "cargo hakari manage-deps").run()?;
    println!("Workspace-hack updated.");
    Ok(())
}
