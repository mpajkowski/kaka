use std::{
    fs::{create_dir_all, remove_dir, remove_file},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use duct::cmd;
use glob::glob;

#[derive(Debug, Parser)]
pub enum Task {
    Coverage {
        #[clap(long)]
        open: bool,
    },
}

fn main() -> Result<()> {
    let task = Task::parse();

    match task {
        Task::Coverage { open } => coverage(open),
    }
}

fn coverage(open: bool) -> Result<()> {
    let _ = remove_dir("coverage");
    create_dir_all("coverage")?;

    println!("=== running coverage ===");
    cmd!("cargo", "test")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env("LLVM_PROFILE_FILE", "cargo-test-%p-%m.profraw")
        .run()?;
    println!("ok.");

    println!("=== generating report ===");
    cmd!(
        "grcov",
        ".",
        "--excl-start",
        "mod test \\{",
        "--excl-line",
        "#\\[derive\\(",
        "--binary-path",
        "./target/debug/deps",
        "-s",
        ".",
        "-t",
        "html",
        "--branch",
        "--ignore-not-existing",
        "--ignore",
        "../*",
        "--ignore",
        "/*",
        "--ignore",
        "xtask/*",
        "--ignore",
        "*/src/tests/*",
        "-o",
        "coverage/html",
    )
    .run()?;
    println!("ok.");

    println!("=== cleaning up ===");
    clean_files("**/*.profraw")?;

    if open {
        println!("=== opening report ===");
        let _ = cmd!("xdg-open", "./coverage/html/index.html")
            .stdout_null()
            .stderr_null()
            .start()?;
    }

    Ok(())
}

fn clean_files(pattern: &str) -> Result<()> {
    let files: Result<Vec<PathBuf>, _> = glob(pattern)?.collect();
    Ok(files?.iter().try_for_each(remove_file)?)
}
