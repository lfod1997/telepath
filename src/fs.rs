use std::{io, fs};
use std::path::Path;

#[inline(always)]
pub(crate) fn is_separator(c: char) -> bool {
    #[cfg(target_family = "windows")] { c == '/' || c == '\\' }
    #[cfg(target_family = "unix")] { c == '/' }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum LinkType { Hard, Symbolic }

pub(crate) fn create_file_link<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q, link_type: LinkType, overwrite: bool) -> io::Result<()> {
    let original = original.as_ref();
    let link = link.as_ref();
    if !original.is_absolute() || !link.is_absolute() {
        panic!("Bug in source: create_file_link only accepts absolute paths")
    }
    if original == link {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the intended link \"{}\" points to itself", link.display())));
    }
    if overwrite {
        erase_entity(link)?;
    }
    match link_type {
        LinkType::Symbolic => {
            #[cfg(target_family = "windows")] { std::os::windows::fs::symlink_file(original, link) }
            #[cfg(target_family = "unix")] { std::os::unix::fs::symlink(original, link) }
        }
        LinkType::Hard => fs::hard_link(original, link),
    }
}

pub(crate) fn erase_entity<P: AsRef<Path>>(entity: P) -> io::Result<()> {
    let entity = entity.as_ref();
    if entity.is_symlink() || entity.is_file() {
        fs::remove_file(&entity)
    } else if entity.is_dir() {
        fs::remove_dir_all(&entity)
    } else { Ok(()) }
}

pub(crate) fn ensure_dir<P: AsRef<Path>>(dir: P, overwrite: bool) -> io::Result<()> {
    let dir = dir.as_ref();
    let mut should_create = false;
    if dir.is_symlink() || dir.is_file() {
        if overwrite {
            fs::remove_file(&dir)?;
            should_create = true;
        } else {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!("\"{}\" already exists and is not a directory", dir.display())));
        }
    } else if !dir.is_dir() {
        should_create = true;
    }
    match should_create {
        true => fs::create_dir_all(&dir),
        false => Ok(()),
    }
}
