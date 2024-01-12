#![feature(absolute_path)]

mod path;
mod fs;
mod glob;
mod cli;

use std::process::exit;
use clap::Parser;
use anstream::eprintln;
use owo_colors::OwoColorize as _;
use crate::cli::{Cli, run};

fn main() {
    const EXIT_CODE_GENERAL_ERROR: i32 = 1;
    if let Err(e) = run(Cli::parse()) {
        eprintln!("{} {}", "error:".red().bold(), e);
        exit(EXIT_CODE_GENERAL_ERROR);
    }
}
