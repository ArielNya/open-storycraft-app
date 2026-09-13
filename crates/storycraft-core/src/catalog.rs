//! Skill catalog. Frontmatter + linked refs only — never the whole library body.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;

/// Parsed skill package. The `SKILL.md` body is not retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillManifest {
    /// Directory name; this is the skill id the orchestrator uses.
    pub name: String,
    /// One-line description from frontmatter.
    pub description: String,
    /// `metadata.version` if present.
    pub version: Option<String>,
    /// `metadata.category` if present.
    pub category: Option<String>,
    /// `metadata.workflow-position` if present.
    pub workflow_position: Option<String>,
    /// Skill ids this skill lists as requires.
    pub requires: Vec<String>,
    /// `metadata.next-skill` if present.
    pub next_skill: Option<String>,
    /// `metadata.output-format` if present.
    pub output_format: Option<String>,
    /// Markdown / JSON files the skill text actually links.
    pub reference_files: Vec<String>,
    /// Folder that contains `SKILL.md`.
    pub skill_dir: PathBuf,
}

impl SkillManifest {
    /// Path of this skill's `SKILL.md`.
    #[must_use]
    pub fn skill_md(&self) -> PathBuf {
        self.skill_dir.join("SKILL.md")
    }
}

/// On-disk index of child skill folders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    root: PathBuf,
    skills: BTreeMap<String, SkillManifest>,
}

impl Catalog {
    /// Load every immediate child of `skills_dir` that contains `SKILL.md`.
    ///
    /// The orchestrator's own root `SKILL.md` is not a child folder, so it is
    /// not indexed. Each file is read, frontmatter extracted, body dropped.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the skills directory cannot be read, or
    /// [`Error::InvalidManifest`] if a `SKILL.md` is not valid UTF-8.
    pub fn load(skills_dir: &Path) -> Result<Self, Error> {
        let mut skills = BTreeMap::new();
        let entries = fs::read_dir(skills_dir).map_err(|err| Error::io(skills_dir, err))?;
        for entry in entries {
            let entry = entry.map_err(|err| Error::io(skills_dir, err))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.is_file() {
                continue;
            }
            let manifest = load_manifest(&path, &skill_md)?;
            skills.insert(manifest.name.clone(), manifest);
        }
        Ok(Self {
            root: skills_dir.to_path_buf(),
            skills,
        })
    }

    /// Directory the catalog was loaded from.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Look up a skill by folder name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&SkillManifest> {
        self.skills.get(name)
    }

    /// Skills in name order.
    pub fn iter(&self) -> impl Iterator<Item = &SkillManifest> {
        self.skills.values()
    }

    /// Skills visible with the current overlay enables.
    pub fn iter_visible<'a>(
        &'a self,
        enabled_overlays: &'a [String],
    ) -> impl Iterator<Item = &'a SkillManifest> + 'a {
        self.skills
            .values()
            .filter(|skill| crate::overlay::overlay_allowed(&skill.name, enabled_overlays))
    }

    /// Number of indexed skills.
    #[must_use]
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Whether the catalog is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

