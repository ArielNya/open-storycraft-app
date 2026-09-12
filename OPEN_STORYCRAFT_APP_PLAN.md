# Open Storycraft — Mobile-First Rust App

**Document type:** architecture and delivery plan  
**Status:** v0.1 — planning only, no implementation yet  
**Date:** 2026-09-12  
**Source library:** `open-storycraft.zip` (42 craft skills + `open-storycraft` orchestrator)

This plan turns the existing Agent-Skill library into a **memory-efficient, mobile-first Rust application** that:

- runs the Storycraft writing pipeline on-device
- talks to any **OpenAI-compatible** endpoint
- can sign in with **xAI Grok OAuth** so a SuperGrok (or eligible X Premium+) subscription can be used instead of an API key
- ships as a **sideloadable Android APK** (and later Play AAB), plus desktop from the same crate

---

## 1. What we are building

A local writing studio, not a chatbot wrapper.

The user owns a **book project** on disk (`Wiki/` + `Chapters/`). The app is a thin orchestrator that:

1. Detects project state from files.
2. Picks **one skill** at a time (the existing rule).
3. Builds a **budgeted context pack** from that skill’s `SKILL.md`, only the references that skill names, and only the Wiki files that skill requires.
4. Calls an LLM (Grok via OAuth, or any OpenAI-compat provider).
5. Writes the skill’s output file, updates the status board, and stops for confirmation.

That is exactly how the current `open-storycraft` orchestrator already works. The app is a host for that contract, not a rewrite of the craft skills.

### 1.1 Non-goals for v1

- Multi-user cloud collaboration
- Running a local 7B+ model on-phone as the default writer (optional later; RAM killer)
- Auto-chaining every editorial pass
- Reimplementing Grok.com’s full agent computer / connectors
- iOS App Store in the first APK milestone (iOS can share the Rust core later)
- Silently impersonating official xAI products

### 1.2 Product name and crate layout (proposed)

Working name: **Open Storycraft**  
Android applicationId: `dev.openstorycraft.app`  
Workspace:

```
open-storycraft-app/
├── crates/
│   ├── storycraft-core/      # project, skills, orchestrator, context packer
│   ├── storycraft-llm/       # providers, streaming, token accounting
│   ├── storycraft-auth/      # API keys + xAI device-code OAuth
│   ├── storycraft-tools/     # local generators / diagnostics (no LLM)
│   └── storycraft-app/       # Tauri 2 shell (Android + desktop)
├── skills/                   # vendored copy of the 42+1 skill folders
├── ui/                       # tiny Svelte (or vanilla) frontend
└── docs/
```

---

## 2. Constraints that shape the design

### 2.1 Memory (phones first)

Target device: mid-range Android, 4–6 GB RAM, WebView + Rust process.

Hard rules:

- **Never load the whole skill library into RAM.** Catalog is an on-disk index (~dozens of KB). Only the active skill’s `SKILL.md` plus named references are read.
- **Never load a whole book into the context window.** Packer has a token budget (default 24k–48k input depending on model) and a file-priority list per skill.
- Chapter files are streamed from disk. Editor keeps one chapter + a sliding window of neighbors.
- SQLite (or `redb`) for project index, not an in-memory graph of every markdown file.
- Frontend is a thin shell. All heavy work is Rust. No Node runtime on device.
- Skill Python scripts (name-generator, town-generator, burstiness `measure.py`) are **ported to Rust**. Do not ship CPython on Android.
- Release APK: LTO, `opt-level = "s"` or `"z"` for the UI crate, strip symbols, split per ABI (`arm64-v8a` primary).

Budget sketch (steady state, one chapter open):

| Piece | Target RSS |
|---|---|
| Rust core + SQLite | 20–40 MB |
| Skill + context pack buffers | 5–15 MB |
| WebView / UI | 80–150 MB |
| HTTP streaming buffers | 2–8 MB |
| **Total** | **~120–200 MB** |

If we miss that, the fallback is Slint/egui for the chrome and WebView only for the chapter editor.

### 2.2 Mobile-first UX

One-handed, interruptible, offline-capable for everything except LLM calls.

- Bottom nav: **Home / Book / Write / Edit / Settings**
- Status board is the home screen (the orchestrator’s board, not a chat log)
- Each skill run is a **job card**: questions → generate → diff/preview → save / reject
- Long generations survive app backgrounding via a foreground service + resumable SSE
- All confirmation gates from the skills (“What should I change?”) become explicit UI buttons, not buried chat
- Files live in app storage; optional SAF folder so the user can put the book on `/Documents/Books/<slug>/`

