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

/// [`find_storybible`] as a path relative to `root`, with `/` separators — the
/// form a job's `output_path` takes.
#[must_use]
pub fn find_storybible_rel(root: &Path) -> Option<String> {
    let found = find_storybible(root)?;
    let rel = found.strip_prefix(root).ok()?;
    Some(rel.to_string_lossy().replace('\\', "/"))
}

/// Whether `text` is already a storybible the importer can unpack.
#[must_use]
pub fn is_importable(text: &str) -> bool {
    parse_storybible(text).is_ok()
}

/// Cut a free-form source into pieces of at most `max_chars`, at headings
/// where possible, then at blank lines, and only as a last resort mid-text.
///
/// Every byte of `text` lands in exactly one piece, in order, so converting
/// the pieces one at a time loses nothing.
#[must_use]
pub fn split_source(text: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(1);
    // Sections: a heading line starts a new one.
    let mut sections: Vec<String> = Vec::new();
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with('#') || sections.is_empty() {
            sections.push(String::new());
        }
        if let Some(last) = sections.last_mut() {
            last.push_str(line);
        }
    }
    let mut pieces: Vec<String> = Vec::new();
    let mut current = String::new();
    for section in sections {
        for part in fit(&section, max_chars) {
            if !current.is_empty() && current.len() + part.len() > max_chars {
                pieces.push(std::mem::take(&mut current));
            }
            current.push_str(&part);
        }
    }
    if !current.trim().is_empty() {
        pieces.push(current);
    }
    pieces
}

