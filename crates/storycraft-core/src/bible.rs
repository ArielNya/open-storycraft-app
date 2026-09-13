//! The storybible: one markdown file that carries a whole book.
//!
//! A storybible is a sequence of documents. Each document opens with a `---`
//! frontmatter block naming where that document lands — `path:` for an exact
//! destination, or `slot:` for one of the canonical slots — followed by the
//! markdown that becomes the file. Anything outside a document (a title, a
//! table of contents, notes to the reader) is ignored by the host.
//!
//! ```markdown
//! ---
//! path: Wiki/Style/genre.md
//! working_title: "Salt Ledger"
//! genre: Fantasy
//! ---
//!
//! # Genre
//!
//! Tone notes, tropes, and what they commit the story to.
//! ```
//!
//! The same file round-trips: `fiction-storybible` writes it, and
//! `storybible-import` splits it back into `Wiki/` files. The routing keys are
//! stripped on the way out, so a written file keeps only its own frontmatter.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::Error;
use crate::output::slug_filename;
use crate::wiki::{default_chapter_rel, default_psych_rel, default_scene_rel};

/// Portable bible file name, looked for beside the book folder's `Wiki/`.
pub const STORYBIBLE_FILE: &str = "storybible.md";

/// Frontmatter keys the host consumes as routing, never written to a file.
const ROUTING_KEYS: [&str; 2] = ["path", "slot"];

/// Slot names that map to a fixed destination.
const FIXED_SLOTS: [(&str, &str); 7] = [
    ("genre", "Wiki/Style/genre.md"),
    ("audience", "Wiki/Style/audience.md"),
    ("theme", "Wiki/Story/theme.md"),
    ("synopsis", "Wiki/Story/synopsis.md"),
    ("style", "Wiki/Style/style_guide.md"),
    ("voice", "Wiki/Style/voice_prompt.md"),
    ("outline", "Wiki/Outline/outline.md"),
];

/// Slot names that need a `name:` in their frontmatter.
const NAMED_SLOTS: [(&str, &str); 5] = [
    ("character", "Wiki/Characters"),
    ("location", "Wiki/Locations"),
    ("organization", "Wiki/Organizations"),
    ("system", "Wiki/Systems"),
    ("event", "Wiki/Events"),
];

/// One document of a storybible with its destination resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibleDoc {
    /// Destination relative to the book folder (`Wiki/Style/genre.md`).
    pub path: String,
    /// Frontmatter written through to the file, routing keys removed.
    pub frontmatter: String,
    /// Body markdown.
    pub body: String,
}

impl BibleDoc {
    /// The file as it lands on disk.
    #[must_use]
    pub fn markdown(&self) -> String {
        let mut out = String::from("---\n");
        let frontmatter = self.frontmatter.trim();
        if !frontmatter.is_empty() {
            out.push_str(frontmatter);
            out.push('\n');
        }
        out.push_str("---\n");
        let body = self.body.trim();
        if !body.is_empty() {
            out.push('\n');
            out.push_str(body);
            out.push('\n');
        }
        out
    }

    /// The document as it appears in a storybible, routing key included.
    #[must_use]
    pub fn document(&self) -> String {
        let mut out = String::from("---\n");
        let _ = writeln!(out, "path: {}", self.path);
        let frontmatter = self.frontmatter.trim();
        if !frontmatter.is_empty() {
            out.push_str(frontmatter);
            out.push('\n');
        }
        out.push_str("---\n");
        let body = self.body.trim();
        if !body.is_empty() {
            out.push('\n');
            out.push_str(body);
            out.push('\n');
        }
        out
    }
}

/// Find the book's storybible: beside the folder, in its `Wiki/`, or one level
/// down in exactly one subfolder.
#[must_use]
pub fn find_storybible(start: &Path) -> Option<PathBuf> {
    let direct = start.join(STORYBIBLE_FILE);
    if direct.is_file() {
        return Some(direct);
    }
    let in_wiki = start.join("Wiki").join(STORYBIBLE_FILE);
    if in_wiki.is_file() {
        return Some(in_wiki);
    }
    let mut candidates = Vec::new();
    let entries = fs::read_dir(start).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let nested = path.join(STORYBIBLE_FILE);
        if nested.is_file() {
            candidates.push(nested);
        }
    }
    candidates.sort();
    candidates.into_iter().next()
}

