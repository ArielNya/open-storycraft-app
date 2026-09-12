//! Zip a book's `Wiki/` and `Chapters/` trees. Jobs and secrets stay out.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use walkdir::WalkDir;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::Error;
use crate::project::ProjectRoot;

/// Write `Wiki/` and `Chapters/` from `project` into `dest`.
///
/// `.storycraft/`, VCS, and build dirs are skipped. If `dest` lives inside
/// the project, the zip file itself is not included.
///
/// # Errors
///
/// Returns [`Error::Io`] or [`Error::Export`].
pub fn export_zip(project: &ProjectRoot, dest: &Path) -> Result<(), Error> {
    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
    }
    let file = File::create(dest).map_err(|err| Error::io(dest, err))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for folder in ["Wiki", "Chapters"] {
        let base = project.path().join(folder);
        if !base.exists() {
            continue;
        }
        for entry in WalkDir::new(&base).follow_links(false).max_depth(8) {
            let entry = entry.map_err(|err| Error::Export(err.to_string()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path == dest {
                continue;
            }
            if skip_dir(path) {
                continue;
            }
            let rel = path
                .strip_prefix(project.path())
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = std::fs::read(path).map_err(|err| Error::io(path, err))?;
            zip.start_file(&rel, options)
                .map_err(|err| Error::Export(err.to_string()))?;
            zip.write_all(&bytes).map_err(|err| Error::io(path, err))?;
        }
    }
    zip.finish().map_err(|err| Error::Export(err.to_string()))?;
    Ok(())
}

fn skip_dir(path: &Path) -> bool {
    path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some(".storycraft" | ".git" | "target" | "node_modules")
        )
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use std::io::Read;
    use zip::ZipArchive;

    #[test]
    fn zip_contains_wiki_and_skips_jobs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("book");
        std::fs::create_dir_all(root.join("Wiki/Style")).unwrap();
        std::fs::write(root.join("Wiki/Style/genre.md"), "genre: Fantasy\n").unwrap();
        std::fs::create_dir_all(root.join("Chapters")).unwrap();
        std::fs::write(root.join("Chapters/Chapter-001.md"), "# One\n").unwrap();
        std::fs::create_dir_all(root.join(".storycraft/jobs")).unwrap();
        std::fs::write(root.join(".storycraft/jobs/secret.json"), "{}").unwrap();
        let dest = tmp.path().join("book.zip");
        export_zip(&ProjectRoot::new(&root), &dest).unwrap();
        let file = File::open(&dest).unwrap();
        let mut zip = ZipArchive::new(file).unwrap();
        let mut names = Vec::new();
        for i in 0..zip.len() {
            names.push(zip.by_index(i).unwrap().name().to_owned());
        }
        names.sort();
        assert!(names.iter().any(|n| n == "Wiki/Style/genre.md"));
        assert!(names.iter().any(|n| n == "Chapters/Chapter-001.md"));
        assert!(names.iter().all(|n| !n.contains(".storycraft")));
        let mut genre = zip.by_name("Wiki/Style/genre.md").unwrap();
        let mut body = String::new();
        genre.read_to_string(&mut body).unwrap();
        assert!(body.contains("Fantasy"));
    }
}