/// Walk up from `start` (then cwd) looking for `open-storycraft/` or `skills/`.
#[must_use]
pub fn find_skills_dir(start: &Path) -> Option<PathBuf> {
    if let Ok(from_env) = std::env::var("STORYCRAFT_SKILLS") {
        let path = PathBuf::from(from_env);
        if looks_like_skills_dir(&path) {
            return Some(path);
        }
    }
    let mut dir = start.to_path_buf();
    loop {
        for name in ["open-storycraft", "skills"] {
            let candidate = dir.join(name);
            if looks_like_skills_dir(&candidate) {
                return Some(candidate);
            }
        }
        if looks_like_skills_dir(&dir) {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn looks_like_skills_dir(path: &Path) -> bool {
    path.join("fiction-genre/SKILL.md").is_file()
}

fn load_manifest(skill_dir: &Path, skill_md: &Path) -> Result<SkillManifest, Error> {
    let bytes = fs::read(skill_md).map_err(|err| Error::io(skill_md, err))?;
    let text = std::str::from_utf8(&bytes).map_err(|_| Error::InvalidManifest {
        path: skill_md.to_path_buf(),
        detail: "not valid UTF-8".to_owned(),
    })?;
    let (yaml, body) = split_frontmatter(text);
    let fields = parse_frontmatter_fields(yaml.unwrap_or(""));
    let folder_name = skill_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| Error::InvalidManifest {
            path: skill_md.to_path_buf(),
            detail: "skill directory has no name".to_owned(),
        })?;
    let requires = parse_requires(fields.get("requires").map(String::as_str).unwrap_or(""));
    Ok(SkillManifest {
        name: folder_name,
        description: fields.get("description").cloned().unwrap_or_default(),
        version: fields.get("version").cloned(),
        category: fields.get("category").cloned(),
        workflow_position: fields.get("workflow-position").cloned(),
        requires,
        next_skill: fields.get("next-skill").cloned(),
        output_format: fields.get("output-format").cloned(),
        reference_files: extract_linked_files(body),
        skill_dir: skill_dir.to_path_buf(),
    })
}

/// First YAML frontmatter document, if the file starts with `---`.
#[must_use]
pub fn split_frontmatter(text: &str) -> (Option<&str>, &str) {
    let text = text.trim_start_matches('\u{feff}');
    let Some(rest) = text.strip_prefix("---") else {
        return (None, text);
    };
    let rest = rest.strip_prefix('\r').unwrap_or(rest);
    let Some(rest) = rest.strip_prefix('\n') else {
        return (None, text);
    };
    let mut search_from = 0;
    while let Some(rel) = rest[search_from..].find("\n---") {
        let abs = search_from + rel;
        let after = &rest[abs + 4..];
        let after = after.strip_prefix('\r').unwrap_or(after);
        if after.starts_with('\n') || after.is_empty() {
            let yaml = &rest[..abs];
            let body = after.strip_prefix('\n').unwrap_or(after);
            return (Some(yaml), body);
        }
        search_from = abs + 4;
    }
    (None, text)
}

/// Scalar `key:` from the first frontmatter block.
#[must_use]
pub fn frontmatter_scalar(text: &str, key: &str) -> Option<String> {
    let (yaml, _) = split_frontmatter(text);
    parse_frontmatter_fields(yaml?).remove(key)
}

fn parse_frontmatter_fields(yaml: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for raw in yaml.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "metadata:" || trimmed.starts_with("metadata:") {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = unquote(value.trim());
        if value.is_empty() {
            continue;
        }
        fields.insert(key.to_owned(), value);
    }
    fields
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].to_owned();
        }
    }
    value.to_owned()
}

fn parse_requires(value: &str) -> Vec<String> {
    if value.is_empty() || value == "null" || value == "~" {
        return Vec::new();
    }
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty() && *item != "null")
        .map(ToOwned::to_owned)
        .collect()
}

fn extract_linked_files(body: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("](") {
        rest = &rest[start + 2..];
        let Some(end) = rest.find(')') else {
            break;
        };
        let target = rest[..end].split_whitespace().next().unwrap_or("");
        let target = target.trim_matches('"').split('#').next().unwrap_or(target);
        rest = &rest[end + 1..];
        if (target.starts_with("references/")
            || target.starts_with("assets/")
            || target.starts_with("data/"))
            && !files.iter().any(|existing| existing == target)
        {
            files.push(target.to_owned());
        }
    }
    files
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn splits_standard_frontmatter() {
        let text = "---\nname: fiction-genre\n---\n\n# Genre\n";
        let (yaml, body) = split_frontmatter(text);
        assert_eq!(yaml, Some("name: fiction-genre"));
        assert!(body.trim_start().starts_with("# Genre"));
    }

    #[test]
    fn extracts_only_linked_refs() {
        let body = "See [a](references/genre-fantasy.md) and [b](https://example.com) and [c](assets/output-template.json).";
        let files = extract_linked_files(body);
        assert_eq!(
            files,
            vec!["references/genre-fantasy.md", "assets/output-template.json"]
        );
    }

    #[test]
    fn parses_comma_requires() {
        assert_eq!(
            parse_requires("fiction-synopsis, fiction-characters"),
            vec!["fiction-synopsis", "fiction-characters"]
        );
        assert!(parse_requires("null").is_empty());
    }

    #[test]
    fn iter_visible_hides_overlays_until_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("fiction-genre")).unwrap();
        fs::write(
            tmp.path().join("fiction-genre/SKILL.md"),
            "---\nname: fiction-genre\ndescription: genre\n---\n# Genre\n",
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("ao3-writer")).unwrap();
        fs::write(
            tmp.path().join("ao3-writer/SKILL.md"),
            "---\nname: ao3-writer\ndescription: overlay\n---\n# AO3\n",
        )
        .unwrap();
        let catalog = Catalog::load(tmp.path()).unwrap();
        let hidden: Vec<&str> = catalog
            .iter_visible(&[])
            .map(|skill| skill.name.as_str())
            .collect();
        assert_eq!(hidden, vec!["fiction-genre"]);
        let enabled = vec!["ao3-writer".to_owned()];
        let shown: Vec<&str> = catalog
            .iter_visible(&enabled)
            .map(|skill| skill.name.as_str())
            .collect();
        assert_eq!(shown, vec!["ao3-writer", "fiction-genre"]);
    }
}
