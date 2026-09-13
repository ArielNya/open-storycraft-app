//! Order-2 Markov names, ported from `name-generator/scripts/generate.py`.

use std::collections::HashMap;
use std::path::Path;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::Error;

const START: char = '\u{0000}';
const END: char = '\u{0001}';

/// Parameters for [`generate_from_path`] and [`generate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateOpts {
    /// How many unique names to try for.
    pub count: usize,
    /// Minimum generated length (inclusive).
    pub min_len: usize,
    /// Maximum generated length (inclusive).
    pub max_len: usize,
    /// Markov order. The Python tools default to 2.
    pub order: usize,
    /// Optional RNG seed for tests and retries.
    pub seed: Option<u64>,
}

impl Default for GenerateOpts {
    fn default() -> Self {
        Self {
            count: 10,
            min_len: 4,
            max_len: 12,
            order: 2,
            seed: None,
        }
    }
}

/// Load non-empty trimmed lines from a `data/*.txt` list.
///
/// # Errors
///
/// Returns [`Error::Io`] or [`Error::EmptyList`].
pub fn load_list(path: &Path) -> Result<Vec<String>, Error> {
    let text = std::fs::read_to_string(path).map_err(|err| Error::io(path, err))?;
    parse_list(&text)
}

/// Parse a name/town list from already-read text.
///
/// # Errors
///
/// Returns [`Error::EmptyList`] when every line is blank.
pub fn parse_list(text: &str) -> Result<Vec<String>, Error> {
    let names: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if names.is_empty() {
        Err(Error::EmptyList)
    } else {
        Ok(names)
    }
}

/// Generate unique names from a list file.
///
/// # Errors
///
/// Returns [`Error::Io`], [`Error::EmptyList`], [`Error::InvalidOrder`],
/// or [`Error::Exhausted`].
pub fn generate_from_path(path: &Path, opts: &GenerateOpts) -> Result<Vec<String>, Error> {
    let names = load_list(path)?;
    generate(&names, opts)
}

/// Generate unique names from an in-memory list.
///
/// Partial results are returned when some, but not all, names could be
/// produced. [`Error::Exhausted`] is only for zero names.
///
/// # Errors
///
/// Returns [`Error::EmptyList`], [`Error::InvalidOrder`], or [`Error::Exhausted`].
pub fn generate(names: &[String], opts: &GenerateOpts) -> Result<Vec<String>, Error> {
    if names.is_empty() {
        return Err(Error::EmptyList);
    }
    if !(1..=3).contains(&opts.order) {
        return Err(Error::InvalidOrder);
    }
    let model = build_model(names, opts.order);
    let mut rng = match opts.seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_entropy(),
    };
    let wanted = opts.count.max(1);
    let mut generated = Vec::with_capacity(wanted);
    let mut seen = std::collections::HashSet::new();
    let max_attempts = wanted.saturating_mul(500);
    for _ in 0..max_attempts {
        if generated.len() >= wanted {
            break;
        }
        let Some(name) = generate_one(&model, opts, &mut rng) else {
            continue;
        };
        let key = name.to_ascii_lowercase();
        if seen.insert(key) {
            generated.push(name);
        }
    }
    if generated.is_empty() {
        Err(Error::Exhausted)
    } else {
        Ok(generated)
    }
}

fn build_model(names: &[String], order: usize) -> HashMap<String, Vec<char>> {
    let mut model: HashMap<String, Vec<char>> = HashMap::new();
    for name in names {
        let lower = name.trim().to_ascii_lowercase();
        if lower.chars().count() < order + 1 {
            continue;
        }
        let mut padded = String::new();
        for _ in 0..order {
            padded.push(START);
        }
        padded.push_str(&lower);
        padded.push(END);
        let chars: Vec<char> = padded.chars().collect();
        if chars.len() <= order {
            continue;
        }
        for i in 0..chars.len() - order {
            let key: String = chars[i..i + order].iter().collect();
            model.entry(key).or_default().push(chars[i + order]);
        }
    }
    model
}

fn generate_one(
    model: &HashMap<String, Vec<char>>,
    opts: &GenerateOpts,
    rng: &mut StdRng,
) -> Option<String> {
    for _ in 0..200 {
        let mut key: String = std::iter::repeat_n(START, opts.order).collect();
        let mut result = String::new();
        for _ in 0..opts.max_len.saturating_add(opts.order) {
            let choices = model.get(&key)?;
            if choices.is_empty() {
                break;
            }
            let idx = rng.gen_range(0..choices.len());
            let next = choices[idx];
            if next == END {
                break;
            }
            result.push(next);
            let mut chars: Vec<char> = key.chars().collect();
            if !chars.is_empty() {
                chars.remove(0);
            }
            chars.push(next);
            key = chars.into_iter().collect();
        }
        let len = result.chars().count();
        if len >= opts.min_len && len <= opts.max_len {
            return Some(capitalize(&result));
        }
    }
    None
}

fn capitalize(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut out = String::with_capacity(name.len());
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
            out
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn fantasy_list() -> Vec<String> {
        [
            "Aelindra",
            "Baelon",
            "Caelith",
            "Daerion",
            "Elowyn",
            "Faeron",
            "Galadwen",
            "Haelith",
            "Iorath",
            "Jaelindra",
            "Kaelthas",
            "Laeris",
            "Maelorn",
            "Naelith",
            "Orandel",
            "Paelora",
            "Quenara",
            "Raelith",
            "Saelwyn",
            "Taelon",
        ]
        .into_iter()
        .map(ToOwned::to_owned)
        .collect()
    }

    #[test]
    fn seed_is_deterministic() {
        let opts = GenerateOpts {
            count: 5,
            seed: Some(42),
            ..GenerateOpts::default()
        };
        let a = generate(&fantasy_list(), &opts).unwrap();
        let b = generate(&fantasy_list(), &opts).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 5);
        for name in &a {
            let n = name.chars().count();
            assert!((4..=12).contains(&n), "{name}");
            let first = name.chars().next().unwrap();
            assert!(first.is_uppercase(), "{name}");
        }
    }

    #[test]
    fn empty_list_errors() {
        let err = generate(&[], &GenerateOpts::default()).unwrap_err();
        assert!(matches!(err, Error::EmptyList));
    }

    #[test]
    fn rejects_bad_order() {
        let opts = GenerateOpts {
            order: 4,
            ..GenerateOpts::default()
        };
        let err = generate(&fantasy_list(), &opts).unwrap_err();
        assert!(matches!(err, Error::InvalidOrder));
    }
}