/// Parse a storybible into documents with destinations resolved.
///
/// # Errors
///
/// Returns [`Error::InvalidStoryBible`] when no document is found, a document
/// names no `path:` / `slot:`, a slot is unknown, a slot is missing its
/// `name:` / `chapter:`, or a destination is not a safe relative `.md` path.
pub fn parse_storybible(text: &str) -> Result<Vec<BibleDoc>, Error> {
    let lines: Vec<&str> = text.lines().collect();
    let mut docs = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !is_fence(lines[i]) {
            i += 1;
            continue;
        }
        let Some(end) = next_fence(&lines, i + 1) else {
            break;
        };
        let keys = parse_keys(&lines[i + 1..end]);
        if !keys.contains_key("path") && !keys.contains_key("slot") {
            // A horizontal rule, or a frontmatter block that is not a document.
            i += 1;
            continue;
        }
        // The body runs to the next fence that opens a real document.
        let body_start = end + 1;
        let mut body_end = lines.len();
        let mut scan = body_start;
        while scan < lines.len() {
            if is_fence(lines[scan]) {
                if let Some(close) = next_fence(&lines, scan + 1) {
                    let ahead = parse_keys(&lines[scan + 1..close]);
                    if ahead.contains_key("path") || ahead.contains_key("slot") {
                        body_end = scan;
                        break;
                    }
                }
            }
            scan += 1;
        }
        let frontmatter = strip_routing_keys(&lines[i + 1..end]);
        let body = lines[body_start..body_end].join("\n");
        docs.push(BibleDoc {
            path: resolve_path(&keys)?,
            frontmatter,
            body: body.trim().to_owned(),
        });
        i = body_end;
    }
    if docs.is_empty() {
        return Err(Error::InvalidStoryBible(
            "no documents found; each one needs a `---` frontmatter block with `path:` or `slot:`"
                .to_owned(),
        ));
    }
    Ok(docs)
}

/// Working title carried by a bible, if any document names one.
#[must_use]
pub fn storybible_title(text: &str) -> Option<String> {
    let docs = parse_storybible(text).ok()?;
    for doc in &docs {
        for key in ["working_title", "title"] {
            let needle = format!("{key}:");
            for line in doc.frontmatter.lines() {
                let Some(value) = line.trim().strip_prefix(&needle) else {
                    continue;
                };
                let value = value.trim().trim_matches('"').trim_matches('\'').trim();
                if !value.is_empty() {
                    return Some(value.to_owned());
                }
            }
        }
    }
    None
}

/// Canonical storybible text for `docs`, with a readable header.
#[must_use]
pub fn render_storybible(docs: &[BibleDoc], source: &Path) -> String {
    let mut out = String::new();
    out.push_str("# Storybible\n\n");
    let _ = writeln!(out, "Source: {}", source.display());
    let _ = writeln!(out, "Files: {}\n", docs.len());
    for doc in docs {
        let _ = writeln!(out, "- {}", doc.path);
    }
    out.push('\n');
    for doc in docs {
        out.push_str(&doc.document());
        out.push('\n');
    }
    out
}

/// Read the book's storybible and turn it into files ready to commit.
///
/// Destinations stay relative to `start`, so a bible kept in a subfolder
/// produces `sub/Wiki/...`.
///
/// # Errors
///
/// Returns [`Error::InvalidStoryBible`] when no bible is found under `start`,
/// or the parse errors of [`parse_storybible`].
pub fn import_preview(start: &Path) -> Result<String, Error> {
    let bible = find_storybible(start).ok_or_else(|| {
        Error::InvalidStoryBible(format!(
            "no {STORYBIBLE_FILE} under {}; write one with fiction-storybible",
            start.display()
        ))
    })?;
    let text = fs::read_to_string(&bible).map_err(|err| Error::io(&bible, err))?;
    let mut docs = parse_storybible(&text)?;
    let prefix = prefix_of(start, &bible);
    for doc in &mut docs {
        doc.path = format!("{prefix}{}", doc.path);
    }
    Ok(render_storybible(&docs, &bible))
}

/// Destination for a document, from `path:` or from `slot:` plus its metadata.
fn resolve_path(keys: &BTreeMap<String, String>) -> Result<String, Error> {
    if let Some(path) = keys.get("path") {
        return safe_rel(path);
    }
    let slot = keys.get("slot").map(String::as_str).unwrap_or_default();
    slot_path(slot, keys)
}

/// Canonical destination for a `slot:` document.
fn slot_path(slot: &str, keys: &BTreeMap<String, String>) -> Result<String, Error> {
    for (name, path) in FIXED_SLOTS {
        if slot == name {
            return Ok(path.to_owned());
        }
    }
    for (name, dir) in NAMED_SLOTS {
        if slot == name || slot == format!("{name}s") {
            let file = slug_filename(&required(keys, slot, "name")?);
            return Ok(format!("{dir}/{file}.md"));
        }
    }
    match slot {
        "scene" | "scenes" => return Ok(default_scene_rel(chapter_of(keys, slot)?)),
        "psych" => return Ok(default_psych_rel(chapter_of(keys, slot)?)),
        "chapter" | "chapters" => return Ok(default_chapter_rel(chapter_of(keys, slot)?)),
        _ => {}
    }
    let known: Vec<&str> = FIXED_SLOTS
        .iter()
        .map(|(name, _)| *name)
        .chain(NAMED_SLOTS.iter().map(|(name, _)| *name))
        .chain(["scene", "psych", "chapter"])
        .collect();
    Err(Error::InvalidStoryBible(format!(
        "unknown slot `{slot}`; use `path:` or one of {}",
        known.join(", ")
    )))
}

