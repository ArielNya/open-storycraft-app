//! Storybible import: one bible file becomes a book the spine can continue.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

use storycraft_core::{
    JobStatus, JobStore, Mode, NewJob, STORYBIBLE_FILE, StatusBoard, storybible_import_preview,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Copy the sample bible into `dir`, the way a user would drop it in a folder.
fn place_bible(dir: &std::path::Path) {
    fs::create_dir_all(dir).unwrap();
    fs::copy(
        fixture("bible-only").join(STORYBIBLE_FILE),
        dir.join(STORYBIBLE_FILE),
    )
    .unwrap();
}

/// Run the importer end to end: preview, then the confirmed save.
fn import(root: &std::path::Path) -> Vec<String> {
    let preview = storybible_import_preview(root).unwrap();
    let store = JobStore::open(root).unwrap();
    let mut job = store
        .create(NewJob {
            skill: "storybible-import".into(),
            mode: Mode::SingleSkill,
            chapter: 1,
            answers: Vec::new(),
            packed_context_hash: "local".into(),
            provider: "local".into(),
            model: "none".into(),
            output_path: None,
        })
        .unwrap();
    store.write_preview(&job.id, &preview).unwrap();
    store.set_status(&mut job, JobStatus::NeedsConfirm).unwrap();
    storycraft_core::validate_preview("storybible-import", &preview).unwrap();
    store.commit(&mut job).unwrap();
    assert_eq!(job.status, JobStatus::Saved);
    fs::read_to_string(store.preview_path(&job.id))
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("- ").map(ToOwned::to_owned))
        .collect()
}

#[test]
fn one_bible_becomes_the_book_files() {
    let tmp = tempfile::tempdir().unwrap();
    let book = tmp.path().join("salt-ledger");
    place_bible(&book);

    let paths = import(tmp.path());
    assert!(
        paths.contains(&"salt-ledger/Wiki/Style/genre.md".to_owned()),
        "the bible keeps the book folder it was found in: {paths:?}"
    );

    for rel in [
        "salt-ledger/Wiki/Style/genre.md",
        "salt-ledger/Wiki/Style/audience.md",
        "salt-ledger/Wiki/Story/theme.md",
        "salt-ledger/Wiki/Story/synopsis.md",
        "salt-ledger/Wiki/Style/style_guide.md",
        "salt-ledger/Wiki/Style/voice_prompt.md",
        "salt-ledger/Wiki/Outline/outline.md",
        "salt-ledger/Wiki/Characters/Nia.md",
        "salt-ledger/Wiki/Characters/Kael_Veyra.md",
        "salt-ledger/Wiki/Locations/The_Long_Pier.md",
        "salt-ledger/Wiki/Outline/Chapter_01_Scene.md",
        "salt-ledger/Wiki/Psych/Chapter_01_Psych.md",
    ] {
        let path = tmp.path().join(rel);
        assert!(path.is_file(), "missing {rel}");
        assert!(
            fs::metadata(&path).unwrap().len() > 40,
            "{rel} has no substance"
        );
    }

    let genre = fs::read_to_string(book.join("Wiki/Style/genre.md")).unwrap();
    assert!(genre.starts_with("---\n"));
    assert!(genre.contains("genre: Mystery"));
    assert!(
        !genre.contains("path:"),
        "routing keys must not leak into the written file:\n{genre}"
    );
    assert!(!book.join("Wiki/Style/genre.md.tmp").exists());
}

#[test]
fn the_imported_book_is_ready_to_draft() {
    let tmp = tempfile::tempdir().unwrap();
    let book = tmp.path().join("salt-ledger");
    place_bible(&book);

    let before = StatusBoard::inspect_folder(tmp.path(), Mode::Resume, 1).unwrap();
    assert_eq!(before.project(), None);
    assert_eq!(before.next().skill.as_deref(), Some("storybible-import"));

    import(tmp.path());

    let after = StatusBoard::inspect_folder(tmp.path(), Mode::Resume, 1).unwrap();
    assert_eq!(
        after.project().unwrap().path(),
        fs::canonicalize(&book).unwrap().as_path(),
        "the board finds the imported book"
    );
    for slot in [
        "genre",
        "audience",
        "theme",
        "synopsis",
        "style",
        "characters",
        "voice",
    ] {
        let state = after
            .snapshot()
            .slots
            .into_iter()
            .find(|s| s.name == slot)
            .expect("slot")
            .state;
        assert_eq!(state, "yes", "slot {slot} should be filled by the import");
    }
    assert_eq!(
        after.next().skill.as_deref(),
        Some("fiction-writechapter"),
        "with planning imported the only thing left is prose: {}",
        after.next().why
    );
}

#[test]
fn a_folder_without_a_bible_still_starts_at_genre() {
    let tmp = tempfile::tempdir().unwrap();
    let board = StatusBoard::inspect_folder(tmp.path(), Mode::Resume, 1).unwrap();
    assert_eq!(board.next().skill.as_deref(), Some("fiction-genre"));
    assert!(board.storybible().is_none());
}

#[test]
fn the_importer_reports_a_missing_bible() {
    let tmp = tempfile::tempdir().unwrap();
    let err = storybible_import_preview(tmp.path()).unwrap_err();
    assert!(err.to_string().contains("no storybible.md"));
}

#[test]
fn assembling_a_bible_reads_the_existing_canon() {
    let project = storycraft_core::discover_one(&fixture("planning-done")).unwrap();
    let rels: Vec<String> = storycraft_core::wiki_pack_files(&project, "fiction-storybible", 1)
        .iter()
        .map(|file| file.relative.clone())
        .collect();
    for expected in [
        "Wiki/Style/genre.md",
        "Wiki/Style/audience.md",
        "Wiki/Story/theme.md",
        "Wiki/Story/synopsis.md",
        "Wiki/Style/style_guide.md",
        "Wiki/Style/voice_prompt.md",
        "Wiki/Outline/outline.md",
        "Wiki/Characters/Nia.md",
    ] {
        assert!(rels.contains(&expected.to_owned()), "missing {expected}");
    }
}

#[test]
fn a_saved_preview_is_itself_a_storybible() {
    let tmp = tempfile::tempdir().unwrap();
    place_bible(tmp.path());
    let preview = storybible_import_preview(tmp.path()).unwrap();
    let docs = storycraft_core::parse_storybible(&preview).unwrap();
    assert_eq!(docs.len(), 12, "every document survives the round trip");
    assert!(docs.iter().any(|doc| doc.path == "Wiki/Style/genre.md"));
}