### 2.3 Skill fidelity

The zip is the source of truth. The app must not “run a skill in spirit.”

- Skills stay as folders with `SKILL.md` + `references/` + `assets/` + `data/`
- Orchestrator modes stay: `spark | new-project | resume | single-skill | draft | edit | world-pack | meta`
- One active skill at a time
- Optional overlays (`ao3-*`, `anti-slop-editor`) stay off unless the user enables them
- Proper names come from `name-generator` when a skill asks for invented names

---

## 3. Auth and model providers

Two first-class providers. The user asked for both.

### 3.1 OpenAI-compatible API (supported, stable)

Config per provider profile:

```
name: "local-lmstudio"
base_url: "http://192.168.1.10:1234/v1"
api_key: "sk-..." | none
models: [user-picked]
api_style: chat_completions | responses
```

Works with: OpenAI, xAI Console keys (`https://api.x.ai/v1`), Groq, Together, OpenRouter, vLLM, llama.cpp server, LM Studio, Ollama’s OpenAI shim, etc.

Implementation:

- `reqwest` + event-source parser for `text/event-stream`
- Support both `/v1/chat/completions` and `/v1/responses` (xAI docs now prefer Responses)
- Tool/function calling when the provider advertises it; otherwise the host performs tools and continues the turn
- Per-request timeout, retry on 429 with `Retry-After`, cancel token tied to the job card

Secrets stored in Android Keystore / OS keyring via Tauri plugin, never in plaintext markdown.

### 3.2 Grok OAuth / SuperGrok (requested, unofficial surface)

**Reality check — this must be in the product copy.**

Official xAI developer billing is **API keys at console.x.ai**. SuperGrok / X Premium+ is a **consumer subscription**. They are separate billing tracks.

A community OAuth path exists and is already used by Hermes, OpenClaw, LobeHub, OpenCode, Home Assistant, and others:

| Item | Value |
|---|---|
| Auth server | `https://auth.x.ai` |
| Device code | `POST /oauth2/device/code` |
| Token | `POST /oauth2/token` |
| Grant | `urn:ietf:params:oauth:grant-type:device_code` |
| Public client id used by Grok CLI ecosystem | `b1a00492-073a-47ea-816f-4c329264a828` |
| Scopes seen in the wild | `openid profile email offline_access grok-cli:access api:access` |
| Inference | `https://api.x.ai/v1` with `Authorization: Bearer <access_token>` |

On Android, **device-code is the correct grant**. Loopback `127.0.0.1:56121` (the desktop PKCE redirect used by some CLIs) does not work well in an APK. Flow:

1. App requests a device code.
2. UI shows the user code + “Open xAI” button (`verification_uri_complete` if present).
3. Custom Tab / system browser opens `auth.x.ai`.
4. App polls the token endpoint.
5. Access + refresh tokens go into Keystore.
6. Refresh a couple of minutes before expiry.

**Known failure modes we must handle in UI:**

- Login works, inference returns **403 tier denied** — xAI allowlists OAuth API access. SuperGrok is not a guarantee.
- Subscription bought on a different identity (Google vs X vs Apple vs email) does not attach.
- Client id / scopes can change without notice. Make client id + auth URLs configurable.
- Reusing the public Grok-CLI client id is an ecosystem convention, not an xAI partner agreement. Call it **“Sign in with xAI (community OAuth)”** and keep API-key as the reliable fallback.

Plan for v1:

1. Ship API-key path first (it will always work).
2. Ship device-code OAuth behind a settings toggle, with the 403 fallback copy.
3. Do **not** scrape grok.com cookies. That is brittle and ToS-hostile.
4. If xAI later offers an official public client for third-party apps, swap the client id in one config file.

### 3.3 Model picker

Default profiles:

- `grok-oauth` → whatever `/v1/models` returns for the token (prefer `grok-4.6` / current flagship)
- `xai-apikey` → same base URL, key from console
- `openai-compat` → user base URL

Per-skill model overrides later (cheap model for kill-passes, flagship for draft). v1 uses one default model per project.

---

## 4. Domain model

### 4.1 Project on disk (unchanged from the library)