fn required(keys: &BTreeMap<String, String>, slot: &str, key: &str) -> Result<String, Error> {
    keys.get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| {
            Error::InvalidStoryBible(format!("slot `{slot}` needs a `{key}:` in its frontmatter"))
        })
}

fn chapter_of(keys: &BTreeMap<String, String>, slot: &str) -> Result<u32, Error> {
    let raw = required(keys, slot, "chapter")?;
    raw.trim()
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| {
            Error::InvalidStoryBible(format!("slot `{slot}` has a non-numeric `chapter: {raw}`"))
        })
}

/// Reject a destination that would leave the book folder or is not markdown.
///
/// # Errors
///
/// Returns [`Error::InvalidStoryBible`] for absolute paths, `..`, ``.``
/// components, empty values, and anything that is not a `.md` file.
pub fn safe_rel(rel: &str) -> Result<String, Error> {
    let cleaned = rel.trim().replace('\\', "/");
    let cleaned = cleaned.trim_start_matches("./").to_owned();
    let reject = |why: &str| Error::InvalidStoryBible(format!("`{rel}` {why}"));
    if cleaned.is_empty() {
        return Err(reject("is empty"));
    }
    if cleaned.starts_with('/') || cleaned.contains(':') {
        return Err(reject("must be relative to the book folder"));
    }
    if !cleaned.to_ascii_lowercase().ends_with(".md") {
        return Err(reject("must be a .md file"));
    }
    if Path::new(&cleaned)
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(reject("must not contain `.` or `..`"));
    }
    Ok(cleaned)
}

fn prefix_of(start: &Path, bible: &Path) -> String {
    let Some(dir) = bible.parent() else {
        return String::new();
    };
    let Ok(relative) = dir.strip_prefix(start) else {
        return String::new();
    };
    let text = relative.to_string_lossy().replace('\\', "/");
    if text.is_empty() {
        String::new()
    } else {
        format!("{}/", text.trim_end_matches('/'))
    }
}

fn is_fence(line: &str) -> bool {
    line.trim() == "---"
}

fn next_fence(lines: &[&str], from: usize) -> Option<usize> {
    (from..lines.len()).find(|idx| is_fence(lines[*idx]))
}

