//! Resume / new-project routing. World pack is optional and never blocks.

use crate::Mode;
use crate::project::ProjectRoot;
use crate::status::{NextAction, Slot, SlotState};

/// Planning spine, excluding the optional world pack.
///
/// Order matches `open-storycraft/SKILL.md` Step 3 (`new-project` / `resume`):
/// genre → audience → theme → synopsis → style → characters → voice →
/// outline → scenes → psych → writechapter.
const SPINE: &[(&str, Slot)] = &[
    ("fiction-genre", Slot::Genre),
    ("fiction-audience", Slot::Audience),
    ("fiction-theme", Slot::Theme),
    ("fiction-synopsis", Slot::Synopsis),
    ("fiction-style", Slot::Style),
    ("fiction-characters", Slot::Characters),
    ("fiction-voiceprompt", Slot::Voice),
    ("fiction-outline", Slot::Outline),
    ("fiction-scenes", Slot::Scenes),
    ("fiction-psych", Slot::Psych),
    ("fiction-writechapter", Slot::Chapters),
];

pub(crate) fn next_action(
    project: Option<&ProjectRoot>,
    mode: Mode,
    chapter: u32,
    slots: &[SlotState; 12],
) -> NextAction {
    match mode {
        Mode::Spark => NextAction {
            skill: Some("fiction-story-sparks".to_owned()),
            why: "spark mode never writes Wiki files".to_owned(),
            missing: Vec::new(),
        },
        Mode::Meta => NextAction {
            skill: Some("skill-builder".to_owned()),
            why: "library / skill-authoring job".to_owned(),
            missing: Vec::new(),
        },
        Mode::Edit => NextAction {
            skill: None,
            why: "name one editorial pass; do not run the whole ladder".to_owned(),
            missing: Vec::new(),
        },
        Mode::SingleSkill => NextAction {
            skill: None,
            why: "name the skill to run".to_owned(),
            missing: Vec::new(),
        },
        Mode::WorldPack => NextAction {
            skill: Some("fiction-world".to_owned()),
            why: "optional world pack; use name-generator / town-generator for invented names"
                .to_owned(),
            missing: missing_if_incomplete(slots, &[Slot::Synopsis, Slot::Characters]),
        },
        Mode::NewProject | Mode::Resume => resume_spine(project, chapter, slots),
        Mode::Draft => draft_next(chapter, slots),
    }
}

fn slot(slots: &[SlotState; 12], which: Slot) -> SlotState {
    slots[which as usize]
}

fn missing_if_incomplete(slots: &[SlotState; 12], required: &[Slot]) -> Vec<Slot> {
    required
        .iter()
        .copied()
        .filter(|item| !slot(slots, *item).is_complete())
        .collect()
}

fn resume_spine(
    project: Option<&ProjectRoot>,
    chapter: u32,
    slots: &[SlotState; 12],
) -> NextAction {
    if project.is_none() {
        return NextAction {
            skill: Some("fiction-genre".to_owned()),
            why: "no Wiki found; start a new project".to_owned(),
            missing: vec![Slot::Genre],
        };
    }
    for (skill, required) in SPINE {
        if !slot(slots, *required).is_complete() {
            return NextAction {
                skill: Some((*skill).to_owned()),
                why: why_incomplete(*required, chapter),
                missing: vec![*required],
            };
        }
    }
    NextAction {
        skill: None,
        why: "spine complete; pick a chapter to draft or an editorial pass".to_owned(),
        missing: Vec::new(),
    }
}

fn draft_next(chapter: u32, slots: &[SlotState; 12]) -> NextAction {
    let contract = missing_if_incomplete(slots, &[Slot::Synopsis, Slot::Outline]);
    if !contract.is_empty() {
        let skill = if !slot(slots, Slot::Synopsis).is_complete() {
            "fiction-synopsis"
        } else {
            "fiction-outline"
        };
        return NextAction {
            skill: Some(skill.to_owned()),
            why: "draft requires synopsis and outline".to_owned(),
            missing: contract,
        };
    }
    if !slot(slots, Slot::Scenes).is_complete() {
        return NextAction {
            skill: Some("fiction-scenes".to_owned()),
            why: format!("offer scene plan for chapter {chapter} before drafting"),
            missing: vec![Slot::Scenes],
        };
    }
    if !slot(slots, Slot::Psych).is_complete() {
        return NextAction {
            skill: Some("fiction-psych".to_owned()),
            why: format!("offer psych pass for chapter {chapter} before drafting"),
            missing: vec![Slot::Psych],
        };
    }
    NextAction {
        skill: Some("fiction-writechapter".to_owned()),
        why: format!("draft chapter {chapter} into a preview, not the live file"),
        missing: Vec::new(),
    }
}

fn why_incomplete(slot: Slot, chapter: u32) -> String {
    match slot {
        Slot::Genre => "Wiki/Style/genre.md is missing or has no genre field".to_owned(),
        Slot::Audience => "Wiki/Style/audience.md is missing or empty".to_owned(),
        Slot::Theme => "Wiki/Story/theme.md is missing or empty".to_owned(),
        Slot::Synopsis => "Wiki/Story/synopsis.md is missing or an empty stub".to_owned(),
        Slot::Style => "Wiki/Style/style_guide.md is missing or empty".to_owned(),
        Slot::Characters => "Wiki/Characters/ has no real character sheet".to_owned(),
        Slot::World => "world files are optional and not required for next".to_owned(),
        Slot::Outline => "Wiki/Outline/outline.md is missing or empty".to_owned(),
        Slot::Scenes => format!("no scene/beat file for chapter {chapter}"),
        Slot::Voice => "Wiki/Style/voice_prompt.md is missing or empty".to_owned(),
        Slot::Psych => format!("no Wiki/Psych file for chapter {chapter}"),
        Slot::Chapters => format!("no chapter prose file for chapter {chapter}"),
    }
}
