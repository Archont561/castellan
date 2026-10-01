//! The one tool that has to be a program rather than a task: typed
//! repository automation.
//!
//! Everything that used to be (or would have become) a repo-targeting shell
//! script lives here as a clap subcommand, so one runtime covers every
//! caller: `bun run codegen`, `pixi run xtask <subcommand>` and a plain
//! `cargo run -p castellan-xtask -- <subcommand>` are the same program with
//! the same arguments. (Same job, same reasoning as pixi-sandbox's xtask;
//! the difference is that castellan's TypeScript output is committed, so a
//! stale `packages/protocol` is a reviewable diff, not a build mystery.)
//!
//! Commands take an explicit `--root` (default: the repository this crate
//! was built from, derived from its manifest — callers run from the repo
//! root, from `crates/`, or from wherever pixi drops them, and none of them
//! should have to know the layout).
//!
//! ```console
//! $ cargo run -p castellan-xtask -- codegen
//! $ cargo run -p castellan-xtask -- version
//! ```

mod codegen;
mod version;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "Repository automation for castellan")]
struct Args {
    /// Repository root the commands operate on.
    #[arg(long)]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Derive packages/protocol/src/generated from the Rust wire types.
    Codegen,
    /// Print the workspace version (the single source of truth in Cargo.toml).
    Version,
}

fn main() {
    if let Err(error) = run() {
        // One line, error chain included: the callers that read output
        // programmatically (command substitution, turbo) keep it parseable,
        // and a human in a terminal gets the whole causal story.
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    // Default to the repository this crate was built from, not the working
    // directory: `cargo run` and the pixi `xtask` task are both documented
    // to run from the root, but "documented to" is not "cannot be run
    // otherwise", and a wrong root is a confusing error far from its cause.
    // crates/xtask → crates → repo root: two levels up, expressed as
    // ancestors().nth(2) because ancestors() includes the starting path.
    let root = match args.root {
        Some(root) => root,
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .context("no repo root")?
            .to_path_buf(),
    };
    match args.command {
        Command::Codegen => codegen::run(&root),
        Command::Version => version::print_version(&root),
    }
}
