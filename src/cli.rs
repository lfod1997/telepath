use std::collections::HashSet;
use std::iter::FilterMap;
use std::io;
use std::path::{Path, PathBuf};
use clap::Parser;
use anstream::eprintln;
use owo_colors::OwoColorize as _;
use crate::path::get_relative;
use crate::fs::{LinkType, create_file_link, erase_entity, ensure_dir};
use crate::glob::{build_globs, ScopedGlobSet};

fn walk<P: AsRef<Path>>(root: P, max_depth: usize) -> FilterMap<walkdir::IntoIter, fn(walkdir::Result<walkdir::DirEntry>) -> Option<walkdir::DirEntry>> {
    walkdir::WalkDir::new(root).max_depth(max_depth).into_iter().filter_map(|it| it.ok())
}

fn establish_linkage<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q, link_type: LinkType, overwrite: bool, print: bool, return_error: bool) -> io::Result<()> {
    let result = create_file_link(&original, &link, link_type, overwrite);
    match result {
        Ok(_) => {
            if print { println!("{} -> {}", link.as_ref().display(), original.as_ref().display()); }
        }
        Err(e) => {
            if return_error { return Err(e); } else {
                eprintln!(
                    "{} failed to create link from \"{}\" to \"{}\" due to error: {}",
                    "warning:".yellow().bold(), link.as_ref().display(), original.as_ref().display(), e);
            }
        }
    }
    Ok(())
}

#[derive(Parser, Debug)]
#[clap(author, version)]
pub(crate) struct Cli {
    /// The root directory to create that contains all generated links
    #[arg(id = "LINK_ROOT")]
    link_root: String,

    /// Entities to link to, can be directories or files, names should not collide
    #[arg(id = "TARGET", value_delimiter = ' ', num_args = 1.., required = true)]
    targets: Vec<String>,

    /// Filter files using a glob pattern, only applies to directory entities in TARGET
    #[arg(long, short, value_delimiter = ' ')]
    glob: Vec<String>,

    /// Directory depth limitation, set to 0 if a "shallow" scan is intended
    #[arg(long, short = 'D', default_value = "255")]
    depth: usize,

    /// Create hard links instead of symbolic links
    #[arg(long, short = 'H')]
    hard: bool,

    /// Overwrite any existing entity if necessary
    #[arg(long, short = 'f')]
    overwrite: bool,

    /// Stop working if an error occurred when creating a link
    #[arg(long, short = 'P')]
    panic: bool,

    /// Do not print anything into standard output
    #[arg(long, short)]
    quiet: bool,

    /// Do not create folder tree, put all links in a single folder
    #[arg(long)]
    no_tree: bool,

    /// Do not create a folder under LINK_ROOT for each directory entity in TARGET
    #[arg(long)]
    combine: bool,

    /// Delete the entire LINK_ROOT directory before creating any link, this is rarely needed and lazily executed; if you only intend to overwrite existing entities, set -f instead
    #[arg(long)]
    purge: bool,
}

#[inline(always)]
fn is_path_accepted<P: AsRef<Path>>(rel: P, maybe_glob: &Option<ScopedGlobSet>) -> bool {
    match maybe_glob {
        None => true,
        Some(g) => g.is_match(rel),
    }
}

pub(crate) fn run(args: Cli) -> io::Result<()> {
    // dbg!(&args);
    // Link metadata
    let link_root = std::path::absolute(&args.link_root)?;
    let link_type = match args.hard {
        true => LinkType::Hard,
        false => LinkType::Symbolic,
    };

    // Glob set
    let glob = match args.glob.is_empty() {
        true => None,
        false => Some(build_globs(&args.glob)?),
    };

    // Lazy purging of LINK_ROOT
    let mut should_erase_root = args.purge;

    // Speed things up
    let mut ensured_dirs = HashSet::<PathBuf>::new();

    // Main program
    for target in &args.targets {
        let target = std::path::absolute(target)?;
        if target.is_file() {
            /* Ensure dirs & chore */ {
                if should_erase_root {
                    erase_entity(&link_root)?;
                    should_erase_root = false;
                }
                if !ensured_dirs.contains(&link_root) {
                    ensure_dir(&link_root, args.overwrite)?;
                    ensured_dirs.insert(link_root.clone());
                }
            }
            establish_linkage(&target, link_root.join(target.file_name().unwrap()), link_type, args.overwrite, !args.quiet, args.panic)?; // `target` is a file so has to have a leaf
        } else if target.is_dir() {
            let target_folder = link_root.join(target.file_name().unwrap_or_default()); // only empty when target is system root on Unix or drive prefix on Windows
            let target_root = match args.combine {
                true => &link_root,
                false => &target_folder,
            };
            for entry in walk(&target, args.depth + 1) {
                let entry = entry.path();
                let rel = get_relative(entry, &target).unwrap(); // `entry` is walked out from `target`
                if is_path_accepted(&rel, &glob) && entry.is_file() {
                    let link = match args.no_tree {
                        true => target_root.join(rel.file_name().unwrap()), // rel points to a concrete file so unwrap is safe
                        false => target_root.join(rel),
                    };
                    /* Ensure dirs & chore */ {
                        if should_erase_root {
                            erase_entity(&link_root)?;
                            should_erase_root = false;
                        }
                        let parent = link.parent().unwrap(); // `link` comes from a `join` so has to have a parent
                        if !ensured_dirs.contains(parent) {
                            ensure_dir(parent, args.overwrite)?;
                            ensured_dirs.insert(parent.to_path_buf());
                        }
                    }
                    establish_linkage(entry, &link, link_type, args.overwrite, !args.quiet, args.panic)?;
                }
            }
        }
    }
    Ok(())
}
