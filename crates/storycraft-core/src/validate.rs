//! Cheap shape checks before a preview is offered for save.

use crate::Error;

/// Reject empty or obviously wrong previews.
///
/// # Errors
///
/// Returns [`Error::InvalidPreview`] when the text is empty or missing
/// headings the skill's output contract requires.
pub fn validate_preview(skill: &str, text: &str) -> Result<(), Error> {
    let trimmed = text.trim();
    if trimmed.len() < 40 {
        return Err(Error::InvalidPreview("output is empty or a stub".into()));
    }
    match skill {
        "fiction-story-sparks" => {
            require(trimmed, "Premise:")?;
            require(trimmed, "Central question:")?;
        }
        "fiction-genre" => {
            let lower = trimmed.to_ascii_lowercase();
            if !lower.contains("genre:") && !lower.contains("# genre") {
                return Err(Error::InvalidPreview(
                    "genre preview has no genre field or heading".into(),
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn require(text: &str, needle: &str) -> Result<(), Error> {
    if text.contains(needle) {
        Ok(())
    } else {
        Err(Error::InvalidPreview(format!("missing `{needle}`")))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn sparks_need_premise_and_question() {
        let ok = "Someone: a clerk\nSomewhere: a dock\nSomething: a ledger\nIt happens: the tide lies\n\nPremise: The harbour clerk finds the posted tide is a foot off.\nReader promise: A closed-circle correction of the public record.\nCentral question: Who gets to write the town's official weather?\nAnswer and payoff: She posts the real numbers.\n";
        validate_preview("fiction-story-sparks", ok).unwrap();
        validate_preview("fiction-story-sparks", "just a vibe").unwrap_err();
    }

    #[test]
    fn genre_needs_genre_marker() {
        let ok =
            "---\ngenre: Fantasy\nsubgenre: Low\n---\n\n# Genre\n\n## tone_notes\nWarm streets.\n";
        validate_preview("fiction-genre", ok).unwrap();
        validate_preview(
            "fiction-genre",
            "# Audience Profile\n\nLots of text about readers but no genre marker at all here.",
        )
        .unwrap_err();
    }
}
