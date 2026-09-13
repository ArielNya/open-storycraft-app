# Open Storycraft

A local writing studio for the [Storycraft skill library](open-storycraft/README.md). Rust core, Tauri 2 shell, plain HTML/CSS/JS front end. It runs the craft skills one at a time along a planning spine — genre → audience → theme → synopsis → style → characters → voice → outline → scenes → psych → draft — and edits chapters with the editorial ladder.

Everything is read from disk, never from memory: the home screen is a status board derived from the files in the book folder. Model calls go to any OpenAI-compatible endpoint (xAI console keys, OpenAI, Groq, Together, OpenRouter, vLLM, llama.cpp, LM Studio, Ollama's OpenAI shim). Some skills run on device with no provider at all.

The long-form design notes are in [OPEN_STORYCRAFT_APP_PLAN.md](OPEN_STORYCRAFT_APP_PLAN.md).

## What works today

- **Status board** — twelve spine slots, each `yes` / `partial` / `no`, plus a `Next:` skill and the reason it is next.
- **Job cards** — every skill run streams into a preview. Nothing touches the Wiki until you save it; `Diff` and `Reject` are there for the change you did not want.
- **Book view** — file list and preview for the whole `Wiki/` and `Chapters/`, with the current file opened for you.
- **Write view** — chapters listed from the outline, each with a `Draft` button.
- **Edit view** — the ladder (cold read, dev edit, style review, AI-tells, prose, line, filter words, fragments, nominalizations, kill passes) plus the local burstiness report.
- **Storybible** — write the whole book out as one portable `storybible.md`, or unpack one into a book folder. See [Storybible](#storybible).
- **Model picker** — the settings screen asks your provider what it serves and offers it in a dropdown. See [Configuring a provider](#configuring-a-provider).
- **Sideload builds** — desktop binary, `.deb`, and a signed Android APK.

Not here yet: OS keyring storage for the API key (see [Where the key lives](#where-the-key-lives)), iOS, and the world-pack UI (the world skills run, the board slot is there, but there is no world browser).

## Requirements

**Linux desktop**

- Rust (the toolchain is pinned in `rust-toolchain.toml`) with `rustfmt` and `clippy`.
- Runtime: `libwebkit2gtk-4.1-0`, `libgtk-3-0`.
- To *build*: the dev packages — `libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev`.

If `libwebkit2gtk-4.1-dev` is not installable on your distro, unpack its headers into a prefix and point pkg-config at it:

```bash
PKG_CONFIG_PATH=$HOME/.local/tauri-linux-dev/usr/lib/x86_64-linux-gnu/pkgconfig \
  cargo build -p storycraft-app
```

The runtime `.so` is enough to *run*; only linking needs the `-dev` package.

**Android APK**

- JDK 17+ (`JAVA_HOME`).
- Android SDK with `platform-tools`, `build-tools`, a platform, and an NDK (`ANDROID_HOME`, `NDK_HOME`).
- The `aarch64-linux-android` Rust target (`rustup target add aarch64-linux-android`).
- The Gradle build writes to `~/.gradle` and tauri's plugin build scripts write into `$CARGO_HOME/registry`, so both must be writable.

## Build and run

```bash
# desktop, dev
cargo run -p storycraft-app

# desktop, release binary
cargo build --release -p storycraft-app     # target/release/open-storycraft

# desktop, installable package
cd crates/storycraft-app && cargo tauri build --bundles deb
```

The front end is not bundled or transpiled — `ui/index.html`, `ui/app.css`, `ui/app.js` are loaded as they are, and they are compiled into the binary at build time. Edit them, then rebuild.

### Where the skills come from

Resolution order (`crates/storycraft-app/src/paths.rs`):

1. `skills_dir` from settings, if set.
2. The bundled resources — `<resource dir>/skills/fiction-genre/SKILL.md`, which is how the `.deb` and the APK ship the pack.
3. A walk up from the working directory for `open-storycraft/` or `skills/`.

So a release binary run from the repo root finds the vendored pack, and `STORYCRAFT_SKILLS=/path/to/pack` overrides everything.

## Configuring a provider

Settings live in the app config directory — `~/.config/dev.openstorycraft.app/app.json` on Linux — written mode `0600`. Override the path with `STORYCRAFT_APP_SETTINGS=/tmp/app.json` (useful for throwaway runs).

| Field | Notes |
|---|---|
| Provider | `openai-compat`, `xai-apikey`, or `grok-oauth` (community OAuth, needs `Sign in`) |
| Base URL | The base **including** `/v1`, e.g. `https://api.x.ai/v1`, `http://127.0.0.1:1234/v1` |
| API key | Bearer token. Empty for local servers that do not want one |
| API style | `chat_completions` or `responses` (xAI prefers Responses) |
| Model | Type an id, or pick one from the provider |
| Cheap model | Optional. Editorial and kill-pass skills use it when set |
| Per-skill models | `kill-flat=mini` per line, overrides both of the above |
| Token budget | Packer character budget. `0` uses the core default |
| Skills dir | Override for the vendored pack |
| Overlays | `ao3-*` / `anti-slop-editor`. Off unless enabled, and only if the folders exist |

### The model picker

Fill in the URL and key, then press **Load models from the API**. The app calls `GET {base_url}/models` with your bearer token and fills both dropdowns — *Pick a model* and *Pick the cheap model*. Choosing an entry writes into the text field next to it; the text field stays the source of truth, so a hand-typed id is kept and labelled `not in the provider list`. Saving does not require a successful listing.

Details worth knowing:

- A server that only answers under the versioned path is retried once at `{base_url}/v1/models`, so pasting a bare host URL still works.
- The listing is sorted and de-duplicated, and understands `{"data":[{"id":…}]}`, `{"models":[…]}` (with `id` or `name`), and a bare array.
- Failures are reported inline under the button, not as a toast: a bad key says `provider rejected credentials`, a dead host says `HTTP transport error`, an unrecognised body says `provider returned no model list; type the model id instead`.
- A failed refresh keeps the last good list, so options you already loaded are never thrown away.

```bash
# what a provider serves, without the app
curl -s -H "Authorization: Bearer $STORYCRAFT_API_KEY" https://api.x.ai/v1/models | head -c 400
```

### Where the key lives

The key is stored in plain text in the settings file above, which is written `0600` so only your user can read it. It is never logged, and the HTTP client marks the auth header sensitive. It is **not** in an OS keyring yet — that is the one gap against the plan's auth section, so treat that file like a credential.

## Storybible

A storybible is one markdown file that carries a whole book. Two skills, a pair:

| Skill | Does | Runs |
|---|---|---|
| `fiction-storybible` | Writes `storybible.md` beside `Wiki/` — assembling existing canon verbatim, or authoring a new book | model call |
| `storybible-import` | Splits that file into `Wiki/…` and `Chapters/…` | on device, no key |

Each document in the file is a `---` frontmatter block naming where it lands (`path:` for an exact destination, `slot:` for a canonical one), then the body:

```markdown
---
path: Wiki/Style/genre.md
working_title: "Salt Ledger"
genre: Mystery
---

# Genre
```

The routing keys are stripped on the way out, so every written file keeps only its own frontmatter — which means the file round-trips. The full format, including what each destination must contain, is [open-storycraft/fiction-storybible/references/storybible-format.md](open-storycraft/fiction-storybible/references/storybible-format.md).

In the app: **Book → Storybible**. `Write` drafts a bible from the book; `Import` unpacks one. `Import` stays disabled until a `storybible.md` exists, and the status board routes a folder that has a bible but no `Wiki/` straight to `Next: storybible-import`.

A complete sample lives at [crates/storycraft-core/tests/fixtures/bible-only/storybible.md](crates/storycraft-core/tests/fixtures/bible-only/storybible.md). To try it from the CLI:

```bash
mkdir -p /tmp/books/salt-ledger && cp "crates/storycraft-core/tests/fixtures/bible-only/storybible.md" /tmp/books/salt-ledger/
storycraft status /tmp/books                    # Next: storybible-import
storycraft run storybible-import /tmp/books --commit
storycraft status /tmp/books/salt-ledger        # Now: Next: fiction-writechapter
```

## CLI

`cargo run -p storycraft-cli --` (binary name `storycraft`) drives the same core without the GUI.

```bash
storycraft status [PATH]                  # the board, as text
storycraft next [PATH]                    # just the next skill and why
storycraft skills                         # the vendored pack, frontmatter only
storycraft pack <skill>                   # files a run would pack, not the whole folder
storycraft run <skill> [PATH] [--commit]  # one skill; preview unless --commit
storycraft jobs|save|reject|diff [PATH]   # job plumbing
storycraft export [PATH]                  # zip Wiki/ and Chapters/
storycraft auth login|status|logout       # xAI community OAuth
storycraft namegen|town|burstiness        # local tools, no model call
```

`run` takes the provider as flags: `--base-url`, `--api-key` (or `STORYCRAFT_API_KEY`), `--model`, `--cheap-model`, `--route skill=model`, `--api-style`, `--budget`.

## Repository layout

```
crates/storycraft-core   Wiki discovery, status board, packing, jobs, storybible format
crates/storycraft-llm    OpenAI-compatible streaming client (chat_completions, responses)
crates/storycraft-auth   Device-code OAuth and token storage
crates/storycraft-tools  On-device tools: Markov names/towns, burstiness
crates/storycraft-cli    `storycraft` binary
crates/storycraft-app    Tauri 2 shell: commands, host loop, settings, window sizing
ui/                      The whole front end (HTML, CSS, JS; no bundler)
open-storycraft/         The vendored skill pack (see its own README)
book/                    Working fixture used during development
docs/adr/                Architecture decisions
```

## Tests and checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo doc --workspace --no-deps      # broken doc links are denied
```

Fixtures under `crates/storycraft-core/tests/fixtures/` include a planned book (`planning-done`) and a bible-only folder (`bible-only`).

## Building the APK

```bash
export JAVA_HOME=~/path/to/jdk17
export ANDROID_HOME=~/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/<version>

cd crates/storycraft-app
cargo tauri android build --apk --target aarch64     # arm64 phones
cargo tauri android build --apk --target x86_64      # emulator
adb install -r gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

A release APK must be signed. Gradle reads `gen/android/keystore.properties` (gitignored), which points at a keystore:

```bash
cd crates/storycraft-app/gen/android
keytool -genkeypair -v -keystore upload-keystore.jks -keyalg RSA -keysize 2048 \
  -validity 10000 -alias upload
printf 'keyAlias=upload\npassword=<password>\nstoreFile=../upload-keystore.jks\n' > keystore.properties
```

Keep the keystore and its password: Android refuses to install an update signed with a different key. A debug APK (`--debug`) needs no keystore but is roughly twenty times larger and unoptimized. `storeFile` is resolved relative to `gen/android/app/`, which is why the example writes `../upload-keystore.jks`.

## Known limitations

- The API key is on disk in plain text (`0600`), not in a keyring.
- The bundle identifier `dev.openstorycraft.app` ends in `.app`; tauri warns about it. Harmless on Android and Linux, but changing it later changes the Android package name and the desktop config directory.
- The APK ships `arm64-v8a` only unless you build more ABIs.
- No CI configuration is checked in; the commands above are the gates.
