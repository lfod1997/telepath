use std::io;
use std::path::Path;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use anstream::eprintln;
use owo_colors::OwoColorize as _;
use crate::fs::is_separator;

#[inline(always)]
fn is_path_glob(pattern: &str) -> bool {
    pattern.contains(is_separator)
}

#[inline(always)]
fn is_leaf_glob(pattern: &str) -> bool {
    !is_path_glob(pattern)
}

pub(crate) struct ScopedGlobSet {
    path_globs: Option<GlobSet>,
    leaf_globs: Option<GlobSet>,
}

impl ScopedGlobSet {
    pub(crate) fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        if let Some(glob) = &self.path_globs {
            if glob.is_match(&path) { return true; }
        }
        if let Some(glob) = &self.leaf_globs {
            if let Some(leaf) = path.as_ref().file_name() {
                if glob.is_match(leaf) { return true; }
            }
        }
        false
    }
}

fn build_globs_impl<S: AsRef<str>, T: Iterator<Item = S>>(patterns: T, strip_prefix: bool) -> io::Result<Option<GlobSet>> {
    let mut builder = GlobSetBuilder::new();
    let mut valid_globs = 0u32;
    for pattern in patterns {
        let pattern = pattern.as_ref();
        let processed = match strip_prefix {
            true => pattern.strip_prefix(is_separator).unwrap_or_else(|| pattern),
            false => pattern,
        };
        match GlobBuilder::new(processed).literal_separator(true).build() {
            Ok(v) => {
                builder.add(v);
                valid_globs += 1;
            }
            Err(_) => { eprintln!("{} ignoring invalid glob \"{}\"", "warning:".yellow().bold(), pattern); }
        }
    }
    match valid_globs {
        0 => Ok(None),
        _ => match builder.build() {
            Ok(globs) => Ok(Some(globs)),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("failed to build globs, {}", e))),
        }
    }
}

pub(crate) fn build_globs(patterns: &[String]) -> io::Result<ScopedGlobSet> {
    match (
        build_globs_impl(patterns.iter().filter(|p| is_path_glob(p)), true)?,
        build_globs_impl(patterns.iter().filter(|p| is_leaf_glob(p)), false)?
    ) {
        (None, None) => Err(io::Error::new(io::ErrorKind::InvalidInput, "all globs are invalid")),
        (path_globs, leaf_globs) => Ok(ScopedGlobSet { path_globs, leaf_globs })
    }
}