```
<project>/
├── Wiki/
│   ├── Style/     genre.md audience.md style_guide.md review_guide.md voice_prompt.md
│   ├── Story/     theme.md synopsis.md
│   ├── Outline/   outline.md
│   ├── Characters/
│   ├── Locations/ Organizations/ Systems/ Events/
│   ├── Psych/
│   └── Scenes/
├── Chapters/
└── .storycraft/
    ├── project.toml      # title, default provider, token budget
    └── jobs/             # run logs, not canon
```

The orchestrator still finds a project by locating `Wiki/`. Multiple Wikis → picker.

### 4.2 Skill package

On disk, identical to the zip. Runtime struct:

```text
SkillManifest {
  name, description, version, category,
  workflow_position,
  requires: [skill names],
  next_skill,
  output_format,
  mode_hints,
  reference_files,   # declared, not “every md in the folder”
  local_tools,       # e.g. namegen, burstiness
}
```

Frontmatter already exists on most skills. A one-time `skill-index.json` is generated at build time so Android does not parse 42 YAML headers on every launch.

### 4.3 Job

A job is one skill execution:

```text
Job {
  id, project_id, skill, mode,
  inputs: { chapter, extra_paths },
  questions_answered,
  packed_context_hash,
  provider, model,
  status: queued | running | needs_confirm | saved | rejected | failed,
  preview_path,          # temp file
  output_path,           # Wiki / Chapters target
}
```

Jobs are resumable. That is how mobile backgrounding works.

---

## 5. Runtime architecture

```
┌─────────────────────────────────────────────┐
│  UI (Svelte, mobile-first, no framework bloat) │
│  status board · job cards · markdown editor   │
└──────────────┬──────────────────────────────┘
               │ Tauri IPC / invoke
┌──────────────▼──────────────────────────────┐
│  storycraft-app  (commands, Android plugins) │
└──────────────┬──────────────────────────────┘
               │
     ┌─────────┼────────────┬──────────────┐
     ▼         ▼            ▼              ▼
  core      llm          auth           tools
  orchestrator  streaming   keystore     namegen
  packer     openai-compat  oauth-device burstiness
  wiki-fs    token-count    refresh      markov towns
```

### 5.1 Orchestrator (port of `open-storycraft/SKILL.md`)

Exact mode table from the library:

| Mode | First move |
|---|---|
| spark | `fiction-story-sparks` only, no Wiki writes |
| new-project | spine starting at `fiction-genre` |
| resume | first missing required file on the spine |
| single-skill | named skill, refuse if requires are missing |
| draft | require synopsis + outline; offer scenes/psych; then `fiction-writechapter` |
| edit | narrowest editorial ladder step |
| world-pack | matching world skill + name/town generators |
| meta | `skill-builder` |

Spine order stays:

1. genre → audience → theme → synopsis → style → characters → voice  
2. optional world pack  
3. outline → scenes → psych → writechapter  
4. editorial ladder on demand

The app prints the status board from disk every time, never from memory.

### 5.2 Context packer (the memory-critical piece)

Input: active skill + project + chapter number + token budget.

Algorithm:

1. Always include: skill `SKILL.md` (maybe truncated after the procedure section if huge).
2. Include only references the skill text actually links (`references/foo.md`), not the whole `fiction-genre/references/` dump (93 files).
3. Include required Wiki files in skill-defined priority. Example for `fiction-writechapter`: scene plan for N, voice_prompt, style_guide, synopsis short section, previous chapter tail, psych for N, character sheets **mentioned in the scene**.
4. Hard caps: per-file max chars, total tokens via tiktoken-compatible crate (`tiktoken-rs`) or a cheap byte heuristic if the model is unknown.
5. If over budget, drop in this order: world files → unused character sheets → previous-chapter body (keep last 1.5k words) → style guide examples.
6. Persist the pack hash on the job so a retry is deterministic.

This is how we stay faithful to the orchestrator rule: *“Do not dump every skill’s references into context.”*

### 5.3 LLM loop

v1 loop is simple and cheap:

1. System = compiled skill instructions + output contract (path, format, confirmation gate).
2. User = packed files + the user’s answers to the skill’s questions.
3. Model returns markdown (or JSON when the skill has `assets/output-template.json`).
4. Host validates basic shape (non-empty, expected headings if templated).
5. Show preview. User saves → atomic write to the Wiki path.

v1.5 adds tools the model can call:

- `read_file(path, span)`
- `list_wiki()`
- `namegen(culture, n)`
- `town_name(n)`
- `burstiness_report(path)`

