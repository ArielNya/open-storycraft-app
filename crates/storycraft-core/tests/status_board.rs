//! Golden status-board checks against fixture books.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)]

use std::path::{Path, PathBuf};

use storycraft_core::{Mode, Slot, SlotState, StatusBoard, discover_one, infer_chapter};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn board(path: &Path, mode: Mode) -> StatusBoard {
    let project = discover_one(path).ok();
    let chapter = project.as_ref().map(infer_chapter).unwrap_or(1);
    StatusBoard::inspect(project.as_ref(), mode, chapter).expect("valid chapter")
}

fn snapshot_text(board: &StatusBoard) -> String {
    board
        .to_string()
        .lines()
        .map(|line| {
            if let Some(rest) = line.strip_prefix("Project: ") {
                let name = Path::new(rest)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| rest.to_owned());
                format!("Project: {name}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn empty_wiki_starts_at_genre() {
    let board = board(&fixture("empty-wiki"), Mode::Resume);
    assert_eq!(board.slot(Slot::Genre), SlotState::No);
    assert_eq!(board.next().skill.as_deref(), Some("fiction-genre"));
    insta::assert_snapshot!(snapshot_text(&board));
}

#[test]
fn half_spine_stops_at_incomplete_style() {
    let board = board(&fixture("half-spine"), Mode::Resume);
    assert_eq!(board.slot(Slot::Genre), SlotState::Yes);
    assert_eq!(board.slot(Slot::Audience), SlotState::Yes);
    assert_eq!(board.slot(Slot::Theme), SlotState::Yes);
    assert_eq!(board.slot(Slot::Synopsis), SlotState::Yes);
    assert_eq!(board.slot(Slot::Style), SlotState::Partial);
    assert_eq!(board.slot(Slot::Characters), SlotState::Yes);
    assert_eq!(board.slot(Slot::World), SlotState::No);
    assert_eq!(board.slot(Slot::Outline), SlotState::No);
    assert_eq!(board.next().skill.as_deref(), Some("fiction-style"));
    insta::assert_snapshot!(snapshot_text(&board));
}

#[test]
fn planning_done_offers_writechapter_for_chapter_one() {
    let board = board(&fixture("planning-done"), Mode::Resume);
    assert_eq!(board.chapter(), 1);
    assert_eq!(board.slot(Slot::Outline), SlotState::Yes);
    assert_eq!(board.slot(Slot::Scenes), SlotState::Yes);
    assert_eq!(board.slot(Slot::Psych), SlotState::Yes);
    assert_eq!(board.slot(Slot::Voice), SlotState::Yes);
    assert_eq!(board.slot(Slot::Chapters), SlotState::No);
    assert_eq!(board.slot(Slot::World), SlotState::No);
    assert_eq!(board.next().skill.as_deref(), Some("fiction-writechapter"));
    insta::assert_snapshot!(snapshot_text(&board));
}

#[test]
fn draft_mode_still_requires_synopsis_and_outline() {
    let board = board(&fixture("half-spine"), Mode::Draft);
    assert_eq!(board.next().skill.as_deref(), Some("fiction-outline"));
    assert!(board.next().missing.contains(&Slot::Outline));
}

#[test]
fn spark_ignores_wiki() {
    let board = board(&fixture("half-spine"), Mode::Spark);
    assert_eq!(board.next().skill.as_deref(), Some("fiction-story-sparks"));
}

/// Copy a fixture with every line ending turned into CRLF, the way an editor on
/// Windows saves it.
fn crlf_copy(src: &Path, dst: &Path) {
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.unwrap();
        let target = dst.join(entry.path().strip_prefix(src).unwrap());
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target).unwrap();
        } else {
            let text = std::fs::read_to_string(entry.path()).unwrap();
            std::fs::write(&target, text.replace("\r\n", "\n").replace('\n', "\r\n")).unwrap();
        }
    }
}

#[test]
fn crlf_books_read_the_same_as_lf_books() {
    for name in ["half-spine", "planning-done"] {
        let tmp = tempfile::tempdir().unwrap();
        let book = tmp.path().join(name);
        crlf_copy(&fixture(name), &book);
        assert_eq!(
            snapshot_text(&board(&book, Mode::Resume)),
            snapshot_text(&board(&fixture(name), Mode::Resume)),
            "{name} with CRLF endings"
        );
    }
}