/// One section as parts no longer than `max_chars`: whole, else by paragraph,
/// else hard cuts on character boundaries.
fn fit(section: &str, max_chars: usize) -> Vec<String> {
    if section.len() <= max_chars {
        return vec![section.to_owned()];
    }
    let mut parts = Vec::new();
    let mut current = String::new();
    for para in section.split_inclusive("\n\n") {
        if !current.is_empty() && current.len() + para.len() > max_chars {
            parts.push(std::mem::take(&mut current));
        }
        let mut rest = para;
        while rest.len() > max_chars {
            let mut cut = max_chars;
            while !rest.is_char_boundary(cut) {
                cut -= 1;
            }
            parts.push(rest[..cut].to_owned());
            rest = &rest[cut..];
        }
        current.push_str(rest);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// Fold documents that land on the same file into one, keeping first-seen
/// order.
///
/// A converted bible often describes one character or place in several
/// sections. The importer writes one file per document, so without this the
/// last mention would silently replace the others. Frontmatter keys from the
/// first document win; later ones only add keys it lacked. Bodies are joined,
/// skipping exact repeats.
#[must_use]
pub fn merge_documents(docs: Vec<BibleDoc>) -> Vec<BibleDoc> {
    let mut merged: Vec<BibleDoc> = Vec::new();
    for doc in docs {
        let Some(existing) = merged.iter_mut().find(|m| m.path == doc.path) else {
            merged.push(doc);
            continue;
        };
        let known: Vec<String> = frontmatter_entries(&existing.frontmatter)
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        for (key, entry) in frontmatter_entries(&doc.frontmatter) {
            if !known.contains(&key) {
                if !existing.frontmatter.is_empty() {
                    existing.frontmatter.push('\n');
                }
                existing.frontmatter.push_str(&entry);
            }
        }
        let body = doc.body.trim();
        if !body.is_empty() && !existing.body.contains(body) {
            if !existing.body.is_empty() {
                existing.body.push_str("\n\n");
            }
            existing.body.push_str(body);
        }
    }
    merged
}

/// Frontmatter as `(key, entry)` pairs, where an entry is the `key:` line plus
/// the indented or `- ` list lines that belong to it.
fn frontmatter_entries(frontmatter: &str) -> Vec<(String, String)> {
    let mut entries: Vec<(String, String)> = Vec::new();
    for line in frontmatter.lines() {
        let continues = line.starts_with(' ') || line.starts_with('\t') || line.starts_with("- ");
        match entries.last_mut() {
            Some((_, entry)) if continues => {
                entry.push('\n');
                entry.push_str(line);
            }
            _ => {
                let key = line.split_once(':').map_or(line, |(k, _)| k).trim();
                entries.push((key.to_owned(), line.to_owned()));
            }
        }
    }
    entries
}

/// The documents of a storybible as one importable text, with no header.
#[must_use]
pub fn documents_text(docs: &[BibleDoc]) -> String {
    let mut out = String::new();
    for doc in docs {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&doc.document());
    }
    out
}

/// Join the per-section answers of a bible conversion into one importable
/// storybible, merging documents that land on the same file.
///
/// A section answered with `NO DOCUMENTS` (or nothing) is skipped on purpose.
///
/// # Errors
///
/// Returns [`Error::InvalidStoryBible`] naming the first section whose answer
/// is not storybible documents — dropping it would silently lose canon — or
/// when no section produced any document.
pub fn assemble_converted(answers: &[String]) -> Result<String, Error> {
    let mut docs = Vec::new();
    for (i, answer) in answers.iter().enumerate() {
        let text = crate::chunk::unwrap_model_output(answer);
        let text = text.trim();
        if text.is_empty() || text == "NO DOCUMENTS" {
            continue;
        }
        let section = parse_storybible(text).map_err(|err| {
            Error::InvalidStoryBible(format!(
                "section {} of {} did not convert: {err}",
                i.saturating_add(1),
                answers.len()
            ))
        })?;
        docs.extend(section);
    }
    if docs.is_empty() {
        return Err(Error::InvalidStoryBible(
            "the conversion produced no documents; nothing in the source mapped to the book".into(),
        ));
    }
    let docs = merge_documents(docs);
    let mut out = String::from("# Storybible\n\nConverted by storybible-convert. Files:\n\n");
    for doc in &docs {
        let _ = writeln!(out, "- {}", doc.path);
    }
    out.push('\n');
    out.push_str(&documents_text(&docs));
    Ok(out)
}

/// Route a model answer for a skill that writes several files.
///
/// The prompt asks for storybible documents (`path:` in each frontmatter), and
/// when the answer has them they are used as given, restricted to `paths`.
/// Models also answer with one fenced block per file, or with plain files in a
/// row; those are mapped onto `paths` in order. Fewer files than `paths` is
/// accepted — the board shows what is still missing — more is not.
///
/// # Errors
///
/// Returns [`Error::InvalidPreview`] when nothing routable is found, a routed
/// document names a file the skill does not write, or there are more files
/// than destinations.
pub fn route_files(text: &str, paths: &[&str]) -> Result<String, Error> {
    if let Ok(docs) = parse_storybible(text) {
        if let Some(stray) = docs.iter().find(|doc| !paths.contains(&doc.path.as_str())) {
            return Err(Error::InvalidPreview(format!(
                "the answer writes {}, which this skill does not produce (expected: {})",
                stray.path,
                paths.join(", ")
            )));
        }
        return Ok(documents_text(&merge_documents(docs)));
    }
    let parts = fenced_blocks(text).unwrap_or_else(|| split_files(text));
    if parts.is_empty() {
        return Err(Error::InvalidPreview("output is empty".into()));
    }
    if parts.len() > paths.len() {
        return Err(Error::InvalidPreview(format!(
            "the answer has {} files; this skill writes {} ({})",
            parts.len(),
            paths.len(),
            paths.join(", ")
        )));
    }
    let docs: Vec<BibleDoc> = parts
        .iter()
        .zip(paths)
        .map(|(part, path)| {
            let (frontmatter, body) = split_frontmatter(part);
            BibleDoc {
                path: (*path).to_owned(),
                frontmatter,
                body,
            }
        })
        .collect();
    Ok(documents_text(&docs))
}

/// The contents of each top-level ```` ``` ```` block, if the text has any.
fn fenced_blocks(text: &str) -> Option<Vec<String>> {
    let mut blocks = Vec::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            match current.take() {
                Some(block) => blocks.push(block),
                None => current = Some(String::new()),
            }
            continue;
        }
        if let Some(block) = current.as_mut() {
            block.push_str(line);
            block.push('\n');
        }
    }
    if blocks.is_empty() {
        None
    } else {
        Some(blocks)
    }
}

/// Plain files written one after another: a new file starts at each
/// frontmatter block that opens after the previous file's body.
fn split_files(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut starts = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if is_fence(lines[i])
            && (i == 0 || lines[i - 1].trim().is_empty())
            && let Some(end) = next_fence(&lines, i + 1)
            && !parse_keys(&lines[i + 1..end]).is_empty()
        {
            starts.push(i);
            i = end + 1;
            continue;
        }
        i += 1;
    }
    if starts.first() != Some(&0) {
        starts.insert(0, 0);
    }
    starts
        .iter()
        .enumerate()
        .map(|(n, &start)| {
            let end = starts.get(n + 1).copied().unwrap_or(lines.len());
            lines[start..end].join("\n")
        })
        .filter(|part| !part.trim().is_empty())
        .collect()
}