Tool results are appended and the model continues. Cap tool rounds (e.g. 8) so a phone cannot burn the subscription on a loop.

Skills that currently say “dispatch focused workers on 40-line chunks” (`fiction-line-editor`, `kill-*`, `fragment-hunter`) become a **host-side chunker**:

- Split chapter into 40-line windows
- Run N sequential (not parallel-unbounded) LLM calls
- Merge patches
- One confirmation at the end

That is both more faithful and more memory-safe than sending the whole chapter plus the whole editor skill in one shot.

### 5.4 Local tools (no network)

Port from the zip, do not shell out to Python:

| Skill | Port |
|---|---|
| `name-generator` | Rust Markov (`order=2`) over `data/*.txt` |
| `town-generator` | same |
| `burstiness-check` / `measure.py` | Rust stats: sentence length variance, opener repeats, dialogue ratio |
| later: fragment regex | already specified as 1–3 word narration sentences |

These run instantly on-device and are the right default before spending tokens.

---

## 6. UI map (mobile)

### Home

- Project picker
- Status board: each spine slot `yes / partial / no`
- Primary CTA: **Next: \<skill\> — \<why\>**
- Secondary: Spark / Open chapter / Run named skill

### Book

- File tree of `Wiki/` and `Chapters/`
- Markdown viewer with cheap virtualization
- Character / location cards generated from filenames, not a second database

### Write

- Chapter list from outline
- Editor (CodeMirror or Milkdown — one of them, not both)
- “Draft this chapter” → job card for `fiction-writechapter`
- Streaming tokens into a preview pane, not into the live file

### Edit

- Editorial ladder as a checklist
- Tap one pass. Do not offer “run everything”
- Diff view (original vs proposed)
- Burstiness numbers rendered as a small chart (local)

### Settings

- Providers (OAuth + API key + custom base URL)
- Default model
- Token budget
- Skill pack path / enable optional overlays
- Export project as zip
- About + OAuth disclaimer

Chat is an implementation detail of a job, not the main surface. Writers think in books and passes, not threads.

---

## 7. Android / APK plan

### 7.1 Why Tauri 2

- One Rust core for desktop and Android
- Official `tauri android build --apk`
- Uses system WebView (no bundled Chromium) → smaller APK, lower RAM than Electron
- Plugins for filesystem, biometric/keystore, custom tabs, background tasks

Alternatives rejected for v1:

| Option | Why not first |
|---|---|
| Pure Kotlin + UniFFI | Best RAM, two UIs to maintain |
| Dioxus mobile | Moving target, weaker Android packaging |
| Slint only | Poor rich-text editing |
| Flutter + rust FFI | Extra toolchain, larger APK |
| Capacitor + WASM | WASM memory + JS editor is the opposite of the brief |

### 7.2 Toolchain

- Rust stable, targets: `aarch64-linux-android` (ship), `x86_64-linux-android` (emulator)
- Android SDK + NDK via Android Studio
- JDK 17
- `cargo-mobile2` / Tauri CLI
- Min SDK: 26 (Keystore + notification channels). Target SDK: current Play requirement at build time

### 7.3 Permissions (keep tiny)

- `INTERNET`
- `FOREGROUND_SERVICE` + typed data-sync for generation
- Optional `POST_NOTIFICATIONS` for job complete
- Storage via SAF, not broad `MANAGE_EXTERNAL_STORAGE`

### 7.4 Build outputs

```
npm/pnpm install   # UI
cargo tauri android init
cargo tauri android build --apk --target aarch64
```

Artifacts:

- debug APK for sideload
- release APK signed with a local keystore
- later: AAB + Play signing

APK size target: **< 25 MB** arm64 release (UI assets + skills as compressed assets + Rust cdylib). Skills can live as an asset pack or extracted on first run into app files.

### 7.5 Background generation

Android will kill a silent WebView mid-stream.

- Start a foreground service when a job enters `running`
- Stream tokens into a file, not only into JS memory
- If the activity dies, reopening the job card tails the file
- User can cancel from the notification

---

## 8. Security and privacy

- Book files never leave the device except as LLM prompt payload the user initiated
- Redact API keys in logs
- Option: “don’t send previous chapters, only scene card” for paranoid mode
- OAuth tokens in Keystore, encrypted-at-rest
- No analytics in v1
- Clear export/delete project
- Prompt firewall: system prompt is the skill text we vendored; user Wiki is data, not instructions with higher priority than the skill’s confirmation gate