fn parse_keys(frontmatter: &[&str]) -> BTreeMap<String, String> {
    let mut keys = BTreeMap::new();
    for raw in frontmatter {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("- ") {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim().trim_matches('"').trim_matches('\'').trim();
        if !value.is_empty() {
            keys.insert(key.trim().to_owned(), value.to_owned());
        }
    }
    keys
}

fn strip_routing_keys(frontmatter: &[&str]) -> String {
    frontmatter
        .iter()
        .filter(|line| {
            let trimmed = line.trim();
            !ROUTING_KEYS
                .iter()
                .any(|key| trimmed.starts_with(&format!("{key}:")))
        })
        .map(|line| line.trim_end())
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    const BIBLE: &str = "# My book\n\nNotes for the reader.\n\n---\npath: Wiki/Style/genre.md\nworking_title: \"Salt Ledger\"\ngenre: Fantasy\n---\n\n# Genre\n\nTone notes for the harbour town story.\n\n---\nslot: character\nname: Nia Farrow\nrole: protagonist\n---\n\n# Nia\n\nHarbour clerk who keeps the real tide numbers.\n";

    #[test]
    fn parses_explicit_paths_and_slots() {
        let docs = parse_storybible(BIBLE).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].path, "Wiki/Style/genre.md");
        assert!(docs[0].frontmatter.contains("genre: Fantasy"));
        assert!(!docs[0].frontmatter.contains("path:"));
        assert_eq!(docs[1].path, "Wiki/Characters/Nia_Farrow.md");
        assert!(docs[1].body.starts_with("# Nia"));
    }

    #[test]
    fn text_outside_documents_is_ignored() {
        let docs =
            parse_storybible("preamble\n\n---\nslot: synopsis\n---\n\nbody\n\ntrailing\n").unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].path, "Wiki/Story/synopsis.md");
        assert_eq!(docs[0].body, "body\n\ntrailing");
    }

    #[test]
    fn a_horizontal_rule_in_a_body_does_not_split_the_document() {
        let text = "---\nslot: theme\n---\n\nfirst part\n\n---\n\nsecond part\n";
        let docs = parse_storybible(text).unwrap();
        assert_eq!(docs.len(), 1);
        assert!(docs[0].body.contains("second part"));
    }

    #[test]
    fn round_trips_through_render() {
        let docs = parse_storybible(BIBLE).unwrap();
        let text = render_storybible(&docs, Path::new("/books/salt/storybible.md"));
        let again = parse_storybible(&text).unwrap();
        assert_eq!(docs, again);
    }

    #[test]
    fn chapter_scoped_slots_need_a_number() {
        let text = "---\nslot: scene\nchapter: 2\n---\n\nbeats\n";
        assert_eq!(
            parse_storybible(text).unwrap()[0].path,
            "Wiki/Outline/Chapter_02_Scene.md"
        );
        let psych = "---\nslot: psych\nchapter: 12\n---\n\ninner life\n";
        assert_eq!(
            parse_storybible(psych).unwrap()[0].path,
            "Wiki/Psych/Chapter_12_Psych.md"
        );
        let chapter = "---\nslot: chapter\nchapter: 3\n---\n\nprose\n";
        assert_eq!(
            parse_storybible(chapter).unwrap()[0].path,
            "Chapters/Chapter-003.md"
        );
        let missing = parse_storybible("---\nslot: scene\n---\n\nbeats\n").unwrap_err();
        assert!(missing.to_string().contains("chapter"));
    }

    #[test]
    fn named_slots_need_a_name() {
        let err = parse_storybible("---\nslot: location\n---\n\nDocks\n").unwrap_err();
        assert!(err.to_string().contains("name"));
        let ok =
            parse_storybible("---\nslot: location\nname: The Long Pier\n---\n\nDocks\n").unwrap();
        assert_eq!(ok[0].path, "Wiki/Locations/The_Long_Pier.md");
    }

    #[test]
    fn escapes_and_unknown_slots_are_rejected() {
        assert!(parse_storybible("---\npath: ../outside.md\n---\n\nx\n").is_err());
        assert!(parse_storybible("---\npath: /etc/passwd\n---\n\nx\n").is_err());
        assert!(parse_storybible("---\npath: Wiki/notes.txt\n---\n\nx\n").is_err());
        let unknown = parse_storybible("---\nslot: vibes\n---\n\nx\n").unwrap_err();
        assert!(unknown.to_string().contains("unknown slot"));
    }

    #[test]
    fn an_empty_file_is_not_a_bible() {
        let err = parse_storybible("# nothing here\n").unwrap_err();
        assert!(matches!(err, Error::InvalidStoryBible(_)));
    }

    #[test]
    fn finds_a_bible_beside_the_folder_or_one_level_down() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("salt-ledger");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join(STORYBIBLE_FILE), BIBLE).unwrap();
        assert_eq!(
            find_storybible(tmp.path()).unwrap(),
            nested.join(STORYBIBLE_FILE),
            "a bible one level down still names the book folder"
        );
        fs::write(tmp.path().join(STORYBIBLE_FILE), BIBLE).unwrap();
        assert_eq!(
            find_storybible(tmp.path()).unwrap(),
            tmp.path().join(STORYBIBLE_FILE),
            "the folder's own bible wins"
        );
    }

    #[test]
    fn import_preview_keeps_subfolder_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("salt-ledger");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join(STORYBIBLE_FILE), BIBLE).unwrap();
        let preview = import_preview(tmp.path()).unwrap();
        assert!(preview.contains("- salt-ledger/Wiki/Style/genre.md"));
        let docs = parse_storybible(&preview).unwrap();
        assert_eq!(docs[0].path, "salt-ledger/Wiki/Style/genre.md");
    }

    #[test]
    fn reads_a_working_title_out_of_any_document() {
        assert_eq!(storybible_title(BIBLE).as_deref(), Some("Salt Ledger"));
        assert_eq!(storybible_title("\n# nothing\n"), None);
    }

    #[test]
    fn slot_table_matches_the_skill_output_map() {
        for (slot, path) in FIXED_SLOTS {
            let skill = match slot {
                "genre" => "fiction-genre",
                "audience" => "fiction-audience",
                "theme" => "fiction-theme",
                "synopsis" => "fiction-synopsis",
                "style" => "fiction-style",
                "voice" => "fiction-voiceprompt",
                "outline" => "fiction-outline",
                other => panic!("unmapped slot {other}"),
            };
            assert_eq!(crate::output::output_rel_path(skill), Some(path));
        }
    }
}
