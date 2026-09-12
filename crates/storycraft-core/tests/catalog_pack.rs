//! Catalog + packer: load the vendored library without dumping every reference.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use storycraft_core::{Catalog, pack_skill};

fn library_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../open-storycraft")
}

#[test]
fn indexes_craft_skills_from_frontmatter_only() {
    let catalog = Catalog::load(&library_dir()).expect("vendored library");
    assert!(
        catalog.len() >= 40,
        "expected the full craft library, got {}",
        catalog.len()
    );
    let genre = catalog.get("fiction-genre").expect("fiction-genre");
    assert!(genre.description.to_ascii_lowercase().contains("genre"));
    assert!(genre.next_skill.as_deref() == Some("fiction-audience"));
    assert!(genre.requires.is_empty());
}

#[test]
fn genre_pack_does_not_pull_the_whole_reference_dump() {
    let catalog = Catalog::load(&library_dir()).expect("vendored library");
    let genre = catalog.get("fiction-genre").expect("fiction-genre");
    let pack = pack_skill(genre);

    let references_dir = library_dir().join("fiction-genre/references");
    let on_disk = std::fs::read_dir(&references_dir)
        .expect("references dir")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .count();
    assert!(
        on_disk > 50,
        "fixture assumption: genre references/ is a large dump ({on_disk} files)"
    );

    assert!(
        pack.reference_count() < 20,
        "packer must not dump every genre reference (got {})",
        pack.reference_count()
    );
    assert!(
        pack.files
            .iter()
            .any(|file| file.relative == "references/genre-fantasy.md")
    );
    assert!(
        pack.files
            .iter()
            .any(|file| file.relative == "references/json-schema.json")
    );
    assert!(
        !pack
            .files
            .iter()
            .any(|file| file.relative == "references/adventure-heist.md"),
        "unlinked subgenre files must stay on disk"
    );
}
