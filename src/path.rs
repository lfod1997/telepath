use std::path::{Component, Path, PathBuf};

pub(crate) fn get_relative<P: AsRef<Path>, B: AsRef<Path>>(path: P, base: B) -> Option<PathBuf> {
    let path = path.as_ref();
    let base = base.as_ref();
    if path.is_absolute() != base.is_absolute() { None } else {
        let mut ita = path.components();
        let mut itb = base.components();
        let mut rel = Vec::new();
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => { break; }
                (Some(a), None) => {
                    rel.push(a);
                    rel.extend(ita.by_ref());
                    break;
                }
                (None, _) => { rel.push(Component::ParentDir); }
                (Some(a), Some(b)) if rel.is_empty() && a == b => {}
                (Some(a), Some(b)) if b == Component::CurDir => { rel.push(a); }
                (Some(_), Some(b)) if b == Component::ParentDir => { return None; }
                (Some(a), Some(_)) => {
                    rel.push(Component::ParentDir);
                    for _ in itb {
                        rel.push(Component::ParentDir);
                    }
                    rel.push(a);
                    rel.extend(ita.by_ref());
                    break;
                }
            }
        }
        Some(rel.iter().map(|c| c.as_os_str()).collect())
    }
}