---

## 9. Delivery phases

### Phase 0 — Plan (this document)

Approve stack, auth stance, and UX map before code.

### Phase 1 — Core library (desktop CLI first)

Faster to test than an emulator.

- `storycraft-core`: discover Wiki, status board, skill index
- `storycraft-tools`: namegen + burstiness port
- `storycraft` CLI:

```
storycraft status
storycraft next
storycraft run fiction-genre
storycraft pack fiction-writechapter --chapter 3
```

Acceptance: against a fixture book, status board matches `project-layout.md`.

### Phase 2 — LLM + auth

- OpenAI-compat streaming client
- Device-code OAuth module with fake server tests
- Job store + preview writes
- Run `fiction-story-sparks` and `fiction-genre` end-to-end against a mock provider

### Phase 3 — Tauri desktop shell

- Status board, file tree, job card, markdown preview
- Settings for providers
- Confirms the IPC surface the phone will reuse

### Phase 4 — Android APK

- `tauri android init`
- Keystore + Custom Tabs for OAuth
- Foreground service
- Sideload APK on a real phone
- Memory profile with Android Studio

### Phase 5 — Writing loop complete

- Full planning spine with confirmation gates
- `fiction-writechapter` + chunked editorial ladder
- Diff/save
- Export zip

### Phase 6 — Polish

- Optional overlays
- Per-skill model routing
- iOS later
- Play Store listing if you want distribution beyond sideload

---

## 10. Testing strategy

- Fixture library: a tiny book with half the spine filled
- Golden status-board snapshots
- Packer tests: genre skill must **not** pull 90 reference files
- Chunk merge tests for kill-passes
- OAuth: recorded device-code HTTP fixtures
- Provider: wiremock for `/v1/chat/completions` and `/v1/responses`
- Android: one instrumentation test that creates a project and writes `genre.md`

No live Grok calls in CI.

---

## 11. Risks

| Risk | Mitigation |
|---|---|
| SuperGrok OAuth 403 / allowlist | API-key fallback; configurable client id; honest UI copy |
| xAI changes device-code or client id | Isolate in `storycraft-auth`; ship a remote-optional well-known override file |
| WebView RAM on cheap phones | Virtualize editor; consider Slint chrome if we blow 250 MB |
| Skills assume a chatty agent host | Host asks the skill’s questions as forms; do not free-chat the spine |
| Token cost of editorial chunking | Local tools first; cheap model override; show estimate before run |
| Skill text drift vs app parser | Keep skills verbatim; parser only reads frontmatter + linked refs |
| Play policy / sideload only | v1 is sideload; don’t claim official xAI partnership |

---

## 12. Immediate next implementation slice (after you approve)

If you say go, the first concrete code drop should be:

1. Cargo workspace + `storycraft-core` status board
2. Vendored `skills/` from the zip
3. CLI `status` / `next`
4. One-page written decision record for OAuth vs API-key

Then we do not start the Android project until the core can answer “what skill is next?” from a real Wiki folder with zero LLM.

---

## 13. Decisions I am recommending

1. **Tauri 2 + Rust core + tiny Svelte UI** as the APK vehicle.
2. **Skills remain markdown packages**, not hardcoded Rust workflows.
3. **One skill, one job, confirmation gate** — same as the orchestrator.
4. **OpenAI-compat first, Grok device-code OAuth second**, with the allowlist caveat in the UI.
5. **Port Python tools to Rust.** No interpreter on the phone.
6. **Context packer with a hard token budget** is a first-class module, not an afterthought.
7. **Sideload APK is the v1 distribution.** Play Store is phase 6.

---

## 14. What I need from you before writing code

Answer these so phase 1 is not built twice:

1. Confirm Tauri 2 vs “Rust library + Jetpack Compose” (lower RAM, slower to ship).
2. App display name and package id (or keep `Open Storycraft` / `dev.openstorycraft.app`).
3. v1 platforms: Android only, or Android + Linux/Windows desktop from day one?
4. Are optional overlays (`ao3-*`, `anti-slop-editor`) in-scope for the first APK or later?
5. Default model you actually use on SuperGrok today (so the picker can prefer it).
6. Where should books live on the phone — app-private storage, or a user-picked folder from the start?

Once those are settled, implementation can start at the CLI core.