/// `(frontmatter, body)` of one file's text.
fn split_frontmatter(text: &str) -> (String, String) {
    let lines: Vec<&str> = text.trim().lines().collect();
    if lines.first().is_some_and(|line| is_fence(line))
        && let Some(end) = next_fence(&lines, 1)
    {
        return (
            lines[1..end].join("\n").trim().to_owned(),
            lines[end + 1..].join("\n").trim().to_owned(),
        );
    }
    (String::new(), text.trim().to_owned())
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
    fn a_crlf_bible_routes_like_an_lf_one() {
        let crlf = parse_storybible(&BIBLE.replace('\n', "\r\n")).unwrap();
        let lf = parse_storybible(BIBLE).unwrap();
        let paths = |docs: &[_]| {
            docs.iter()
                .map(|d: &BibleDoc| d.path.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(paths(&crlf), paths(&lf));
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
        assert_eq!(
            find_storybible_rel(tmp.path()).as_deref(),
            Some("salt-ledger/storybible.md"),
            "relative, with `/` on every platform"
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
    fn split_source_keeps_every_byte_in_order() {
        let text = "# One\nalpha\n\n## Two\nbeta beta beta\n\n# Three\n".to_owned()
            + &"long paragraph. ".repeat(20)
            + "\n\nend\n";
        let pieces = split_source(&text, 60);
        assert!(pieces.len() > 2, "{pieces:?}");
        assert!(pieces.iter().all(|p| p.len() <= 60), "{pieces:?}");
        assert_eq!(pieces.concat(), text);
        assert!(pieces[0].starts_with("# One"));
    }

    #[test]
    fn same_destination_documents_are_merged_not_replaced() {
        let docs = parse_storybible(
            "---\nslot: character\nname: Mira\nrole: protagonist\n---\n\nCounts coins twice.\n\n\
             ---\nslot: genre\ngenre: Fantasy\n---\n\n# Genre\n\n\
             ---\nslot: character\nname: Mira\nrole: villain\nage: 28\n---\n\nHates the tide office.\n",
        )
        .unwrap();
        let merged = merge_documents(docs);
        assert_eq!(merged.len(), 2);
        let mira = &merged[0];
        assert_eq!(mira.path, "Wiki/Characters/Mira.md");
        assert!(mira.frontmatter.contains("role: protagonist"));
        assert!(!mira.frontmatter.contains("villain"), "first value wins");
        assert!(mira.frontmatter.contains("age: 28"), "new keys are added");
        assert!(mira.body.contains("Counts coins") && mira.body.contains("tide office"));
    }

    const STYLE_FILES: [&str; 2] = ["Wiki/Style/style_guide.md", "Wiki/Style/review_guide.md"];

    #[test]
    fn two_fenced_files_route_in_order() {
        let answer = "```markdown\n---\ntitle: \"Style Guide\"\nperson: third\n---\n\n## Style\n\nClose third.\n```\n\n```markdown\n---\ntitle: \"Review Guide\"\n---\n\n## Forbidden Words\n- suddenly\n```\n";
        let routed = route_files(answer, &STYLE_FILES).unwrap();
        let docs = parse_storybible(&routed).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].path, STYLE_FILES[0]);
        assert!(docs[0].frontmatter.contains("person: third"));
        assert!(docs[0].body.starts_with("## Style"));
        assert_eq!(docs[1].path, STYLE_FILES[1]);
        assert!(docs[1].body.contains("suddenly"));
        assert!(!routed.contains("```"));
    }

    #[test]
    fn plain_files_in_a_row_route_in_order() {
        let answer = "---\ntitle: Style\n---\n\n## Style\n\nClose third.\n\n---\ntitle: Review\n---\n\n## Crutch Words\n- felt\n";
        let docs = parse_storybible(&route_files(answer, &STYLE_FILES).unwrap()).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[1].path, STYLE_FILES[1]);
        assert!(docs[1].body.contains("felt"));
    }

    #[test]
    fn routed_documents_are_used_as_given_but_only_for_known_files() {
        let answer = "---\npath: Wiki/Style/review_guide.md\n---\n\n## Forbidden Words\n\n---\npath: Wiki/Style/style_guide.md\nperson: first\n---\n\n## Style\n";
        let docs = parse_storybible(&route_files(answer, &STYLE_FILES).unwrap()).unwrap();
        assert_eq!(docs[0].path, STYLE_FILES[1]);
        let stray = "---\npath: Wiki/Story/theme.md\n---\n\nnope\n";
        assert!(route_files(stray, &STYLE_FILES).is_err());
        let one = "---\ntitle: Style\n---\n\n## Style\n\nOnly the guide.\n";
        assert_eq!(
            parse_storybible(&route_files(one, &STYLE_FILES).unwrap())
                .unwrap()
                .len(),
            1
        );
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
